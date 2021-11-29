use std::borrow::BorrowMut;
use std::collections::HashMap;
use crate::catcodes::{CategoryCode, CategoryCodeScheme, STARTING_SCHEME};
use crate::commands::TeXCommand;
use crate::interpreter::{Interpreter, TeXMode};
use crate::utils::{kpsewhich, PWD, TeXError, TeXString, TeXStr};
use crate::{TeXErr,log};

#[derive(Copy,Clone,PartialEq)]
pub enum GroupType {
    Token,
    Begingroup,
    Box(BoxMode)
}
impl Display for GroupType {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(f,"{}",match self {
            GroupType::Token => "{",
            GroupType::Begingroup => "\\begingroup",
            GroupType::Box(_) => "\\box"
        })
    }
}

#[derive(Clone)]
struct StackFrame {
    //parent: Option<&'a StackFrame<'a>>,
    pub(crate) catcodes: CategoryCodeScheme,
    pub(crate) newlinechar: u8,
    pub(crate) endlinechar: u8,
    pub(crate) commands: HashMap<TeXStr,Option<TeXCommand>>,
    pub(crate) registers: HashMap<i16,i32>,
    pub(crate) dimensions: HashMap<i16,i32>,
    pub(crate) skips : HashMap<i16,Skip>,
    pub(crate) muskips : HashMap<i16,MuSkip>,
    pub(crate) toks : HashMap<i16,Vec<Token>>,
    pub(in crate::interpreter::state) tp : Option<GroupType>,
    pub(crate) sfcodes : HashMap<u8,i32>,
    pub(crate) lccodes : HashMap<u8,u8>,
    pub(crate) uccodes : HashMap<u8,u8>,
    pub(crate) mathcodes : HashMap<u8,i32>,
    pub(crate) delcodes : HashMap<u8,i32>,
    pub(crate) boxes: HashMap<i16,TeXBox>,
    pub(crate) currfont : Rc<Font>
}

impl StackFrame {
    pub(crate) fn initial_pdf_etex() -> StackFrame {
        use crate::commands::conditionals::conditional_commands;
        use crate::commands::primitives::tex_commands;
        use crate::commands::pdftex::pdftex_commands;
        let mut cmds: HashMap<TeXStr,Option<TeXCommand>> = HashMap::new();
        for c in conditional_commands() {
            let c = c.as_command();
            cmds.insert(c.name().unwrap().clone(),Some(c));
        }
        for c in tex_commands() {
            let c = c.as_command();
            cmds.insert(c.name().unwrap().clone(),Some(c));
        }
        for c in pdftex_commands() {
            let c = c.as_command();
            cmds.insert(c.name().unwrap().clone(),Some(c));
        }
        let mut reg: HashMap<i16,i32> = HashMap::new();
        reg.insert(-crate::utils::u8toi16(crate::commands::primitives::MAG.index),1000);

        let dims: HashMap<i16,i32> = HashMap::new();
        let skips: HashMap<i16,Skip> = HashMap::new();
        let muskips: HashMap<i16,MuSkip> = HashMap::new();
        let toks: HashMap<i16,Vec<Token>> = HashMap::new();
        let sfcodes: HashMap<u8,i32> = HashMap::new();
        let mut lccodes: HashMap<u8,u8> = HashMap::new();
        let mut uccodes: HashMap<u8,u8> = HashMap::new();
        for i in 97..123 {
            uccodes.insert(i,i-32);
            lccodes.insert(i-32,i);
        }
        let boxes: HashMap<i16,TeXBox> = HashMap::new();
        let mathcodes : HashMap<u8,i32> = HashMap::new();
        let delcodes : HashMap<u8,i32> = HashMap::new();
        StackFrame {
            //parent: None,
            catcodes: STARTING_SCHEME.clone(),
            commands: cmds,
            newlinechar: 10,
            endlinechar:13,
            registers:reg,
            dimensions:dims,
            skips,toks,sfcodes,lccodes,uccodes,muskips,boxes,mathcodes,delcodes,
            tp:None,
            currfont:Nullfont.try_with(|x| x.clone()).unwrap()
        }
    }
    pub(crate) fn new(parent: &StackFrame,tp : GroupType) -> StackFrame {
        let reg: HashMap<i16,i32> = HashMap::new();
        let dims: HashMap<i16,i32> = HashMap::new();
        let skips: HashMap<i16,Skip> = HashMap::new();
        let muskips: HashMap<i16,MuSkip> = HashMap::new();
        let toks: HashMap<i16,Vec<Token>> = HashMap::new();
        let sfcodes: HashMap<u8,i32> = HashMap::new();
        let mut lccodes: HashMap<u8,u8> = HashMap::new();
        let mut uccodes: HashMap<u8,u8> = HashMap::new();
        for i in 97..123 {
            uccodes.insert(i,i-32);
            lccodes.insert(i-32,i);
        }
        let boxes: HashMap<i16,TeXBox> = HashMap::new();
        let mathcodes : HashMap<u8,i32> = HashMap::new();
        let delcodes : HashMap<u8,i32> = HashMap::new();
        StackFrame {
            //parent: Some(parent),
            catcodes: parent.catcodes.clone(),
            commands: HashMap::new(),
            newlinechar: parent.newlinechar,
            endlinechar: parent.newlinechar,
            registers:reg,
            dimensions:dims,
            skips,toks,sfcodes,lccodes,uccodes,muskips,boxes,mathcodes,delcodes,
            tp:Some(tp),currfont:parent.currfont.clone()
        }
    }
}

