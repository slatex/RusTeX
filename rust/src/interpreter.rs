pub enum TeXMode {
    Vertical, InternalVertical, Horizontal, RestrictedHorizontal, Math, Displaymath, Script, ScriptScript
}

use std::any::Any;
use std::borrow::Borrow;
use crate::ontology::{CharacterToken, PrimitiveCharacterToken, PrimitiveToken, Token};
use crate::catcodes::CategoryCodeScheme;
use mouth::Mouth;
use crate::references::SourceReference;

pub mod mouth;
pub mod state;
mod files;

fn tokenize(s : &str,cats: &CategoryCodeScheme) -> Vec<PrimitiveCharacterToken> {
    let mut ns = s.as_bytes();
    let mut retvec: Vec<PrimitiveCharacterToken> = Vec::new();
    for next in ns {
        let b = match cats.get_code(*next) {
            cc =>
                PrimitiveCharacterToken::new(*next,cc,SourceReference::None)
        };
        retvec.push(b)
    }
    retvec
}

use crate::interpreter::state::{State,default_pdf_latex_state};
use crate::utils::{FilePath, kpsewhich};
use crate::interpreter::files::VFile;

pub struct Interpreter<'a> {
    state : Option<State<'a>>,
    pub mode : TeXMode,
    mouths: Vec<Mouth<'a>>,
    job : Option<Rc<VFile>>,
}

use std::rc::Rc;
use crate::utils::PWD;

impl<'a> Interpreter<'a> {
    pub fn new() -> Interpreter<'a> {
        let mut ret = Interpreter {
            state: Some(default_pdf_latex_state()),
            mode: TeXMode::Vertical,
            mouths:Vec::new(),
            job:None
        };
        //ret.state = Some(default_pdf_latex_state());
        ret
    }
    pub fn new_from_state(mut state:State<'a>) ->Interpreter<'a> {
        let mut ret = Interpreter {
            state:Some(state),
            mode: TeXMode::Vertical,
            mouths:Vec::new(),
            job:None
        };
        ret
    }

    pub fn string_to_tokens(s : &str) -> Vec<PrimitiveCharacterToken> {
        use std::mem;
        use crate::catcodes::OTHER_SCHEME;
        tokenize(s,&OTHER_SCHEME)
    }

    pub(in crate::interpreter) fn kill_state(&mut self) -> State<'a> {
        self.state.take().expect("State killed already")
    }

    pub fn do_file(&mut self,file:FilePath) {

    }

    fn do_v_mode(&mut self) {

    }

}
