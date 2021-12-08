use crate::interpreter::Interpreter;
use crate::utils::{TeXError, TeXStr};
use std::rc::Rc;
use crate::fonts::Font;
use crate::interpreter::dimensions::Skip;
use crate::references::SourceFileReference;

#[derive(Copy,Clone,PartialEq)]
pub enum BoxMode { H,V,M,DM,Void }

#[derive(Clone)]
pub struct HBox {
    pub children:Vec<Whatsit>,
    pub spread:i64,
    pub _width:Option<i64>,
    pub _height:Option<i64>,
    pub _depth:Option<i64>
}

#[derive(Clone)]
pub struct VBox {
    pub children:Vec<Whatsit>,
    pub center:bool,
    pub spread:i64,
    pub _width:Option<i64>,
    pub _height:Option<i64>,
    pub _depth:Option<i64>
}

#[derive(Clone)]
pub enum TeXBox {
    Void,H(HBox),V(VBox)
}

static WIDTH_CORRECTION : i64 = 0;
static HEIGHT_CORRECTION : i64 = 0;

impl TeXBox {
    pub fn has_ink(&self) -> bool {
        match self {
            TeXBox::Void => false,
            TeXBox::H(hb) => {
                for c in &hb.children { if c.has_ink() { return true } }
                false
            }
            TeXBox::V(vb) => {
                for c in &vb.children { if c.has_ink() { return true } }
                false
            }
        }
    }
    pub fn width(&self) -> i64 {
        match self {
            TeXBox::Void => 0,
            TeXBox::H(hb) => match hb._width {
                Some(i) => i,
                None => {
                    let mut w = hb.spread;
                    for c in &hb.children { w += c.width() + WIDTH_CORRECTION }
                    w
                }
            },
            TeXBox::V(vb) => match vb._width {
                Some(i) => i,
                None => {
                    let mut w = 0;
                    for c in &vb.children {
                        let wd = c.width();
                        if wd > w { w = wd }
                    }
                    w
                }
            },
        }
    }
    pub fn height(&self) -> i64 {
        match self {
            TeXBox::Void => 0,
            TeXBox::H(hb) =>  match hb._height {
                Some(i) => i,
                None => {
                    let mut w = 0;
                    for c in &hb.children {
                        let ht = c.height();
                        if ht > w { w = ht }
                    }
                    w
                }
            },
            TeXBox::V(vb) => match vb._height {
                Some(i) => i,
                None => {
                    let mut w = vb.spread;
                    for c in &vb.children { w += c.height() + HEIGHT_CORRECTION }
                    w
                }
            },
        }
    }
    pub fn depth(&self) -> i64 {
        match self {
            TeXBox::Void => 0,
            TeXBox::H(hb) => todo!(),
            TeXBox::V(vb) => todo!(),
        }
    }
}

#[derive(Clone)]
pub struct MathWI {
    pub tp : TeXStr,
    pub children:Vec<Whatsit>
}
impl MathWI {
    pub fn width(&self) -> i64 { todo!( )}
    pub fn height(&self) -> i64 { todo!( )}
    pub fn depth(&self) -> i64 { todo!( )}
}

#[derive(Clone)]
pub enum Whatsit {
    Exec(Rc<ExecutableWhatsit>),
    Box(TeXBox),
    Ext(Rc<dyn ExtWhatsit>),
    GroupLike(WIGroup),
    Simple(SimpleWI),
    Char(u8,Rc<Font>,Option<SourceFileReference>),
    Ls(Vec<Whatsit>),
    Grouped(WIGroup)
}

impl Whatsit {
    pub fn has_ink(&self) -> bool {
        use Whatsit::*;
        match self {
            Exec(_) => false,
            Box(b) => b.has_ink(),
            Ext(e) => e.has_ink(),
            GroupLike(w) => w.has_ink(),
            Grouped(w) => w.has_ink(),
            Simple(s) => s.has_ink(),
            Char(u,f,_) => true,
            Ls(_) => unreachable!()
        }
    }
    pub fn width(&self) -> i64 {
        use Whatsit::*;
        match self {
            Exec(_) => 0,
            Box(b) => b.width(),
            Ext(e) => e.width(),
            GroupLike(w) => w.width(),
            Grouped(w) => w.width(),
            Simple(s) => s.width(),
            Char(u,f,_) => f.get_width(*u as u16),
            Ls(_) => unreachable!()
        }
    }
    pub fn height(&self) -> i64 {
        use Whatsit::*;
        match self {
            Exec(_) => 0,
            Box(b) => b.height(),
            Ext(e) => e.height(),
            GroupLike(w) => w.height(),
            Grouped(w) => w.height(),
            Simple(s) => s.height(),
            Char(u,f,_) => f.get_height(*u as u16),
            Ls(_) => unreachable!()
        }
    }
    pub fn depth(&self) -> i64 {
        use Whatsit::*;
        match self {
            Exec(_) => 0,
            Box(b) => b.depth(),
            Ext(e) => e.depth(),
            GroupLike(w) => w.depth(),
            Grouped(w) => w.depth(),
            Simple(s) => s.depth(),
            Char(u,f,_) => f.get_depth(*u as u16),
            Ls(_) => unreachable!()
        }
    }
}

