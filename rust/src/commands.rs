mod primitives;
pub mod etex;

use crate::ontology::{Command, Expansion};
use crate::interpreter::Interpreter;
use std::rc::Rc;

pub struct TeXCommand {
    pub apply:fn(cs:Rc<Command>,itp:&Interpreter) -> Expansion,
    pub expandable : bool,
    pub protected: bool
}