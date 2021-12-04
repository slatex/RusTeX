use crate::interpreter::Interpreter;
use crate::utils::{TeXError, TeXStr};
use std::rc::Rc;
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
impl TeXBox {
    pub fn width(&self) -> i64 {
        match self {
            TeXBox::Void => 0,
            TeXBox::H(hb) => todo!(),
            TeXBox::V(vb) => todo!(),
        }
    }
    pub fn height(&self) -> i64 {
        match self {
            TeXBox::Void => 0,
            TeXBox::H(hb) => todo!(),
            TeXBox::V(vb) => todo!(),
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
}

impl Whatsit {
    pub fn width(&self) -> i64 {
        use Whatsit::*;
        match self {
            Exec(_) => 0,
            Box(b) => b.width(),
            Ext(e) => e.width(),
            GroupLike(w) => w.width(),
            Simple(s) => s.width()
        }
    }
    pub fn height(&self) -> i64 {
        use Whatsit::*;
        match self {
            Exec(_) => 0,
            Box(b) => b.height(),
            Ext(e) => e.height(),
            GroupLike(w) => w.height(),
            Simple(s) => s.height()
        }
    }
    pub fn depth(&self) -> i64 {
        use Whatsit::*;
        match self {
            Exec(_) => 0,
            Box(b) => b.depth(),
            Ext(e) => e.depth(),
            GroupLike(w) => w.depth(),
            Simple(s) => s.depth()
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
    Pdfxform(Option<TeXStr>,Option<TeXStr>,TeXBox,Option<SourceFileReference>)
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