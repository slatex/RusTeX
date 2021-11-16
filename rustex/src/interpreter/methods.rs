use std::borrow::BorrowMut;
use crate::catcodes::CategoryCode;
use crate::interpreter::Interpreter;
use crate::ontology::Token;
use crate::utils::TeXError;
use std::str::FromStr;
use crate::commands::{Expandable, TeXCommand};
use crate::{TeXErr,FileEnd};
use crate::commands::primitives::NOEXPAND;


impl Interpreter<'_> {

    // General -------------------------------------------------------------------------------------

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

    pub fn read_keyword(&self,mut kws:Vec<&str>) -> Result<Option<String>,TeXError> {
        use std::str;
        let mut tokens:Vec<Token> = Vec::new();
        let mut ret : String = "".to_string();
        self.skip_ws();
        while self.has_next() {
            let next = self.next_token();
            match next.catcode {
                CategoryCode::Space | CategoryCode::EOL => break,
                CategoryCode::Active | CategoryCode::Escape => {
                     match self.get_command(&next.cmdname())?.as_expandable_with_protected() {
                        Ok(e) => {e.expand(next,self)?;}
                        Err(_) => {
                            tokens.push(next);
                            break;
                        }
                    }
                },
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
            Ok(Some(ret))
        } else {
            self.push_tokens(tokens);
            Ok(None)
        }
    }

    pub fn read_string(&self) -> Result<String,TeXError> {
        use std::str::from_utf8;
        let mut ret : Vec<u8> = Vec::new();
        let mut quoted = false;
        self.skip_ws();
        while self.has_next() {
            let next = self.next_token();
            match next.catcode {
                CategoryCode::Escape | CategoryCode::Active => todo!(),
                CategoryCode::Space | CategoryCode::EOL if !quoted => return Ok(from_utf8(ret.as_slice()).unwrap().to_owned()),
                CategoryCode::BeginGroup if ret.is_empty() => todo!(),
                _ if next.char == 34 && !quoted => quoted = true,
                _ if next.char == 34 => {
                    self.skip_ws();
                    return Ok(from_utf8(ret.as_slice()).unwrap().to_owned())
                }
                _ => ret.push(next.char)
            }
        }
        FileEnd!(self)
    }

    pub fn read_command_token(&self) -> Result<Token,TeXError> {
        let mut cmd: Option<Token> = None;
        while self.has_next() {
            self.skip_ws();
            let next = self.next_token();
            match next.catcode {
                CategoryCode::Escape | CategoryCode::Active => {
                    let p = self.state_get_command(&next.cmdname());
                    match p {
                        None =>{ cmd = Some(next); break }
                        Some(p) => match p {
                            TeXCommand::Cond(c) => { c.expand(next, self)?; },
                            TeXCommand::Primitive(p) if p.expandable => Expandable::Primitive(p).expand(next, self)?,
                            _ => { cmd = Some(next); break }
                        }
                    }
                }
                _ => TeXErr!(self,"Command expected; found: {}",next)
            }
        };
        match cmd {
            Some(t) => Ok(t),
            _ => FileEnd!(self)
        }
    }

    // Token lists ---------------------------------------------------------------------------------

    #[inline(always)]
    pub fn read_argument(&self) -> Result<Vec<Token>,TeXError> {
        let next = self.next_token();
        if next.catcode != CategoryCode::BeginGroup {
            return Ok(vec!(next))
        }
        self.read_token_list(false,false)
    }

    #[inline(always)]
    pub fn read_token_list(&self,expand:bool,the:bool) -> Result<Vec<Token>,TeXError> {
        self.read_token_list_map(expand,the,Box::new(|x,_| Ok(Some(x))))
    }

    #[inline(always)]
    pub fn read_token_list_map<'a,T>(&self,expand:bool,the:bool,f:Box<dyn Fn(Token,&Interpreter) -> Result<Option<T>,TeXError> + 'a>) -> Result<Vec<T>,TeXError> {
        use crate::commands::primitives::THE;
        use crate::commands::etex::UNEXPANDED;
        use std::rc::Rc;
        let mut ingroups : i8 = 0;
        let mut ret : Vec<T> = vec!();
        while self.has_next() {
            let next = self.next_token();
            match next.catcode {
                CategoryCode::Active | CategoryCode::Escape if expand => {
                    let cmd = self.get_command(&next.cmdname())?.as_expandable();
                    match cmd {
                        Ok(Expandable::Primitive(x)) if *x == NOEXPAND => {
                            self.assert_has_next()?;
                            for t in (f)(self.next_token(),self)? {ret.push(t)}
                        }
                        Ok(Expandable::Primitive(x)) if the && (*x == THE || *x == UNEXPANDED) => {
                            match (x._apply)(next,self)? {
                                Some(e) => {
                                    let rc = Rc::new(e);
                                    for tk in &rc.exp {
                                        for t in (f)(tk.copied(Rc::clone(&rc)),self)? {ret.push(t)}
                                    }
                                }
                                None => ()
                            }
                        }
                        Ok(e) => e.expand(next,self)?,
                        Err(_) => for t in (f)(next,self)? {ret.push(t)}
                    }
                }
                CategoryCode::EndGroup if ingroups == 0 => return Ok(ret),
                CategoryCode::BeginGroup => {
                    ingroups += 1;
                    for t in (f)(next,self)? {ret.push(t)};
                }
                CategoryCode::EndGroup => {
                    ingroups -= 1;
                    for t in (f)(next,self)? {ret.push(t)};
                }
                _ => for t in (f)(next,self)? {ret.push(t)}
            }
        }
        FileEnd!(self)
    }

    // Numbers -------------------------------------------------------------------------------------

    fn num_do_ret(&self,ishex:bool,isnegative:bool,ret:String) -> Result<i32,TeXError> {
        let num = if ishex {
            i32::from_str_radix(ret.as_str(), 16)
        } else {
            i32::from_str(ret.as_str())
        };
        match num {
            Ok(n) => return Ok(if isnegative { -n } else { n }),
            Err(_s) => TeXErr!(self,"Number error (should be impossible)")
        }
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
        FileEnd!(self)
    }

    pub fn read_number(&self) -> Result<i32,TeXError> {
        let mut isnegative = false;
        let mut ishex = false;
        let mut ret = "".to_string();
        self.skip_ws();
        while self.has_next() {
            let next = self.next_token();
            match next.catcode {
                CategoryCode::Escape | CategoryCode::Active if ret.is_empty() && !ishex => {
                    let p = self.get_command(&next.cmdname())?;
                    match p.as_hasnum() {
                        Ok(hn) => return Ok(if isnegative { -hn.get(self)? } else { hn.get(self)? }),
                        Err(p) => match p.as_expandable_with_protected() {
                            Ok(e) => e.expand(next,self)?,
                            Err(e) => match e {
                                TeXCommand::Char((_,tk)) => return Ok(if isnegative {-(tk.char as i32)} else {tk.char as i32}),
                                _ => TeXErr!(self,"Number expected; found {}",next)
                            }
                        }
                    };
                }
                CategoryCode::Escape | CategoryCode::Active => {
                    let p = self.get_command(&next.cmdname())?;
                    match p.as_expandable_with_protected() {
                        Ok(e) => e.expand(next,self)?,
                        _ => {
                            self.requeue(next);
                            return self.num_do_ret(ishex,isnegative,ret)
                        }
                    }
                }
                CategoryCode::Space | CategoryCode::EOL if !ret.is_empty() => return self.num_do_ret(ishex,isnegative,ret),
                _ if next.char.is_ascii_digit() => ret += &next.name(),
                _ if next.char.is_ascii_hexdigit() && ishex => ret += &next.name(),
                _ if next.char == 45 && ret.is_empty() => isnegative = !isnegative,
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
                _ => todo!("{},{}: {}",next.as_string(),self.current_line(),self.preview())
            }
        }
        FileEnd!(self)
    }

    // Dimensions ----------------------------------------------------------------------------------

    fn point_to_int(&self,f:f32) -> Result<i32,TeXError> {
        use crate::interpreter::dimensions::*;
        let _istrue = self.read_keyword(vec!("true"))?.is_some();
        match self.read_keyword(vec!("sp","pt","pc","in","bp","cm","mm","dd","cc","em","ex","px","mu"))? {
            Some(s) if s == "mm" => Ok(mm(f).round() as i32),
            Some(s) if s == "in" => Ok(inch(f).round() as i32),
            Some(o) => todo!("{}",o),
            None => todo!("{}",self.current_line())
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
                CategoryCode::Escape | CategoryCode::Active if ret.is_empty() =>
                    {
                        let p = self.get_command(&next.cmdname())?;
                        match p {
                            _ => todo!("{}",next.as_string())
                        }
                    }
                CategoryCode::Space | CategoryCode::EOL if !ret.is_empty() =>
                    {
                        let num = f32::from_str(ret.as_str());
                        match num {
                            Ok(n) => return self.point_to_int(if isnegative {-n} else {n}),
                            Err(_s) => TeXErr!(self,"Number error (should be impossible)")
                        }
                    }
                _ if next.char.is_ascii_digit() => ret += &next.name(),
                _ if next.char == 46 && !isfloat => { isfloat = true; ret += "." }
                _ => todo!("{}",next.as_string())
            }
        }
        FileEnd!(self)
    }
}