use std::collections::HashMap;
use crate::utils::{TeXError, TeXStr};
use std::sync::Arc;
use crate::fonts::Font;
use crate::references::SourceFileReference;
use crate::{htmlliteral};
use crate::interpreter::params::InterpreterParams;
use crate::interpreter::state::State;

pub trait HasWhatsitIter {
    fn iter_wi(&self) -> WhatsitIter;
}

impl HasWhatsitIter for Vec<Whatsit> {
    fn iter_wi(&self) -> WhatsitIter {
        WhatsitIter::new(self)
    }
}

pub struct WhatsitIter<'a> {
    children:&'a [Whatsit],
    parent:Option<Box<WhatsitIter<'a>>>
}

impl WhatsitIter<'_> {
    pub fn new(v:&Vec<Whatsit>) -> WhatsitIter {
        WhatsitIter {
            children:v.as_slice(),
            parent:None
        }
    }
}

impl <'a> Iterator for WhatsitIter<'a> {
    type Item = &'a Whatsit;
    fn next(&mut self) -> Option<Self::Item> {
        match self.children.get(0) {
            None => match self.parent.take() {
                Some(p) =>{
                    *self = *p;
                    self.next()
                }
                None => None
            }
            Some(Whatsit::Grouped(g)) if !g.opaque() => {
                self.children = &self.children[1..];
                *self = WhatsitIter {
                    children:g.children().as_slice(),
                    parent:Some(Box::new(std::mem::take(self)))
                };
                self.next()
            }
            Some(s) => {
                self.children = &self.children[1..];
                Some(s)
            }
        }
    }
}
impl<'a> Default for WhatsitIter<'a> {
    fn default() -> Self {
        WhatsitIter { children: &[], parent: None }
    }
}

pub static WIDTH_CORRECTION : i32 = 0;
pub static HEIGHT_CORRECTION : i32 = 0;

pub trait WhatsitTrait {
    fn as_whatsit(self) -> Whatsit;
    fn width(&self) -> i32;
    fn height(&self) -> i32;
    fn depth(&self) -> i32;
    fn as_xml_internal(&self,prefix: String) -> String;
    fn has_ink(&self) -> bool;

    fn as_xml(&self) -> String {
        self.as_xml_internal("".to_string())
    }
    fn normalize(self,mode:&ColonMode,ret:&mut Vec<Whatsit>,scale:Option<f32>);
    fn as_html(self,mode:&ColonMode,colon:&mut HTMLColon,node_top:&mut Option<HTMLParent>);
    fn get_ref(&self) -> Option<SourceFileReference>;
}

use crate::stomach::boxes::TeXBox;
use crate::stomach::colon::ColonMode;
use crate::stomach::groups::{GroupClose, WIGroup, WIGroupTrait};
use crate::stomach::html::{HTMLChild, HTMLColon, HTMLParent, HTMLStr};
use crate::stomach::math::{Above, MathGroup};
use crate::stomach::paragraph::Paragraph;
use crate::stomach::simple::SimpleWI;

// -------------------------------------------------------------------------------------------------

#[derive(Clone)]
pub enum Whatsit {
    Exec(Arc<ExecutableWhatsit>),
    Box(TeXBox),
    GroupOpen(WIGroup),
    GroupClose(GroupClose),
    Simple(SimpleWI),
    Char(PrintChar),
    Space(SpaceChar),
    Accent(Accent),
    Math(MathGroup),
    Ls(Vec<Whatsit>),
    Grouped(WIGroup),
    Par(Paragraph),
    Inserts(Insert),
    Above(Above),
    Float(TeXBox)
}

macro_rules! pass_on {
    ($s:tt,$e:ident$(,$tl:expr)*) => (match $s {
        Whatsit::Exec(g) => WhatsitTrait::$e(g $(,$tl)*),
        Whatsit::Box(g) => TeXBox::$e(g $(,$tl)*),
        Whatsit::GroupOpen(g) => WIGroup::$e(g $(,$tl)*),
        Whatsit::GroupClose(g) => GroupClose::$e(g $(,$tl)*),
        Whatsit::Simple(g) => SimpleWI::$e(g $(,$tl)*),
        Whatsit::Char(g) => PrintChar::$e(g $(,$tl)*),
        Whatsit::Space(g) => SpaceChar::$e(g $(,$tl)*),
        Whatsit::Accent(g) => Accent::$e(g $(,$tl)*),
        Whatsit::Math(g) => MathGroup::$e(g $(,$tl)*),
        Whatsit::Ls(_) => panic!("Should never happen!"),
        Whatsit::Grouped(g) => WIGroup::$e(g $(,$tl)*),
        Whatsit::Par(g) => Paragraph::$e(g $(,$tl)*),
        Whatsit::Inserts(g) => Insert::$e(g $(,$tl)*),
        Whatsit::Float(g) => TeXBox::$e(g $(,$tl)*),
        Whatsit::Above(g) => Above::$e(g $(,$tl)*),
        }
    )
}

