use crate::commands::{PrimitiveExecutable, TeXCommand};
use crate::interpreter::Interpreter;
use crate::ontology::{Command, Expansion, PrimitiveCharacterToken, Token, TokenI};
use crate::VERSION_INFO;
use std::rc::Rc;
use std::slice::Iter;

pub static PAR : TeXCommand = TeXCommand::Primitive(&PrimitiveExecutable {
    expandable:false,
    name:"par",
    apply:|cs: Rc<Command>, int: &Interpreter| {
        Expansion {
            cs,
            exp: vec![]
        }
    }
});
pub static RELAX : TeXCommand = TeXCommand::Primitive(&PrimitiveExecutable {
    expandable:false,
    name:"relax",
    apply:|cs: Rc<Command>, int: &Interpreter| {
        Expansion {
            cs,
            exp: vec![]
        }
    }
});

pub fn tex_commands() -> Vec<&'static TeXCommand> {vec![
    &PAR, &RELAX
]}