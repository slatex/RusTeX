
#[derive(Copy,Clone,PartialEq)]
pub enum TeXMode {
    Vertical, InternalVertical, Horizontal, RestrictedHorizontal, Math, Displaymath
}

use std::borrow::BorrowMut;
use std::cell::RefCell;
use std::collections::HashMap;
use crate::ontology::{Expansion, Token};
use crate::catcodes::{CategoryCode, CategoryCodeScheme};
use crate::references::SourceReference;
use std::path::{Path, PathBuf};
use crate::commands::{TeXCommand,PrimitiveTeXCommand};
use crate::interpreter::files::{FileStore, VFile};
use crate::interpreter::mouth::Mouths;
use crate::interpreter::state::{GroupType, State};
use crate::utils::{TeXError, TeXString, TeXStr, kpsewhich};
use std::rc::Rc;

pub mod mouth;
pub mod state;
mod files;
pub mod dimensions;
pub mod methods;


pub fn tokenize(s : TeXString,cats: &CategoryCodeScheme) -> Vec<Token> {
    let mut retvec: Vec<Token> = Vec::new();
    for next in s.0 {
        retvec.push(Token::new(next,cats.get_code(next),None,SourceReference::None,true))
    }
    retvec
}

use chrono::{DateTime,Local};

pub struct Jobinfo {
    pub path : PathBuf,
    pub time: DateTime<Local>
}

impl Jobinfo {
    pub fn new(p : PathBuf) -> Jobinfo {
        Jobinfo {
            path:p,
            time:Local::now()
        }
    }
    pub fn in_file(&self) -> &Path {
        self.path.parent().unwrap()
    }
}

pub struct Interpreter<'a> {
    pub (in crate) state:RefCell<State>,
    pub jobinfo:Jobinfo,
    mouths:RefCell<Mouths>,
    filestore:RefCell<FileStore>,
    catcodes:RefCell<CategoryCodeScheme>,
    pub stomach:RefCell<&'a mut dyn Stomach>
}
use crate::{TeXErr,FileEnd};
use crate::commands::PrimitiveTeXCommand::Whatsit;

pub fn string_to_tokens(s : TeXString) -> Vec<Token> {
    use crate::catcodes::OTHER_SCHEME;
    tokenize(s,&OTHER_SCHEME)
}
pub fn tokens_to_string_default(tks:&Vec<Token>) -> TeXString {
    tokens_to_string(tks,&crate::catcodes::OTHER_SCHEME)
}

pub fn tokens_to_string(tks:&Vec<Token>,catcodes:&CategoryCodeScheme) -> TeXString {
    let mut ret : Vec<u8> = vec!();
    let escapechar = catcodes.escapechar;
    for tk in tks {
        match tk.catcode {
            CategoryCode::Escape => {
                let name = tk.name();
                if escapechar != 255 { ret.push(catcodes.escapechar) }
                for s in name.iter() { ret.push(*s) }
                if name.len() > 1 {
                    ret.push(32)
                } else if name.len() == 1 {
                    match catcodes.get_code(*name.iter().first().unwrap()) {
                        CategoryCode::Letter => ret.push(32),
                        _ => ()
                    }
                } else {
                    ret.append(&mut vec!(99,115,110,97,109,101)); // csname
                    if catcodes.escapechar != 255 { ret.push(catcodes.escapechar) }
                    ret.append(&mut vec!(101,110,100,99,115,110,97,109,101)) // endcsname
                }
            }
            _ => ret.push(tk.char)
        }
    }
    ret.into()
}

use crate::stomach::{NoShipoutRoutine, Stomach};
use crate::stomach::whatsits::{SimpleWI};

