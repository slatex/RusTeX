mod primitives;
pub mod etex;

use crate::ontology::{Command, Expansion, Token};
use crate::interpreter::Interpreter;
use std::rc::Rc;
use crate::references::SourceReference;

pub fn primitive_meaning<'a>(itp_opt:&'a Option<Interpreter>,name:&str) -> &'a str {
    todo!()
}

pub struct PrimitiveTeXCommand {
    pub apply:fn(cs:Rc<Command>,itp:&Interpreter) -> Expansion,
    pub expandable : bool,
}
pub enum TeXCommand {
    Primitive(&'static PrimitiveTeXCommand),
    Def
}
impl TeXCommand {
    pub fn defmacro(tks : Vec<Token>,source:Rc<Token>,protected:bool) -> TeXCommand {
        todo!()
    }
}