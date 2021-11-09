use std::any::Any;
use std::borrow::BorrowMut;
use std::collections::HashMap;
use std::rc::Rc;
use std::sync::Mutex;
use crate::catcodes::{CategoryCodeScheme,STARTING_SCHEME,DEFAULT_SCHEME};
use crate::commands::{RegisterReference, TeXCommand};
use crate::interpreter::files::VFile;
use crate::interpreter::Interpreter;
use crate::utils::{kpsewhich,PWD};

#[derive(Clone)]
struct StackFrame<'a> {
    parent: Option<&'a StackFrame<'a>>,
    pub(crate) catcodes: CategoryCodeScheme,
    pub(crate) newlinechar: u8,
    pub(crate) endlinechar: u8,
    pub(crate) commands: HashMap<&'a str,Option<&'a TeXCommand>>,
    pub(crate) registers: HashMap<i8,Option<i32>>,
    pub(crate) dimensions: HashMap<i8,Option<i32>>
}

impl StackFrame<'_> {
    pub(crate) fn initial_pdf_etex<'a>() -> StackFrame<'a> {
        use crate::commands::etex::etex_commands;
        use crate::commands::primitives::tex_commands;
        use crate::commands::pdftex::pdftex_commands;
        let mut cmds: HashMap<&str,Option<&TeXCommand>> = HashMap::new();
        for c in tex_commands() {
            cmds.insert(c.name(),Some(c));
        }
        for c in etex_commands() {
            cmds.insert(c.name(),Some(c));
        }
        for c in pdftex_commands() {
            cmds.insert(c.name(),Some(c));
        }
        let mut reg: HashMap<i8,Option<i32>> = HashMap::new();
        let mut dims: HashMap<i8,Option<i32>> = HashMap::new();
        StackFrame {
            parent: None,
            catcodes: STARTING_SCHEME.clone(),
            commands: cmds,
            newlinechar: 10,
            endlinechar:13,
            registers:reg,
            dimensions:dims
        }
    }
    pub(crate) fn new<'a>(parent: &'a StackFrame<'a>) -> StackFrame<'a> {
        let reg: HashMap<i8,Option<i32>> = HashMap::new();
        let dims: HashMap<i8,Option<i32>> = HashMap::new();
        StackFrame {
            parent: Some(parent),
            catcodes: parent.catcodes.clone(),
            commands: HashMap::new(),
            newlinechar: parent.newlinechar,
            endlinechar: parent.newlinechar,
            registers:reg,
            dimensions:dims
        }
    }
    pub(crate) fn get_command(&self, name:&str) -> Option<&TeXCommand> {
        match self.commands.get(name) {
            Some(r) => *r,
            None => match self.parent {
                Some(p) => p.get_command(name),
                None => None
            }
        }
    }
    pub(crate) fn get_register(&self,index:i8) -> Option<i32> {
        match self.registers.get(&index) {
            Some(r) => *r,
            None => match self.parent {
                Some(p) => p.get_register(index),
                None => None
            }
        }
    }
    pub(crate) fn get_dimension(&self,index:i8) -> Option<i32> {
        match self.dimensions.get(&index) {
            Some(r) => *r,
            None => match self.parent {
                Some(p) => p.get_dimension(index),
                None => None
            }
        }
    }
}

// ------------------------------------------------------------------------------------------------

#[derive(Clone)]
pub struct State<'a> {
    stacks: Vec<Box<StackFrame<'a>>>,
}

impl State<'_> {
    pub fn new<'a>() -> State<'a> {
        State {
            stacks: vec![Box::new(StackFrame::initial_pdf_etex())]
        }
    }
    pub fn get_command(&self, name: &str) -> Option<&TeXCommand> {
        self.stacks.last().unwrap().get_command(name)
    }
    pub fn get_register(&self, index:i8) -> i32 {
        match self.stacks.last().unwrap().get_register(index) {
            Some(i) => i,
            None => 0
        }
    }
    pub fn get_dimension(&self, index:i8) -> i32 {
        match self.stacks.last().unwrap().get_dimension(index) {
            Some(i) => i,
            None => 0
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
    pub fn change(&mut self,change:StateChange) {
        match change {
            StateChange::Register(regch) => {
                if regch.global {
                    self.stacks.iter_mut().map(|s| s.registers.insert(regch.index,Some(regch.value)));
                } else {
                    self.stacks.last_mut().unwrap().registers.insert(regch.index,Some(regch.value));
                }
            }
            _ => todo!()
        }
    }
}


pub fn default_pdf_latex_state<'a>() -> State<'a> {
    let mut st = State::new();
    let pdftex_cfg = kpsewhich("pdftexconfig.tex",&PWD).expect("pdftexconfig.tex not found");
    let latex_ltx = kpsewhich("latex.ltx",&PWD).expect("No latex.ltx found");

    println!("{}",pdftex_cfg.to_str().expect("wut"));
    println!("{}",latex_ltx.to_str().expect("wut"));
    st = Interpreter::do_file_with_state(&pdftex_cfg,st);
    st = Interpreter::do_file_with_state(&latex_ltx,st);
    st
    /*

    let mut interpreter = Interpreter::new_from_state(st);
    {interpreter.do_file(pdftex_cfg.as_path());}
    {interpreter.do_file(latex_ltx.as_path());}
    interpreter.kill_state()

 */
}

pub struct RegisterStateChange {
    pub index:i8,
    pub value:i32,
    pub global:bool
}

pub enum StateChange {
    Register(RegisterStateChange)
}