#[derive(Clone)]
pub enum WIGroup {
    FontChange(Rc<Font>,Option<SourceFileReference>,bool,Vec<Whatsit>),
    ColorChange(TeXStr,Option<SourceFileReference>,Vec<Whatsit>),
    ColorEnd(Option<SourceFileReference>)
}
impl WIGroup {
    pub fn has_ink(&self) -> bool {
        use Whatsit::*;
        for x in self.children() { if x.has_ink() {return true} }
        false
    }
    pub fn children_d(self) -> Vec<Whatsit> {
        match self {
            WIGroup::FontChange(_,_,_,v) => v,
            WIGroup::ColorChange(_,_,v) => v,
            WIGroup::ColorEnd(_) => unreachable!()
        }
    }
    pub fn children(&self) -> &Vec<Whatsit> {
        match self {
            WIGroup::FontChange(_,_,_,v) => v,
            WIGroup::ColorChange(_,_,v) => v,
            WIGroup::ColorEnd(_) => unreachable!()
        }
    }
    pub fn new_from(&self,children:Vec<Whatsit>) -> WIGroup {
        match self {
            WIGroup::FontChange(f,r,b,_) => WIGroup::FontChange(f.clone(),r.clone(),*b,children),
            WIGroup::ColorChange(c,r,_) => WIGroup::ColorChange(c.clone(),r.clone(),children),
            WIGroup::ColorEnd(_) => unreachable!()
        }
    }
    pub fn width(&self) -> i64 { todo!( )}
    pub fn height(&self) -> i64 { todo!( )}
    pub fn depth(&self) -> i64 { todo!( )}
    pub fn closesWithGroup(&self) -> bool {
        match self {
            WIGroup::FontChange(_,_,b,_) => !*b,
            _ => false
        }
    }
}

#[derive(Clone)]
pub enum ActionSpec {
    User(TeXStr),
    GotoNum(i64),
    //   file    name    window
    File(TeXStr,TeXStr,Option<TeXStr>),
    FilePage(TeXStr,i64,Option<TeXStr>),
    Name(TeXStr),
    Page(i64)
}

#[derive(Clone)]
pub enum SimpleWI {
    //                                  height       width      depth
    VRule(Option<SourceFileReference>,Option<i64>,Option<i64>,Option<i64>),
    VFil(Option<SourceFileReference>),
    VFill(Option<SourceFileReference>),
    VSkip(Skip,Option<SourceFileReference>),
    HSkip(Skip,Option<SourceFileReference>),
    HFil(Option<SourceFileReference>),
    HFill(Option<SourceFileReference>),
    Penalty(i64),
    PdfLiteral(TeXStr,Option<SourceFileReference>),
    //          attr            resource
    Pdfxform(Option<TeXStr>,Option<TeXStr>,TeXBox,Option<SourceFileReference>),
    Raise(i64,TeXBox,Option<SourceFileReference>),
    Kern(i64,Option<SourceFileReference>),
    PdfDest(TeXStr,TeXStr,Option<SourceFileReference>)
}
impl SimpleWI {
    pub fn has_ink(&self) -> bool {
        use SimpleWI::*;
        match self {
            VRule(_,_,_,_) => true,
            VFil(_) | VFill(_) | VSkip(_,_) | HSkip(_,_) | HFil(_) | HFill(_) | Penalty(_) |
            PdfLiteral(_,_) | Pdfxform(_,_,_,_) | Kern(_,_) | PdfDest(_,_,_) => false,
            Raise(_,bx,_) => bx.has_ink()
        }
    }
    pub fn width(&self) -> i64 { todo!( )}
    pub fn height(&self) -> i64 { todo!( )}
    pub fn depth(&self) -> i64 { todo!( )}
}


pub struct ExecutableWhatsit {
    pub _apply : Box<dyn FnOnce(&Interpreter) -> Result<(),TeXError>>
}

pub trait ExtWhatsit {
    fn name(&self) -> TeXStr;
    fn reference(&self) -> Option<SourceFileReference>;
    fn children(&self) -> Vec<Whatsit>;
    fn isGroup(&self) -> bool;
    fn height(&self) -> i64;
    fn width(&self) -> i64;
    fn depth(&self) -> i64;
    fn has_ink(&self) -> bool;
}

// -------------------------------------------------------------------------------------------------

pub struct VRule {
    reference : SourceFileReference
}