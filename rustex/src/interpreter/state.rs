use std::borrow::BorrowMut;
use std::collections::HashMap;
use crate::catcodes::{CategoryCode, CategoryCodeScheme, STARTING_SCHEME};
use crate::commands::TeXCommand;
use crate::interpreter::Interpreter;
use crate::utils::{kpsewhich, PWD, TeXError};
use crate::{TeXErr,log};

#[derive(Copy,Clone)]
pub enum GroupType {
    Token,
    Begingroup
}
impl Display for GroupType {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(f,"{}",match self {
            GroupType::Token => "{",
            GroupType::Begingroup => "\\begingroup"
        })
    }
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
    pub(crate) skips : HashMap<i16,Skip>,
    pub(crate) toks : HashMap<i16,Vec<Token>>,
    pub(in crate::interpreter::state) tp : Option<GroupType>,
    pub(crate) sfcodes : HashMap<u8,i32>
}

impl StackFrame {
    pub(crate) fn initial_pdf_etex() -> StackFrame {
        use crate::commands::conditionals::conditional_commands;
        use crate::commands::etex::etex_commands;
        use crate::commands::primitives::tex_commands;
        use crate::commands::pdftex::pdftex_commands;
        let mut cmds: HashMap<String,Option<TeXCommand>> = HashMap::new();
        for c in conditional_commands() {
            cmds.insert(c.name().unwrap().to_string(),Some(c));
        }
        for c in tex_commands() {
            cmds.insert(c.name().unwrap().to_string(),Some(c));
        }
        for c in etex_commands() {
            cmds.insert(c.name().unwrap().to_string(),Some(c));
        }
        for c in pdftex_commands() {
            cmds.insert(c.name().unwrap().to_string(),Some(c));
        }
        let mut reg: HashMap<i16,i32> = HashMap::new();
        reg.insert(-crate::utils::u8toi16(crate::commands::primitives::MAG.index),1000);

        let dims: HashMap<i16,i32> = HashMap::new();
        let skips: HashMap<i16,Skip> = HashMap::new();
        let toks: HashMap<i16,Vec<Token>> = HashMap::new();
        let sfcodes: HashMap<u8,i32> = HashMap::new();
        StackFrame {
            //parent: None,
            catcodes: STARTING_SCHEME.clone(),
            commands: cmds,
            newlinechar: 10,
            endlinechar:13,
            registers:reg,
            dimensions:dims,
            skips,toks,sfcodes,
            tp:None
        }
    }
    pub(crate) fn new(parent: &StackFrame,tp : GroupType) -> StackFrame {
        let reg: HashMap<i16,i32> = HashMap::new();
        let dims: HashMap<i16,i32> = HashMap::new();
        let skips: HashMap<i16,Skip> = HashMap::new();
        let toks: HashMap<i16,Vec<Token>> = HashMap::new();
        let sfcodes: HashMap<u8,i32> = HashMap::new();
        StackFrame {
            //parent: Some(parent),
            catcodes: parent.catcodes.clone(),
            commands: HashMap::new(),
            newlinechar: parent.newlinechar,
            endlinechar: parent.newlinechar,
            registers:reg,
            dimensions:dims,
            skips,toks,sfcodes,
            tp:Some(tp)
        }
    }
}

// ------------------------------------------------------------------------------------------------

#[derive(Clone)]
pub struct State {
    stacks: Vec<StackFrame>,
    pub(in crate) conditions:Vec<Option<bool>>,
    pub(in crate) outfiles:HashMap<u8,VFile>,
    pub(in crate) infiles:HashMap<u8,StringMouth>
}

