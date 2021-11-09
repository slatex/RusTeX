use crate::commands::{DimenReference, PrimitiveExecutable, RegisterReference, TeXCommand};
use crate::interpreter::Interpreter;
use crate::ontology::{Command, Expansion, PrimitiveCharacterToken, Token, TokenI};
use crate::VERSION_INFO;
use std::rc::Rc;
use std::slice::Iter;

pub static PDFOUTPUT : TeXCommand = TeXCommand::Register(&RegisterReference {
    name: "pdfoutput",
    index:-35
});

pub static PDFPAGEHEIGHT : TeXCommand = TeXCommand::Dimen(&DimenReference {
    name: "pdfpageheight",
    index:-17
});

pub fn pdftex_commands() -> Vec<&'static TeXCommand> {vec![
    &PDFOUTPUT,&PDFPAGEHEIGHT
]}