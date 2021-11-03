mod primitives;
pub mod etex;

use crate::ontology::{Command, Expansion, Token};
use crate::interpreter::Interpreter;
use std::rc::Rc;
use crate::references::SourceReference;
use std::fmt;
use std::fmt::Formatter;

pub struct PrimitiveTeXCommand {
    pub apply:fn(cs:Rc<Command>,itp:&Interpreter) -> Expansion,
    pub expandable : bool,
    pub name: &'static str
}

#[derive(Clone)]
pub enum TeXCommand {
    Primitive(&'static PrimitiveTeXCommand),
    Def
}
impl fmt::Display for TeXCommand {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        match self {
            TeXCommand::Primitive(p) =>
                write!(f,"{}",p.name),
            _ => todo!("commands.rs 27")
        }
    }
}
impl TeXCommand {
    pub fn defmacro(tks : Vec<Token>,source:Rc<Token>,protected:bool) -> TeXCommand {
        todo!("commands.rs 33")
    }
}