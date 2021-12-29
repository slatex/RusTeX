use std::cmp::{max, min, Ordering};
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
    Ext(Rc<dyn ExtWhatsit>),
    GroupOpen(WIGroup),
    GroupClose(GroupClose),
    Simple(SimpleWI),
    Char(u8,Rc<Font>,Option<SourceFileReference>),
    Math(MathGroup),
    MathInfix(MathInfix),
    Ls(Vec<Whatsit>),
    Grouped(WIGroup),
    Par(Paragraph),
    Inserts(Vec<Vec<Whatsit>>),
    Float(TeXBox)
}

impl Whatsit {
    pub fn as_xml(&self) -> String {
        self.as_xml_internal("".to_string())
    }
    pub fn as_xml_internal(&self,prefix: String) -> String {
        use Whatsit::*;
        match self {
            Exec(_) | GroupOpen(_) | GroupClose(_) => "".to_string(),
            Ext(e) => e.as_xml_internal(prefix),
            Math(m) => m.as_xml_internal(prefix),
            Simple(s) => s.as_xml_internal(prefix),
            Grouped(g) => g.as_xml_internal(prefix),
            Par(p) => p.as_xml_internal(prefix),
            Box(b) => b.as_xml_internal(prefix),
            Char(u,_,_) => {
                fn is_ascii(u:&u8) -> bool {
                    (32 <= *u && *u <= 126) || *u > 160
                }
                if *u == 60 {
                    "&lt;".to_string()
                } else if *u == 62 {
                    "&gt;".to_string()
                } else if *u == 38 {
                    "&amp;".to_string()
                } else if is_ascii(u) {
                    std::char::from_u32(*u as u32).unwrap().to_string()
                } else {
                    "<char value=\"".to_string() + &u.to_string() + "\"/>"
                }
            },
            MathInfix(i) => i.as_xml_internal(prefix),
            Inserts(vs) => {
                let mut ret = "\n".to_string() + &prefix + "<inserts>";
                for v in vs {
                    ret += "\n  ";
                    ret += &prefix;
                    ret += "<insert>";
                    for w in v {
                        ret  += &w.as_xml_internal(prefix.clone() + "    ")
                    }
                    ret += "\n  ";
                    ret += &prefix;
                    ret += "</insert>";
                }
                ret + "\n" + &prefix + "</inserts>"
            }
            Float(bx) => {
                let mut ret = "\n".to_string() + &prefix + "<float>";
                ret  += &bx.as_xml_internal(prefix.clone() + "  ");
                ret + "\n" + &prefix + "</float>"
            }
            Ls(_) => unreachable!(),
            _ => todo!()
        }

    }
    pub fn has_ink(&self) -> bool {
        use Whatsit::*;
        match self {
            Exec(_) | GroupClose(_) => false,
            Box(b) => b.has_ink(),
            Ext(e) => e.has_ink(),
            GroupOpen(w) => w.has_ink(),
            Grouped(w) => w.has_ink(),
            Simple(s) => s.has_ink(),
            Char(_,_,_) | Par(_) | Float(_) | Inserts(_) | MathInfix(_) => true,
            Math(m) => m.has_ink(),
            Ls(_) => unreachable!()
        }
    }
    pub fn width(&self) -> i32 {
        use Whatsit::*;
        match self {
            Exec(_) | GroupClose(_) => 0,
            Box(b) => b.width(),
            Ext(e) => e.width(),
            GroupOpen(w) => w.width(),
            Grouped(w) => w.width(),
            Simple(s) => s.width(),
            Char(u,f,_) => f.get_width(*u as u16),
            Math(m) => m.width(),
            Par(p) => p.width(),
            Float(s) => s.width(),
            Inserts(s) => todo!(),
            MathInfix(m) => m.width(),
            Ls(_) => unreachable!(),

        }
    }
    pub fn height(&self) -> i32 {
        use Whatsit::*;
        match self {
            Exec(_) | GroupClose(_) => 0,
            Box(b) => b.height(),
            Ext(e) => e.height(),
            GroupOpen(w) => w.height(),
            Grouped(w) => w.height(),
            Simple(s) => s.height(),
            Char(u,f,_) => f.get_height(*u as u16),
            Math(m) => m.height(),
            Par(p) => p.height(),
            Ls(_) => unreachable!(),
            Float(s) => s.height(),
            Inserts(s) => todo!(),
            MathInfix(m) => m.height(),
            _ => todo!()
        }
    }
    pub fn depth(&self) -> i32 {
        use Whatsit::*;
        match self {
            Exec(_) | GroupClose(_) => 0,
            Box(b) => b.depth(),
            Ext(e) => e.depth(),
            GroupOpen(w) => w.depth(),
            Grouped(w) => w.depth(),
            Simple(s) => s.depth(),
            Char(u,f,_) => f.get_depth(*u as u16),
            Math(m) => m.depth(),
            Par(p) => p.depth(),
            Ls(_) => unreachable!(),
            Float(s) => s.depth(),
            Inserts(s) => todo!(),
            MathInfix(m) => m.depth(),
            _ => todo!()
        }
    }
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

pub struct ExecutableWhatsit {
    pub _apply : Box<dyn FnOnce(&Interpreter) -> Result<(),TeXError>>
}

pub trait ExtWhatsit {
    fn name(&self) -> TeXStr;
    fn reference(&self) -> Option<SourceFileReference>;
    fn children(&self) -> Vec<Whatsit>;
    fn isGroup(&self) -> bool;
    fn height(&self) -> i32;
    fn width(&self) -> i32;
    fn depth(&self) -> i32;
    fn has_ink(&self) -> bool;
    fn as_xml_internal(&self,prefix:String) -> String;
}