impl WhatsitTrait for Whatsit {
    fn get_ref(&self) -> Option<SourceFileReference> { pass_on!(self,get_ref) }
    /*fn test(&self) {
        match self {
            Whatsit::Exec(e) => {
                let test = e.deref();
            }
            _ => ()
        }
    }*/
    fn normalize(self, mode: &ColonMode, ret: &mut Vec<Whatsit>, scale: Option<f32>) {
        pass_on!(self,normalize,mode,ret,scale)
    }
    fn as_html(self, mode: &ColonMode, colon: &mut HTMLColon, node_top: &mut Option<HTMLParent>) {
        pass_on!(self,as_html,mode,colon,node_top)
    }
    fn as_whatsit(self) -> Whatsit { self }
    fn width(&self) -> i32 { pass_on!(self,width) }
    fn height(&self) -> i32 { pass_on!(self,height) }
    fn depth(&self) -> i32 { pass_on!(self,depth) }
    fn as_xml_internal(&self, prefix: String) -> String {
        pass_on!(self,as_xml_internal,prefix)
    }
    fn has_ink(&self) -> bool { pass_on!(self,has_ink) }
}

#[derive(Clone)]
pub enum ActionSpec {
    User(TeXStr),
    GotoNum(i32),
    //    file   name    window
    File(TeXStr,TeXStr,Option<TeXStr>),
    FilePage(TeXStr,i32,Option<TeXStr>),
    Name(TeXStr),
    Page(i32)
}

impl ActionSpec {
    pub fn as_link(&self) -> String {
        use ActionSpec::*;
        match self {
            User(str) => {
                let str = str.to_string();
                if str.contains("/URI(") {
                    str.split("/URI(").last().unwrap().split(")").next().unwrap().to_string()
                } else if str.contains("/F(") {
                    str.split("/F(").last().unwrap().split(")").next().unwrap().to_string()
                } else {
                    "".to_string()//TeXErr!("TODO")
                }
            }
            Name(str) => "#".to_string() + &str.to_string(),
            _ => "".to_string()//TeXErr!("TODO")
        }
    }
    pub fn as_xml(&self) -> String {
        use ActionSpec::*;
        match self {
            User(s) => " user=\"".to_string() + &s.to_string() + "\"",
            GotoNum(s) => " goto=\"#".to_string() + &s.to_string() + "\"",
            File(s,t,_) => " file=\"".to_string() + &s.to_string() +
                "#" + &t.to_string() + "\"",
            FilePage(s,t,_) => " filepage=\"".to_string() + &s.to_string() +
                "#" + &t.to_string() + "\"",
            Name(s) => " name=\"".to_string() + &s.to_string() + "\"",
            Page(s) => " page=\"".to_string() + &s.to_string() + "\"",
        }
    }
}

// -------------------------------------------------------------------------------------------------

pub struct ExecutableWhatsit {
    pub _apply : Box<dyn (Fn(&mut State,&dyn InterpreterParams) -> Result<(),TeXError>) + Send + Sync>
}
impl ExecutableWhatsit {
    pub fn as_whatsit(self) -> Whatsit {
        Whatsit::Exec(Arc::new(self))
    }
}
impl WhatsitTrait for Arc<ExecutableWhatsit> {
    fn get_ref(&self) -> Option<SourceFileReference> { None }
    fn normalize(self, _: &ColonMode, ret: &mut Vec<Whatsit>, _: Option<f32>) {
        ret.push(self.as_whatsit())
    }
    fn as_whatsit(self) -> Whatsit {
        Whatsit::Exec(self)
    }
    fn width(&self) -> i32 { 0 }
    fn height(&self) -> i32 { 0 }
    fn depth(&self) -> i32 { 0 }
    fn as_xml_internal(&self, _: String) -> String {
        "".to_string()
    }
    fn has_ink(&self) -> bool { false }
    fn as_html(self, _: &ColonMode, _: &mut HTMLColon, _: &mut Option<HTMLParent>) {}
}

// -------------------------------------------------------------------------------------------------

