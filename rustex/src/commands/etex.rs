use crate::commands::{PrimitiveExecutable, TeXCommand};
use crate::interpreter::Interpreter;
use crate::ontology::{Expansion, Token};
use crate::VERSION_INFO;
use std::rc::Rc;

pub static ETEXREVISION : PrimitiveExecutable = PrimitiveExecutable {
    expandable:true,
    apply: |cs: Token, _int: &mut Interpreter| {
        Ok(Expansion {
            cs: cs,
            exp:Interpreter::string_to_tokens(VERSION_INFO.etexrevision())
        })
    },
    name: "etexrevision"
};

pub fn etex_commands() -> Vec<TeXCommand<'static>> {vec![
    TeXCommand::Primitive(&ETEXREVISION)
]}