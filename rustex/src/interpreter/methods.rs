use crate::catcodes::CategoryCode;
use crate::interpreter::Interpreter;
use crate::ontology::Token;
use crate::utils::{TeXError, TeXString};
use std::str::FromStr;
use crate::commands::{Expandable, TeXCommand, TokReference};
use crate::{TeXErr,FileEnd,log};
use crate::interpreter::dimensions::{Skip, Numeric, SkipDim};
use crate::utils::u8toi16;

impl Interpreter<'_> {

    // General -------------------------------------------------------------------------------------

    pub fn insert_every(&self,tr:&TokReference) {
        let i = -u8toi16(tr.index);
        self.push_tokens(self.state_tokens(i))
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
        if kws.len() >= 1 && kws.contains(&ret.as_str()) {
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

    pub fn read_argument(&self) -> Result<Vec<Token>,TeXError> {
        let next = self.next_token();
        if next.catcode == CategoryCode::BeginGroup {
            self.read_token_list(false,false)
        } else {
            Ok(vec!(next))
        }
    }

    #[inline(always)]
    pub fn read_token_list(&self,expand:bool,the:bool) -> Result<Vec<Token>,TeXError> {
        self.read_token_list_map(expand,the,Box::new(|x,_| Ok(Some(x))))
    }

    #[inline(always)]
    pub fn read_token_list_map<'a,T>(&self,expand:bool,the:bool,f:Box<dyn Fn(Token,&Interpreter) -> Result<Option<T>,TeXError> + 'a>) -> Result<Vec<T>,TeXError> {
        use crate::commands::primitives::{THE,UNEXPANDED};
        use std::rc::Rc;
        let mut ingroups : i8 = 0;
        let mut ret : Vec<T> = vec!();
        while self.has_next() {
            let next = self.next_token();
            match next.catcode {
                CategoryCode::Active | CategoryCode::Escape if expand && next.expand => {
                    let cmd = self.get_command(&next.cmdname())?;
                    match cmd.clone().as_expandable() {
                        Ok(Expandable::Primitive(x)) if the && (*x == THE || *x == UNEXPANDED) => {
                            match (x._apply)(&next,self)? {
                                Some(e) => {
                                    for tk in e {
                                        for t in (f)(tk.copied(&next,cmd.clone()),self)? {ret.push(t)}
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

    fn num_do_ret(&self,ishex:bool,isnegative:bool,allowfloat:bool,ret:TeXString) -> Result<Numeric,TeXError> {
        let num = if ishex {
            Numeric::Int(i32::from_str_radix(&ret.to_utf8(), 16).or_else(|_| TeXErr!(self,"Number error (should be impossible)"))?)
        } else if allowfloat {
            Numeric::Float(f32::from_str(&ret.to_utf8()).or_else(|_| TeXErr!(self,"Number error (should be impossible)"))?)
        } else {
            Numeric::Int(i32::from_str(&ret.to_utf8()).or_else(|_| TeXErr!(self,"Number error (should be impossible)"))?)
        };
        Ok(if isnegative {num.negate()} else {num})
    }

    pub fn expand_until(&self,eat_space:bool) -> Result<(),TeXError> {
        while self.has_next() {
            let next = self.next_token();
            match next.catcode {
                CategoryCode::Active | CategoryCode::Escape => {
                    let p = self.get_command(&next.cmdname())?;
                    match p.as_expandable_with_protected() {
                        Ok(p) => p.expand(next,self)?,
                        Err(TeXCommand::Char(tk)) if eat_space && tk.catcode == CategoryCode::Space => return Ok(()),
                        Err(_) => {
                            self.requeue(next);
                            return Ok(())
                        }
                    }
                },
                CategoryCode::Space if eat_space => return Ok(()),
                _ => {
                    self.requeue(next);
                    return Ok(())
                }
            }
        }
        FileEnd!(self)

    }
/*
    pub fn expand_until_space(&self,i:Numeric) -> Result<Numeric,TeXError> {
        while self.has_next() {
            let next = self.next_token();
            match next.catcode {
                CategoryCode::Active | CategoryCode::Escape => {
                    let p = self.get_command(&next.cmdname())?;
                    match p.as_expandable_with_protected() {
                        Ok(p) => p.expand(next,self)?,
                        Err(TeXCommand::Char(tk)) if tk.catcode == CategoryCode::Space => return Ok(i),
                        Err(_) => {
                            self.requeue(next);
                            return Ok(i)
                        }
                    }
                },
                CategoryCode::Space => return Ok(i),
                _ => {
                    self.requeue(next);
                    return Ok(i)
                }
            }
        }
        FileEnd!(self)
    }

 */

    pub(crate) fn read_number_i(&self,allowfloat:bool) -> Result<Numeric,TeXError> {
        let mut isnegative = false;
        let mut ishex = false;
        let mut isfloat = false;
        let mut ret : TeXString = "".into();
        self.skip_ws();
        log!("Reading number {}",self.preview());
        while self.has_next() {
            let next = self.next_token();
            match next.catcode {
                CategoryCode::Escape | CategoryCode::Active if ret.is_empty() && !ishex => {
                    let p = self.get_command(&next.cmdname())?;
                    match p.as_hasnum() {
                        Ok(hn) => return Ok(if isnegative { hn.get(self)?.negate() } else { hn.get(self)? }),
                        Err(p) => match p.as_expandable_with_protected() {
                            Ok(e) => e.expand(next,self)?,
                            Err(e) => match e {
                                TeXCommand::Char(tk) => return Ok(Numeric::Int(if isnegative {-(tk.char as i32)} else {tk.char as i32})),
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
                            return self.num_do_ret(ishex,isnegative,allowfloat,ret)
                        }
                    }
                }
                CategoryCode::Space | CategoryCode::EOL if !ret.is_empty() => return self.num_do_ret(ishex,isnegative,allowfloat,ret),
                _ if next.char.is_ascii_digit() => ret += next.name(),
                _ if next.char.is_ascii_hexdigit() && ishex => ret += next.name(),
                _ if next.char == 45 && ret.is_empty() => isnegative = !isnegative,
                _ if next.char == 46 && allowfloat && !isfloat => { isfloat = true; ret += ".".into() }
                _ if next.char == 96 => while self.has_next() {
                    let next = self.next_token();
                    match next.catcode {
                        CategoryCode::Escape if next.cmdname().len() == 1 => {
                            let num = *next.cmdname().0.first().unwrap() as i32;
                            self.expand_until(true)?;
                            return Ok(Numeric::Int(if isnegative { -num } else { num }))
                        }
                        CategoryCode::Active | CategoryCode::Escape => todo!("{} ({}) >>{}{}",self.current_line(),next.cmdname().len(),next,self.preview()),
                        _ => {
                            self.expand_until(true)?;
                            return Ok(Numeric::Int(if isnegative {-(next.char as i32)} else {next.char as i32}))
                        }
                    }
                }
                _ if !ret.is_empty() => {
                    self.requeue(next);
                    return self.num_do_ret(ishex,isnegative,allowfloat,ret)
                }
                _ => todo!("{},{}: {}",next,self.current_line(),self.preview())
            }
        }
        FileEnd!(self)
    }

    pub fn read_number(&self) -> Result<i32,TeXError> {
        match self.read_number_i(false)? {
            Numeric::Int(i) => Ok(i),
            Numeric::Dim(i) => Ok(i),
            Numeric::Float(_) => unreachable!(),
            Numeric::Skip(sk) => Ok(sk.base)
        }
    }

    // Dimensions ----------------------------------------------------------------------------------

    fn point_to_int(&self,f:f32,allowfills:bool) -> Result<SkipDim,TeXError> {
        use crate::interpreter::dimensions::*;
        let mut kws = vec!("sp","pt","pc","in","bp","cm","mm","dd","cc","em","ex","px","mu");
        if allowfills {
            kws.push("fil");
            kws.push("fill");
            kws.push("filll");
        }
        let istrue = self.read_keyword(vec!("true"))?.is_some();
        match self.read_keyword(kws)? {
            Some(s) if s == "mm" => Ok(SkipDim::Pt(self.make_true(mm(f),istrue))),
            Some(s) if s == "in" => Ok(SkipDim::Pt(self.make_true(inch(f),istrue))),
            Some(s) if s == "sp" => Ok(SkipDim::Pt(self.make_true(f,istrue))),
            Some(s) if s == "pt" => Ok(SkipDim::Pt(self.make_true(pt(f),istrue))),
            Some(s) if s == "fil" => Ok(SkipDim::Fil(self.make_true(pt(f),istrue))),
            Some(s) if s == "fill" => Ok(SkipDim::Fill(self.make_true(pt(f),istrue))),
            Some(s) if s == "filll" => Ok(SkipDim::Filll(self.make_true(pt(f),istrue))),
            Some(o) => todo!("{}",o),
            None => TeXErr!(self,"expected unit for dimension")
        }
    }

    fn make_true(&self,f : f32,istrue:bool) -> i32 {
        use crate::utils::u8toi16;
        use crate::commands::primitives::MAG;
        if istrue {
            let mag = (self.state_register(-u8toi16(MAG.index)) as f32) / 1000.0;
            (f * mag).round() as i32
        } else {f.round() as i32}
    }

    pub fn read_dimension(&self) -> Result<i32,TeXError> {
        match match self.read_number_i(true)? {
            Numeric::Dim(i) => return Ok(i),
            Numeric::Int(i) => self.point_to_int(i as f32,false)?,
            Numeric::Float(f) => self.point_to_int(f,false)?,
            Numeric::Skip(sk) => return Ok(sk.base)
        } {
            SkipDim::Pt(i) => Ok(i),
            _ => unreachable!()
        }
    }

    pub fn read_skip(&self) -> Result<Skip,TeXError> {
        match self.read_number_i(true)? {
            Numeric::Dim(i) => self.rest_skip(i),
            Numeric::Float(f) => self.rest_skip(match self.point_to_int(f,false)? {
                SkipDim::Pt(i) => i,
                _ => unreachable!()
            }),
            Numeric::Int(f) => self.rest_skip(match self.point_to_int(f as f32,false)? {
                SkipDim::Pt(i) => i,
                _ => unreachable!()
            }),
            Numeric::Skip(s) => Ok(s)
        }
    }

    fn rest_skip(&self,dim:i32) -> Result<Skip,TeXError> {
        match self.read_keyword(vec!("plus","minus"))? {
            None => Ok(Skip {
                base: dim,
                stretch:None,
                shrink:None
            }),
            Some(p) if p == "plus" => {
                let stretch = Some(self.read_skipdim()?);
                match self.read_keyword(vec!("minus"))? {
                    None => Ok(Skip {
                        base:dim,
                        stretch,
                        shrink:None
                    }),
                    Some(_) => Ok(Skip {
                        base:dim,
                        stretch,
                        shrink:Some(self.read_skipdim()?)
                    })
                }
            }
            Some(p) if p == "minus" => {
                let shrink = Some(self.read_skipdim()?);
                match self.read_keyword(vec!("plus"))? {
                    None => Ok(Skip {
                        base:dim,
                        stretch:None,
                        shrink
                    }),
                    Some(_) => Ok(Skip {
                        base:dim,
                        shrink,
                        stretch:Some(self.read_skipdim()?)
                    })
                }
            }
            _ => unreachable!()
        }
    }

    fn read_skipdim(&self) -> Result<SkipDim,TeXError> {
        match self.read_number_i(true)? {
            Numeric::Dim(i) => Ok(SkipDim::Pt(i)),
            Numeric::Int(i) => self.point_to_int(i as f32,true),
            Numeric::Float(f) => self.point_to_int(f,true),
            Numeric::Skip(sk) => Ok(SkipDim::Pt(sk.base))
        }
    }
}