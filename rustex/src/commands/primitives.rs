use crate::commands::{PrimitiveExecutable, TeXCommand};
use crate::interpreter::Interpreter;
use crate::ontology::{Command, Expansion};
use std::rc::Rc;

pub static PAR : PrimitiveExecutable = PrimitiveExecutable {
    expandable:false,
    name:"par",
    apply:|cs: Rc<Command>, _int: &Interpreter| {
        Expansion {
            cs,
            exp: vec![]
        }
    }
};
pub static RELAX : PrimitiveExecutable = PrimitiveExecutable {
    expandable:false,
    name:"relax",
    apply:|cs: Rc<Command>, _int: &Interpreter| {
        Expansion {
            cs,
            exp: vec![]
        }
    }
};

pub fn tex_commands() -> Vec<TeXCommand<'static>> {vec![
    TeXCommand::Primitive(&PAR), TeXCommand::Primitive(&RELAX)
]}