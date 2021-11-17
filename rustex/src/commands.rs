pub mod primitives;
pub mod etex;
pub mod pdftex;
pub mod conditionals;

use crate::ontology::{Expansion, Token};
use crate::interpreter::Interpreter;
use std::rc::Rc;
use std::fmt;
use std::fmt::{Display, Formatter, Pointer};
use std::str::from_utf8;
use crate::catcodes::{CategoryCode, CategoryCodeScheme};
use crate::interpreter::dimensions::Numeric;
use crate::utils::{TeXError, TeXString};
use crate::log;

pub struct PrimitiveExecutable {
    pub (in crate) _apply:fn(cs:Token,itp:&Interpreter) -> Result<Option<Expansion>,TeXError>,
    pub expandable : bool,
    pub name: &'static str
}
impl PrimitiveExecutable {
    pub fn apply(&self,cs:Token,itp:&Interpreter) -> Result<Option<Expansion>,TeXError> {
        (self._apply)(cs,itp)
    }
}
pub struct Conditional {
    pub name: &'static str,
    _apply:fn(int:&Interpreter,cond:u8,unless:bool) -> Result<(),TeXError>
}
impl Conditional {
    pub fn expand(&self,_tk:Token,int:&Interpreter) -> Result<(),TeXError> {
        (self._apply)(int,int.pushcondition(),false)
    }
}

impl PartialEq for PrimitiveExecutable {
    fn eq(&self, other: &Self) -> bool {
        self.name == other.name
    }
}

pub struct AssValue<T> {
    pub name: &'static str,
    pub _assign: fn(int: &Interpreter,global: bool) -> Result<(),TeXError>,
    pub _getvalue: fn(int: &Interpreter) -> Result<T,TeXError>
}
impl<T> PartialEq for AssValue<T> {
    fn eq(&self, other: &Self) -> bool {
        self.name == other.name
    }
}

pub struct IntCommand {
    pub _getvalue: fn(int: &Interpreter) -> Result<i32,TeXError>,
    pub name : &'static str
}

#[derive(PartialEq)]
pub struct RegisterReference {
    pub index: u8,
    pub name: &'static str
}

#[derive(PartialEq)]
pub struct DimenReference {
    pub index: u8,
    pub name: &'static str
}

#[derive(PartialEq)]
pub struct SkipReference {
    pub index: u8,
    pub name: &'static str
}

#[derive(PartialEq)]
pub struct TokReference {
    pub index: u8,
    pub name: &'static str
}

pub struct PrimitiveAssignment {
    pub name: &'static str,
    pub _assign: fn(int: &Interpreter,global: bool) -> Result<(),TeXError>
}
impl PartialEq for PrimitiveAssignment {
    fn eq(&self, other: &Self) -> bool {
        self.name == other.name
    }
}

#[derive(Clone)]
pub struct DefMacro {
    pub protected:bool,
    pub long:bool,
    pub sig:Signature,
    pub ret:Vec<ParamToken>
}
impl Display for DefMacro {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        write!(f,"{}->{}{}{}",self.sig,"{",ParamList(&self.ret),"}")
    }
}

use crate::stomach::whatsits::ExecutableWhatsit;

pub struct ProvidesExecutableWhatsit {
    pub name: &'static str,
    pub _get: fn(tk:Token,int: &Interpreter) -> Result<ExecutableWhatsit,TeXError>
}

// -------------------------------------------------------------------------------------------------

