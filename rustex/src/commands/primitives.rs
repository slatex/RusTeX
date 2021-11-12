use crate::commands::{PrimitiveExecutable, TeXCommand};
use crate::interpreter::Interpreter;
use crate::ontology::{Token, Expansion};
use std::rc::Rc;

pub static PAR : PrimitiveExecutable = PrimitiveExecutable {
    expandable:false,
    name:"par",
    apply:|cs: Token, _int: &mut Interpreter| {
        Ok(Expansion {
            cs,
            exp: vec![]
        })
    }
};
pub static RELAX : PrimitiveExecutable = PrimitiveExecutable {
    expandable:false,
    name:"relax",
    apply:|cs: Token, _int: &mut Interpreter| {
        Ok(Expansion {
            cs,
            exp: vec![]
        })
    }
};

pub fn tex_commands() -> Vec<TeXCommand<'static>> {vec![
    TeXCommand::Primitive(&PAR), TeXCommand::Primitive(&RELAX)
]}