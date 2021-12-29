use std::cmp::{max, min, Ordering};
use std::ops::Deref;
use std::path::PathBuf;
use crate::interpreter::Interpreter;
use crate::utils::{TeXError, TeXStr};
use std::rc::Rc;
use std::str::from_utf8;
use image::{DynamicImage, GenericImageView};
use crate::commands::MathWhatsit;
use crate::fonts::Font;
use crate::interpreter::dimensions::{dimtostr, MuSkip, Skip};
use crate::references::SourceFileReference;
use crate::stomach::StomachGroup;
use crate::Token;

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
}

use crate::stomach::boxes::{BoxMode,TeXBox};
use crate::stomach::groups::{GroupClose, WIGroup, WIGroupTrait};
use crate::stomach::math::{MathGroup, MathInfix};
use crate::stomach::paragraph::Paragraph;
use crate::stomach::simple::SimpleWI;

// -------------------------------------------------------------------------------------------------

#[derive(Clone)]
pub enum Whatsit {
    Exec(Rc<ExecutableWhatsit>),
    Box(TeXBox),
    GroupOpen(WIGroup),
    GroupClose(GroupClose),
    Simple(SimpleWI),
    Char(PrintChar),
    Math(MathGroup),
    MathInfix(MathInfix),
    Ls(Vec<Whatsit>),
    Grouped(WIGroup),
    Par(Paragraph),
    Inserts(Insert),
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
        Whatsit::Math(g) => MathGroup::$e(g $(,$tl)*),
        Whatsit::MathInfix(g) => MathInfix::$e(g $(,$tl)*),
        Whatsit::Ls(_) => panic!("Should never happen!"),
        Whatsit::Grouped(g) => WIGroup::$e(g $(,$tl)*),
        Whatsit::Par(g) => Paragraph::$e(g $(,$tl)*),
        Whatsit::Inserts(g) => Insert::$e(g $(,$tl)*),
        Whatsit::Float(g) => TeXBox::$e(g $(,$tl)*),
        }
    )
}

impl WhatsitTrait for Whatsit {
    /*fn test(&self) {
        match self {
            Whatsit::Exec(e) => {
                let test = e.deref();
            }
            _ => ()
        }
    }*/
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
    pub _apply : Box<dyn FnOnce(&Interpreter) -> Result<(),TeXError>>
}
impl ExecutableWhatsit {
    pub fn as_whatsit(self) -> Whatsit {
        Whatsit::Exec(Rc::new(self))
    }
}
impl WhatsitTrait for Rc<ExecutableWhatsit> {
    fn as_whatsit(self) -> Whatsit {
        Whatsit::Exec(self)
    }
    fn width(&self) -> i32 { 0 }
    fn height(&self) -> i32 { 0 }
    fn depth(&self) -> i32 { 0 }
    fn as_xml_internal(&self, prefix: String) -> String {
        "".to_string()
    }
    fn has_ink(&self) -> bool { false }
}

// -------------------------------------------------------------------------------------------------

#[derive(Clone)]
pub struct PrintChar {
    pub char : u8,
    pub font : Rc<Font>,
    pub sourceref:Option<SourceFileReference>
}
impl WhatsitTrait for PrintChar {
    fn as_whatsit(self) -> Whatsit { Whatsit::Char(self) }
    fn width(&self) -> i32 { self.font.get_width(self.char as u16) }
    fn height(&self) -> i32 { self.font.get_height(self.char as u16) }
    fn depth(&self) -> i32 { self.font.get_depth(self.char as u16) }
    fn as_xml_internal(&self, prefix: String) -> String {
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
}

#[derive(Clone)]
pub struct Insert(pub Vec<Vec<Whatsit>>);
impl WhatsitTrait for Insert {
    fn as_whatsit(self) -> Whatsit { Whatsit::Inserts(self) }
    fn width(&self) -> i32 { todo!() }
    fn height(&self) -> i32 { todo!() }
    fn depth(&self) -> i32 { todo!() }
    fn as_xml_internal(&self, prefix: String) -> String {
        let mut ret = "<inserts>".to_string();
        for v in &self.0 {
            for w in v {ret += &w.as_xml_internal(prefix.clone())}
        }
        ret + "</inserts"
    }
    fn has_ink(&self) -> bool { true }
}