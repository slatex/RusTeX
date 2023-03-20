use crate::catcodes::CategoryCode;
use crate::interpreter::{Interpreter, TeXMode};
use crate::ontology::Token;
use crate::utils::{TeXError, TeXString};
use std::str::FromStr;
use crate::commands::{TokReference, PrimitiveTeXCommand};
use crate::{TeXErr,FileEnd,log};
use crate::catcodes::CategoryCode::BeginGroup;
use crate::commands::primitives::{HANGINDENT, PARSHAPE};
use crate::interpreter::dimensions::{Skip, Numeric, SkipDim, MuSkipDim, MuSkip, round_f};
use crate::interpreter::state::GroupType;
use crate::stomach::whatsits::Whatsit;
use crate::stomach::boxes::{BoxMode,TeXBox};

impl Interpreter<'_> {

    // General -------------------------------------------------------------------------------------

    pub fn insert_every(&mut self,tr:&TokReference) {
        let i = tr.index - 1;
        let insert = self.state.toks_prim.get(&i);
        self.push_tokens(insert)
    }

    pub fn skip_ws(&mut self) {
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

    pub fn read_eq(&mut self) {
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

    pub fn read_keyword(&mut self,mut kws:Vec<&str>) -> Result<Option<String>,TeXError> {
        use std::str;
        let mut tokens:Vec<Token> = Vec::new();
        let mut ret : String = "".to_string();
        self.skip_ws();
        while self.has_next() {
            let next = self.next_token();
            match next.catcode {
                CategoryCode::Space | CategoryCode::EOL => break,
                CategoryCode::Active | CategoryCode::Escape => {
                    let cmd = self.get_command(&next.cmdname())?;
                    if cmd.expandable(true) {
                        cmd.expand(next,self)?;
                    } else {
                        self.requeue(next);
                        break
                    }
                },
                _ => {
                    let ret2 = ret.clone() + str::from_utf8(crate::utils::as_ascii(&next.char).as_slice()).unwrap(); //str::from_utf8(&[next.char]).unwrap();
                    if kws.iter().any(|x| x.starts_with(&ret2)) {
                        kws = kws.iter().filter(|s| s.starts_with(&ret2)).map(|x| *x).collect();
                        ret = ret2;
                        tokens.push(next);
                        //if kws.is_empty() { break }
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
            self.push_tokens(tokens[ret.len()..].to_vec());
            Ok(Some(ret))
        } else {
            self.push_tokens(tokens);
            Ok(None)
        }
    }

    pub fn read_string(&mut self) -> Result<String,TeXError> {
        use std::str::from_utf8;
        let mut ret : Vec<u8> = Vec::new();
        let mut quoted = false;
        self.skip_ws();
        while self.has_next() {
            let next = self.next_token();
            match next.catcode {
                CategoryCode::Escape | CategoryCode::Active => {
                    let cmd = self.get_command(&next.cmdname())?;
                    if cmd.expandable(true) {
                        cmd.expand(next,self)?;
                    } else {
                        self.requeue(next);
                        return Ok(from_utf8(ret.as_slice()).unwrap().to_owned())
                    }
                },
                CategoryCode::Space | CategoryCode::EOL if !quoted => return Ok(from_utf8(ret.as_slice()).unwrap().to_owned()),
                CategoryCode::BeginGroup if ret.is_empty() => {
                    let tks = self.read_token_list(true,false,false,true)?;
                    return Ok(self.tokens_to_string(&tks).to_string())
                },
                _ if next.char == 34 && !quoted => quoted = true,
                _ if next.char == 34 => {
                    self.skip_ws();
                    return Ok(from_utf8(ret.as_slice()).unwrap().to_owned())
                }
                _ => ret.push(next.char)
            }
        }
        FileEnd!()
    }

    pub fn read_command_token(&mut self) -> Result<Token,TeXError> {
        let mut cmd: Option<Token> = None;
        while self.has_next() {
            self.skip_ws();
            let next = self.next_token();
            match next.catcode {
                CategoryCode::Escape | CategoryCode::Active => {
                    let p = self.state.commands.get(&next.cmdname());
                    match p {
                        None =>{ cmd = Some(next); break }
                        Some(p) => match *p.orig {
                            PrimitiveTeXCommand::Cond(c) => { c.expand(self)?; },
                            PrimitiveTeXCommand::Primitive(pr) if pr.expandable => p.expand(next, self)?,
                            _ => { cmd = Some(next); break }
                        }
                    }
                }
                _ => TeXErr!(next.clone() => "Command expected; found: {}",next)
            }
        };
        match cmd {
            Some(t) => Ok(t),
            _ => FileEnd!()
        }
    }

    // Token lists ---------------------------------------------------------------------------------

    pub fn read_argument(&mut self) -> Result<Vec<Token>,TeXError> {
        let next = self.next_token();
        if next.catcode == CategoryCode::BeginGroup {
            self.read_token_list(false,false,false,true)
        } else {
            Ok(vec!(next))
        }
    }

    #[inline(always)]
    pub fn read_token_list(&mut self,expand:bool,protect:bool,the:bool,allowunknowns:bool) -> Result<Vec<Token>,TeXError> {
        use crate::commands::primitives::{THE,UNEXPANDED};
        let mut ingroups : u8 = 0;
        let mut ret : Vec<Token> = Vec::with_capacity(50);
        while self.has_next() {
            let next = self.next_token();
            match next.catcode {
                CategoryCode::Active | CategoryCode::Escape if expand && next.expand => {
                    match self.state.commands.get(&next.cmdname()) {
                        Some(cmd) => match &*cmd.orig {
                            PrimitiveTeXCommand::Primitive(x) if (the && **x == THE) || **x == UNEXPANDED => {
                                match cmd.get_expansion(next,self)? {
                                    Some(exp) => {
                                        //let rf = exp.get_ref();
                                        for tk in exp.2 {
                                            match tk.catcode {
                                                CategoryCode::Parameter if the => {
                                                    ret.push(tk.clone());
                                                    ret.push(tk);
                                                }
                                                _ => ret.push(tk)
                                            }
                                        }
                                    }
                                    None => {}
                                }
                            }
                            //PrimitiveTeXCommand::Char(tk) => ret.push(tk.clone()),
                            _ => {
                                if cmd.expandable(!protect) {
                                    cmd.expand(next, self)?;
                                } else {
                                    ret.push(next);
                                }
                            }
                        }
                        None if allowunknowns => ret.push(next),
                        None => {self.get_command(&next.cmdname())?;}
                    }
                }
                CategoryCode::EndGroup if ingroups == 0 => return Ok(ret),
                CategoryCode::BeginGroup => {
                    if ingroups > 245 {
                        TeXErr!(next => "Group overflow")
                    }
                    ingroups += 1;
                    ret.push(next)
                }
                CategoryCode::EndGroup => {
                    ingroups -= 1;
                    ret.push(next);
                }
                _ => ret.push(next)
            }
        }
        FileEnd!()
    }

    pub fn expand_until(&mut self,eat_space:bool) -> Result<(),TeXError> {
        while self.has_next() {
            let next = self.next_token();
            match next.catcode {
                CategoryCode::Active | CategoryCode::Escape => {
                    let cmd = match self.state.commands.get(&next.cmdname()) {
                        None => return Ok(()),
                        Some(p) => p
                    };
                    if cmd.expandable(true) {
                        cmd.expand(next,self)?;
                    } else {
                        match &*cmd.orig {
                            PrimitiveTeXCommand::Char(tk) if eat_space && (tk.catcode == CategoryCode::Space || tk.catcode == CategoryCode::EOL) => return Ok(()),
                            _ => {
                                log!("expand_until ended by {}",next);
                                self.requeue(next);
                                return Ok(())
                            }
                        }
                    }
                },
                CategoryCode::Space | CategoryCode::EOL if eat_space => return Ok(()),
                _ => {
                    log!("expand_until ended by {}",next);
                    self.requeue(next);
                    return Ok(())
                }
            }
        }
        FileEnd!()
    }

    pub fn read_balanced_argument(&mut self,expand:bool,protect:bool,the:bool,allowunknowns:bool) -> Result<Vec<Token>,TeXError> {
        self.expand_until(true)?;
        let next = self.next_token();
        match next.catcode {
            CategoryCode::BeginGroup => {}
            CategoryCode::Active | CategoryCode::Escape => {
                let p = self.get_command(&next.cmdname())?;
                match &*p.orig {
                    PrimitiveTeXCommand::Char(tk) if tk.catcode == BeginGroup => {},
                    _ => TeXErr!(next => "Expected Begin Group Token")
                }
            }
            _ => TeXErr!(next => "Expected Begin Group Token")
        }
        self.read_token_list(expand, protect,the,allowunknowns)
    }

    pub fn set_relax(&mut self,cmd:&Token) {
        use crate::commands::primitives::RELAX;
        self.state.commands.set(cmd.cmdname(),Some(PrimitiveTeXCommand::Primitive(&RELAX).as_command()),false);
    }

    pub fn eat_relax(&mut self) {
        use crate::commands::primitives::RELAX;
        if self.has_next() {
            let next = self.next_token();
            match next.catcode {
                CategoryCode::Escape | CategoryCode::Active => {
                    match self.state.commands.get(&next.cmdname()).map(|x| x.orig) {
                        Some(p)  => match &*p {
                            PrimitiveTeXCommand::Primitive(r) if **r == RELAX => (),
                            _ => self.requeue(next)
                        }
                        _ => self.requeue(next)
                    }
                }
                _ => {
                    self.requeue(next)
                }
            }
        }
    }

    // Boxes & Whatsits ----------------------------------------------------------------------------

    pub fn read_whatsit_group(&mut self,bm : BoxMode,insertevery:bool) -> Result<Vec<Whatsit>,TeXError> {
        self.expand_until(false)?;
        let next = self.next_token();
        let tk = match next.catcode {
            CategoryCode::BeginGroup => next,
            CategoryCode::Active | CategoryCode::Escape => {
                let p = self.get_command(&next.cmdname())?;
                match &*p.orig {
                    PrimitiveTeXCommand::Char(tk) if tk.catcode == BeginGroup => tk.clone(),
                    _ => TeXErr!(next => "Expected Begin Group Token")
                }
            }
            _ => TeXErr!(next => "Expected Begin Group Token")
        };
        let _oldmode = self.state.mode;
        self.state.push(self.stomach,GroupType::Box(bm));
        self.state.mode = match bm {
            BoxMode::H => {
                if insertevery { self.insert_every(&crate::commands::registers::EVERYHBOX) };
                TeXMode::RestrictedHorizontal
            },
            BoxMode::V => {
                if insertevery { self.insert_every(&crate::commands::registers::EVERYVBOX) };
                TeXMode::InternalVertical
            },
            BoxMode::M => TeXMode::Math,
            BoxMode::DM => TeXMode::Displaymath,
            _ => TeXErr!("read_whatsit_group requires non-void box mode")
        };
        if self.state.insetbox {
            self.state.insetbox = false;
            self.insert_afterassignment();
        }
        self.requeue(tk);
        self.read_whatsits()?;
        let ret = self.get_whatsit_group(GroupType::Box(bm))?;
        self.state.mode = _oldmode;
        Ok(ret)
    }

    pub fn read_box(&mut self) -> Result<TeXBox,TeXError> {
        use crate::commands::ProvidesWhatsit;
        self.expand_until(false)?;
        let next = self.next_token();
        match next.catcode {
            CategoryCode::Escape | CategoryCode::Active => {
                let cmd = self.get_command(&next.cmdname())?;
                match &*cmd.orig {
                    PrimitiveTeXCommand::Whatsit(ProvidesWhatsit::Box(b)) => {
                        (b._get)(&next,self)
                    }
                    _ => TeXErr!(next => "Expected box")
                }
            }
            _ => TeXErr!(next => "Expected Begin Group Token or Whatsit")
        }
    }

    pub fn read_whatsits(&mut self) -> Result<(),TeXError> {
        self.expand_until(false)?;
        let next = self.next_token();
        match next.catcode {
            CategoryCode::BeginGroup => {}
            CategoryCode::Escape | CategoryCode::Active => {
                let cmd = self.get_command(&next.cmdname())?;
                if cmd.has_whatsit() {
                    TeXErr!("TODO")
                } else {
                    TeXErr!(next => "Expected Begin Group Token or Whatsit")
                }
            }
            _ => TeXErr!(next => "Expected Begin Group Token or Whatsit")
        }
        let mut ingroups : u8 = 0;
        //let mut ret : Vec<Whatsit> = vec!();
        while self.has_next() {
            let next = self.next_token();
            match next.catcode {
                CategoryCode::EndGroup if ingroups == 0 => return Ok(()),
                CategoryCode::BeginGroup => {
                    ingroups += 1;
                    self.state.push(self.stomach,GroupType::Token);
                }
                CategoryCode::EndGroup => {
                    ingroups -= 1;
                    self.pop_group(GroupType::Token)?;
                }
                _ => {
                    self.do_top(next,true)?
                }
            }
        }
        FileEnd!()
    }

    // Numbers -------------------------------------------------------------------------------------

    fn num_do_ret(&self,ishex:bool,isoct:bool,isnegative:bool,allowfloat:bool,ret:TeXString) -> Result<Numeric,TeXError> {
        let utf = match ret.to_utf8() {
            s if s == "." => "0.0".to_string(),
            s => s
        };
        let num = if ishex {
            Numeric::Int(i32::from_str_radix(&utf, 16).or_else(|_| {
                TeXErr!("Number error (should be impossible)")
            })?)
        } else if isoct {
            Numeric::Int(i32::from_str_radix(&utf, 8).or_else(|_| {
                TeXErr!("Number error (should be impossible)")
            })?)
        } else if allowfloat {
            Numeric::Float(f64::from_str(&utf).or_else(|_| {
                TeXErr!("Number error (should be impossible)")
            })?)
        } else {
            Numeric::Int(i32::from_str(&utf).or_else(|_| {
                TeXErr!("Number error (should be impossible)")
            })?)
        };
        Ok(if isnegative {num.negate()} else {num})
    }

    pub(crate) fn read_number_i(&mut self,allowfloat:bool) -> Result<Numeric,TeXError> {
        self.skip_ws();
        match self.read_number_i_opt(allowfloat)? {
            Some(i) => Ok(i),
            _ => TeXErr!("Number expected")
        }
    }

    pub(crate) fn read_number_i_opt(&mut self,allowfloat:bool) -> Result<Option<Numeric>,TeXError> {
        let mut isnegative = false;
        let mut ishex = false;
        let mut isoct = false;
        let mut isfloat = false;
        let mut ret : TeXString = "".into();
        log!("Reading number {}",self.preview());
        while self.has_next() {
            let next = self.next_token();
            match next.catcode {
                CategoryCode::Escape | CategoryCode::Active if ret.is_empty() && !ishex && !isoct => {
                    let p = self.get_command(&next.cmdname())?;
                    if p.has_num() {
                        return Ok(Some(if isnegative { p.get_num(self)?.negate() } else { p.get_num(self)? }))
                    } else if p.expandable(true) {
                        p.expand(next,self)?;
                    } else {
                        match &*p.orig {
                            PrimitiveTeXCommand::Char(tk) => return Ok(Some(Numeric::Int(if isnegative { -(tk.char as i32) } else { tk.char as i32 }))),
                            PrimitiveTeXCommand::Primitive(p) if **p == PARSHAPE => return Ok(Some(Numeric::Int(self.state.parshape.get().len() as i32))),
                            PrimitiveTeXCommand::Primitive(p) if **p == HANGINDENT => return Ok(Some(Numeric::Dim(self.state.hangindent.get()))),
                            _ => TeXErr!(next.clone() => "Number expected; found {}\n{}",next,self.preview())
                        }
                    }
                }
                CategoryCode::Escape | CategoryCode::Active => {
                    let p = self.get_command(&next.cmdname())?;
                    if p.expandable(true) {
                        p.expand(next,self)?
                    } else {
                        match &*p.orig {
                            PrimitiveTeXCommand::Char(tk) if tk.catcode == CategoryCode::Space || tk.catcode == CategoryCode::EOL =>
                                return Ok(Some(self.num_do_ret(ishex, isoct, isnegative, allowfloat, ret)?)),
                            _ => {
                                self.requeue(next);
                                return Ok(Some(self.num_do_ret(ishex, isoct, isnegative, allowfloat, ret)?))
                            }
                        }
                    }
                }
                CategoryCode::Space | CategoryCode::EOL if !ret.is_empty() => return Ok(Some(self.num_do_ret(ishex,isoct,isnegative,allowfloat,ret)?)),
                _ if next.char.is_ascii_digit() => ret += next.name(),
                _ if next.char.is_ascii_hexdigit() && ishex => ret += next.name(),
                _ if next.char == 45 && ret.is_empty() => isnegative = !isnegative,
                _ if next.char == 43 && ret.is_empty() => (),
                _ if (next.char == 46 || next.char == 44) && allowfloat && !isfloat => { isfloat = true; ret += "." }
                _ if next.char == 34 && ret.is_empty() && !ishex && !isoct => ishex = true,
                _ if next.char == 39 && ret.is_empty() && !ishex && !isoct => isoct = true,
                _ if next.char == 96 => while self.has_next() {
                    let next = self.next_token();
                    match next.catcode {
                        CategoryCode::Escape if next.cmdname().len() == 1 => {
                            let num = *next.cmdname().iter().first().unwrap() as i32;
                            self.expand_until(true)?;
                            return Ok(Some(Numeric::Int(if isnegative { -num } else { num })))
                        }
                        CategoryCode::Escape => {
                            let cmd = self.get_command(&next.cmdname())?;
                            if cmd.expandable(true) {
                                cmd.expand(next,self)?;
                            } else {
                                match &*cmd.orig {
                                    PrimitiveTeXCommand::Char(c) => {
                                        self.expand_until(true)?;
                                        return Ok(Some(Numeric::Int(if isnegative {-(c.char as i32)} else {c.char as i32})))
                                    }
                                    _ => TeXErr!(next.clone() => "Number expected; found {}",next)
                                }
                            }
                        }
                        _ => {
                            self.expand_until(true)?;
                            return Ok(Some(Numeric::Int(if isnegative {-(next.char as i32)} else {next.char as i32})))
                        }
                    }
                }
                _ if !ret.is_empty() => {
                    self.requeue(next);
                    return Ok(Some(self.num_do_ret(ishex,isoct,isnegative,allowfloat,ret)?))
                }
                _ => {
                    self.requeue(next);
                    if isnegative {
                        self.push_tokens(vec!(Token::new(45,self.state.catcodes.get_scheme().get_code(45),None,None,true)))
                    }
                    return Ok(None)
                }
            }
        }
        FileEnd!()
    }

    pub fn read_number(&mut self) -> Result<i32,TeXError> {
        match self.read_number_i(false)? {
            Numeric::BigInt(i) => Ok(i as i32),
            Numeric::Int(i) => Ok(i),
            Numeric::Dim(i) => Ok(i),
            Numeric::Float(_) => TeXErr!("Should be unreachable!"),
            Numeric::Skip(sk) => Ok(sk.base),
            Numeric::MuSkip(sk) => Ok(sk.base),
        }
    }

    // Dimensions ----------------------------------------------------------------------------------

    fn point_to_int(&mut self,f:f64,allowfills:bool) -> Result<SkipDim,TeXError> {
        use crate::interpreter::dimensions::*;
        let mut kws = vec!("sp","pt","pc","in","bp","cm","mm","dd","cc","em","ex","px");
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
            Some(s) if s == "cm" => Ok(SkipDim::Pt(self.make_true(cm(f),istrue))),
            Some(s) if s == "bp" => Ok(SkipDim::Pt(self.make_true(pt(f),istrue))),
            Some(s) if s == "dd" => Ok(SkipDim::Pt(self.make_true(dd(f),istrue))),
            Some(s) if s == "cc" => Ok(SkipDim::Pt(self.make_true(cc(f),istrue))),
            Some(s) if s == "pc" => Ok(SkipDim::Pt(self.make_true(pc(f),istrue))),
            Some(s) if s == "ex" => Ok(SkipDim::Pt(self.make_true(self.state.currfont.get().get_dimen(5) as f64 * f,istrue))),
            Some(s) if s == "em" => Ok(SkipDim::Pt(self.make_true(self.state.currfont.get().get_dimen(6) as f64 * f,istrue))),
            Some(s) if s == "px" => Ok(SkipDim::Pt(self.make_true(
                self.state.dimensions_prim.get(&(crate::commands::registers::PDFPXDIMEN.index - 1)) as f64 * f,
                istrue))),
            Some(s) if s == "fil" => Ok(SkipDim::Fil(self.make_true(pt(f),istrue))),
            Some(s) if s == "fill" => Ok(SkipDim::Fill(self.make_true(pt(f),istrue))),
            Some(s) if s == "filll" => Ok(SkipDim::Filll(self.make_true(pt(f),istrue))),
            Some(o) => todo!("{}",o),
            None => {
                let r = self.read_dimension()?;
                Ok(SkipDim::Pt(((r as f64 * (f * 65536.0).floor()) / 65536.0).floor() as i32))
            }
                //TeXErr!((self,None),"expected unit for dimension : {}",f)
        }
    }

    fn make_true(&self,f : f64,istrue:bool) -> i32 {
        use crate::commands::registers::MAG;
        if istrue {
            let mag = (self.state.registers_prim.get(&(MAG.index - 1)) as f64) / 1000.0;
            round_f(f * mag)
        } else { round_f(f) }
    }

    pub fn read_dimension(&mut self) -> Result<i32,TeXError> {
        match match self.read_number_i(true)? {
            Numeric::BigInt(i) => return Ok(i as i32),
            Numeric::Dim(i) => return Ok(i),
            Numeric::Int(i) => self.point_to_int(i as f64,false)?,
            Numeric::Float(f) => self.point_to_int(f,false)?,
            Numeric::Skip(sk) => return Ok(sk.base),
            Numeric::MuSkip(_) => TeXErr!("Dimension expected; muskip found")
        } {
            SkipDim::Pt(i) => Ok(i),
            _ => TeXErr!("Should be unreachable!")
        }
    }

    pub fn read_skip(&mut self) -> Result<Skip,TeXError> {
        let (a,b,c) = match self.read_number_i(true)? {
            Numeric::Dim(i) => (i,None,None),
            Numeric::Float(f) => (match self.point_to_int(f,false)? {
                SkipDim::Pt(i) => i,
                _ => TeXErr!("Should be unreachable!")
            },None,None),
            Numeric::BigInt(f) => (match self.point_to_int(f as f64,false)? {
                SkipDim::Pt(i) => i,
                _ => TeXErr!("Should be unreachable!")
            },None,None),
            Numeric::Int(f) => (match self.point_to_int(f as f64,false)? {
                SkipDim::Pt(i) => i,
                _ => TeXErr!("Should be unreachable!")
            },None,None),
            Numeric::Skip(s) => return Ok(s),//(s.base,s.stretch,s.shrink),
            Numeric::MuSkip(_) => TeXErr!("Skip expected; muskip found")
        };
        self.rest_skip(a,b,c)
    }

    pub fn read_muskip(&mut self) -> Result<MuSkip,TeXError> {
        let (a,b,c) = match self.read_number_i(true)? {
            Numeric::Dim(i) => (i,None,None),
            Numeric::Float(f) => (match self.point_to_muskip(f)? {
                MuSkipDim::Mu(i) => i,
                _ => TeXErr!("Should be unreachable!")
            },None,None),
            Numeric::BigInt(f) => (match self.point_to_muskip(f as f64)? {
                MuSkipDim::Mu(i) => i,
                _ => TeXErr!("Should be unreachable!")
            },None,None),
            Numeric::Int(f) => (match self.point_to_muskip(f as f64)? {
                MuSkipDim::Mu(i) => i,
                _ => TeXErr!("Should be unreachable!")
            },None,None),
            Numeric::Skip(_) => TeXErr!("MuSkip expected; skip found"),
            Numeric::MuSkip(s) =>  (s.base,s.stretch,s.shrink)
        };
        self.rest_muskip(a,b,c)
    }



    fn point_to_muskip(&mut self,f:f64) -> Result<MuSkipDim,TeXError> {
        use crate::interpreter::dimensions::*;
        let kws = vec!("mu","fil","fill","filll");
        //let istrue = self.read_keyword(vec!("true"))?.is_some();
        match self.read_keyword(kws)? {
            Some(s) if s == "mu" => Ok(MuSkipDim::Mu(round_f(pt(f)))),
            Some(s) if s == "fil" => Ok(MuSkipDim::Fil(round_f(pt(f)))),
            Some(s) if s == "fill" => Ok(MuSkipDim::Fill(round_f(pt(f)))),
            Some(s) if s == "filll" => Ok(MuSkipDim::Filll(round_f(pt(f)))),
            None => {
                let r = self.read_dimension()?;
                Ok(MuSkipDim::Mu(((r as f64 * (f * 65536.0).floor()) / 65536.0).floor() as i32))
            }
            _ => TeXErr!("Should be unreachable!")
            //TeXErr!((self,None),"expected unit for dimension : {}",f)
        }
    }

    fn rest_skip(&mut self,dim:i32,plus:Option<SkipDim>,minus:Option<SkipDim>) -> Result<Skip,TeXError> {
        match self.read_keyword(vec!("plus","minus"))? {
            None => Ok(Skip {
                base: dim,
                stretch:plus,
                shrink:minus
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
            _ => TeXErr!("Should be unreachable!")
        }
    }


    fn rest_muskip(&mut self,dim:i32,plus:Option<MuSkipDim>,minus:Option<MuSkipDim>) -> Result<MuSkip,TeXError> {
        match self.read_keyword(vec!("plus","minus"))? {
            None => Ok(MuSkip {
                base: dim,
                stretch:plus,
                shrink:minus
            }),
            Some(p) if p == "plus" => {
                let stretch = Some(self.read_muskipdim()?);
                match self.read_keyword(vec!("minus"))? {
                    None => Ok(MuSkip {
                        base:dim,
                        stretch,
                        shrink:None
                    }),
                    Some(_) => Ok(MuSkip {
                        base:dim,
                        stretch,
                        shrink:Some(self.read_muskipdim()?)
                    })
                }
            }
            Some(p) if p == "minus" => {
                let shrink = Some(self.read_muskipdim()?);
                match self.read_keyword(vec!("plus"))? {
                    None => Ok(MuSkip {
                        base:dim,
                        stretch:None,
                        shrink
                    }),
                    Some(_) => Ok(MuSkip {
                        base:dim,
                        shrink,
                        stretch:Some(self.read_muskipdim()?)
                    })
                }
            }
            _ => TeXErr!("Should be unreachable!")
        }
    }

    fn read_skipdim(&mut self) -> Result<SkipDim,TeXError> {
        match self.read_number_i(true)? {
            Numeric::Dim(i) => Ok(SkipDim::Pt(i)),
            Numeric::BigInt(i) => self.point_to_int(i as f64,true),
            Numeric::Int(i) => self.point_to_int(i as f64,true),
            Numeric::Float(f) => self.point_to_int(f,true),
            Numeric::Skip(sk) => Ok(SkipDim::Pt(sk.base)),
            Numeric::MuSkip(_) => TeXErr!("Skip expected; muskip found")
        }
    }

    fn read_muskipdim(&mut self) -> Result<MuSkipDim,TeXError> {
        match self.read_number_i(true)? {
            Numeric::Dim(i) => Ok(MuSkipDim::Mu(i)),
            Numeric::BigInt(i) => self.point_to_muskip(i as f64),
            Numeric::Int(i) => self.point_to_muskip(i as f64),
            Numeric::Float(f) => self.point_to_muskip(f),
            Numeric::Skip(_) => TeXErr!("MuSkip expected; skip found"),
            Numeric::MuSkip(sk) => Ok(MuSkipDim::Mu(sk.base))
        }
    }
}