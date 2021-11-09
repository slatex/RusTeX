pub enum TeXMode {
    Vertical, InternalVertical, Horizontal, RestrictedHorizontal, Math, Displaymath, Script, ScriptScript
}

use std::any::Any;
use std::borrow::{Borrow, BorrowMut};
use std::collections::HashMap;
use std::ops::Deref;
use crate::ontology::{CharacterToken, LaTeXFile, PrimitiveCharacterToken, PrimitiveToken, Token};
use crate::catcodes::CategoryCodeScheme;
use crate::references::SourceReference;
use std::path::{Path, PathBuf};
use std::rc::Rc;
use std::str::FromStr;
use crate::commands::TeXCommand;
use crate::interpreter::files::{FileStore, VFile};
use crate::interpreter::mouth::Mouths;
use crate::interpreter::state::{RegisterStateChange, State, StateChange};

pub mod mouth;
pub mod state;
mod files;

fn tokenize(s : &str,cats: &CategoryCodeScheme) -> Vec<PrimitiveCharacterToken> {
    let mut ns = s.as_bytes();
    let mut retvec: Vec<PrimitiveCharacterToken> = Vec::new();
    for next in ns {
        let b = match cats.get_code(*next) {
            cc =>
                PrimitiveCharacterToken::new(*next,cc,SourceReference::None)
        };
        retvec.push(b)
    }
    retvec
}

pub struct Jobinfo<'a> {
    path : &'a Path
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

pub struct Interpreter<'state,'inner> {
    pub state:State<'state>,
    pub jobinfo:Jobinfo<'inner>,
    mouths:Mouths,
    filestore:FileStore
}
impl Interpreter<'_,'_> {
    pub fn string_to_tokens(s : &str) -> Vec<PrimitiveCharacterToken> {
        use std::mem;
        use crate::catcodes::OTHER_SCHEME;
        tokenize(s,&OTHER_SCHEME)
    }
    pub fn do_file_with_state<'a,'b>(p : &'b Path, s : State<'a>) -> State<'a> {
        let mut int = Interpreter {
            state:s,
            jobinfo:Jobinfo::new(p),
            mouths:Mouths::new(),
            filestore:FileStore {
                files:HashMap::new()
            }
        };
        let vf:VFile  = VFile::new(p,int.jobinfo.in_file(),int.filestore.borrow_mut());
        int.push_file(vf);
        while int.has_next() {
            match int.do_top() {
                Ok(_) => {},
                Err(s) => panic!("{}",s)
            }
        }
        int.state
    }

    pub fn do_assignment(&mut self,p : &TeXCommand,globally:bool) -> Result<(),String> {
        let global = globally; // TODO!
        match p {
            TeXCommand::Register(reg) => {
                self.read_eq();
                let num = self.read_number();
                match num {
                    Err(s) => return Err(s),
                    Ok(i) => {
                        /*
                        self.state.change(StateChange::Register(RegisterStateChange {
                            index: reg.index,
                            value: i,
                            global
                        })); */
                        Ok(())
                    }
                }
            }
            _ => todo!()
        }
    }

    pub fn get_command(&self,s : &str) -> Result<&TeXCommand,String> {
        match self.state.get_command(s) {
            Some(p) => Ok(p),
            None => Err("Unknown control sequence: ".to_owned() + s)
        }
    }

    pub fn do_top(&mut self) -> Result<(),String> {
        use crate::commands::{TeXCommand,RegisterReference};
        let next = self.mouths.next_token(&self.state);
        println!("85: {}",next.as_string());
        match next.deref() {
            Token::Command(cmd) => {
                let proc = self.get_command(cmd.name());
                match proc {
                    Ok(p) => {
                        match p {
                            TeXCommand::Register(reg) => return self.do_assignment(p,false),
                            _ => todo!()
                        }
                    },
                    Err(s) => Err(s)
                }
            },
            Token::Char(ch) => todo!()
        }
    }

    pub fn skip_ws(&mut self) {
        use crate::catcodes::CategoryCode;
        while self.has_next() {
            let next = self.mouths.next_token(&self.state);
            println!("114: {}",next.as_string());
            match next.deref() {
                Token::Char(ch) =>
                match ch.catcode() {
                    CategoryCode::Space | CategoryCode::EOL => {}
                    _ => {
                        self.mouths.requeue(next);
                        break
                    }
                }
                _ => {
                    self.mouths.requeue(next);
                    break
                }
            }
        }
    }

    pub fn read_eq(&mut self) {
        use crate::catcodes::CategoryCode;
        self.skip_ws();
        let next = self.mouths.next_token(&self.state);
        println!("136: {}",next.as_string());
        match next.deref() {
            Token::Char(ch) =>
                match ch.get_char() {
                    61 => {
                        let next = self.mouths.next_token(&self.state);
                        println!("142: {}",next.as_string());
                        match next.deref() {
                            Token::Char(ch) =>
                                match ch.catcode() {
                                    CategoryCode::Space => {},
                                    _ => self.mouths.requeue(next)
                                }
                            _ => self.mouths.requeue(next)
                        }
                    }
                    _ => self.mouths.requeue(next)
                }
            _ => self.mouths.requeue(next)
        }
    }

    pub fn read_number(&mut self) -> Result<i32,String> {
        use crate::catcodes::CategoryCode;
        use crate::commands::{TeXCommand,RegisterReference,DimenReference};
        use std::str;
        let mut isnegative = false;
        let mut ishex = false;
        let mut ret = "".to_string();
        self.skip_ws();
        while self.has_next() {
            let next = self.mouths.next_token(&self.state);
            println!("166: {}",next.as_string());
            match next.deref() {
                Token::Char(ct) =>
                    match ct.catcode() {
                        CategoryCode::Space | CategoryCode::EOL if !ret.is_empty() =>
                            {
                                let num = if ishex {
                                    i32::from_str_radix(ret.as_str(),16)
                                } else {
                                    i32::from_str(ret.as_str())
                                };
                                match num {
                                    Ok(n) => return Ok((if isnegative {-n} else {n})),
                                    Err(s) => return Err("Number error (should be impossible)".to_string())
                                }
                            }
                        _ if ct.get_char().is_ascii_digit() =>
                            {
                                ret += str::from_utf8(&[ct.get_char()]).unwrap()
                            }
                        _ if ct.get_char().is_ascii_hexdigit() && ishex =>
                            {
                                ret += str::from_utf8(&[ct.get_char()]).unwrap()
                            }
                        _ =>
                            todo!("{}",next.as_string())
                    }
                Token::Command(cmd) =>
                    match self.get_command(cmd.name()) {
                        Err(s) => return Err(s),
                        Ok(p) => {
                            match p {
                                TeXCommand::Register(reg) => {
                                    if isnegative {
                                        return Ok(-self.state.get_register(reg.index))
                                    } else {
                                        return Ok(self.state.get_register(reg.index))
                                    }
                                }
                                _ => todo!("{}",cmd.as_string())
                            }
                        }

                    }
            }
        }
        Err("File ended unexpectedly".to_string())
    }

    fn expand_until_space(i:i32) -> Result<i32,String> {
        todo!()
    }
}

