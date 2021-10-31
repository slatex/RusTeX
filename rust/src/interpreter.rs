pub enum Mode {
    Vertical, InternalVertical, Horizontal, RestrictedHorizontal, Math, Displaymath, Script, ScriptScript
}

use std::any::Any;
use std::borrow::Borrow;
use crate::state::{State, default_pdf_latex_state};
use crate::ontology::{CharacterToken, PrimitiveCharacterToken, PrimitiveToken, Token};
use crate::catcodes::CategoryCodeScheme;
use crate::references::SourceReference;

fn tokenize(s : &str,cats: &CategoryCodeScheme) -> Vec<Box<dyn Token>> {
    let mut ns = s.as_bytes();
    let mut retvec: Vec<Box<dyn Token>> = Vec::new();
    for next in ns {
        let b = match cats.get_code(*next) {
            cc =>
                Box::new(PrimitiveCharacterToken::new(*next,cc,SourceReference::None))
        };
        retvec.push(b)
    }
    retvec
}

pub struct Interpreter<'a> {
    state : Option<State<'a>>,
    pub mode : Mode
}


impl<'a> Interpreter<'a> {
    pub fn new() -> Interpreter<'a> {
        let mut ret = Interpreter {
            state: None,
            mode: Mode::Vertical
        };
        ret.state = Some(default_pdf_latex_state());
        ret
    }
    pub fn new_from_state(state:State<'a>) -> Interpreter<'a> {
        let ret = Interpreter {
            state:Some(state),
            mode: Mode::Vertical
        };
        ret
    }

    pub fn string_to_tokens(s : &str) -> Vec<Box<dyn Token>> {
        use std::mem;
        use crate::catcodes::OTHER_SCHEME;
        tokenize(s,&OTHER_SCHEME)
    }


    pub fn kill_state(self) -> State<'a> {
        self.state.expect("State killed already")
    }

}