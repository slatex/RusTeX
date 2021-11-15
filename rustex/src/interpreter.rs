pub enum TeXMode {
    Vertical, InternalVertical, Horizontal, RestrictedHorizontal, Math, Displaymath, Script, ScriptScript
}

use std::borrow::BorrowMut;
use std::cell::RefCell;
use std::collections::HashMap;
use std::ops::Deref;
use crate::ontology::Token;
use crate::catcodes::{CategoryCode, CategoryCodeScheme};
use crate::references::SourceReference;
use std::path::Path;
use std::rc::Rc;
use std::str::FromStr;
use crate::commands::{Assignment, TeXCommand};
use crate::interpreter::files::{FileStore, VFile};
use crate::interpreter::mouth::Mouths;
use crate::interpreter::state::State;
use crate::utils::TeXError;

pub mod mouth;
pub mod state;
mod files;
pub mod dimensions;
pub mod methods;


fn tokenize(s : &str,cats: &CategoryCodeScheme) -> Vec<Token> {
    let ns = s.as_bytes();
    let mut retvec: Vec<Token> = Vec::new();
    for next in ns {
        retvec.push(Token {
            catcode: cats.get_code(*next),
            name_opt: None,
            char: *next,
            reference: Box::new(SourceReference::None)
        })
    }
    retvec
}

pub struct Jobinfo<'a> {
    pub path : &'a Path
}

impl Jobinfo<'_> {
    pub fn new(p : &Path) -> Jobinfo {
        Jobinfo {
            path:p
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
impl Interpreter<'_> {
    pub fn string_to_tokens(s : &str) -> Vec<Token> {
        use crate::catcodes::OTHER_SCHEME;
        tokenize(s,&OTHER_SCHEME)
    }
    pub fn get_file(&self,filename : &str) -> Result<VFile,TeXError> {
        use crate::utils::kpsewhich;
        match kpsewhich(filename,self.jobinfo.in_file()) {
            None => Err(TeXError::new("File ".to_owned() + filename + " not found")),
            Some(p) => Ok(VFile::new(&p,self.jobinfo.in_file(),&mut self.filestore.borrow_mut()))
        }
    }
    pub fn do_file_with_state(p : &Path, s : State) -> State {
        let catcodes = s.catcodes().clone();
        let mut int = Interpreter {
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
        while int.has_next() {
            match int.do_top() {
                Ok(_) => {},
                Err(s) => s.throw()
            }
        }
        let ret = int.state.borrow().clone(); ret
    }

    pub fn do_assignment(&self,a : Assignment,globally:bool) -> Result<(),TeXError> {
        let global = globally; // TODO!
        a.assign(self,global)
    }

    pub fn get_command(&self,s : &str) -> Result<TeXCommand,TeXError> {
        match self.state.borrow().get_command(s) {
            Some(p) => Ok(p),
            None => Err(TeXError::new("Unknown control sequence: ".to_owned() + s + " at " + self.current_line().as_str()))
        }
    }

    pub fn do_top(&self) -> Result<(),TeXError> {
        use crate::commands::primitives;
        let next = self.next_token();
        match next.catcode {
            CategoryCode::Active | CategoryCode::Escape => {
                let mut p = self.get_command(&next.cmdname())?;
                p = match p.as_assignment() {
                    Ok(a) => return self.do_assignment(a,false),
                    Err(x) => x
                };
                p = match p.as_expandable() {
                    Ok(e) => return e.expand(next,self),
                    Err(x) => x
                };
                match p {
                    //TeXCommand::Register(_) | TeXCommand::Dimen(_) => return self.do_assignment(p,false),
                    TeXCommand::Primitive(p) if *p == primitives::PAR && matches!(self.mode,TeXMode::Vertical) => Ok(()),
                    TeXCommand::Primitive(p) => p.apply(next,self),
                    TeXCommand::Ext(exec) =>
                        exec.execute(self).map_err(|x| x.derive("External Command ".to_owned() + exec.name().as_str() + " errored!")),
                    _ => todo!("{}",next.as_string())

                }
            },
            CategoryCode::Space | CategoryCode::EOL if matches!(self.mode,TeXMode::Vertical) => Ok(()),
            _ => todo!("Character: {}, {}, {}",next.char,next.catcode,self.current_line())
        }
    }

    pub fn current_line(&self) -> String {
        self.mouths.borrow().current_line()
    }
}