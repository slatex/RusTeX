use crate::commands::{PrimitiveExecutable, TeXCommand};
use crate::interpreter::Interpreter;
use crate::ontology::{Command, Expansion, PrimitiveCharacterToken, Token, TokenI};
use crate::VERSION_INFO;
use std::rc::Rc;
use std::slice::Iter;

pub static PAR : PrimitiveExecutable = PrimitiveExecutable {
    expandable:false,
    name:"par",
    apply:|cs: Rc<Command>, int: &Interpreter| {
        Expansion {
            cs,
            exp: vec![]
        }
    }
};
pub static RELAX : PrimitiveExecutable = PrimitiveExecutable {
    expandable:false,
    name:"relax",
    apply:|cs: Rc<Command>, int: &Interpreter| {
        Expansion {
            cs,
            exp: vec![]
        }
    }
};

pub fn tex_commands() -> Vec<TeXCommand<'static,'static>> {vec![
    TeXCommand::Primitive(&PAR), TeXCommand::Primitive(&RELAX)
]}