/*
use crate::interpreter::state::{State,default_pdf_latex_state};
use crate::utils::kpsewhich;
use crate::interpreter::files::VFile;

pub struct Interpreter<'state,'mouths> {
    state : State<'state>,
    pub mode : TeXMode,
    job : Option<PathBuf>,
    mouths:Mouths<'mouths>
}

use std::rc::Rc;
use crate::interpreter::mouth::Mouths;
use crate::utils::PWD;


impl<'state,'mouths> Interpreter<'state,'mouths> {
    pub fn new<'a,'b>() -> Interpreter<'a,'b> {
        let mut ret = Interpreter {
            state: default_pdf_latex_state(),
            mode: TeXMode::Vertical,
            mouths:Mouths::new(),
            job:None
        };
        //ret.state = Some(default_pdf_latex_state());
        //ret
        todo!()
    }
    pub fn new_from_state<'a>(state:State<'a>) ->Interpreter {
        let mut ret = Interpreter {
            state:state,
            mode: TeXMode::Vertical,
            mouths:Mouths::new(),
            job:None
        };
        ret
    }


    pub(in crate::interpreter) fn kill_state<'b>(&mut self) -> State<'b> {
        //self.state
        todo!()
    }

    pub fn jobname(&self) -> &str {
        let job = self.job.as_ref().expect("Interpreter without running job has no jobname");
        job.file_stem().unwrap().to_str().unwrap()
    }
    fn in_file(&self) -> &Path {
        self.job.as_ref().expect("Interpreter without running job has no jobname").parent().unwrap()
    }

    pub fn kpsewhich(&self,filename:&str) -> PathBuf {
        match kpsewhich(filename,self.in_file()) {
            None => PathBuf::from(self.in_file().to_str().unwrap().to_owned() + "/" + filename).canonicalize().unwrap(),
            Some(fp) => fp
        }
    }

    pub fn do_file(&'mouths mut self, file:&Path) {
        if !file.exists() {
            return ()//Result::Err("File does not exist")
        }
        //self.job = Some(file.canonicalize().expect("File name not canonicalizable").to_path_buf());
        //let vf = self.borrow_mut().getvf(file);
        let mut vf = VFile::new(file,self);
        self.mouths.push_file(&self.state,vf);
        todo!("interpreter.rs 101");
        while self.mouths.has_next() {
            self.do_v_mode()
        }
    }

    fn do_v_mode<'a>(&'a mut self) {
        todo!("interpreter.rs 105")
    }

}

 */
