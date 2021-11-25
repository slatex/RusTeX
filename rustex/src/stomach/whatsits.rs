use crate::interpreter::Interpreter;
use crate::utils::TeXError;
use std::rc::Rc;

#[derive(Copy,Clone,PartialEq)]
pub enum BoxMode { H,V,M,DM,Void }

#[derive(Clone,PartialEq)]
pub struct TeXBox {
    pub mode:BoxMode,
    pub children:Vec<Whatsit>
}

#[derive(Clone,PartialEq)]
pub enum Whatsit {
    Exec(Rc<ExecutableWhatsit>),
    Box(TeXBox)
}

pub struct ExecutableWhatsit {
    pub _apply : Box<dyn FnOnce(&Interpreter) -> Result<(),TeXError>>
}
impl PartialEq for ExecutableWhatsit {
    fn eq(&self, other: &Self) -> bool { false }
}