pub mod primitives;
pub mod pdftex;
pub mod conditionals;

use crate::ontology::{Expansion, ExpansionRef, Token};
use crate::interpreter::Interpreter;
use std::rc::Rc;
use std::fmt;
use std::fmt::{Display, Formatter, Pointer};
use std::str::from_utf8;
use crate::catcodes::{CategoryCode, CategoryCodeScheme};
use crate::interpreter::dimensions::Numeric;
use crate::utils::{TeXError, TeXString};
use crate::{COPY_COMMANDS_FULL, log};

pub struct PrimitiveExecutable {
    pub (in crate) _apply:fn(tk:&mut Expansion,itp:&Interpreter) -> Result<(),TeXError>,
    pub expandable : bool,
    pub name: &'static str
}
impl PrimitiveExecutable {
    pub fn apply(&self,tk:&mut Expansion,itp:&Interpreter) -> Result<(),TeXError> {
        (self._apply)(tk,itp)
    }
}
pub struct Conditional {
    pub name: &'static str,
    _apply:fn(int:&Interpreter,cond:u8,unless:bool) -> Result<(),TeXError>
}
impl Conditional {
    pub fn expand(&self,int:&Interpreter) -> Result<(),TeXError> {
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
    pub _assign: fn(rf:ExpansionRef,int: &Interpreter,global: bool) -> Result<(),TeXError>,
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
    pub _assign: fn(rf:ExpansionRef,int: &Interpreter,global: bool) -> Result<(),TeXError>
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
    pub ret:Vec<Token>
}
impl Display for DefMacro {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        write!(f,"{}->{}{}{}",self.sig,"{",TokenList(&self.ret),"}")
    }
}

use crate::stomach::whatsits::ExecutableWhatsit;

pub struct ProvidesExecutableWhatsit {
    pub name: &'static str,
    pub _get: fn(tk:&Token,int: &Interpreter) -> Result<ExecutableWhatsit,TeXError>
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

pub struct HasNum(pub (in crate) TeXCommand);

impl HasNum {
    pub(crate) fn get(&self,int:&Interpreter) -> Result<Numeric,TeXError> {
        use PrimitiveTeXCommand::*;
        use AssignableValue::*;
        use crate::utils::u8toi16;
        match self.0.get_orig() {
            AV(Dim(i)) => Ok(Numeric::Dim(int.state_dimension(u8toi16(i)))),
            AV(Register(i)) => Ok(Numeric::Int(int.state_register(u8toi16(i)))),
            AV(Skip(i)) => Ok(Numeric::Skip(int.state_skip(u8toi16(i)))),
            AV(AssignableValue::Int(i)) => Ok(Numeric::Int((i._getvalue)(int)?)),
            PrimitiveTeXCommand::Int(i) => Ok(Numeric::Int((i._getvalue)(int)?)),
            AV(PrimReg(r)) => Ok(Numeric::Int(int.state_register(-u8toi16(r.index)))),
            AV(PrimDim(r)) => Ok(Numeric::Dim(int.state_dimension(-u8toi16(r.index)))),
            AV(PrimSkip(r)) => Ok(Numeric::Skip(int.state_skip(-u8toi16(r.index)))),
            Ext(r) => r.get_num(int),
            Char(u) => Ok(Numeric::Int(u.char as i32)),
            MathChar(u) => Ok(Numeric::Int(u as i32)),
            _ => unreachable!("{}",self.0.get_orig())
        }
    }
}

pub struct Expandable(pub (in crate) TeXCommand);

use crate::TeXErr;
use crate::references::SourceReference;

impl Expandable {
    pub fn get_expansion(&self,tk:Token,int:&Interpreter) -> Result<Option<Expansion>,TeXError> {
        use PrimitiveTeXCommand::*;
        log!("Expanding {}",tk);
        match self.0.get_orig() {
            Cond(c) => {c.expand(int)?; Ok(None)},
            Primitive(p) => {
                let mut exp = Expansion(tk,Rc::new(self.0.clone()),vec!());
                (p._apply)(&mut exp,int)?;
                Ok(Some(exp))
            },
            Ext(p) => {
                let mut exp = Expansion(tk,Rc::new(self.0.clone()),vec!());
                p.expand(&mut exp,int)?;
                Ok(Some(exp))
            },
            Def(d) => Ok(Some(self.doDef(tk,int,d)?)),
            _ => unreachable!()
        }
    }
    pub fn expand(&self,tk:Token,int:&Interpreter) -> Result<(),TeXError> {
        use PrimitiveTeXCommand::*;
        match self.get_expansion(tk,int)? {
            Some(exp) => Ok(int.push_expansion(exp)),
            None => Ok(())
        }
    }

