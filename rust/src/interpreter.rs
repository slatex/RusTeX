pub enum TeXMode {
    Vertical, InternalVertical, Horizontal, RestrictedHorizontal, Math, Displaymath, Script, ScriptScript
}

use std::any::Any;
use std::borrow::{Borrow, BorrowMut};
use crate::ontology::{CharacterToken, LaTeXFile, PrimitiveCharacterToken, PrimitiveToken, Token};
use crate::catcodes::CategoryCodeScheme;
use mouth::Mouth;
use crate::references::SourceReference;
use std::path::{Path, PathBuf};

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
use crate::utils::kpsewhich;
use crate::interpreter::files::VFile;

pub struct Interpreter<'a> {
    state : Option<&'a State<'a>>,
    pub mode : TeXMode,
    mouths: Vec<Mouth<'a>>,
    job : Option<PathBuf>,
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

    pub fn jobname(&self) -> &str {
        let job = self.job.as_ref().expect("Interpreter without running job has no jobname");
        job.file_stem().unwrap().to_str().unwrap()
    }
    fn in_file(&self) -> &Path {
        self.job.as_ref().expect("Interpreter without running job has no jobname").parent().unwrap()
    }

    pub fn kpsewhich(&self,filename:&str) -> PathBuf {
        match kpsewhich(filename,self.in_file()) {
            None => PathBuf::from(self.in_file().to_str().unwrap().to_owned() + "/" + filename).canonicalize().unwrap(),
            Some(fp) => fp
        }
    }

    pub fn do_file(&'a mut self, file:&Path) {
        if !file.exists() {
            return ()//Result::Err("File does not exist")
        }
        self.job = Some(file.canonicalize().expect("File name not canonicalizable").to_path_buf());
        //let vf = self.borrow_mut().getvf(file);
        let mut vf = VFile::new(file,self);
        self.push_file(vf);
        while self.has_next() {
            self.do_v_mode()
        }
        todo!("interpreter.rs 101")
    }

    fn do_v_mode(&mut self) {
        todo!("interpreter.rs 105")
    }

}