impl Interpreter<'_> {
    pub fn tokens_to_string(&self,tks:&Vec<Token>) -> TeXString {
        let catcodes = self.catcodes.borrow();
        tokens_to_string(tks,&catcodes)
    }

    pub fn kpsewhich(&self,filename: &str) -> Option<PathBuf> {
        kpsewhich(filename,self.jobinfo.in_file())
    }

    pub fn get_file(&self,filename : &str) -> Result<VFile,TeXError> {
        match self.kpsewhich(filename) {
            None =>TeXErr!((self,None),"File {} not found",filename),
            Some(p) => Ok(VFile::new(&p,self.jobinfo.in_file(),&mut self.filestore.borrow_mut()))
        }
    }
    pub fn with_state(s : State,stomach: &mut dyn Stomach) -> Interpreter {
        let catcodes = s.catcodes().clone();
        Interpreter {
            state:RefCell::new(s),
            jobinfo:Jobinfo::new(PathBuf::new()),
            mouths:RefCell::new(Mouths::new()),
            filestore:RefCell::new(FileStore {
                files:HashMap::new()
            }),
            catcodes:RefCell::new(catcodes),
            stomach:RefCell::new(stomach)
        }
    }
    pub fn do_file(&mut self,p:&Path) {
        self.jobinfo = Jobinfo::new(p.to_path_buf());
        let vf:VFile  = VFile::new(p,self.jobinfo.in_file(),&mut self.filestore.borrow_mut());
        self.push_file(vf);
        self.insert_every(&crate::commands::primitives::EVERYJOB);
        while self.has_next() {
            let next = self.next_token();
            if !self.state.borrow().indocument {
                let line = self.state.borrow().indocument_line;
                match line {
                    Some(i) if self.line_no() > i => {
                        self.state.borrow_mut().indocument_line = None;
                        self.stomach.borrow_mut().on_begin_document(self)
                    }
                    _ => match next.catcode {
                        CategoryCode::Escape if next.cmdname() == "document" => {
                            self.state.borrow_mut().indocument_line = Some(self.line_no())
                        }
                        _ => ()
                    }
                }
            }
            match self.do_top(next,false) {
                Ok(_) => {},
                Err(s) => s.throw()
            }
        }
    }
    pub fn do_file_with_state(p : &Path, s : State) -> State {
        let mut stomach = NoShipoutRoutine::new();
        let mut int = Interpreter::with_state(s,stomach.borrow_mut());
        int.jobinfo = Jobinfo::new(p.to_path_buf());
        let vf:VFile  = VFile::new(p,int.jobinfo.in_file(),&mut int.filestore.borrow_mut());
        int.push_file(vf);
        int.insert_every(&crate::commands::primitives::EVERYJOB);
        while int.has_next() {
            let next = int.next_token();
            if !int.state.borrow().indocument {
                let line = int.state.borrow().indocument_line;
                match line {
                    Some(i) if int.line_no() > i => {
                        int.state.borrow_mut().indocument_line = None;
                        int.stomach.borrow_mut().on_begin_document(&int)
                    }
                    _ => match next.catcode {
                        CategoryCode::Escape if next.cmdname() == "document" => {
                            int.state.borrow_mut().indocument_line = Some(int.line_no())
                        }
                        _ => ()
                    }
                }
            }
            match int.do_top(next,false) {
                Ok(_) => {},
                Err(s) => s.throw()
            }
        }
        let ret = int.state.borrow().clone();
        ret.close(int)
    }

    pub fn get_command(&self,s : &TeXStr) -> Result<TeXCommand,TeXError> {
        match self.state.borrow().get_command(s) {
            Some(p) => Ok(p),
            None if s.len() == 1 => {
                let char = *s.iter().first().unwrap();
                let catcode = self.catcodes.borrow().get_code(char);
                let tk = Token::new(char,catcode,None,SourceReference::None,true);
                Ok(PrimitiveTeXCommand::Char(tk).as_command())
            }
            None => TeXErr!((self,None),"Unknown control sequence: \\{}",s)
        }
    }

    pub fn do_top(&self,next:Token,inner:bool) -> Result<(),TeXError> {
        use crate::commands::primitives;
        let mode = self.get_mode();
        use crate::catcodes::CategoryCode::*;
        use TeXMode::*;
        use PrimitiveTeXCommand::*;
        match (next.catcode,mode) {
            (Active | Escape,_) => {
                let p = self.get_command(&next.cmdname())?;
                if p.assignable() {
                    return p.assign(next,self,false)
                } else if p.expandable(true) {
                    return p.expand(next,self)
                }
                match (&*p.orig,mode) {
                    (Primitive(p),Vertical | InternalVertical) if **p == primitives::PAR => {
                        self.stomach.borrow_mut().reset_par();
                        Ok(())
                    },
                    (Primitive(p),Vertical | InternalVertical) if **p == primitives::INDENT || **p == primitives::NOINDENT => {
                        self.switch_to_H(next)
                    }
                    (Primitive(p),Horizontal) if **p == primitives::PAR => self.end_paragraph(inner),
                    (Primitive(np),_) => {
                        let mut exp = Expansion(next,Rc::new(p.clone()),vec!());
                        np.apply(&mut exp,self)?;
                        if !exp.2.is_empty() {
                            self.push_expansion(exp)
                        }
                        Ok(())
                    },
                    (Ext(exec),_) =>
                        exec.execute(self).map_err(|x| x.derive("External Command ".to_owned() + &exec.name() + " errored!")),
                    (Char(tk),_) => {
                        self.requeue(tk.clone());
                        Ok(())
                    },
                    (Whatsit(w),_) if w.allowed_in(mode) => {
                        let next = w.get(&next,self)?;
                        self.stomach.borrow_mut().add(self,next)
                    },
                    (Whatsit(_), Vertical | InternalVertical) => {
                        self.switch_to_H(next)
                    }
                    _ => TeXErr!((self,Some(next.clone())),"TODO: {} in {}",next,self.current_line())

                }
            },
            (BeginGroup,_) => Ok(self.new_group(GroupType::Token)),
            (EndGroup,_) => self.pop_group(GroupType::Token),
            (Space | EOL, Vertical | InternalVertical) => Ok(()),
            (Letter | Other | Space, Horizontal | RestrictedHorizontal) => {
                let font = self.get_font();
                let rf = self.update_reference(&next);
                self.stomach.borrow_mut().add(self,crate::stomach::Whatsit::Char(next.char,font,rf))
            }
            (MathShift, Horizontal | RestrictedHorizontal) => {
                self.start_math(inner)
            }
            (Letter | Other | Space | MathShift,Vertical | InternalVertical) => self.switch_to_H(next),
            _ => TeXErr!((self,Some(next)),"Urgh!"),
        }
    }

    fn switch_to_H(&self,next:Token) -> Result<(),TeXError> {
        let indent = match next.catcode {
            CategoryCode::Escape | CategoryCode::Active => {
                let pr = self.get_command(next.cmdname())?;
                match &*pr.orig {
                    PrimitiveTeXCommand::Primitive(c) if **c == crate::commands::primitives::NOINDENT => 0,
                    PrimitiveTeXCommand::Primitive(c) if **c == crate::commands::primitives::INDENT =>
                        self.state_dimension(-(crate::commands::primitives::PARINDENT.index as i32)),
                    _ => {
                        self.requeue(next);
                        self.state_dimension(-(crate::commands::primitives::PARINDENT.index as i32))
                    }
                }
            }
            _ => {
                self.requeue(next);
                self.state_dimension(-(crate::commands::primitives::PARINDENT.index as i32))
            }
        };
        self.state.borrow_mut().mode = TeXMode::Horizontal;
        self.insert_every(&crate::commands::primitives::EVERYPAR);
        let parskip = self.state_skip(-(crate::commands::primitives::PARSKIP.index as i32));
        self.stomach.borrow_mut().start_paragraph(parskip.base);
        if indent != 0 {
            self.stomach.borrow_mut().add(self,crate::stomach::Whatsit::Simple(SimpleWI::Indent(indent,None)))?
        }
        Ok(())
    }

    fn end_paragraph(&self,inner : bool) -> Result<(),TeXError> {
        if inner { self.set_mode(TeXMode::InternalVertical) } else { self.set_mode(TeXMode::Vertical) }
        self.stomach.borrow_mut().end_paragraph(self)?;
        for w in self.state.borrow_mut().vadjust.drain(..) {
            self.stomach.borrow_mut().add(self,w)?
        }
        Ok(())
    }

    fn start_math(&self, inner : bool) -> Result<(),TeXError> {
        let _oldmode = self.get_mode();
        let mode = if inner { TeXMode::Math } else {
            let next = self.next_token();
            match next.catcode {
                CategoryCode::MathShift => TeXMode::Displaymath,
                _ => {
                    self.requeue(next);
                    TeXMode::Math
                }
            }
        };
        self.set_mode(mode);
        self.new_group(GroupType::Math);
        while self.has_next() {
            let next = self.next_token();
            match next.catcode {
                CategoryCode::MathShift if self.get_mode() == TeXMode::Displaymath => todo!(),
                CategoryCode::MathShift => {
                    self.set_mode(_oldmode);
                    self.pop_group(GroupType::Math)?;
                    return Ok(())
                }
                _ => self.do_top(next,false)?
            }
        }
        FileEnd!(self)
    }

    pub fn current_line(&self) -> String {
        self.mouths.borrow().current_line()
    }
    pub fn line_no(&self) -> usize {
        self.mouths.borrow().line_no().0
    }

    pub fn assert_has_next(&self) -> Result<(),TeXError> {
        if self.has_next() {Ok(())} else  {
            FileEnd!(self)
        }
    }
}