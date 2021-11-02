use std::any::Any;
use std::collections::HashMap;
use std::rc::Rc;
use std::sync::Mutex;
use crate::catcodes::{CategoryCodeScheme,STARTING_SCHEME};
use crate::commands::TeXCommand;

#[derive(Clone)]
struct StackFrame<'a> {
    parent: Option<&'a StackFrame<'a>>,
    pub(in crate::state) catcodes: CategoryCodeScheme,
    pub(in crate::state) newlinechar: u8,
    pub(in crate::state) endlinechar: u8,
    pub(in crate::state) commands: HashMap<&'a str,Rc<TeXCommand>>
}

impl StackFrame<'_> {
    pub(in crate::state) fn initial_pdf_etex<'a>() -> StackFrame<'a> {
        use crate::commands::etex::etex_commands;
        let mut cmds: HashMap<&str,Rc<TeXCommand>> = HashMap::new();
        for (n,c) in etex_commands() {
            cmds.insert(n,c);
        }
        StackFrame {
            parent: None,
            catcodes: STARTING_SCHEME.clone(),
            commands: cmds,
            newlinechar: 10,
            endlinechar:13
        }
    }
    pub(in crate::state) fn new<'a>(parent: &'a StackFrame<'a>) -> StackFrame<'a> {
        StackFrame {
            parent: Some(parent),
            catcodes: parent.catcodes.clone(),
            commands: HashMap::new(),
            newlinechar: parent.newlinechar,
            endlinechar: parent.newlinechar
        }
    }
    pub(in crate::state) fn get_command(&self,name:&str) -> Option<Rc<TeXCommand>> {
        match self.commands.get(name) {
            Some(cmd) => Some(Rc::clone(cmd)),
            None => match self.parent {
                Some(p) => p.get_command(name),
                None => None
            }
        }
    }
}

#[derive(Clone)]
pub struct State<'a> {
    stacks: Vec<Box<StackFrame<'a>>>
}

impl State<'_> {
    pub(in crate::state) fn new<'a>() -> State<'a> {
        State {
            stacks: vec![Box::new(StackFrame::initial_pdf_etex())]
        }
    }
    pub fn get_command(&self, name: &str) -> Option<Rc<TeXCommand>> {
        self.stacks.last().unwrap().get_command(name)
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

pub fn default_pdf_latex_state<'a>() -> State<'a> {
    use std::env;
    use crate::utils::{kpsewhich,FilePath};
    use crate::interpreter::TeXMode;

    let maindir = FilePath::from_path(env::current_dir().expect("No current directory!"));
    //let mut st = State::new();
    let mut interpreter = Interpreter::new_from_state(State::new());
    let pdftex_cfg = kpsewhich("pdftexconfig.tex",&maindir).expect("pdftexconfig.tex not found");
    let latex_ltx = kpsewhich("latex.ltx",&maindir).expect("No latex.ltx found");

    // TODO
    println!("{}",pdftex_cfg.path());
    println!("{}",latex_ltx.path());

    interpreter.kill_state()
}