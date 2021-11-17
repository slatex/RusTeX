pub enum TeXMode {
    Vertical, InternalVertical, Horizontal, RestrictedHorizontal, Math, Displaymath, Script, ScriptScript
}

use std::cell::RefCell;
use std::collections::HashMap;
use crate::ontology::Token;
use crate::catcodes::{CategoryCode, CategoryCodeScheme};
use crate::references::SourceReference;
use std::path::Path;
use std::str::from_utf8;
use crate::commands::{Assignment, TeXCommand};
use crate::interpreter::files::{FileStore, VFile};
use crate::interpreter::mouth::Mouths;
use crate::interpreter::state::{GroupType, State};
use crate::utils::{TeXError, TeXString};

pub mod mouth;
pub mod state;
mod files;
pub mod dimensions;
pub mod methods;


pub fn tokenize(s : TeXString,cats: &CategoryCodeScheme) -> Vec<Token> {
    let mut retvec: Vec<Token> = Vec::new();
    for next in s.0 {
        retvec.push(Token {
            catcode: cats.get_code(next),
            name_opt: None,
            char: next,
            reference: Box::new(SourceReference::None),
            expand:true
        })
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

impl Interpreter<'_> {
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
                    for s in tk.name_opt.unwrap().0 { ret.push(s) }
                    ret.push(32)
                }
                _ => ret.push(tk.char)
            }
        }
        ret.into()
    }
    pub fn tokens_to_string(&self,tks:Vec<Token>) -> TeXString {
        let catcodes = self.catcodes.borrow();
        let mut ret : Vec<u8> = vec!();
        for tk in tks {
            match tk.catcode {
                CategoryCode::Escape if catcodes.escapechar != 255 => {
                    ret.push(catcodes.escapechar);
                    for s in tk.name_opt.unwrap().0 { ret.push(s) }
                    ret.push(32)
                }
                _ => ret.push(tk.char)
            }
        }
        ret.into()
    }

    pub fn get_file(&self,filename : &str) -> Result<VFile,TeXError> {
        use crate::utils::kpsewhich;
        match kpsewhich(filename,self.jobinfo.in_file()) {
            None =>TeXErr!(self,"File {} not found",filename),
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

    pub fn do_assignment(&self,a : Assignment,globally:bool) -> Result<(),TeXError> {
        let global = globally; // TODO!
        a.assign(self,global)
    }

    pub fn get_command(&self,s : &TeXString) -> Result<TeXCommand,TeXError> {
        match self.state.borrow().get_command(s) {
            Some(p) => Ok(p),
            None => TeXErr!(self,"Unknown control sequence: \\{}",s)
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
                p = match p.as_expandable_with_protected() {
                    Ok(e) => return e.expand(next,self),
                    Err(x) => x
                };
                match p {
                    //TeXCommand::Register(_) | TeXCommand::Dimen(_) => return self.do_assignment(p,false),
                    TeXCommand::Primitive(p) if *p == primitives::PAR && matches!(self.mode,TeXMode::Vertical) => Ok(()),
                    TeXCommand::Primitive(p) => match p.apply(next,self)? {
                        None => Ok(()),
                        Some(e) => Ok(self.push_expansion(e))
                    },
                    TeXCommand::Ext(exec) =>
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