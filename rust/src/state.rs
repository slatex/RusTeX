use crate::catcodes::{CategoryCodeScheme,STARTING_SCHEME};
#[derive(Clone)]
struct StackFrame<'a> {
    parent : Option<&'a StackFrame<'a>>,
    pub catcodes:CategoryCodeScheme,
    pub newlinechar: u8,
    pub endlinechar: u8
}
impl StackFrame<'_> {
    pub fn new<'a>(parent: Option<&'a StackFrame>) -> StackFrame<'a> {
        StackFrame {
            parent:parent,
            catcodes:match parent {
                Some(p) => p.catcodes.clone(),
                None => STARTING_SCHEME.clone()
            },
            newlinechar:match parent {
                Some(p) => p.newlinechar,
                None => 10
            },
            endlinechar:match parent {
                Some(p) => p.newlinechar,
                None => 13
            },
        }
    }
}
#[derive(Clone)]
pub struct State<'a> {
    stacks : Vec<Box<StackFrame<'a>>>
}
impl State<'_> {
    pub fn dummy(&self) {}
    pub fn new<'a>() -> State<'a> {
        State {
            stacks:vec![Box::new(StackFrame::new(None))]
        }
    }
    pub fn catcodes(&self) -> &CategoryCodeScheme {
        &self.stacks.last().expect("Stack frames empty").catcodes
    }
    pub fn endlinechar(&self) -> u8 {
        self.stacks.last().expect("Stack frames empty").endlinechar
    }
    pub fn newlinechar(&self) -> u8 {
        self.stacks.last().expect("Stack frames empty").newlinechar
    }
}

use crate::interpreter::Interpreter;

lazy_static! {
    static ref DEFAULT_PDF_LATEX_STATE : State<'static> = {
        use std::env;
        use crate::utils::{kpsewhich,FilePath};
        use crate::interpreter::Mode;

        let maindir = FilePath::from_path(env::current_dir().expect("No current directory!"));
        //let mut st = State::new();
        let mut interpreter = Interpreter::new_from_state(State::new());
        let pdftex_cfg = kpsewhich("pdftexconfig.tex",&maindir).expect("pdftexconfig.tex not found");
        let latex_ltx = kpsewhich("latex.ltx",&maindir).expect("No latex.ltx found");

        // TODO
        println!("{}",pdftex_cfg.path());
        println!("{}",latex_ltx.path());

        interpreter.kill_state()
    };
}
pub fn default_pdf_latex_state<'a>() -> State<'a> {
    DEFAULT_PDF_LATEX_STATE.clone()
}