#[derive(Clone)]
pub struct SpaceChar {
    pub sourceref:Option<SourceFileReference>,
    pub font : Arc<Font>,
    pub nonbreaking: bool
}
impl WhatsitTrait for SpaceChar {
    fn get_ref(&self) -> Option<SourceFileReference> { self.sourceref.clone() }
    fn normalize(self, _: &ColonMode, ret: &mut Vec<Whatsit>, _: Option<f32>) {
        ret.push(self.as_whatsit())
    }
    fn as_whatsit(self) -> Whatsit { Whatsit::Space(self) }
    fn width(&self) -> i32 { self.font.get_width(32) }
    fn height(&self) -> i32 { self.font.get_height(32) }
    fn depth(&self) -> i32 { self.font.get_depth(32) }
    fn as_xml_internal(&self, _: String) -> String {
        " ".to_string()
    }
    fn has_ink(&self) -> bool { false }
    fn as_html(self, _: &ColonMode, colon: &mut HTMLColon, node_top: &mut Option<HTMLParent>) {
        if self.nonbreaking {htmlliteral!(colon,node_top,"&#160;")} else {htmlliteral!(colon,node_top," ")}
    }
}

#[derive(Clone)]
pub struct Accent {
    pub sourceref:Option<SourceFileReference>,
    pub font : Arc<Font>,
    pub char:PrintChar,
    pub acc:i32
}
impl WhatsitTrait for Accent {
    fn get_ref(&self) -> Option<SourceFileReference> { self.sourceref.clone() }
    fn normalize(self, _: &ColonMode, ret: &mut Vec<Whatsit>, _: Option<f32>) {
        ret.push(self.as_whatsit())
    }
    fn as_whatsit(self) -> Whatsit { Whatsit::Accent(self) }
    fn width(&self) -> i32 { self.char.width() }
    fn height(&self) -> i32 { self.char.height() }
    fn depth(&self) -> i32 { self.char.depth() }
    fn as_xml_internal(&self, ind: String) -> String {
        self.char.as_xml_internal(ind) // TODO
    }
    fn has_ink(&self) -> bool { true }
    fn as_html(self, mode: &ColonMode, colon: &mut HTMLColon, node_top: &mut Option<HTMLParent>) {
        let ch = match &self.char.font.file.chartable {
            Some(ct) => Some(ct.get_char(self.char.char)),
            _ => None
        };
        let acc = match &self.font.file.chartable {
            Some(ct) => Some(ct.get_char(self.acc as u8)),
            _ => None
        };
        match (ch,acc) {
            (Some(ch),Some(acc)) => match ACCENTS.get(&(ch,acc)) {
                Some(s) =>
                    {
                        htmlliteral!(colon,node_top,>{*s}<);
                        return ()
                    }
                _ => ()
            }
            _ => ()
        }
        self.char.as_html(mode,colon,node_top) // TODO
    }
}

lazy_static! {
    static ref ACCENTS : HashMap<(&'static str,&'static str),&'static str> = HashMap::from([
        (("a"," ̈"),"ä"),(("o"," ̈"),"ö"),(("u"," ̈"),"ü"),(("a","^"),"â"),(("e","^"),"ê"),(("ı","^"),"î"),
        (("o","^"),"ô"),(("u","^"),"û"),(("ȷ","^"),"ĵ"),(("c","^"),"ĉ"),(("y","^"),"ŷ"),(("w","^"),"ŵ"),
        (("z","^"),"ẑ"),(("s","^"),"ŝ"),(("g","^"),"ĝ"),(("h","^"),"ĥ"),(("a"," ́"),"á"),(("e"," ́"),"é"),
        (("ı"," ́"),"í"),(("o"," ́"),"ó"),(("u"," ́"),"ú"),(("r"," ́"),"ŕ"),(("z"," ́"),"ź"),(("s"," ́"),"ś"),
        (("g"," ́"),"ǵ"),(("k"," ́"),"ḱ"),(("l"," ́"),"ĺ"),(("y"," ́"),"ý"),(("c"," ́"),"ć"),(("n"," ́"),"ń"),
        (("a","`"),"à"),(("e","`"),"è"),(("ı","`"),"ì"),(("o","`"),"ò"),(("u","`"),"ù"),(("y","`"),"ỳ"),

        (("A"," ̈"),"Ä"),(("O"," ̈"),"Ö"),(("U"," ̈"),"Ü"),(("A","^"),"Â"),(("E","^"),"Ê"),(("I","^"),"Î"),
        (("O","^"),"Ô"),(("U","^"),"Û"),(("J","^"),"Ĵ"),(("C","^"),"Ĉ"),(("Y","^"),"Ŷ"),(("W","^"),"Ŵ"),
        (("Z","^"),"Ẑ"),(("S","^"),"Ŝ"),(("G","^"),"Ĝ"),(("H","^"),"Ĥ"),(("A"," ́"),"Á"),(("E"," ́"),"É"),
        (("I"," ́"),"Í"),(("O"," ́"),"Ó"),(("U"," ́"),"Ú"),(("R"," ́"),"Ŕ"),(("Z"," ́"),"Ź"),(("S"," ́"),"Ś"),
        (("G"," ́"),"Ǵ"),(("K"," ́"),"Ḱ"),(("L"," ́"),"Ĺ"),(("Y"," ́"),"Ý"),(("C"," ́"),"Ć"),(("N"," ́"),"Ń"),
        (("A","`"),"À"),(("E","`"),"È"),(("I","`"),"Ì"),(("O","`"),"Ò"),(("U","`"),"Ù"),(("Y","`"),"Ỳ"),
        ((" ","^"),"^"),(("c","ˇ"),"č")
    ]);
}

