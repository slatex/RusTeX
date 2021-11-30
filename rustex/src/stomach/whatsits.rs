use crate::interpreter::Interpreter;
use crate::utils::{TeXError, TeXStr};
use std::rc::Rc;

#[derive(Copy,Clone,PartialEq)]
pub enum BoxMode { H,V,M,DM,Void }

#[derive(Clone)]
pub struct TeXBox {
    pub mode:BoxMode,
    pub children:Vec<Whatsit>
}

#[derive(Clone)]
pub struct MathWI {
    pub tp : TeXStr,
    pub children:Vec<Whatsit>
}

#[derive(Clone)]
pub enum Whatsit {
    Exec(Rc<ExecutableWhatsit>),
    Box(TeXBox),
    W(Rc<dyn WhatsitT>)
}

pub struct ExecutableWhatsit {
    pub _apply : Box<dyn FnOnce(&Interpreter) -> Result<(),TeXError>>
}

pub trait WhatsitT {
    fn name(&self) -> TeXStr;
}
