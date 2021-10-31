mod primitives;
mod etex;

use crate::ontology::{ControlSequence, Expansion};
use crate::interpreter::Interpreter;

pub trait TeXCommand {}
pub struct Expandable {
    pub apply:fn(cs:ControlSequence,itp:Interpreter) -> Expansion
}
impl TeXCommand for Expandable {}
pub struct Executable {
    pub apply:fn(cs:ControlSequence,itp:Interpreter)
}
impl TeXCommand for Executable {}