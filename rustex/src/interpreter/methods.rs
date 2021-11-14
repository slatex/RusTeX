use crate::catcodes::CategoryCode;
use crate::interpreter::Interpreter;
use crate::ontology::Token;
use crate::utils::TeXError;
use std::str::FromStr;
use std::ops::Deref;
use crate::commands::TeXCommand;

impl Interpreter<'_,'_> {

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

    pub fn read_string(&self) -> Result<String,TeXError> {
        use std::str::from_utf8;
        let mut ret : Vec<u8> = Vec::new();
        let mut quoted = false;
        while self.has_next() {
            let next = self.next_token();
            match next.catcode {
                CategoryCode::Escape | CategoryCode::Active => todo!(),
                CategoryCode::Space | CategoryCode::EOL if !quoted => return Ok(from_utf8(ret.as_slice()).unwrap().to_owned()),
                CategoryCode::BeginGroup if ret.is_empty() => todo!(),
                _ => ret.push(next.char)
            }
        }
        Err(TeXError::new("File ended unexpectedly".to_string()))
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
            Err(_s) => return Err(TeXError::new("Number error (should be impossible)".to_string()))
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
        Err(TeXError::new("File ended unexpectedly".to_string()))
    }

    pub fn read_number(&self) -> Result<i32,TeXError> {
        let mut isnegative = false;
        let mut ishex = false;
        let mut ret = "".to_string();
        self.skip_ws();
        while self.has_next() {
            let next = self.next_token();
            match next.catcode {
                CategoryCode::Escape | CategoryCode::Active if ret.is_empty() => {
                    let p = self.get_command(&next.cmdname())?;
                    match p.as_hasnum() {
                        Ok(hn) => return hn.get(self),
                        _ => return Err(TeXError::new("Number expected; found ".to_owned() + &next.cmdname()))
                    };
                }
                CategoryCode::Space | CategoryCode::EOL if !ret.is_empty() => return self.num_do_ret(ishex,isnegative,ret),
                _ if next.char.is_ascii_digit() => ret += &next.name(),
                _ if next.char.is_ascii_hexdigit() && ishex => ret += &next.name(),
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

    // Dimensions ----------------------------------------------------------------------------------

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
                            Ok(n) => return Ok(self.point_to_int(if isnegative {-n} else {n})),
                            Err(_s) => return Err(TeXError::new("Number error (should be impossible)".to_string()))
                        }
                    }
                _ if next.char.is_ascii_digit() => ret += &next.name(),
                _ if next.char == 46 && !isfloat => { isfloat = true; ret += "." }
                _ => todo!("{}",next.as_string())
            }
        }
        Err(TeXError::new("File ended unexpectedly".to_string()))
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
                            TeXCommand::Cond(c) => { c.expand(next, self); },
                            TeXCommand::Primitive(p) if p.expandable =>
                                { self.push_expansion((p._apply)(next, self)?); }
                            _ => { cmd = Some(next); break }
                        }
                    }
                }
                _ => return Err(TeXError::new("Command expected; found: ".to_owned() + &next.as_string()))
            }
        };
        cmd.ok_or_else(|| TeXError::new("File ended unexpectedly".to_string()))
    }
}