use crate::commands::{PrimitiveExecutable, TeXCommand};
use crate::interpreter::Interpreter;
use crate::ontology::{Command, Expansion, Token, TokenI};
use crate::VERSION_INFO;
use std::rc::Rc;

pub static ETEXREVISION : PrimitiveExecutable = PrimitiveExecutable {
    expandable:true,
    apply: |cs: Rc<Command>, _int: &Interpreter| {
        let mut vs : Vec<Rc<Token>> = Vec::new();
        for x in Interpreter::string_to_tokens(VERSION_INFO.etexrevision()) {
            vs.push(x.as_token())
        }
        Expansion {
            cs: cs,
            exp:vs
        }
    },
    name: "etexrevision"
};

pub fn etex_commands() -> Vec<TeXCommand<'static,'static>> {vec![
    TeXCommand::Primitive(&ETEXREVISION)
]}