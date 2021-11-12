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
use std::str::{from_utf8, FromStr};
use crate::commands::TeXCommand;
use crate::interpreter::files::{FileStore, VFile};
use crate::interpreter::mouth::Mouths;
use crate::interpreter::state::{RegisterStateChange, State, StateChange};
use crate::utils::TeXError;

pub mod mouth;
pub mod state;
mod files;
pub mod dimensions;

fn tokenize(s : &str,cats: &CategoryCodeScheme) -> Vec<Token> {
    let ns = s.as_bytes();
    let mut retvec: Vec<Token> = Vec::new();
    for next in ns {
        retvec.push(Token {
            catcode: cats.get_code(*next),
            nameOpt: None,
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

pub struct Interpreter<'state,'inner> {
    state:RefCell<State<'state>>,
    pub jobinfo:Jobinfo<'inner>,
    mouths:RefCell<Mouths>,
    filestore:FileStore,
    mode:TeXMode
}
impl Interpreter<'_,'_> {
    pub fn string_to_tokens(s : &str) -> Vec<Token> {
        use crate::catcodes::OTHER_SCHEME;
        tokenize(s,&OTHER_SCHEME)
    }
    pub fn do_file_with_state<'a,'b>(p : &'b Path, s : State<'a>) -> State<'a> {
        let mut int = Interpreter {
            state:RefCell::new(s),
            jobinfo:Jobinfo::new(p),
            mouths:RefCell::new(Mouths::new()),
            filestore:FileStore {
                files:HashMap::new()
            },
            mode:TeXMode::Vertical
        };
        let vf:VFile  = VFile::new(p,int.jobinfo.in_file(),int.filestore.borrow_mut());
        int.push_file(vf);
        while int.has_next() {
            match int.do_top() {
                Ok(_) => {},
                Err(s) => s.throw()
            }
        }
        let ret = int.state.borrow().clone(); ret
    }

    pub fn do_assignment(&self,p : Rc<TeXCommand>,globally:bool) -> Result<(),TeXError> {
        let global = globally; // TODO!
        match p.deref() {
            TeXCommand::Dimen(reg) => {
                self.read_eq();
                let dim = self.read_dimension()?;
                self.change_state(StateChange::Register(RegisterStateChange {
                    index: reg.index,
                    value: dim,
                    global
                }));
                Ok(())
            }
            TeXCommand::Register(reg) => {
                self.read_eq();
                let num = self.read_number()?;
                self.change_state(StateChange::Register(RegisterStateChange {
                    index: reg.index,
                    value: num,
                    global
                }));
                Ok(())
            }
            _ => todo!()
        }
    }

    pub fn get_command(&self,s : &str) -> Result<Rc<TeXCommand>,TeXError> {
        match self.state.borrow().get_command(s) {
            Some(p) => Ok(p),
            None => Err(TeXError::new("Unknown control sequence: ".to_owned() + s + " at " + self.current_line().as_str()))
        }
    }

    pub fn do_top(&mut self) -> Result<(),TeXError> {
        use crate::commands::primitives;
        let next = self.next_token();
        match next.catcode {
            CategoryCode::Active | CategoryCode::Escape => {
                let p = self.get_command(&next.cmdname())?;
                match p.deref() {
                    TeXCommand::Register(_reg) => return self.do_assignment(p,false),
                    TeXCommand::Dimen(_reg) => return self.do_assignment(p,false),
                    TeXCommand::Primitive(p) if **p == primitives::PAR && matches!(self.mode,TeXMode::Vertical) => Ok(()),
                    TeXCommand::Primitive(p) => {
                            let ret = p.apply(next,self)?;
                            self.push_expansion(ret);
                            Ok(())
                        }
                    TeXCommand::Ext(exec) =>
                        match exec.execute(self) {
                            true => Ok(()),
                            false => Err(TeXError::new("External Command ".to_owned() + exec.name().as_str() + " errored!"))
                        }
                    _ => todo!("{}",next.as_string())

                }
            },
            CategoryCode::Space | CategoryCode::EOL if matches!(self.mode,TeXMode::Vertical) => Ok(()),
            _ => todo!("Character: {}, {}",next.char,next.catcode)
        }
    }

    pub fn skip_ws(&self) {
        while self.has_next() {
            let next = self.next_token();
            match next.catcode {
                CategoryCode::Space | CategoryCode::EOL => {}
                _ => {
                    self.requeue(next);
                    break
                }
            }
        }
    }

    pub fn read_eq(&self) {
        self.skip_ws();
        let next = self.next_token();
        match next.char {
            61 => {
                let next = self.next_token();
                match next.catcode {
                    CategoryCode::Space => {},
                    _ => self.requeue(next)
                }
            }
            _ => self.requeue(next)
        }
    }

    fn point_to_int(&self,f:f32) -> i32 {
        use crate::interpreter::dimensions::*;
        let _istrue = self.read_keyword(vec!("true")).is_some();
        match self.read_keyword(vec!("sp","pt","pc","in","bp","cm","mm","dd","cc","em","ex","px","mu")) {
            Some(s) if s == "mm" => mm(f).round() as i32,
            Some(s) if s == "in" => inch(f).round() as i32,
            Some(o) => todo!("{}",o),
            None => todo!("{}",self.current_line())
        }
    }

    fn current_line(&self) -> String {
        self.mouths.borrow().current_line()
    }

    pub fn read_keyword(&self,mut kws:Vec<&str>) -> Option<String> {
        use std::str;
        let mut tokens:Vec<Token> = Vec::new();
        let mut ret : String = "".to_string();
        self.skip_ws();
        while self.has_next() {
            let next = self.next_token();
            match next.catcode {
                CategoryCode::Space | CategoryCode::EOL => break,
                CategoryCode::Active | CategoryCode::Escape => todo!("{}",next.as_string()),
                _ => {
                    let ret2 = ret.clone() + str::from_utf8(&[next.char]).unwrap();
                    if kws.iter().any(|x| x.starts_with(&ret2)) {
                        kws = kws.iter().filter(|s| s.starts_with(&ret2)).map(|x| *x).collect();
                        ret = ret2;
                        tokens.push(next);
                        if kws.is_empty() { break }
                        if kws.len() == 1 && kws.contains(&ret.as_str()) { break }
                    } else {
                        if kws.len() == 1 && kws.contains(&ret.as_str()) {
                            self.requeue(next);
                        } else {
                            tokens.push(next);
                        }
                        break
                    }
                }
            }
        }
        if kws.len() == 1 && kws.contains(&ret.as_str()) {
            Some(ret)
        } else {
            self.push_tokens(tokens);
            None
        }
    }

    pub fn read_dimension(&self) -> Result<i32,TeXError> {
        use std::str;
        let mut isnegative = false;
        let mut ret = "".to_string();
        let mut isfloat = false;
        self.skip_ws();
        while self.has_next() {
            let next = self.next_token();
            match next.catcode {
                CategoryCode::Escape | CategoryCode::Active =>
                    {
                        let p = self.get_command(&next.cmdname())?;
                        match p.deref() {
                            TeXCommand::Dimen(reg) if ret.is_empty() => {
                                if isnegative {
                                    return Ok(-self.state_dimension(reg.index))
                                } else {
                                    return Ok(self.state_dimension(reg.index))
                                }
                            }
                            _ => todo!("{}",next.as_string())
                        }
                    }
                CategoryCode::Space | CategoryCode::EOL if !ret.is_empty() =>
                    {
                        let num = f32::from_str(ret.as_str());
                        match num {
                            Ok(n) => return Ok(self.point_to_int(if isnegative {-n} else {n})),
                            Err(_s) => return Err(TeXError::new("Number error (should be impossible)".to_string()))
                        }
                    }
                _ if next.char.is_ascii_digit() =>
                    {
                        ret += &next.name()
                    }
                _ if next.char == 46 && !isfloat =>
                    {
                        isfloat = true;
                        ret += "."
                    }
                _ =>
                    todo!("{}",next.as_string())
            }
        }
        Err(TeXError::new("File ended unexpectedly".to_string()))
    }

    pub fn read_number(&self) -> Result<i32,TeXError> {
        use std::str;
        let mut isnegative = false;
        let mut ishex = false;
        let mut ret = "".to_string();
        self.skip_ws();
        while self.has_next() {
            let next = self.next_token();
            match next.catcode {
                CategoryCode::Escape | CategoryCode::Active =>
                    match self.get_command(&next.cmdname()) {
                        Err(s) => return Err(s),
                        Ok(p) => {
                            match p.deref() {
                                TeXCommand::Register(reg) => {
                                    if isnegative {
                                        return Ok(-self.state_register(reg.index))
                                    } else {
                                        return Ok(self.state_register(reg.index))
                                    }
                                }
                                _ => todo!("{}",next.as_string())
                            }
                        }
                    }
                CategoryCode::Space | CategoryCode::EOL if !ret.is_empty() =>
                    {
                        let num = if ishex {
                            i32::from_str_radix(ret.as_str(),16)
                        } else {
                            i32::from_str(ret.as_str())
                        };
                        match num {
                            Ok(n) => return Ok(if isnegative {-n} else {n}),
                            Err(_s) => return Err(TeXError::new("Number error (should be impossible)".to_string()))
                        }
                    }
                _ if next.char.is_ascii_digit() =>
                    {
                        ret += &next.name()
                    }
                _ if next.char.is_ascii_hexdigit() && ishex =>
                    {
                        ret += &next.name()
                    }
                _ =>
                    todo!("{}",next.as_string())
            }
        }
        Err(TeXError::new("File ended unexpectedly".to_string()))
    }

    fn expand_until_space(_i:i32) -> Result<i32,TeXError> {
        todo!()
    }
}