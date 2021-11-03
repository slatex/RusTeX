use crate::commands::{PrimitiveTeXCommand, TeXCommand};
use crate::interpreter::Interpreter;
use crate::ontology::{Command, Expansion, PrimitiveCharacterToken, Token, TokenI};
use crate::VERSION_INFO;
use std::rc::Rc;
use std::slice::Iter;

pub static ETEXREVISION : PrimitiveTeXCommand = PrimitiveTeXCommand {
    expandable:true,
    apply: |cs: Rc<Command>, int: &Interpreter| {
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

pub fn etex_commands() -> Vec<&'static PrimitiveTeXCommand> {vec![
    &ETEXREVISION
]}