use std::cmp::max;
use std::collections::HashMap;
use std::io::Write;
use crate::utils::{TeXError, TeXStr};
use std::sync::Arc;
use crate::fonts::{ArcFont, Font};
use crate::references::SourceFileReference;
use crate::{htmlliteral, htmlnode, htmlparent};
use crate::interpreter::params::InterpreterParams;
use crate::interpreter::state::State;
use crate::stomach::html::{dimtohtml, HTMLNode};

fn lineheight_i(baselineskip:i32,lineskip:i32,lineskiplimit:i32,fontsize:i32) -> i32 {
    if baselineskip >= (lineskiplimit+fontsize) {baselineskip} else {
        fontsize + lineskip
    }
}
pub fn lineheight(state:&State) -> i32 {
    let bls = state.skips_prim.get(&(crate::commands::registers::BASELINESKIP.index - 1)).base;
    let ls = state.skips_prim.get(&(crate::commands::registers::LINESKIP.index - 1)).base;
    let lim = state.dimensions_prim.get(&(crate::commands::registers::LINESKIPLIMIT.index - 1));
    let fnt = state.currfont.get().get_at();
    lineheight_i(bls,ls,lim,fnt)
}

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
    //fn get_par_width(&self) -> Option<i32>;
    //fn get_par_widths(&self) -> Vec<i32>;
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
    //fn get_par_width(&self) -> Option<i32> { pass_on!(self,get_par_width)}
    //fn get_par_widths(&self) -> Vec<i32> { pass_on!(self,get_par_widths)}
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
    //fn get_par_width(&self) -> Option<i32> { None }
    //fn get_par_widths(&self) -> Vec<i32> { vec!() }
}

// -------------------------------------------------------------------------------------------------

#[derive(Clone)]
pub struct SpaceChar {
    pub sourceref:Option<SourceFileReference>,
    pub font : ArcFont,
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
    fn as_html(self, mode: &ColonMode, colon: &mut HTMLColon, node_top: &mut Option<HTMLParent>) {
        match mode {
            ColonMode::H => htmlnode!(colon,div,self.get_ref(),"rustex-space-in-hbox",node_top,node => {htmlliteral!(colon,htmlparent!(node),<&str as Into<HTMLStr>>::into("&#160;"))}),
            _ => {
                let str: HTMLStr = if self.nonbreaking { "&#160;".into() } else { " ".into() };
                htmlliteral!(colon,node_top,str);
                /*
                let str: HTMLStr = if self.nonbreaking { "&#160;".into() } else { " ".into() };
                let maybetext = match match node_top {
                    Some(HTMLParent::N(n)) => n.children.last_mut(),
                    Some(HTMLParent::A(n)) => n.children.last_mut(),
                    _ => None
                } {
                    Some(HTMLChild::Node(n)) => Some(n),
                    _ => None
                };
                match maybetext {
                    Some(n) if n.classes.contains(&"rustex-text".into()) =>
                        n.children.push(HTMLChild::Str(str.into())),
                    _ => htmlnode!(colon,span,None,"text",node_top,span => {
                        htmlliteral!(colon,htmlparent!(span),str);
                    })
                }
                 */
            }
        }
    }
    //fn get_par_width(&self) -> Option<i32> { None }
    //fn get_par_widths(&self) -> Vec<i32> { vec!() }
}

