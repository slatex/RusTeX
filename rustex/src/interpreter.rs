#[derive(PartialEq)]
pub enum TeXMode {
    Vertical, InternalVertical, Horizontal, RestrictedHorizontal, Math, Displaymath, Script, ScriptScript
}

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

pub struct Jobinfo<'a> {
    pub path : &'a Path,
    pub time: DateTime<Local>
}

impl Jobinfo<'_> {
    pub fn new(p : &Path) -> Jobinfo {
        Jobinfo {
            path:p,
            time:Local::now()
        }
    }
    pub fn in_file(&self) -> &Path {
        self.path.parent().unwrap()
    }
}

pub struct Interpreter<'inner> {
    state:RefCell<State>,
    pub jobinfo:Jobinfo<'inner>,
    mouths:RefCell<Mouths>,
    filestore:RefCell<FileStore>,
    mode:TeXMode,
    catcodes:RefCell<CategoryCodeScheme>
}
use crate::{TeXErr,FileEnd};

pub fn string_to_tokens(s : TeXString) -> Vec<Token> {
    use crate::catcodes::OTHER_SCHEME;
    tokenize(s,&OTHER_SCHEME)
}
pub fn tokens_to_string_default(tks:Vec<Token>) -> TeXString {
    let mut ret : Vec<u8> = vec!();
    for tk in tks {
        match tk.catcode {
            CategoryCode::Escape => {
                ret.push(92);
                for s in tk.name().iter() { ret.push(*s) }
                ret.push(32)
            }
            _ => ret.push(tk.char)
        }
    }
    ret.into()
}

impl Interpreter<'_> {
    pub fn tokens_to_string(&self,tks:Vec<Token>) -> TeXString {
        let catcodes = self.catcodes.borrow();
        let mut ret : Vec<u8> = vec!();
        for tk in tks {
            match tk.catcode {
                CategoryCode::Escape if catcodes.escapechar != 255 => {
                    ret.push(catcodes.escapechar);
                    for s in tk.name().iter() { ret.push(*s) }
                    ret.push(32)
                }
                _ => ret.push(tk.char)
            }
        }
        ret.into()
    }

    pub fn kpsewhich(&self,filename: &str) -> Option<PathBuf> {
        kpsewhich(filename,self.jobinfo.in_file())
    }

    pub fn get_file(&self,filename : &str) -> Result<VFile,TeXError> {
        use crate::utils::kpsewhich;
        match self.kpsewhich(filename) {
            None =>TeXErr!((self,None),"File {} not found",filename),
            Some(p) => Ok(VFile::new(&p,self.jobinfo.in_file(),&mut self.filestore.borrow_mut()))
        }
    }
    pub fn do_file_with_state(p : &Path, s : State) -> State {
        let catcodes = s.catcodes().clone();
        let int = Interpreter {
            state:RefCell::new(s),
            jobinfo:Jobinfo::new(p),
            mouths:RefCell::new(Mouths::new()),
            filestore:RefCell::new(FileStore {
                files:HashMap::new()
            }),
            mode:TeXMode::Vertical,
            catcodes:RefCell::new(catcodes)
        };
        let vf:VFile  = VFile::new(p,int.jobinfo.in_file(),&mut int.filestore.borrow_mut());
        int.push_file(vf);
        int.insert_every(&crate::commands::primitives::EVERYJOB);
        while int.has_next() {
            match int.do_top() {
                Ok(_) => {},
                Err(s) => s.throw()
            }
        }
        let ret = int.state.borrow().clone(); ret
    }

    pub fn get_command(&self,s : &TeXStr) -> Result<TeXCommand,TeXError> {
        match self.state.borrow().get_command(s) {
            Some(p) => Ok(p),
            None => TeXErr!((self,None),"Unknown control sequence: \\{}",s)
        }
    }

    pub fn do_top(&self) -> Result<(),TeXError> {
        use crate::commands::primitives;
        let next = self.next_token();
        match next.catcode {
            CategoryCode::Active | CategoryCode::Escape => {
                let mut p = self.get_command(&next.cmdname())?;
                if p.assignable() {
                    return p.assign(next,self,false)
                } else if p.expandable(true) {
                    return p.expand(next,self)
                }
                match &*p.orig {
                    PrimitiveTeXCommand::Primitive(p) if **p == primitives::PAR && self.mode == TeXMode::Vertical => Ok(()),
                    PrimitiveTeXCommand::Primitive(np) => {
                        let mut exp = Expansion(next,Rc::new(p.clone()),vec!());
                        np.apply(&mut exp,self)?;
                        if !exp.2.is_empty() {
                            self.push_expansion(exp)
                        }
                        Ok(())
                    },
                    PrimitiveTeXCommand::Ext(exec) =>
                        exec.execute(self).map_err(|x| x.derive("External Command ".to_owned() + &exec.name() + " errored!")),
                    _ => todo!("{}",next)

                }
            },
            CategoryCode::BeginGroup => Ok(self.new_group(GroupType::Token)),
            CategoryCode::EndGroup => self.pop_group(GroupType::Token),
            CategoryCode::Space | CategoryCode::EOL if matches!(self.mode,TeXMode::Vertical) => Ok(()),
            _ => todo!("{}, {}",next,self.current_line())
        }
    }

    pub fn current_line(&self) -> String {
        self.mouths.borrow().current_line()
    }

    pub fn assert_has_next(&self) -> Result<(),TeXError> {
        if self.has_next() {Ok(())} else  {
            FileEnd!(self)
        }
    }
}