// ------------------------------------------------------------------------------------------------

use std::rc::Rc;

#[derive(Clone)]
pub struct State {
    stacks: Vec<StackFrame>,
    pub(in crate) conditions:Vec<Option<bool>>,
    pub(in crate) outfiles:HashMap<u8,VFile>,
    pub(in crate) infiles:HashMap<u8,StringMouth>,
    pub(in crate) incs : u8,
    fontfiles: HashMap<TeXStr,Rc<FontFile>>,
    pub(in crate) mode:TeXMode,
    pub(in crate) afterassignment : Option<Token>
}

// sudo apt install libkpathsea-dev

impl State {
    pub fn new() -> State {
        let fonts: HashMap<TeXStr,Rc<FontFile>> = HashMap::new();
        State {
            stacks: vec![StackFrame::initial_pdf_etex()],
            conditions: vec![],
            outfiles:HashMap::new(),
            infiles:HashMap::new(),
            incs:0,
            fontfiles: fonts,
            mode:TeXMode::Vertical,
            afterassignment:None,
        }
    }

    pub fn get_font(&mut self,int:&Interpreter,name:TeXStr) -> Result<Rc<FontFile>,TeXError> {
        match self.fontfiles.get(&name) {
            Some(ff) => Ok(Rc::clone(ff)),
            None => {
                let ret = unsafe{int.kpsewhich(from_utf8_unchecked(name.iter()))};
                match ret {
                    Some(pb) if pb.exists() => {
                        let f = Rc::new(FontFile::new(pb));
                        self.fontfiles.insert(name,Rc::clone(&f));
                        Ok(f)
                    }
                    _ => TeXErr!((int,None),"Font file {} not found",name)
                }
            }
        }
    }

    pub fn with_commands(procs:Vec<TeXCommand>) -> State {
        let mut st = State::new();
        for p in procs {
            let name = p.name().unwrap();
            st.stacks.last_mut().unwrap().commands.insert(name.clone(),Some(p));
        }
        st
    }

