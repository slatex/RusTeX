use crate::commands::{DimenReference, PrimitiveExecutable, RegisterReference, TeXCommand};
use crate::interpreter::Interpreter;
use crate::ontology::{Command, Expansion, PrimitiveCharacterToken, Token, TokenI};
use crate::VERSION_INFO;
use std::rc::Rc;
use std::slice::Iter;

pub static PDFOUTPUT : RegisterReference = RegisterReference {
    name: "pdfoutput",
    index:-35
};

pub static PDFPAGEHEIGHT : DimenReference = DimenReference {
    name: "pdfpageheight",
    index:-17
};

pub fn pdftex_commands() -> Vec<TeXCommand<'static,'static>> {vec![
    TeXCommand::Register(&PDFOUTPUT),TeXCommand::Dimen(&PDFPAGEHEIGHT)
]}