#[derive(Clone)]
pub struct PrintChar {
    pub char : u8,
    pub font : Arc<Font>,
    pub sourceref:Option<SourceFileReference>
}
impl WhatsitTrait for PrintChar {
    fn get_ref(&self) -> Option<SourceFileReference> { self.sourceref.clone() }
    fn normalize(self, _: &ColonMode, ret: &mut Vec<Whatsit>, _: Option<f32>) {
        ret.push(self.as_whatsit())
    }
    fn as_whatsit(self) -> Whatsit { Whatsit::Char(self) }
    fn width(&self) -> i32 { self.font.get_width(self.char as u16) }
    fn height(&self) -> i32 { self.font.get_height(self.char as u16) }
    fn depth(&self) -> i32 { self.font.get_depth(self.char as u16) }
    fn as_xml_internal(&self, _: String) -> String {
        fn is_ascii(u:u8) -> bool {
            (32 <= u && u <= 126) || u > 160
        }
        if self.char == 60 {
            "&lt;".to_string()
        } else if self.char == 62 {
            "&gt;".to_string()
        } else if self.char == 38 {
            "&amp;".to_string()
        } else if is_ascii(self.char) {
            std::char::from_u32(self.char as u32).unwrap().to_string()
        } else {
            "<char value=\"".to_string() + &self.char.to_string() + "\"/>"
        }
    }
    fn has_ink(&self) -> bool { true }
    fn as_html(self, _: &ColonMode, colon: &mut HTMLColon, node_top: &mut Option<HTMLParent>) {
        htmlliteral!(colon,node_top,{
            match &self.font.file.chartable {
                Some(ct) => {
                    let ch = ct.get_char(self.char).to_string();
                    if ch == " " {
                        "&#160;".to_string()
                    } else {HTMLStr::from(ch).html_escape().to_string()}
                }
                None => self.as_xml_internal("".to_string())
            }
        })
    }
}

#[derive(Clone)]
pub struct Insert(pub Vec<Vec<Whatsit>>);
impl WhatsitTrait for Insert {
    fn get_ref(&self) -> Option<SourceFileReference> {
        SourceFileReference::from_wi_lists(&self.0)
    }
    fn as_whatsit(self) -> Whatsit { Whatsit::Inserts(self) }
    fn width(&self) -> i32 { 0 }//TeXErr!("TODO") }
    fn height(&self) -> i32 { 0 }//TeXErr!("TODO") }
    fn depth(&self) -> i32 { 0 }//TeXErr!("TODO") }
    fn as_xml_internal(&self, prefix: String) -> String {
        let mut ret = "<inserts>".to_string();
        for v in &self.0 {
            for w in v {ret += &w.as_xml_internal(prefix.clone())}
        }
        ret + "</inserts"
    }
    fn has_ink(&self) -> bool { true }
    fn normalize(self, mode: &ColonMode, ret: &mut Vec<Whatsit>, scale: Option<f32>) {
        let mut iret : Vec<Vec<Whatsit>> = vec!();
        for v in self.0 {
            let mut iiret : Vec<Whatsit> = vec!();
            for w in v { w.normalize(mode, &mut iiret, scale) }
            if !iiret.is_empty() {iret.push(iiret)}
        }
        if !iret.is_empty() { ret.push(Insert(iret).as_whatsit())}
    }
    fn as_html(self, mode: &ColonMode, colon: &mut HTMLColon, node_top: &mut Option<HTMLParent>) {
        htmlliteral!(colon,node_top,"\n<hr/>\n");
        for v in self.0 {
            for w in v {
                w.as_html(mode,colon,node_top)
            }
        }
    }
}