    fn doDef(&self,tk:Token,int:&Interpreter,d:DefMacro) -> Result<Expansion,TeXError> {
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
                            let mut retarg : Vec<Token> = vec!();
                            loop {
                                int.assert_has_next()?;
                                let next = int.next_token();
                                match next.catcode {
                                    CategoryCode::BeginGroup => {
                                        int.requeue(next);
                                        break
                                    }
                                    _ => retarg.push(next)
                                }
                            }
                            args.push(retarg)
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
        let mut exp = Expansion(tk,Rc::new(self.0.clone()),vec!());
        let rf = exp.get_ref();
        let mut i = 0;
        while i < d.ret.len() {
            let tk = d.ret.get(i).unwrap();
            match tk.catcode {
                CategoryCode::Parameter => {
                    i += 1;
                    let next = d.ret.get(i).unwrap();
                    match next.catcode {
                        CategoryCode::Parameter => {
                            i += 1;
                            exp.2.push(tk.copied(rf.clone()))
                        }
                        _ => {
                            i += 1;
                            let arg = next.char - 49;
                            if (arg < 0 || arg >= d.sig.arity) {
                                TeXErr!(int,"Expected argument number; got:{}",next)
                            }
                            for tk in args.get(arg as usize).unwrap() { exp.2.push(tk.clone()) }
                        }
                    }
                }
                _ => {
                    i += 1;
                    exp.2.push(tk.copied(rf.clone()))
                },
            }
        }
        Ok(exp)
    }
}


pub struct Assignment(pub (in crate) TeXCommand);

use crate::interpreter::state::StateChange;

impl Assignment {
    pub fn assign(&self,tk:Token,int:&Interpreter,globally:bool) -> Result<(),TeXError> {
        use crate::utils::u8toi16;
        use crate::interpreter::dimensions::dimtostr;
        use crate::commands::primitives::GLOBALDEFS;;
        use PrimitiveTeXCommand::*;

        let globals = int.state_register(-u8toi16(GLOBALDEFS.index));
        let global = !(globals < 0) && ( globally || globals > 0 );
        let rf = ExpansionRef(tk,Rc::new(self.0.clone()));

        match self.0.get_orig() {
            Ass(p) => (p._assign)(rf,int, global),
            AV(av) => match av {
                AssignableValue::Int(d) => (d._assign)(rf,int, global),
                AssignableValue::Register(i) => {
                    int.read_eq();
                    let num = int.read_number()?;
                    log!("Assign register {} to {}",i,num);
                    int.change_state(StateChange::Register(u8toi16(i), num, global));
                    Ok(())
                }
                AssignableValue::Dim(i) => {
                    int.read_eq();
                    let num = int.read_dimension()?;
                    log!("Assign dimen register {} to {}",i,dimtostr(num));
                    int.change_state(StateChange::Dimen(u8toi16(i), num, global));
                    Ok(())
                }
                AssignableValue::Skip(i) => {
                    int.read_eq();
                    let num = int.read_skip()?;
                    log!("Assign skip register {} to {}",i,num);
                    int.change_state(StateChange::Skip(u8toi16(i), num, global));
                    Ok(())
                },
                AssignableValue::PrimSkip(r) => {
                    int.read_eq();
                    let num = int.read_skip()?;
                    log!("Assign {} to {}",r.name,num);
                    int.change_state(StateChange::Skip(-u8toi16(r.index), num, global));
                    Ok(())
                },
                AssignableValue::Toks(i) => {
                    int.expand_until(false)?;
                    match int.next_token().catcode {
                        CategoryCode::BeginGroup => {}
                        _ => TeXErr!(int,"Expected Begin Group Token")
                    }
                    let toks = int.read_token_list(false, false)?;
                    int.change_state(StateChange::Tokens(u8toi16(i), toks, global));
                    Ok(())
                },
                AssignableValue::PrimToks(r) => {
                    int.expand_until(false)?;
                    match int.next_token().catcode {
                        CategoryCode::BeginGroup => {}
                        _ => TeXErr!(int,"Expected Begin Group Token")
                    }
                    let toks = int.read_token_list(false, false)?;
                    int.change_state(StateChange::Tokens(-u8toi16(r.index), toks, global));
                    Ok(())
                },
                AssignableValue::PrimReg(r) => {
                    int.read_eq();
                    let num = int.read_number()?;
                    log!("Assign {} to {}",r.name,num);
                    int.change_state(StateChange::Register(-u8toi16(r.index), num, global));
                    Ok(())
                },
                AssignableValue::PrimDim(r) => {
                    int.read_eq();
                    let num = int.read_dimension()?;
                    log!("Assign {} to {}",r.name,dimtostr(num));
                    int.change_state(StateChange::Dimen(-u8toi16(r.index), num, global));
                    Ok(())
                }
            },
            Ext(ext) => ext.assign(int, global),
            _ => unreachable!()
        }
    }
}


pub trait ExternalCommand {
    fn expandable(&self) -> bool;
    fn assignable(&self) -> bool;
    fn has_num(&self) -> bool;
    fn name(&self) -> String;
    fn execute(&self,int : &Interpreter) -> Result<(),TeXError>;
    fn expand(&self,exp:&mut Expansion,int:&Interpreter) -> Result<(),TeXError>;
    fn assign(&self,int:&Interpreter,global:bool) -> Result<(),TeXError>;
    fn get_num(&self,int:&Interpreter) -> Result<Numeric,TeXError>;
}

#[derive(Clone)]
pub enum ParamToken {
    Param(u8,Token),
    Token(Token)
}
impl PartialEq for ParamToken {
    fn eq(&self, other: &Self) -> bool {
        match (self,other) {
            (ParamToken::Param(a1,_),ParamToken::Param(b1,_)) => a1 == b1,
            (ParamToken::Token(a),ParamToken::Token(b)) => a == b,
            _ => false
        }
    }
}
impl Display for ParamToken {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        use ansi_term::Colour::*;
        match self {
            ParamToken::Param(0,u) => write!(f,"{}",Yellow.paint(TeXString(vec!(u.char,u.char)).to_string())),
            ParamToken::Param(i,u) => write!(f,"{}{}",Yellow.paint(TeXString(vec!(u.char)).to_string()),Yellow.paint(i.to_string())),
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
pub enum PrimitiveTeXCommand {
    Primitive(&'static PrimitiveExecutable),
    AV(AssignableValue),
    Ext(Rc<dyn ExternalCommand>),
    Cond(&'static Conditional),
    Int(&'static IntCommand),
    Char(Token),
    Ass(&'static PrimitiveAssignment),
    Def(DefMacro),
    Whatsit(ProvidesWhatsit),
    MathChar(u32)
}

impl PrimitiveTeXCommand {
    pub fn as_ref(self,rf:&ExpansionRef) -> TeXCommand {
        if COPY_COMMANDS_FULL {
            TeXCommand::Ref(ExpansionRef(rf.0.clone(),Rc::new(TeXCommand::Prim(self))))//(rf, Box::new(TeXCommand::Prim(self)))
        } else {
            TeXCommand::Prim(self)
        }
    }
}

#[derive(Clone)]
pub enum TeXCommand {
    Prim(PrimitiveTeXCommand),
    Ref(ExpansionRef)
}
impl TeXCommand {
    pub fn get_orig(&self) -> PrimitiveTeXCommand {
        let mut curr = self;
        loop {
            match curr {
                TeXCommand::Prim(p) => return p.clone(),
                TeXCommand::Ref(c) => curr = &c.1
            }
        }
    }
}
impl PartialEq for TeXCommand {
    fn eq(&self, other: &Self) -> bool {
        self.get_orig() == other.get_orig()
    }
}

impl PartialEq for PrimitiveTeXCommand {
    fn eq(&self, other: &Self) -> bool {
        use PrimitiveTeXCommand::*;
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

impl fmt::Display for PrimitiveTeXCommand {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        use PrimitiveTeXCommand::*;
        match self {
            Primitive(p) =>
                write!(f,"\\{}",p.name),
            Cond(p) =>
                write!(f,"\\{}",p.name),
            Int(p) =>
                write!(f,"\\{}",p.name),
            Ext(p) =>
                write!(f,"External \\{}",p.name()),
            AV(av) => av.fmt(f),
            Def(d) => std::fmt::Display::fmt(&d, f),
            Whatsit(wi) => wi.fmt(f),
            Ass(a) =>
                write!(f,"\\{}",a.name),
            Char(tk) => write!(f,"CHAR({})",tk),
            MathChar(i) => write!(f,"MATHCHAR({})",i)
        }
    }
}

impl fmt::Display for TeXCommand {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        write!(f,"{}",self.get_orig())
    }
}

impl TeXCommand {
    pub fn meaning(&self,catcodes:&CategoryCodeScheme) -> TeXString {
        use PrimitiveTeXCommand::*;
        use std::str::FromStr;
        match self.get_orig() {
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
            Def(d) => {
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
                        ParamToken::Param(0,u) => meaning += vec!(u.char,u.char).into(),
                        ParamToken::Param(i,u) => {
                            meaning += (u.char).into();
                            meaning += i.to_string().into();
                        }
                    }
                }
                meaning += "->".into();
                for tk in &d.ret {
                    match tk.catcode {
                        CategoryCode::Escape => {
                            meaning += escape.clone();
                            meaning += tk.name();
                            meaning += " ".into()
                        }
                        _ => meaning += tk.char.into()
                    }
                }
                meaning
            }
            Int(ic) => {
                let mut ret : TeXString = if catcodes.escapechar != 255 {catcodes.escapechar.into()} else {"".into()};
                ret + ic.name.into()
            }
            _ => todo!("{}",self)
        }
    }
    pub fn name(&self) -> Option<TeXString> {
        use PrimitiveTeXCommand::*;
        match self.get_orig() {
            Char(_) => None,
            Ass(a) => Some(a.name.into()),
            Primitive(pr) => Some(pr.name.into()),
            AV(av) => av.name(),
            Ext(jr) => Some(jr.name().into()),
            Cond(c) => Some(c.name.into()),
            Int(i) => Some(i.name.into()),
            Def(_) => None,
            Whatsit(wi) => wi.name(),
            MathChar(_) => None
        }
    }
    pub fn as_expandable(self) -> Result<Expandable,TeXCommand> {
        use PrimitiveTeXCommand::*;
        match self.get_orig() {
            Cond(c) => Ok(Expandable(self)),
            Ext(e) if e.expandable() => Ok(Expandable(self)),
            Primitive(p) if p.expandable => Ok(Expandable(self)),
            Def(d) if !d.protected => Ok(Expandable(self)),
            _ => Err(self)
        }
    }
    pub fn as_expandable_with_protected(self) -> Result<Expandable,TeXCommand> {
        use PrimitiveTeXCommand::*;
        match self.get_orig() {
            Cond(c) => Ok(Expandable(self)),
            Ext(e) if e.expandable() => Ok(Expandable(self)),
            Primitive(p) if p.expandable => Ok(Expandable(self)),
            Def(d) => Ok(Expandable(self)),
            _ => Err(self)
        }
    }
    pub fn as_hasnum(self) -> Result<HasNum,TeXCommand> {
        match self.get_orig() {
            PrimitiveTeXCommand::AV(ref av) => match av {
                AssignableValue::Register(s) => Ok(HasNum(self)),
                AssignableValue::Dim(s) => Ok(HasNum(self)),
                AssignableValue::Skip(s) => Ok(HasNum(self)),
                AssignableValue::Int(d) => Ok(HasNum(self)),
                AssignableValue::PrimDim(d) => Ok(HasNum(self)),
                AssignableValue::PrimReg(d) => Ok(HasNum(self)),
                AssignableValue::PrimSkip(d) => Ok(HasNum(self)),
                AssignableValue::Toks(s) => Err(self),
                AssignableValue::PrimToks(_) => Err(self),
            },
            PrimitiveTeXCommand::Ext(ext) if ext.has_num() =>Ok(HasNum(self)),
            PrimitiveTeXCommand::Int(i) => Ok(HasNum(self)),
            PrimitiveTeXCommand::MathChar(u) => Ok(HasNum(self)),
            PrimitiveTeXCommand::Char(u) => Ok(HasNum(self)),
            _ => Err(self)
        }
    }
    pub fn as_assignment(self) -> Result<Assignment,TeXCommand> {
        match self.get_orig() {
            PrimitiveTeXCommand::Ass(a) => Ok(Assignment(self)),
            PrimitiveTeXCommand::AV(av) => Ok(Assignment(self)),
            PrimitiveTeXCommand::Ext(ext) if ext.assignable() => Ok(Assignment(self)),
            _ => Err(self)
        }
    }
    pub fn as_ref(self,tk:Token) -> TeXCommand {
        if COPY_COMMANDS_FULL {
            TeXCommand::Ref(ExpansionRef(tk,Rc::new(self)))
        } else { self }
    }
}