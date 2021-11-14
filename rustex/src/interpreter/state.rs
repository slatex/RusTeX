use std::collections::HashMap;
use std::rc::Rc;
use crate::catcodes::{CategoryCode, CategoryCodeScheme, STARTING_SCHEME};
use crate::commands::TeXCommand;
use crate::interpreter::Interpreter;
use crate::utils::{kpsewhich,PWD};

#[derive(Clone)]
struct StackFrame<'a> {
    parent: Option<&'a StackFrame<'a>>,
    pub(crate) catcodes: CategoryCodeScheme,
    pub(crate) newlinechar: u8,
    pub(crate) endlinechar: u8,
    pub(crate) commands: HashMap<String,Option<TeXCommand>>,
    pub(crate) registers: HashMap<i8,Option<i32>>,
    pub(crate) dimensions: HashMap<i8,Option<i32>>
}

impl<'sf> StackFrame<'sf> {
    pub(crate) fn initial_pdf_etex<'a>() -> StackFrame<'a> {
        use crate::commands::conditionals::conditional_commands;
        use crate::commands::etex::etex_commands;
        use crate::commands::primitives::tex_commands;
        use crate::commands::pdftex::pdftex_commands;
        let mut cmds: HashMap<String,Option<TeXCommand>> = HashMap::new();
        for c in conditional_commands() {
            cmds.insert(c.name(),Some(c));
        }
        for c in tex_commands() {
            cmds.insert(c.name(),Some(c));
        }
        for c in etex_commands() {
            cmds.insert(c.name(),Some(c));
        }
        for c in pdftex_commands() {
            cmds.insert(c.name(),Some(c));
        }
        let reg: HashMap<i8,Option<i32>> = HashMap::new();
        let dims: HashMap<i8,Option<i32>> = HashMap::new();
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
    pub(crate) fn get_command(&self, name:&str) -> Option<TeXCommand> {
        match self.commands.get(name) {
            Some(Some(r)) => Some(r.clone()),
            Some(None) => None,
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
    pub(in crate) conditions:Vec<Option<bool>>
}

impl<'s> State<'s> {
    pub fn new<'a>() -> State<'a> {
        State {
            stacks: vec![Box::new(StackFrame::initial_pdf_etex())],
            conditions: vec![]
        }
    }
    pub fn with_commands<'a>(mut procs:Vec<TeXCommand>) -> State<'a> {
        let mut st = State::new();
        while !procs.is_empty() {
            let p = procs.pop().unwrap();
            let name = p.name();
            st.stacks.last_mut().unwrap().commands.insert(name,Some(p));
        }
        st
    }
    pub fn get_command(&self, name: &str) -> Option<TeXCommand> {
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
    pub fn change(&mut self,int:&Interpreter,change:StateChange) {
        match change {
            StateChange::Register(regch) => {
                if regch.global {
                    for s in self.stacks.iter_mut() {
                        s.registers.insert(regch.index,Some(regch.value));
                    }
                } else {
                    self.stacks.last_mut().unwrap().registers.insert(regch.index,Some(regch.value));
                }
            }
            StateChange::Dimen(regch) => {
                if regch.global {
                    for s in self.stacks.iter_mut() {
                        s.dimensions.insert(regch.index,Some(regch.value));
                    }
                } else {
                    self.stacks.last_mut().unwrap().dimensions.insert(regch.index,Some(regch.value));
                }
            }
            StateChange::Cs(cmd) => {
                if cmd.global {
                    for s in self.stacks.iter_mut() {
                        s.commands.remove(&*cmd.name);
                    }
                    self.stacks.first_mut().unwrap().commands.insert(cmd.name.to_string(),cmd.cmd);
                } else {
                    self.stacks.last_mut().unwrap().commands.insert(cmd.name.to_string(),cmd.cmd);
                }
            }
            StateChange::Cat(cc) => {
                int.catcodes.borrow_mut().catcodes.insert(cc.char,cc.catcode);
                if cc.global {
                    for s in self.stacks.iter_mut() {
                        s.catcodes.catcodes.insert(cc.char,cc.catcode);
                    }
                }
            }
            StateChange::Newline(nl) => {
                int.catcodes.borrow_mut().newlinechar = nl.char;
                if nl.global {
                    for s in self.stacks.iter_mut() {
                        s.catcodes.newlinechar = nl.char;
                    }
                }
            }
            //_ => todo!()
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

use std::cell::Ref;
impl<'s> Interpreter<'s,'_> {
    pub fn change_state(&self,change:StateChange) {
        let mut state = self.state.borrow_mut();
        state.change(self,change)
    }

    pub fn state_catcodes(&self) -> Ref<'_,CategoryCodeScheme> {
        self.catcodes.borrow()
    }
    pub fn state_register(&self,i:i8) -> i32 {
        self.state.borrow().get_register(i)
    }
    pub fn state_dimension(&self,i:i8) -> i32 {
        self.state.borrow().get_dimension(i)
    }

    pub fn pushcondition<'a>(&self) -> u8 {
        let mut state = self.state.borrow_mut();
        state.conditions.push(None);
        (state.conditions.len() - 1) as u8
    }
    pub fn setcondition(&self,c : u8,val : bool) -> u8 {
        let mut conds = &mut self.state.borrow_mut().conditions;
        conds.remove(c as usize);
        conds.insert(c as usize,Some(val));
        (conds.len() as u8) - 1 - c
    }
    pub fn popcondition(&self) {
        self.state.borrow_mut().conditions.pop();
    }
    pub fn getcondition(&self) -> Option<(u8,Option<bool>)> {
        let conds = &self.state.borrow().conditions;
        match conds.last() {
            Some(p) => Some((conds.len() as u8,*p)),
            None => None
        }
    }
    pub fn state_get_command(&self,s:&str) -> Option<TeXCommand> {
        self.state.borrow().get_command(s)
    }
}

pub struct RegisterStateChange {
    pub index:i8,
    pub value:i32,
    pub global:bool
}

pub struct CommandChange {
    pub name:String,
    pub cmd:Option<TeXCommand>,
    pub global:bool
}

pub struct CategoryCodeChange {
    pub char:u8,
    pub catcode:CategoryCode,
    pub global:bool
}

pub struct NewlineChange {
    pub char:u8,
    pub global:bool
}

pub enum StateChange {
    Register(RegisterStateChange),
    Dimen(RegisterStateChange),
    Cs(CommandChange),
    Cat(CategoryCodeChange),
    Newline(NewlineChange)
}