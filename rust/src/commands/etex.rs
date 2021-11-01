use crate::commands::{Expandable,TeXCommand,Executable};
use crate::interpreter::Interpreter;
use crate::ontology::{Command, Expansion, PrimitiveCharacterToken, Token, TokenI};
use crate::VERSION_INFO;
use std::rc::Rc;

pub static ETEREVISION : Expandable = Expandable {
    apply: |cs: Rc<Command>, int: &Interpreter| {
        let mut vs : Vec<Rc<Token>> = Vec::new();
        for x in Interpreter::string_to_tokens(VERSION_INFO.etexrevision()) {
            vs.push(x.as_token())
        }
        Expansion {
            cs: cs,
            exp:vs
        }
    }
};

pub fn etex_commands() -> Vec<(&'static str,&'static dyn TeXCommand)> {vec![
    ("etexrevision",&ETEREVISION)
]}