#[derive(Clone)]
pub struct Accent {
    pub sourceref:Option<SourceFileReference>,
    pub font : ArcFont,
    pub char:PrintChar,
    pub acc:i32,
    pub accstr:&'static str
}
impl WhatsitTrait for Accent {
    fn as_whatsit(self) -> Whatsit { Whatsit::Accent(self) }
    fn width(&self) -> i32 { self.char.width() }
    fn height(&self) -> i32 { self.char.height() }
    fn depth(&self) -> i32 { self.char.depth() }
    fn as_xml_internal(&self, ind: String) -> String {
        self.char.as_xml_internal(ind) // TODO
    }
    fn has_ink(&self) -> bool { true }
    fn normalize(self, _: &ColonMode, ret: &mut Vec<Whatsit>, _: Option<f32>) {
        ret.push(self.as_whatsit())
    }
    fn as_html(self, mode: &ColonMode, colon: &mut HTMLColon, node_top: &mut Option<HTMLParent>) {
        let ch = self.char.charstr;
        let acc = self.accstr;
        match (ch,acc) {
            (ch,acc) if ch != "???" && acc != "???" => match ACCENTS.get(&(ch,acc)) {
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
    fn get_ref(&self) -> Option<SourceFileReference> { self.sourceref.clone() }
    //fn get_par_width(&self) -> Option<i32> { None }
    //fn get_par_widths(&self) -> Vec<i32> { vec!() }
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
    pub font : ArcFont,
    pub sourceref:Option<SourceFileReference>,
    pub charstr: &'static str
}
impl WhatsitTrait for PrintChar {
    fn as_whatsit(self) -> Whatsit { Whatsit::Char(self) }
    fn width(&self) -> i32 { self.font.get_width(self.char as u16) }
    fn height(&self) -> i32 {(max(self.font.get_height(self.char as u16),0) * 6) / 5 }
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
    fn normalize(self, _: &ColonMode, ret: &mut Vec<Whatsit>, _: Option<f32>) {
        ret.push(self.as_whatsit())
    }
    fn as_html(self, mode: &ColonMode, colon: &mut HTMLColon, node_top: &mut Option<HTMLParent>) {
        let str: HTMLStr = HTMLStr::from(self.charstr).html_escape();
        match mode {
            ColonMode::H => {
                let maybetext = match match node_top {
                    Some(HTMLParent::N(n)) => n.children.last_mut(),
                    Some(HTMLParent::A(n)) => n.children.last_mut(),
                    _ => None
                } {
                    Some(HTMLChild::Node(n)) => Some(n),
                    _ => None
                };;
                match maybetext {
                    Some(n) if n.classes.contains(&"rustex-text".into()) =>{
                        n.children.push(HTMLChild::Str(str.into()));
                        /*n.style("height".into(),"".into());
                        n.style("width".into(),"".into());
                        n.style("line-height".into(),"".into());*/
                    }
                    _ => htmlnode!(colon,span,None,"rustex-text",node_top,span => {
                        let h = self.font.get_height(self.char as u16);
                        /*span.ht = h;
                        span.style("height".into(),dimtohtml(h));
                        span.style("width".into(),dimtohtml(self.font.get_width(self.char as u16)));
                        span.style("line-height".into(),"0".into());*/
                        htmlliteral!(colon,htmlparent!(span),str);
                    })
                }
            }
            _ =>
                node_top.as_mut().unwrap().push(HTMLChild::Str(str.into()))
        }
    }
    fn get_ref(&self) -> Option<SourceFileReference> { self.sourceref.clone() }
    //fn get_par_width(&self) -> Option<i32> { None }
    //fn get_par_widths(&self) -> Vec<i32> { vec!() }
}

#[derive(Clone)]
pub struct Insert(pub Vec<Vec<Whatsit>>);
impl WhatsitTrait for Insert {
    fn as_whatsit(self) -> Whatsit { Whatsit::Inserts(self) }
    fn width(&self) -> i32 { 0 }
    //TeXErr!("TODO") }
    fn height(&self) -> i32 { 0 }
    //TeXErr!("TODO") }
    fn depth(&self) -> i32 { 0 }
    //TeXErr!("TODO") }
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
    fn get_ref(&self) -> Option<SourceFileReference> {
        SourceFileReference::from_wi_lists(&self.0)
    }
    //fn get_par_width(&self) -> Option<i32> { None }
    //fn get_par_widths(&self) -> Vec<i32> { vec!() }
}