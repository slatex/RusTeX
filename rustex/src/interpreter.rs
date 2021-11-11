pub enum TeXMode {
    Vertical, InternalVertical, Horizontal, RestrictedHorizontal, Math, Displaymath, Script, ScriptScript
}

use std::borrow::BorrowMut;
use std::collections::HashMap;
use std::ops::Deref;
use crate::ontology::{PrimitiveCharacterToken, Token};
use crate::catcodes::{CategoryCode, CategoryCodeScheme};
use crate::references::SourceReference;
use std::path::Path;
use std::rc::Rc;
use std::str::FromStr;
use crate::commands::TeXCommand;
use crate::interpreter::files::{FileStore, VFile};
use crate::interpreter::mouth::Mouths;
use crate::interpreter::state::{RegisterStateChange, State, StateChange};

pub mod mouth;
pub mod state;
mod files;
pub mod dimensions;

fn tokenize(s : &str,cats: &CategoryCodeScheme) -> Vec<PrimitiveCharacterToken> {
    let ns = s.as_bytes();
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
    pub state:State<'state>,
    pub jobinfo:Jobinfo<'inner>,
    mouths:Mouths,
    filestore:FileStore,
    mode:TeXMode
}
impl Interpreter<'_,'_> {
    pub fn string_to_tokens(s : &str) -> Vec<PrimitiveCharacterToken> {
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
            },
            mode:TeXMode::Vertical
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

    pub fn do_assignment(&mut self,p : Rc<TeXCommand>,globally:bool) -> Result<(),String> {
        let global = globally; // TODO!
        match p.deref() {
            TeXCommand::Dimen(reg) => {
                self.read_eq();
                let dim = self.read_dimension();
                match dim {
                    Err(s) => Err(s),
                    Ok(i) => {
                        self.state.change(StateChange::Dimen(RegisterStateChange {
                            index: reg.index,
                            value: i,
                            global
                        }));
                        Ok(())
                    }
                }
            }
            TeXCommand::Register(reg) => {
                self.read_eq();
                let num = self.read_number();
                match num {
                    Err(s) => Err(s),
                    Ok(i) => {
                        self.state.change(StateChange::Register(RegisterStateChange {
                            index: reg.index,
                            value: i,
                            global
                        }));
                        Ok(())
                    }
                }
            }
            _ => todo!()
        }
    }

    pub fn get_command(&self,s : &str) -> Result<Rc<TeXCommand>,String> {
        match self.state.get_command(s) {
            Some(p) => Ok(p),
            None => Err("Unknown control sequence: ".to_owned() + s + " at " + self.current_line().as_str())
        }
    }

    pub fn do_top(&mut self) -> Result<(),String> {
        use crate::commands::primitives;
        let next = self.mouths.next_token(&self.state);
        match next.deref() {
            Token::Command(cmd) => {
                let p = match self.state.get_command(cmd.name()) {
                    Some(pr) => pr,
                    None => return Err("Unknown control sequence: ".to_owned() + cmd.name() + " at " + self.current_line().as_str())
                };
                match p.deref() {
                    TeXCommand::Register(_reg) => return self.do_assignment(p,false),
                    TeXCommand::Dimen(_reg) => return self.do_assignment(p,false),
                    TeXCommand::Primitive(p) if **p == primitives::PAR && matches!(self.mode,TeXMode::Vertical) => Ok(()),
                    TeXCommand::Ext(exec) =>
                        match exec.execute(self) {
                            true => Ok(()),
                            false => Err("External Command ".to_owned() + exec.name().as_str() + " errored!")
                        }
                    _ => todo!("{}",cmd.as_string())

                }
            },
            Token::Char(ch) if matches!(ch.catcode(),CategoryCode::Space) || matches!(ch.catcode(),CategoryCode::EOL) => Ok(()),
            Token::Char(ch) => todo!("Character: {}, {}",ch.get_char(),ch.catcode())
        }
    }

    pub fn skip_ws(&mut self) {
        while self.has_next() {
            let next = self.mouths.next_token(&self.state);
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
        self.skip_ws();
        let next = self.mouths.next_token(&self.state);
        match next.deref() {
            Token::Char(ch) =>
                match ch.get_char() {
                    61 => {
                        let next = self.mouths.next_token(&self.state);
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

    fn point_to_int(&mut self,f:f32) -> i32 {
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
        self.mouths.current_line()
    }

    pub fn read_keyword(&mut self,mut kws:Vec<&str>) -> Option<String> {
        use std::str;
        let mut tokens:Vec<Rc<Token>> = Vec::new();
        let mut ret : String = "".to_string();
        self.skip_ws();
        while self.has_next() {
            let next = self.mouths.next_token(&self.state);
            match next.deref() {
                Token::Char(ct) if matches!(ct.catcode(),CategoryCode::Space) || matches!(ct.catcode(),CategoryCode::EOL) => break,
                Token::Char(ct) => {
                    let ret2 = ret.clone() + str::from_utf8(&[ct.get_char()]).unwrap();
                    if kws.iter().any(|x| x.starts_with(&ret2)) {
                        kws = kws.iter().filter(|s| s.starts_with(&ret2)).map(|x| *x).collect();
                        ret = ret2;
                        tokens.push(Rc::clone(&next));
                        if kws.is_empty() { break }
                        if kws.len() == 1 && kws.contains(&ret.as_str()) { break }
                    } else {
                        if kws.len() == 1 && kws.contains(&ret.as_str()) {
                            self.mouths.requeue(next);
                        } else {
                            tokens.push(Rc::clone(&next));
                        }
                        break
                    }
                }
                _ => todo!("{}",next.as_string())
            }
        }
        if kws.len() == 1 && kws.contains(&ret.as_str()) {
            Some(ret)
        } else {
            self.mouths.push_tokens(tokens);
            None
        }
    }

    pub fn read_dimension(&mut self) -> Result<i32,String> {
        use std::str;
        let mut isnegative = false;
        let mut ret = "".to_string();
        let mut isfloat = false;
        self.skip_ws();
        while self.has_next() {
            let next = self.mouths.next_token(&self.state);
            match next.deref() {
                Token::Char(ct) =>
                    match ct.catcode() {
                        CategoryCode::Space | CategoryCode::EOL if !ret.is_empty() =>
                            {
                                let num = f32::from_str(ret.as_str());
                                match num {
                                    Ok(n) => return Ok(self.point_to_int(if isnegative {-n} else {n})),
                                    Err(_s) => return Err("Number error (should be impossible)".to_string())
                                }
                            }
                        _ if ct.get_char().is_ascii_digit() =>
                            {
                                ret += str::from_utf8(&[ct.get_char()]).unwrap()
                            }
                        _ if ct.get_char() == 46 && !isfloat =>
                            {
                                isfloat = true;
                                ret += "."
                            }
                        _ =>
                            todo!("{}",next.as_string())
                    }
                Token::Command(cmd) =>
                    match self.get_command(cmd.name()) {
                        Err(s) => return Err(s),
                        Ok(p) => {
                            match p.deref() {
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

    pub fn read_number(&mut self) -> Result<i32,String> {
        use std::str;
        let mut isnegative = false;
        let mut ishex = false;
        let mut ret = "".to_string();
        self.skip_ws();
        while self.has_next() {
            let next = self.mouths.next_token(&self.state);
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
                                    Ok(n) => return Ok(if isnegative {-n} else {n}),
                                    Err(_s) => return Err("Number error (should be impossible)".to_string())
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
                            match p.deref() {
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

    fn expand_until_space(_i:i32) -> Result<i32,String> {
        todo!()
    }
}