impl State {
    pub fn new() -> State {
        State {
            stacks: vec![StackFrame::initial_pdf_etex()],
            conditions: vec![],
            outfiles:HashMap::new(),
            infiles:HashMap::new()
        }
    }
    pub fn with_commands(mut procs:Vec<TeXCommand>) -> State {
        let mut st = State::new();
        while !procs.is_empty() {
            let p = procs.pop().unwrap();
            let name = p.name().unwrap().to_string();
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
    pub fn get_sfcode(&self, index:u8) -> i32 {
        for sf in self.stacks.iter().rev() {
            match sf.sfcodes.get(&index) {
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
    pub fn get_skip(&self, index:i16) -> Skip {
        for sf in self.stacks.iter().rev() {
            match sf.skips.get(&index) {
                Some(r) => return *r,
                _ => {}
            }
        }
        Skip{
            base: 0,
            stretch: None,
            shrink: None
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
    pub fn tokens(&self,index:i16) -> Vec<Token> {
        for sf in self.stacks.iter().rev() {
            match sf.toks.get(&index) {
                Some(r) => return r.clone(),
                _ => {}
            }
        }
        vec!()
    }
    pub fn change(&mut self,int:&Interpreter,change:StateChange) {
        match change {
            StateChange::Register(index,value,global) => {
                if global {
                    for s in self.stacks.iter_mut() {
                        s.registers.insert(index,value);
                    }
                } else {
                    self.stacks.last_mut().unwrap().registers.insert(index,value);
                }
            }
            StateChange::Dimen(index,value,global) => {
                if global {
                    for s in self.stacks.iter_mut() {
                        s.dimensions.insert(index,value);
                    }
                } else {
                    self.stacks.last_mut().unwrap().dimensions.insert(index,value);
                }
            }
            StateChange::Skip(index,value,global) => {
                if global {
                    for s in self.stacks.iter_mut() {
                        s.skips.insert(index,value);
                    }
                } else {
                    self.stacks.last_mut().unwrap().skips.insert(index,value);
                }
            }
            StateChange::Cs(name,cmd,global) => {
                if global {
                    for s in self.stacks.iter_mut() {
                        s.commands.remove(&*name);
                    }
                    match cmd {
                        Some(c) => self.stacks.first_mut().unwrap().commands.insert(name.to_string(),Some(c)),
                        None => self.stacks.first_mut().unwrap().commands.remove(&name)
                    };
                } else if self.stacks.len() == 1 {
                    match cmd {
                        Some(c) => self.stacks.first_mut().unwrap().commands.insert(name.to_string(),Some(c)),
                        None => self.stacks.first_mut().unwrap().commands.remove(&name)
                    };
                } else {
                    self.stacks.last_mut().unwrap().commands.insert(name.to_string(),cmd);
                }
            }
            StateChange::Cat(char,catcode,global) => {
                match catcode {
                    CategoryCode::Other => {
                        int.catcodes.borrow_mut().catcodes.remove(&char);
                        if global {
                            for s in self.stacks.iter_mut() {
                                s.catcodes.catcodes.remove(&char);
                            }
                        }

                    }
                    _ => {
                        int.catcodes.borrow_mut().catcodes.insert(char, catcode);
                        if global {
                            for s in self.stacks.iter_mut() {
                                s.catcodes.catcodes.insert(char, catcode);
                            }
                        }
                    }
                }
            }
            StateChange::Newline(char,global) => {
                int.catcodes.borrow_mut().newlinechar = char;
                if global {
                    for s in self.stacks.iter_mut() {
                        s.catcodes.newlinechar = char;
                    }
                }
            }
            StateChange::Endline(char,global) => {
                int.catcodes.borrow_mut().endlinechar = char;
                if global {
                    for s in self.stacks.iter_mut() {
                        s.catcodes.endlinechar = char;
                    }
                }
            }
            StateChange::Sfcode(char,value,global) => {
                if global {
                    for s in self.stacks.iter_mut() {
                        s.sfcodes.insert(char,value);
                    }
                } else {
                    self.stacks.last_mut().unwrap().sfcodes.insert(char,value);
                }
            }
            StateChange::Tokens(i,tks,global) => {
                if global {
                    for s in self.stacks.iter_mut() {
                        s.toks.remove(&i);
                    }
                    self.stacks.first_mut().unwrap().toks.insert(i,tks);
                } else {
                    self.stacks.last_mut().unwrap().toks.insert(i,tks);
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
    pub (in crate::interpreter::state) fn pop(&mut self,int:&Interpreter,_tp : GroupType) -> Result<&CategoryCodeScheme,TeXError> {
        if self.stacks.len() < 2 { TeXErr!(int,"No group here to end!")}
        match self.stacks.pop() {
            Some(sf) => match sf.tp {
                None => TeXErr!(int,"No group here to end!"),
                Some(ltp) if !matches!(ltp,_tp) => TeXErr!(int,"Group opened by {} ended by {}",ltp,_tp),
                _ => Ok(&self.stacks.last().unwrap().catcodes)
            }
            None => TeXErr!(int,"No group here to end!")
        }

    }
}


pub fn default_pdf_latex_state() -> State {
    let mut st = State::new();
    let pdftex_cfg = kpsewhich("pdftexconfig.tex",&PWD).expect("pdftexconfig.tex not found");
    let latex_ltx = kpsewhich("latex.ltx",&PWD).expect("No latex.ltx found");

    //println!("{}",pdftex_cfg.to_str().expect("wut"));
    //println!("{}",latex_ltx.to_str().expect("wut"));
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
use std::fmt::{Display, Formatter};
use crate::interpreter::dimensions::Skip;
use crate::interpreter::files::VFile;
use crate::interpreter::mouth::StringMouth;
use crate::interpreter::Token;

impl Interpreter<'_> {
    pub fn file_read(&self,index:u8,nocomment:bool) -> Result<Vec<Token>,TeXError> {
        use std::io::BufRead;
        match index {
            255 => {
                let stdin = std::io::stdin();
                let string = stdin.lock().lines().next().unwrap().unwrap();
                Ok(crate::interpreter::tokenize(&string,&self.catcodes.borrow()))
            }
            i => {
                match self.state.borrow_mut().infiles.get_mut(&i) {
                    None => TeXErr!(self,"No file open at index {}",i),
                    Some(fm) => Ok(fm.read_line(&self.catcodes.borrow(),nocomment))
                }
            }
        }
    }
    pub fn file_eof(&self,index:u8) -> Result<bool,TeXError> {
        match self.state.borrow_mut().infiles.get_mut(&index) {
            None => TeXErr!(self,"No file open at index {}",index),
            Some(fm) => Ok(!fm.has_next(&self.catcodes.borrow(),false))
        }
    }
    pub fn file_openin(&self,index:u8,file:VFile) -> Result<(),TeXError> {
        let mut state = self.state.borrow_mut();
        if state.infiles.contains_key(&index) {
            TeXErr!(self,"File already open at {}",index)
        }
        let mouth = StringMouth::new_from_file(&self.catcodes.borrow(),&file);
        self.filestore.borrow_mut().files.insert(file.id.clone(),file);
        state.infiles.insert(index,mouth);
        Ok(())
    }
    pub fn file_closein(&self,index:u8) -> Result<(),TeXError> {
        let mut state = self.state.borrow_mut();
        match state.infiles.remove(&index) {
            None => TeXErr!(self,"No file open at index {}",index),
            Some(f) => {
                f.source.pop_file().unwrap();
            }
        }
        Ok(())
    }
    pub fn file_openout(&self,index:u8,file:VFile) -> Result<(),TeXError> {
        let mut state = self.state.borrow_mut();
        if state.outfiles.contains_key(&index) {
            TeXErr!(self,"File already open at {}",index)
        }
        state.outfiles.insert(index,file);
        Ok(())
    }
    pub fn file_write(&self,index:u8,s:String) -> Result<(),TeXError> {
        use ansi_term::Colour::*;
        match index {
            17 => {
                print!("{}",s);
                Ok(())
            }
            16 | 18 => todo!("{}",index),
            255 => {
                print!("{}",Black.on(Blue).paint(s));
                Ok(())
            }
            i if !self.state.borrow().outfiles.contains_key(&i) => todo!("{}",i),
             _ => {
                 let mut state = self.state.borrow_mut();
                 match state.outfiles.get_mut(&index) {
                     Some(f) => match f.string.borrow_mut() {
                         x@None => *x = Some(s),
                         Some(st) => *st += &s
                     }
                     None => TeXErr!(self,"No file open at index {}",index)
                 }
                 Ok(())
             }
        }
    }
    pub fn file_closeout(&self,index:u8) -> Result<(),TeXError> {
        let mut state = self.state.borrow_mut();
        match state.outfiles.remove(&index) {
            Some(vf) => {self.filestore.borrow_mut().files.insert(vf.id.clone(),vf);}
            None => ()//TeXErr!(self,"No file open at index {}",index)
        }
        Ok(())
    }
    pub fn change_state(&self,change:StateChange) {
        let mut state = self.state.borrow_mut();
        state.change(self,change)
    }
    pub fn new_group(&self,tp:GroupType) {
        log!("Push: {}",tp);
        self.state.borrow_mut().push(self.catcodes.borrow().clone(),tp)
    }
    pub fn pop_group(&self,tp:GroupType) -> Result<(),TeXError> {
        log!("Pop: {}",tp);
        let mut state = self.state.borrow_mut();
        let cc = state.pop(self,tp)?;
        let mut scc = self.catcodes.borrow_mut();
        scc.catcodes = cc.catcodes.clone();
        scc.endlinechar = cc.endlinechar;
        scc.newlinechar = cc.newlinechar;
        scc.escapechar = cc.escapechar;
        Ok(())
    }

    pub fn state_catcodes(&self) -> Ref<'_,CategoryCodeScheme> {
        self.catcodes.borrow()
    }
    pub fn state_register(&self,i:i16) -> i32 { self.state.borrow().get_register(i) }
    pub fn state_dimension(&self,i:i16) -> i32 {
        self.state.borrow().get_dimension(i)
    }
    pub fn state_skip(&self,i:i16) -> Skip {
        self.state.borrow().get_skip(i)
    }
    pub fn state_sfcode(&self,i:u8) -> i32 { self.state.borrow().get_sfcode(i) }
    pub fn state_tokens(&self,i:i16) -> Vec<Token> { self.state.borrow().tokens(i)}

    pub fn pushcondition(&self) -> u8 {
        let mut state = self.state.borrow_mut();
        state.conditions.push(None);
        (state.conditions.len() - 1) as u8
    }
    pub fn setcondition(&self,c : u8,val : bool) -> u8 {
        let conds = &mut self.state.borrow_mut().conditions;
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

pub enum StateChange {
    Register(i16,i32,bool),
    Dimen(i16,i32,bool),
    Skip(i16,Skip,bool),
    Cs(String,Option<TeXCommand>,bool),
    Cat(u8,CategoryCode,bool),
    Newline(u8,bool),
    Endline(u8,bool),
    Sfcode(u8,i32,bool),
    Tokens(i16,Vec<Token>,bool)
}