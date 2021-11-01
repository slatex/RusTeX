mod primitives;
mod etex;

use crate::ontology::{Command, Expansion};
use crate::interpreter::Interpreter;
use std::rc::Rc;

pub trait TeXCommand {}
pub struct Expandable {
    pub apply:fn(cs:Rc<Command>,itp:&Interpreter) -> Expansion
}
impl TeXCommand for Expandable {}
pub struct Executable {
    pub apply:fn(cs:Rc<Command>,itp:&Interpreter)
}
impl TeXCommand for Executable {}