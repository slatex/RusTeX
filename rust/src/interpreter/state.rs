use std::any::Any;
use std::collections::HashMap;
use std::rc::Rc;
use std::sync::Mutex;
use crate::catcodes::{CategoryCodeScheme,STARTING_SCHEME};
use crate::commands::TeXCommand;
use crate::interpreter::files::VFile;

#[derive(Clone)]
struct StackFrame<'a> {
    parent: Option<&'a StackFrame<'a>>,
    pub(crate) catcodes: CategoryCodeScheme,
    pub(crate) newlinechar: u8,
    pub(crate) endlinechar: u8,
    pub(crate) commands: HashMap<&'a str,TeXCommand>,
}

impl StackFrame<'_> {
    pub(crate) fn initial_pdf_etex<'a>() -> StackFrame<'a> {
        use crate::commands::etex::etex_commands;
        let mut cmds: HashMap<&str,TeXCommand> = HashMap::new();
        for c in etex_commands() {
            cmds.insert(c.name,TeXCommand::Primitive(c));
        }
        StackFrame {
            parent: None,
            catcodes: STARTING_SCHEME.clone(),
            commands: cmds,
            newlinechar: 10,
            endlinechar:13,
        }
    }
    pub(crate) fn new<'a>(parent: &'a StackFrame<'a>) -> StackFrame<'a> {
        StackFrame {
            parent: Some(parent),
            catcodes: parent.catcodes.clone(),
            commands: HashMap::new(),
            newlinechar: parent.newlinechar,
            endlinechar: parent.newlinechar
        }
    }
    pub(crate) fn get_command(&self, name:&str) -> Option<&TeXCommand> {
        match self.commands.get(name) {
            Some(cmd) => Some(cmd),
            None => match self.parent {
                Some(p) => p.get_command(name),
                None => None
            }
        }
    }
}

// ------------------------------------------------------------------------------------------------

#[derive(Clone)]
pub struct State<'a> {
    stacks: Vec<Box<StackFrame<'a>>>,
    pub(in crate::interpreter) files: HashMap<String,VFile>
}

impl State<'_> {
    pub(crate) fn new<'a>() -> State<'a> {
        State {
            stacks: vec![Box::new(StackFrame::initial_pdf_etex())],
            files:HashMap::new()
        }
    }
    pub fn get_command(&self, name: &str) -> Option<&TeXCommand> {
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
    use crate::utils::kpsewhich;
    use crate::interpreter::TeXMode;
    use crate::utils::PWD;

    let mut st = State::new();
    let mut interpreter = Interpreter::new_from_state(st);
    let pdftex_cfg = kpsewhich("pdftexconfig.tex",&PWD).expect("pdftexconfig.tex not found");
    let latex_ltx = kpsewhich("latex.ltx",&PWD).expect("No latex.ltx found");

    println!("{}",pdftex_cfg.to_str().expect("wut"));
    println!("{}",latex_ltx.to_str().expect("wut"));
    interpreter.do_file(pdftex_cfg.as_path());
    interpreter.do_file(latex_ltx.as_path());
    interpreter.kill_state()
}