#[derive(Clone)]
pub enum AssignableValue {
    Dim(u8),
    Register(u8),
    Skip(u8),
    Toks(u8),
    Int(&'static AssValue<i32>),
    PrimReg(&'static RegisterReference),
    PrimDim(&'static DimenReference),
    PrimSkip(&'static SkipReference),
    PrimToks(&'static TokReference)
}

impl AssignableValue {
    pub fn name(&self) -> Option<TeXString> {
        use AssignableValue::*;
        match self {
            Dim(_) | Register(_) | Skip(_) | Toks(_) => None,
            Int(i) => Some(i.name.into()),
            PrimReg(r) => Some(r.name.into()),
            PrimDim(d) => Some(d.name.into()),
            PrimSkip(d) => Some(d.name.into()),
            PrimToks(d) => Some(d.name.into())
        }
    }
}

pub enum HasNum {
    Dim(u8),
    Register(u8),
    Skip(u8),
    Char(u32),
    AssInt(&'static AssValue<i32>),
    Int(&'static IntCommand),
    PrimReg(&'static RegisterReference),
    PrimSkip(&'static SkipReference),
    PrimDim(&'static DimenReference),
    Ext(Rc<dyn ExternalCommand>)
}

impl HasNum {
    pub(crate) fn get(&self,int:&Interpreter) -> Result<Numeric,TeXError> {
        use HasNum::*;
        use crate::utils::u8toi16;
        match self {
            Dim(i) => Ok(Numeric::Dim(int.state_dimension(u8toi16(*i)))),
            Register(i) => Ok(Numeric::Int(int.state_register(u8toi16(*i)))),
            Skip(i) => Ok(Numeric::Skip(int.state_skip(u8toi16(*i)))),
            AssInt(i) => Ok(Numeric::Int((i._getvalue)(int)?)),
            Int(i) => Ok(Numeric::Int((i._getvalue)(int)?)),
            PrimReg(r) => Ok(Numeric::Int(int.state_register(-u8toi16(r.index)))),
            PrimDim(r) => Ok(Numeric::Dim(int.state_dimension(-u8toi16(r.index)))),
            PrimSkip(r) => Ok(Numeric::Skip(int.state_skip(-u8toi16(r.index)))),
            Ext(r) => r.get_num(int),
            Char(u) => Ok(Numeric::Int(*u as i32))
        }
    }
}

pub enum Expandable {
    Cond(&'static Conditional),
    Primitive(&'static PrimitiveExecutable),
    Ext(Rc<dyn ExternalCommand>),
    Def(Rc<DefMacro>)
}

use crate::TeXErr;
use crate::references::SourceReference;

impl Expandable {
    pub fn get_expansion(&self,tk:Token,int:&Interpreter) -> Result<Vec<Token>,TeXError> {
        use Expandable::*;
        log!("Expanding {}",tk);
        match self {
            Cond(c) => {c.expand(tk,int)?; Ok(vec!())},
            Primitive(p) => match (p._apply)(tk,int)? {
                Some(e) => Ok(e.exp),
                _ => Ok(vec!())
            },
            Ext(p) => Ok(p.expand(int)?.exp),
            Def(d) => Expandable::doDef(int,d)
        }
    }

    fn doDef(int:&Interpreter,d:&Rc<DefMacro>) -> Result<Vec<Token>,TeXError> {
        log!("{}",d);
        let mut args : Vec<Vec<Token>> = Vec::new();
        let mut i = 0;
        while i < d.sig.elems.len() {
            match d.sig.elems.get(i).unwrap() {
                ParamToken::Token(tk) => {
                    int.assert_has_next()?;
                    let next = int.next_token();
                    if *tk != next { TeXErr!(int,"Expected {}; found {} (in {})",tk,next,d) }
                    i += 1;
                }
                ParamToken::Param(_,_) => {
                    i +=1;
                    match d.sig.elems.get(i) {
                        None if d.sig.endswithbrace => {
                            todo!()
                        },
                        None | Some(ParamToken::Param(_,_)) => {
                            int.skip_ws();
                            let next = int.read_argument()?;
                            args.push(next);
                        },
                        Some(ParamToken::Token(tk)) => {
                            let mut delim : Vec<Token> = vec!(tk.clone());
                            i +=1;
                            while i < d.sig.elems.len() {
                                match d.sig.elems.get(i) {
                                    Some(ParamToken::Token(t)) => {
                                        delim.push(t.clone());
                                        i += 1;
                                    },
                                    _ => break
                                }
                            }
                            let mut retarg : Vec<Token> = vec!();
                            let mut groups = 0;
                            let mut totalgroups = 0;
                            while int.has_next() {
                                let next = int.next_token();
                                match next.catcode {
                                    CategoryCode::BeginGroup if groups == 0 => {
                                        groups += 1;
                                        totalgroups += 1;
                                    }
                                    CategoryCode::BeginGroup => groups += 1,
                                    CategoryCode::EndGroup => groups -=1,
                                    _ => ()
                                }
                                retarg.push(next);
                                if groups < 0 {TeXErr!(int,"Missing { somewhere!")}
                                if groups == 0 && retarg.ends_with(delim.as_slice()) {break}
                            }
                            int.assert_has_next()?;
                            for _ in 0..delim.len() { retarg.pop(); }
                            if totalgroups == 1 &&
                                match retarg.first() {Some(tk) => tk.catcode == CategoryCode::BeginGroup, _ => false} &&
                                match retarg.last() {Some(tk) => tk.catcode == CategoryCode::EndGroup, _ => false} {
                                retarg.remove(0);
                                retarg.pop();
                            }
                            args.push(retarg)
                        }
                    }
                }
            }
        }
        let mut ret : Vec<Token> = Vec::new();
        for tk in &d.ret {
            match tk {
                ParamToken::Token(tk) => ret.push(tk.clone()),
                ParamToken::Param(0,c) => {
                    let ntk = Token {
                        char: *c,
                        catcode: CategoryCode::Parameter,
                        name_opt: None,
                        reference: Box::new(SourceReference::None),
                        expand: true
                    };
                    ret.push(ntk)
                }
                ParamToken::Param(i,_) => for tk in args.get((i-1) as usize).unwrap() { ret.push(tk.clone()) }
            }
        }
        Ok(ret)
    }
    pub fn expand(&self,tk:Token,int:&Interpreter) -> Result<(),TeXError> {
        use Expandable::*;
        log!("Expanding {}",tk);
        match self {
            Cond(c) => c.expand(tk,int),
            Primitive(p) => match (p._apply)(tk,int)? {
                Some(e) =>
                    Ok(int.push_expansion(e)),
                _ => Ok(())
            },
            Ext(p) => Ok(int.push_expansion(p.expand(int)?)),
            Def(d) => {
                Ok(int.push_expansion(Expansion {
                    cs: tk,
                    exp: Expandable::doDef(int,d)?
                }))
            }
        }
    }
}


pub enum Assignment {
    Value(AssignableValue),
    Ext(Rc<dyn ExternalCommand>),
    Prim(&'static PrimitiveAssignment)
}

use crate::interpreter::state::StateChange;

impl Assignment {
    pub fn assign(&self,int:&Interpreter,global:bool) -> Result<(),TeXError> {
        use crate::utils::u8toi16;
        use crate::interpreter::dimensions::dimtostr;
        match self {
            Assignment::Prim(p) => (p._assign)(int,global),
            Assignment::Value(av) => match av {
                AssignableValue::Int(d) => (d._assign)(int,global),
                AssignableValue::Register(i) => {
                    int.read_eq();
                    let num = int.read_number()?;
                    log!("Assign register {} to {}",i,num);
                    int.change_state(StateChange::Register(u8toi16(*i),num,global));
                    Ok(())
                }
                AssignableValue::Dim(i) => {
                    int.read_eq();
                    let num = int.read_dimension()?;
                    log!("Assign dimen register {} to {}",i,dimtostr(num));
                    int.change_state(StateChange::Dimen(u8toi16(*i),num,global));
                    Ok(())
                }
                AssignableValue::Skip(i) => {
                    int.read_eq();
                    let num = int.read_skip()?;
                    log!("Assign skip register {} to {}",i,num);
                    int.change_state(StateChange::Skip(u8toi16(*i),num,global));
                    Ok(())
                },
                AssignableValue::PrimSkip(r) => {
                    int.read_eq();
                    let num = int.read_skip()?;
                    log!("Assign {} to {}",r.name,num);
                    int.change_state(StateChange::Skip(-u8toi16(r.index),num,global));
                    Ok(())
                },
                AssignableValue::Toks(i) => {
                    int.expand_until(false)?;
                    match int.next_token().catcode {
                        CategoryCode::BeginGroup => {}
                        _ => TeXErr!(int,"Expected Begin Group Token")
                    }
                    let toks = int.read_token_list(false,false)?;
                    int.change_state(StateChange::Tokens(u8toi16(*i),toks,global));
                    Ok(())
                },
                AssignableValue::PrimToks(r) => {
                    int.expand_until(false)?;
                    match int.next_token().catcode {
                        CategoryCode::BeginGroup => {}
                        _ => TeXErr!(int,"Expected Begin Group Token")
                    }
                    let toks = int.read_token_list(false,false)?;
                    int.change_state(StateChange::Tokens(-u8toi16(r.index),toks,global));
                    Ok(())
                },
                AssignableValue::PrimReg(r) => {
                    int.read_eq();
                    let num = int.read_number()?;
                    log!("Assign {} to {}",r.name,num);
                    int.change_state(StateChange::Register(-u8toi16(r.index),num,global));
                    Ok(())
                },
                AssignableValue::PrimDim(r) => {
                    int.read_eq();
                    let num = int.read_dimension()?;
                    log!("Assign {} to {}",r.name,dimtostr(num));
                    int.change_state(StateChange::Dimen(-u8toi16(r.index),num,global));
                    Ok(())
                }
            } ,
            Assignment::Ext(ext) => ext.assign(int,global)
        }
    }
}


pub trait ExternalCommand {
    fn expandable(&self) -> bool;
    fn assignable(&self) -> bool;
    fn has_num(&self) -> bool;
    fn name(&self) -> String;
    fn execute(&self,int : &Interpreter) -> Result<(),TeXError>;
    fn expand(&self,int:&Interpreter) -> Result<Expansion,TeXError>;
    fn assign(&self,int:&Interpreter,global:bool) -> Result<(),TeXError>;
    fn get_num(&self,int:&Interpreter) -> Result<Numeric,TeXError>;
}

#[derive(Clone)]
pub enum ParamToken {
    Param(u8,u8),
    Token(Token)
}
impl PartialEq for ParamToken {
    fn eq(&self, other: &Self) -> bool {
        match (self,other) {
            (ParamToken::Param(a1,a2),ParamToken::Param(b1,b2)) => a1 == b1 && a2 == b2,
            (ParamToken::Token(a),ParamToken::Token(b)) => a == b,
            _ => false
        }
    }
}
impl Display for ParamToken {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        use ansi_term::Colour::*;
        match self {
            ParamToken::Param(0,u) => write!(f,"{}",Yellow.paint(TeXString(vec!(*u,*u)).to_string())),
            ParamToken::Param(i,u) => write!(f,"{}{}",Yellow.paint(TeXString(vec!(*u)).to_string()),Yellow.paint(i.to_string())),
            ParamToken::Token(t) => write!(f,"{}",t)
        }
    }
}

#[derive(Clone)]
pub struct Signature {
    elems:Vec<ParamToken>,
    endswithbrace:bool,
    arity:u8
}
impl PartialEq for Signature {
    fn eq(&self, other: &Self) -> bool {
        self.arity == other.arity &&
            self.endswithbrace == other.endswithbrace &&
            self.elems.len() == other.elems.len() && {
                for i in 0..self.elems.len() {
                    if self.elems.get(i) != other.elems.get(i) {return false}
                }
                return true
            }
    }
}
impl Display for Signature {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        for e in &self.elems {
            write!(f,"{}",e)?;
        }
        if self.endswithbrace {write!(f,"{}","{")} else {
            write!(f,"")
        }
    }
}
pub struct ParamList<'a>(&'a Vec<ParamToken>);

impl Display for ParamList<'_> {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        for e in self.0 {
            write!(f,"{}",e)?;
        }
        write!(f,"")
    }
}

pub struct TokenList<'a>(&'a Vec<Token>);
impl Display for TokenList<'_> {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        for e in self.0 {
            match e.catcode {
                CategoryCode::Escape => write!(f,"\\{}",e.name())?,
                _ => write!(f,"{}",from_utf8(&[e.char]).unwrap())?
            }
        }
        write!(f,"")
    }
}

#[derive(Clone)]
pub enum ProvidesWhatsit {
    Exec(&'static ProvidesExecutableWhatsit),
    Other
}
impl ProvidesWhatsit {
    pub fn name(&self) -> Option<TeXString> {
        match self {
            ProvidesWhatsit::Exec(e) => Some(e.name.into()),
            _ => todo!()
        }
    }
}

#[derive(Clone)]
pub enum TeXCommand {
    Primitive(&'static PrimitiveExecutable),
    AV(AssignableValue),
    Ext(Rc<dyn ExternalCommand>),
    Cond(&'static Conditional),
    Int(&'static IntCommand),
    Char(Token),
    Ass(&'static PrimitiveAssignment),
    Def(Rc<DefMacro>),
    Whatsit(ProvidesWhatsit),
    MathChar(u32)
}

impl PartialEq for TeXCommand {
    fn eq(&self, other: &Self) -> bool {
        use TeXCommand::*;
        match (self,other) {
            (Primitive(a),Primitive(b)) => a.name == b.name,
            (AV(a),AV(b)) => a.name() == b.name(),
            (Ext(a),Ext(b)) => a.name() == b.name(),
            (Cond(a),Cond(b)) => a.name == b.name,
            (Ass(a),Ass(b)) => a.name == b.name,
            (Whatsit(a),Whatsit(b)) => a.name() == b.name(),
            (MathChar(a),MathChar(b)) => a==b,
            (Def(a),Def(b)) => {
                a.long == b.long &&
                    a.protected == b.protected &&
                    a.sig == b.sig &&
                    a.ret.len() == b.ret.len() &&
                    {
                        for i in 0..a.ret.len() {
                            if a.ret.get(i) != b.ret.get(i) {return false}
                        }
                        true
                    }
            }
            _ => false
        }
    }
}

impl fmt::Display for TeXCommand {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        match self {
            TeXCommand::Primitive(p) =>
                write!(f,"\\{}",p.name),
            TeXCommand::Cond(p) =>
                write!(f,"\\{}",p.name),
            TeXCommand::Int(p) =>
                write!(f,"\\{}",p.name),
            TeXCommand::Ext(p) =>
                write!(f,"External \\{}",p.name()),
            TeXCommand::AV(av) => av.fmt(f),
            TeXCommand::Def(d) => std::fmt::Display::fmt(&d, f),
            TeXCommand::Whatsit(wi) => wi.fmt(f),
            TeXCommand::Ass(a) =>
                write!(f,"\\{}",a.name),
            TeXCommand::Char(tk) => write!(f,"CHAR({})",tk),
            TeXCommand::MathChar(i) => write!(f,"MATHCHAR({})",i)
        }
    }
}

impl TeXCommand {
    pub fn meaning(&self,catcodes:&CategoryCodeScheme) -> TeXString {
        use TeXCommand::*;
        use std::str::FromStr;
        match self {
            Char(c) => match c.catcode {
                CategoryCode::Space => "blank space ".into(),
                CategoryCode::Letter => {
                    let s : TeXString = "the letter ".into();
                    s + c.char.into()
                },
                CategoryCode::Other => {
                    let s : TeXString = "the character ".into();
                    s + c.char.into()
                },
                _ => todo!("{}",self)
            }
            TeXCommand::Def(d) => {
                let escape : TeXString = if catcodes.escapechar != 255 {catcodes.escapechar.into()} else {"".into()};
                let mut meaning : TeXString = "".into();
                if d.protected {
                    meaning += escape.clone();
                    meaning += "protected ".into()
                }
                if d.long {
                    meaning += escape.clone();
                    meaning += "long ".into()
                }
                meaning += "macro:".into();
                for s in &d.sig.elems {
                    match s {
                        ParamToken::Token(tk) => {
                            match tk.catcode {
                                CategoryCode::Escape => {
                                    meaning += escape.clone();
                                    meaning += tk.name();
                                    meaning += " ".into()
                                }
                                _ => meaning += tk.char.into()
                            }
                        },
                        ParamToken::Param(0,u) => meaning += vec!(*u,*u).into(),
                        ParamToken::Param(i,u) => {
                            meaning += (*u).into();
                            meaning += i.to_string().into();
                        }
                    }
                }
                meaning += "->".into();
                for s in &d.ret {
                    match s {
                        ParamToken::Token(tk) => {
                            match tk.catcode {
                                CategoryCode::Escape => {
                                    meaning += escape.clone();
                                    meaning += tk.name();
                                    meaning += " ".into()
                                }
                                _ => meaning += tk.char.into()
                            }
                        },
                        ParamToken::Param(0,u) => meaning += vec!(*u,*u).into(),
                        ParamToken::Param(i,u) => {
                            meaning += (*u).into();
                            meaning += i.to_string().into();
                        }
                    }
                }
                meaning
            }
            _ => todo!("{}",self)
        }
    }
    pub fn name(&self) -> Option<TeXString> {
        match self {
            TeXCommand::Char(_) => None,
            TeXCommand::Ass(a) => Some(a.name.into()),
            TeXCommand::Primitive(pr) => Some(pr.name.into()),
            TeXCommand::AV(av) => av.name(),
            TeXCommand::Ext(jr) => Some(jr.name().into()),
            TeXCommand::Cond(c) => Some(c.name.into()),
            TeXCommand::Int(i) => Some(i.name.into()),
            TeXCommand::Def(_) => None,
            TeXCommand::Whatsit(wi) => wi.name(),
            TeXCommand::MathChar(_) => None
        }
    }
    pub fn as_expandable(self) -> Result<Expandable,TeXCommand> {
        match self {
            TeXCommand::Cond(c) => Ok(Expandable::Cond(c)),
            TeXCommand::Ext(e) if e.expandable() => Ok(Expandable::Ext(e)),
            TeXCommand::Primitive(p) if p.expandable => Ok(Expandable::Primitive(p)),
            TeXCommand::Def(d) if !d.protected => Ok(Expandable::Def(d)),
            _ => Err(self)
        }
    }
    pub fn as_expandable_with_protected(self) -> Result<Expandable,TeXCommand> {
        match self {
            TeXCommand::Cond(c) => Ok(Expandable::Cond(c)),
            TeXCommand::Ext(e) if e.expandable() => Ok(Expandable::Ext(e)),
            TeXCommand::Primitive(p) if p.expandable => Ok(Expandable::Primitive(p)),
            TeXCommand::Def(d) => Ok(Expandable::Def(d)),
            _ => Err(self)
        }
    }
    pub fn as_hasnum(self) -> Result<HasNum,TeXCommand> {
        match self {
            TeXCommand::AV(ref av) => match av {
                AssignableValue::Register(s) => Ok(HasNum::Register(*s)),
                AssignableValue::Dim(s) => Ok(HasNum::Dim(*s)),
                AssignableValue::Skip(s) => Ok(HasNum::Skip(*s)),
                AssignableValue::Int(d) => Ok(HasNum::AssInt(*d)),
                AssignableValue::PrimDim(d) => Ok(HasNum::PrimDim(*d)),
                AssignableValue::PrimReg(d) => Ok(HasNum::PrimReg(*d)),
                AssignableValue::PrimSkip(d) => Ok(HasNum::PrimSkip(*d)),
                AssignableValue::Toks(s) => Err(self),
                AssignableValue::PrimToks(_) => Err(self),
            },
            TeXCommand::Ext(ext) if ext.has_num() => Ok(HasNum::Ext(ext)),
            TeXCommand::Int(i) => Ok(HasNum::Int(i)),
            TeXCommand::MathChar(u) => Ok(HasNum::Char(u)),
            TeXCommand::Char(u) => Ok(HasNum::Char(u.char as u32)),
            _ => Err(self)
        }
    }
    pub fn as_assignment(self) -> Result<Assignment,TeXCommand> {
        match self {
            TeXCommand::Ass(a) => Ok(Assignment::Prim(a)),
            TeXCommand::AV(av) => Ok(Assignment::Value(av)),
            TeXCommand::Ext(ext) if ext.assignable() => Ok(Assignment::Ext(ext)),
            _ => Err(self)
        }
    }
}