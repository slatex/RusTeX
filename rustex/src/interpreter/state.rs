use std::collections::HashMap;
use std::rc::Rc;
use crate::catcodes::{CategoryCode, CategoryCodeScheme, STARTING_SCHEME};
use crate::commands::TeXCommand;
use crate::interpreter::Interpreter;
use crate::utils::{kpsewhich,PWD};

#[derive(Clone)]
pub enum GroupType {
    Token,
    Begingroup
}

#[derive(Clone)]
struct StackFrame {
    //parent: Option<&'a StackFrame<'a>>,
    pub(crate) catcodes: CategoryCodeScheme,
    pub(crate) newlinechar: u8,
    pub(crate) endlinechar: u8,
    pub(crate) commands: HashMap<String,Option<TeXCommand>>,
    pub(crate) registers: HashMap<i16,i32>,
    pub(crate) dimensions: HashMap<i16,i32>,
    pub(in crate::interpreter::state) tp : Option<GroupType>
}

impl StackFrame {
    pub(crate) fn initial_pdf_etex() -> StackFrame {
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
        let reg: HashMap<i16,i32> = HashMap::new();
        let dims: HashMap<i16,i32> = HashMap::new();
        StackFrame {
            //parent: None,
            catcodes: STARTING_SCHEME.clone(),
            commands: cmds,
            newlinechar: 10,
            endlinechar:13,
            registers:reg,
            dimensions:dims,
            tp:None
        }
    }
    pub(crate) fn new(parent: &StackFrame,tp : GroupType) -> StackFrame {
        let reg: HashMap<i16,i32> = HashMap::new();
        let dims: HashMap<i16,i32> = HashMap::new();
        StackFrame {
            //parent: Some(parent),
            catcodes: parent.catcodes.clone(),
            commands: HashMap::new(),
            newlinechar: parent.newlinechar,
            endlinechar: parent.newlinechar,
            registers:reg,
            dimensions:dims,
            tp:Some(tp)
        }
    }
}

// ------------------------------------------------------------------------------------------------

#[derive(Clone)]
pub struct State {
    stacks: Vec<StackFrame>,
    pub(in crate) conditions:Vec<Option<bool>>
}

impl State {
    pub fn new() -> State {
        State {
            stacks: vec![StackFrame::initial_pdf_etex()],
            conditions: vec![]
        }
    }
    pub fn with_commands(mut procs:Vec<TeXCommand>) -> State {
        let mut st = State::new();
        while !procs.is_empty() {
            let p = procs.pop().unwrap();
            let name = p.name();
            st.stacks.last_mut().unwrap().commands.insert(name,Some(p));
        }
        st
    }

    pub fn get_command(&self, name: &str) -> Option<TeXCommand> {
        for sf in self.stacks.iter().rev() {
            match sf.commands.get(name) {
                Some(r) => return r.clone(),
                _ => {}
            }
        }
        None
    }
    pub fn get_register(&self, index:i16) -> i32 {
        for sf in self.stacks.iter().rev() {
            match sf.registers.get(&index) {
                Some(r) => return *r,
                _ => {}
            }
        }
        0
    }
    pub fn get_dimension(&self, index:i16) -> i32 {
        for sf in self.stacks.iter().rev() {
            match sf.dimensions.get(&index) {
                Some(r) => return *r,
                _ => {}
            }
        }
        0
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
                        s.registers.insert(regch.index,regch.value);
                    }
                } else {
                    self.stacks.last_mut().unwrap().registers.insert(regch.index,regch.value);
                }
            }
            StateChange::Dimen(regch) => {
                if regch.global {
                    for s in self.stacks.iter_mut() {
                        s.dimensions.insert(regch.index,regch.value);
                    }
                } else {
                    self.stacks.last_mut().unwrap().dimensions.insert(regch.index,regch.value);
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

    pub (in crate::interpreter::state) fn push(&mut self,cc:CategoryCodeScheme,tp : GroupType) {
        let mut laststack = self.stacks.last_mut().unwrap();
        laststack.catcodes = cc;
        let sf = StackFrame::new(self.stacks.last().unwrap(),tp);
        self.stacks.push(sf)
    }
}


pub fn default_pdf_latex_state() -> State {
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
use std::ops::Deref;

impl Interpreter<'_> {
    pub fn change_state(&self,change:StateChange) {
        let mut state = self.state.borrow_mut();
        state.change(self,change)
    }
    pub fn new_group(&self,tp:GroupType) {
        self.state.borrow_mut().push(self.catcodes.borrow().clone(),tp)
    }

    pub fn state_catcodes(&self) -> Ref<'_,CategoryCodeScheme> {
        self.catcodes.borrow()
    }
    pub fn state_register(&self,i:i16) -> i32 { self.state.borrow().get_register(i) }
    pub fn state_dimension(&self,i:i16) -> i32 {
        self.state.borrow().get_dimension(i)
    }

    pub fn pushcondition(&self) -> u8 {
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
    pub index:i16,
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