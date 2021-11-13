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

pub struct Interpreter<'state,'inner> {
    state:RefCell<State<'state>>,
    pub jobinfo:Jobinfo<'inner>,
    mouths:RefCell<Mouths>,
    filestore:FileStore,
    mode:TeXMode,
    catcodes:RefCell<CategoryCodeScheme>
}
impl Interpreter<'_,'_> {
    pub fn string_to_tokens(s : &str) -> Vec<Token> {
        use crate::catcodes::OTHER_SCHEME;
        tokenize(s,&OTHER_SCHEME)
    }
    pub fn do_file_with_state<'a,'b>(p : &'b Path, s : State<'a>) -> State<'a> {
        let catcodes = s.catcodes().clone();
        let mut int = Interpreter {
            state:RefCell::new(s),
            jobinfo:Jobinfo::new(p),
            mouths:RefCell::new(Mouths::new()),
            filestore:FileStore {
                files:HashMap::new()
            },
            mode:TeXMode::Vertical,
            catcodes:RefCell::new(catcodes)
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

    pub fn do_assignment(&self,a : Assignment,globally:bool) -> Result<(),TeXError> {
        let global = globally; // TODO!
        a.assign(self,global)
    }

    pub fn get_command(&self,s : &str) -> Result<Rc<TeXCommand>,TeXError> {
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
                let p = self.get_command(&next.cmdname())?;
                match p.as_assignment() {
                    Some(a) => return self.do_assignment(a,false),
                    _ => {}
                }
                match p.as_expandable() {
                    Some(e) => return e.expand(next,self),
                    _ => {}
                }
                match p.deref() {
                    //TeXCommand::Register(_) | TeXCommand::Dimen(_) => return self.do_assignment(p,false),
                    TeXCommand::Primitive(p) if **p == primitives::PAR && matches!(self.mode,TeXMode::Vertical) => Ok(()),
                    TeXCommand::Primitive(p) => {
                            let ret = p.apply(next,self)?;
                            self.push_expansion(ret);
                            Ok(())
                        }
                    TeXCommand::Ext(exec) =>
                        exec.execute(self).map_err(|x| x.derive("External Command ".to_owned() + exec.name().as_str() + " errored!")),
                    _ => todo!("{}",next.as_string())

                }
            },
            CategoryCode::Space | CategoryCode::EOL if matches!(self.mode,TeXMode::Vertical) => Ok(()),
            _ => todo!("Character: {}, {}, {}",next.char,next.catcode,self.current_line())
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
                            /*
                            TeXCommand::Dimen(reg) if ret.is_empty() => {
                                if isnegative {
                                    return Ok(-self.state_dimension(reg.index))
                                } else {
                                    return Ok(self.state_dimension(reg.index))
                                }
                            }

                             */
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

    fn num_do_ret(&self,ishex:bool,isnegative:bool,ret:String) -> Result<i32,TeXError> {
        let num = if ishex {
            i32::from_str_radix(ret.as_str(), 16)
        } else {
            i32::from_str(ret.as_str())
        };
        match num {
            Ok(n) => return Ok(if isnegative { -n } else { n }),
            Err(_s) => return Err(TeXError::new("Number error (should be impossible)".to_string()))
        }
    }

    pub fn read_number(&self) -> Result<i32,TeXError> {
        let mut isnegative = false;
        let mut ishex = false;
        let mut ret = "".to_string();
        self.skip_ws();
        while self.has_next() {
            let next = self.next_token();
            match next.catcode {
                CategoryCode::Escape | CategoryCode::Active => {
                    let p = self.get_command(&next.cmdname())?;
                    match p.as_hasnum() {
                        Some(hn) => return hn.get(self),
                        None => return Err(TeXError::new("Number expected; found ".to_owned() + &next.cmdname()))
                    };
                }
                CategoryCode::Space | CategoryCode::EOL if !ret.is_empty() => return self.num_do_ret(ishex,isnegative,ret),
                _ if next.char.is_ascii_digit() =>
                    {
                        ret += &next.name()
                    }
                _ if next.char.is_ascii_hexdigit() && ishex =>
                    {
                        ret += &next.name()
                    }
                _ if next.char == 96 => while self.has_next() {
                    let next = self.next_token();
                    match next.catcode {
                        CategoryCode::Escape if next.cmdname().len() == 1 => {
                            let num = *next.cmdname().as_bytes().first().unwrap() as i32;
                            return self.expand_until_space(if isnegative { -num } else { num })
                        }
                        CategoryCode::Active | CategoryCode::Escape => todo!(),
                        _ => return self.expand_until_space(if isnegative {-(next.char as i32)} else {next.char as i32})
                    }
                }
                _ if !ret.is_empty() => {
                    self.requeue(next);
                    return self.num_do_ret(ishex,isnegative,ret)
                }
                _ => todo!("{},{}",next.as_string(),self.current_line())
            }
        }
        Err(TeXError::new("File ended unexpectedly".to_string()))
    }

    fn expand_until_space(&self,i:i32) -> Result<i32,TeXError> {
        while self.has_next() {
            let next = self.next_token();
            match next.catcode {
                CategoryCode::Active | CategoryCode::Escape => todo!(),
                CategoryCode::Space => return Ok(i),
                _ => {
                    self.requeue(next);
                    return Ok(i)
                }
            }
        }
        Err(TeXError::new("File ended unexpectedly".to_string()))
    }
}