    pub fn get_command(&self, name: &TeXStr) -> Option<TeXCommand> {
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

    pub (in crate::interpreter::state) fn lccode(&self,i:u8) -> u8 {
        for sf in self.stacks.iter().rev() {
            match sf.lccodes.get(&i) {
                Some(r) => return *r,
                _ => {}
            }
        }
        i
    }

    pub (in crate::interpreter::state) fn uccode(&self,i:u8) -> u8 {
        for sf in self.stacks.iter().rev() {
            match sf.uccodes.get(&i) {
                Some(r) => return *r,
                _ => {}
            }
        }
        i
    }

    pub (in crate::interpreter::state) fn mathcode(&self,i:u8) -> i32 {
        for sf in self.stacks.iter().rev() {
            match sf.mathcodes.get(&i) {
                Some(r) => return *r,
                _ => {}
            }
        }
        0
    }

    pub (in crate::interpreter::state) fn delcode(&self,i:u8) -> i32 {
        for sf in self.stacks.iter().rev() {
            match sf.delcodes.get(&i) {
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
            StateChange::Font(f,global) => {
                if global {
                    for s in self.stacks.iter_mut() {
                        s.currfont = f.clone()
                    }
                } else {
                    self.stacks.last_mut().unwrap().currfont = f
                }
            }
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
            StateChange::MuSkip(index,value,global) => {
                if global {
                    for s in self.stacks.iter_mut() {
                        s.muskips.insert(index,value);
                    }
                } else {
                    self.stacks.last_mut().unwrap().muskips.insert(index,value);
                }
            }
            StateChange::Cs(name,cmd,global) => {
                if global {
                    for s in self.stacks.iter_mut() {
                        s.commands.remove(&name);
                    }
                    match cmd {
                        Some(c) => self.stacks.first_mut().unwrap().commands.insert(name,Some(c)),
                        None => self.stacks.first_mut().unwrap().commands.remove(&name)
                    };
                } else if self.stacks.len() == 1 {
                    match cmd {
                        Some(c) => self.stacks.first_mut().unwrap().commands.insert(name,Some(c)),
                        None => self.stacks.first_mut().unwrap().commands.remove(&name)
                    };
                } else {
                    self.stacks.last_mut().unwrap().commands.insert(name,cmd);
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
            StateChange::Escapechar(char,global) => {
                int.catcodes.borrow_mut().escapechar = char;
                if global {
                    for s in self.stacks.iter_mut() {
                        s.catcodes.escapechar = char;
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
            StateChange::Mathcode(char,value,global) => {
                if global {
                    for s in self.stacks.iter_mut() {
                        s.mathcodes.insert(char,value);
                    }
                } else {
                    self.stacks.last_mut().unwrap().mathcodes.insert(char,value);
                }
            }
            StateChange::Delcode(char,value,global) => {
                if global {
                    for s in self.stacks.iter_mut() {
                        s.delcodes.insert(char,value);
                    }
                } else {
                    self.stacks.last_mut().unwrap().delcodes.insert(char,value);
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
            StateChange::Lccode(i,u,global) => {
                if global {
                    for s in self.stacks.iter_mut() {
                        s.lccodes.insert(i,u);
                    }
                } else {
                    self.stacks.last_mut().unwrap().lccodes.insert(i,u);
                }
            }
            StateChange::Uccode(i,u,global) => {
                if global {
                    for s in self.stacks.iter_mut() {
                        s.uccodes.insert(i,u);
                    }
                } else {
                    self.stacks.last_mut().unwrap().uccodes.insert(i,u);
                }
            }
            StateChange::Box(index,value,global) => {
                if global {
                    for s in self.stacks.iter_mut() {
                        s.boxes.remove(&index);
                    }
                    self.stacks.first_mut().unwrap().boxes.insert(index,value);
                } else {
                    self.stacks.last_mut().unwrap().boxes.insert(index,value);
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
        if self.stacks.len() < 2 { TeXErr!((int,None),"No group here to end!")}
        match self.stacks.pop() {
            Some(sf) => match sf.tp {
                None => TeXErr!((int,None),"No group here to end!"),
                Some(ltp) if !matches!(ltp,_tp) => TeXErr!((int,None),"Group opened by {} ended by {}",ltp,_tp),
                _ => Ok(&self.stacks.last().unwrap().catcodes)
            }
            None => TeXErr!((int,None),"No group here to end!")
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
use std::str::from_utf8_unchecked;
use crate::fonts::{Font, FontFile, Nullfont};
use crate::interpreter::dimensions::{MuSkip, Skip};
use crate::interpreter::files::VFile;
use crate::interpreter::mouth::StringMouth;
use crate::interpreter::Token;
use crate::stomach::whatsits::{BoxMode, TeXBox};

impl Interpreter<'_> {
    pub fn file_read(&self,index:u8,nocomment:bool) -> Result<Vec<Token>,TeXError> {
        use std::io::BufRead;
        match index {
            255 => {
                let stdin = std::io::stdin();
                let string = stdin.lock().lines().next().unwrap().unwrap();
                Ok(crate::interpreter::tokenize(string.into(),&self.catcodes.borrow()))
            }
            i => {
                match self.state.borrow_mut().infiles.get_mut(&i) {
                    None => TeXErr!((self,None),"No file open at index {}",i),
                    Some(fm) => Ok(fm.read_line(&self.catcodes.borrow(),nocomment))
                }
            }
        }
    }
    pub fn file_eof(&self,index:u8) -> Result<bool,TeXError> {
        match self.state.borrow_mut().infiles.get_mut(&index) {
            None => TeXErr!((self,None),"No file open at index {}",index),
            Some(fm) => {
                let ret = fm.has_next(&self.catcodes.borrow(),false,false);
                Ok(!ret)
            }
        }
    }
    pub fn file_openin(&self,index:u8,file:VFile) -> Result<(),TeXError> {
        let mut state = self.state.borrow_mut();
        /*if state.infiles.contains_key(&index) {
            TeXErr!((self,None),"File already open at {}",index)
        }*/
        let mouth = StringMouth::new_from_file(&self.catcodes.borrow(),&file);
        self.filestore.borrow_mut().files.insert(file.id.clone(),file);
        state.infiles.insert(index,mouth);
        Ok(())
    }
    pub fn file_closein(&self,index:u8) -> Result<(),TeXError> {
        let mut state = self.state.borrow_mut();
        match state.infiles.remove(&index) {
            None => TeXErr!((self,None),"No file open at index {}",index),
            Some(f) => {
                f.source.pop_file().unwrap();
            }
        }
        Ok(())
    }
    pub fn file_openout(&self,index:u8,file:VFile) -> Result<(),TeXError> {
        let mut state = self.state.borrow_mut();
        /*if state.outfiles.contains_key(&index) {
            TeXErr!((self,None),"File already open at {}",index)
        }*/
        state.outfiles.insert(index,file);
        Ok(())
    }
    pub fn file_write(&self,index:u8,s:TeXString) -> Result<(),TeXError> {
        use ansi_term::Colour::*;
        use std::io::Write;
        match index {
            17 => {
                print!("{}",s);
                std::io::stdout().flush();
                Ok(())
            }
            16 => {
                print!("{}",White.bold().paint(s.to_utf8()));
                std::io::stdout().flush();
                Ok(())
            }
            18 => todo!("{}",index),
            255 => {
                println!("{}",Black.on(Blue).paint(s.to_utf8()));
                std::io::stdout().flush();
                Ok(())
            }
            i if !self.state.borrow().outfiles.contains_key(&i) => {
                println!("{}",Black.on(Blue).paint(s.to_utf8()));
                std::io::stdout().flush();
                Ok(())
            }
             _ => {
                 let mut state = self.state.borrow_mut();
                 match state.outfiles.get_mut(&index) {
                     Some(f) => match f.string.borrow_mut() {
                         x@None => *x = Some(s),
                         Some(st) => *st += s
                     }
                     None => TeXErr!((self,None),"No file open at index {}",index)
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
    pub fn state_lccode(&self,i:u8) -> u8 { self.state.borrow().lccode(i) }
    pub fn state_uccode(&self,i:u8) -> u8 { self.state.borrow().uccode(i) }

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
    pub fn newincs(&self) -> u8 {
        let mut state = self.state.borrow_mut();
        state.incs += 1;
        state.incs
    }
    pub fn currcs(&self) -> u8 {
        self.state.borrow().incs
    }
    pub fn popcs(&self) -> Result<(),TeXError> {
        let mut state = self.state.borrow_mut();
        if state.incs > 0 {
            state.incs -= 1;
            Ok(())
        } else {
            TeXErr!((self,None),"spurious \\endcsname")
        }
    }
    pub fn state_get_command(&self,s:&TeXStr) -> Option<TeXCommand> {
        self.state.borrow().get_command(s)
    }
    pub fn state_get_font(&self,name:&str) -> Result<Rc<FontFile>,TeXError> {
        self.state.borrow_mut().get_font(self,name.into())
    }
    pub fn state_get_mathcode(&self,i:u8) -> i32 {
        self.state.borrow().mathcode(i)
    }
    pub fn state_get_delcode(&self,i:u8) -> i32 {
        self.state.borrow().delcode(i)
    }
    pub fn get_mode(&self) -> TeXMode {
        self.state.borrow().mode
    }
    pub fn set_mode(&self,tm:TeXMode) {
        self.state.borrow_mut().mode = tm
    }
    pub fn state_set_afterassignment(&self,tk:Token) {
        self.state.borrow_mut().afterassignment = Some(tk)
    }
    pub fn insert_afterassignment(&self) {
        match self.state.borrow_mut().afterassignment.take() {
            Some(tk) => self.push_tokens(vec!(tk)),
            _ => ()
        }
    }
    pub fn get_font(&self) -> Rc<Font> {
        self.state.borrow().stacks.last().unwrap().currfont.clone()
    }
}

pub enum StateChange {
    Register(i16,i32,bool),
    Dimen(i16,i32,bool),
    Skip(i16,Skip,bool),
    MuSkip(i16,MuSkip,bool),
    Cs(TeXStr,Option<TeXCommand>,bool),
    Cat(u8,CategoryCode,bool),
    Newline(u8,bool),
    Endline(u8,bool),
    Escapechar(u8,bool),
    Sfcode(u8,i32,bool),
    Tokens(i16,Vec<Token>,bool),
    Lccode(u8,u8,bool),
    Uccode(u8,u8,bool),
    Box(i16,TeXBox,bool),
    Mathcode(u8,i32,bool),
    Delcode(u8,i32,bool),
    Font(Rc<Font>,bool)
}