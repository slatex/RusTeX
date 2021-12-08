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
    Ls(Vec<Whatsit>)
}

impl Whatsit {
    pub fn width(&self) -> i64 {
        use Whatsit::*;
        match self {
            Exec(_) => 0,
            Box(b) => b.width(),
            Ext(e) => e.width(),
            GroupLike(w) => w.width(),
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
            Simple(s) => s.depth(),
            Char(u,f,_) => f.get_depth(*u as u16),
            Ls(_) => unreachable!()
        }
    }
}

#[derive(Clone)]
pub enum WIGroup {

}
impl WIGroup {
    pub fn width(&self) -> i64 { todo!( )}
    pub fn height(&self) -> i64 { todo!( )}
    pub fn depth(&self) -> i64 { todo!( )}
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
    HFil(Option<SourceFileReference>),
    HFill(Option<SourceFileReference>),
    Penalty(i64),
    PdfLiteral(TeXStr,Option<SourceFileReference>),
    //          attr            resource
    Pdfxform(Option<TeXStr>,Option<TeXStr>,TeXBox,Option<SourceFileReference>),
    Raise(i64,TeXBox,Option<SourceFileReference>),
    Kern(i64,Option<SourceFileReference>),
    Skip(Skip,Option<SourceFileReference>),
    PdfDest(TeXStr,TeXStr,Option<SourceFileReference>)
}
impl SimpleWI {
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
}

// -------------------------------------------------------------------------------------------------

pub struct VRule {
    reference : SourceFileReference
}