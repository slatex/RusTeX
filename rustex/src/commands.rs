pub mod primitives;
pub mod etex;
pub mod pdftex;
pub mod conditionals;

use crate::ontology::{Expansion, Token};
use crate::interpreter::Interpreter;
use std::rc::Rc;
use std::fmt;
use std::fmt::{Display, Formatter};
use crate::utils::TeXError;

pub struct PrimitiveExecutable {
    pub (in crate) _apply:fn(cs:Token,itp:&Interpreter) -> Result<(),TeXError>,
    pub expandable : bool,
    pub name: &'static str
}
impl PrimitiveExecutable {
    pub fn apply(&self,cs:Token,itp:&Interpreter) -> Result<(),TeXError> {
        (self._apply)(cs,itp)
    }
}
pub struct Conditional {
    pub name: &'static str,
    _apply:fn(int:&Interpreter,cond:u8,unless:bool) -> Result<(),TeXError>
}
impl Conditional {
    pub fn expand(&self,tk:Token,int:&Interpreter) -> Result<(),TeXError> {
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

#[derive(Clone)]
pub enum AssignableValue {
    Dim((u8,String)),
    Register((u8, String)),
    Int(&'static AssValue<i32>),
    PrimReg(&'static RegisterReference),
    PrimDim(&'static DimenReference)
}

impl AssignableValue {
    pub fn name(&self) -> String {
        use AssignableValue::*;
        match self {
            Dim((_,s)) => s.to_string(),
            Register((_,s)) => s.to_string(),
            Int(i) => i.name.to_string(),
            PrimReg(r) => r.name.to_string(),
            PrimDim(d) => d.name.to_string()
        }
    }
}

pub struct IntCommand {
    pub _getvalue: fn(int: &Interpreter) -> Result<i32,TeXError>,
    pub name : &'static str
}

pub enum HasNum {
    Dim((u8,String)),
    Register((u8,String)),
    AssInt(&'static AssValue<i32>),
    Int(&'static IntCommand),
    PrimReg(&'static RegisterReference),
    PrimDim(&'static DimenReference),
    Ext(Rc<dyn ExternalCommand>)
}

impl HasNum {
    pub fn get(&self,int:&Interpreter) -> Result<i32,TeXError> {
        use HasNum::*;
        use crate::utils::u8toi16;
        match self {
            Dim((i,_)) => Ok(int.state_dimension(u8toi16(*i))),
            Register((i,_)) => Ok(int.state_register(u8toi16(*i))),
            AssInt(i) => (i._getvalue)(int),
            Int(i) => (i._getvalue)(int),
            PrimReg(r) => Ok(int.state_register(-u8toi16(r.index))),
            PrimDim(r) => Ok(int.state_dimension(-u8toi16(r.index))),
            Ext(r) => r.get_num(int),
        }
    }
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

pub enum Expandable {
    Cond(&'static Conditional),
    Primitive(&'static PrimitiveExecutable),
    Ext(Rc<dyn ExternalCommand>),
    Def(DefMacro)
}

impl Expandable {
    pub fn expand(&self,tk:Token,int:&Interpreter) -> Result<(),TeXError> {
        use Expandable::*;
        match self {
            Cond(c) => c.expand(tk,int),
            Primitive(p) => (p._apply)(tk,int),
            Ext(p) => Ok(int.push_expansion(p.expand(int)?)),
            Def(d) => todo!()
        }
    }
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

pub enum Assignment {
    //Register(&'a RegisterReference),
    //Dimen(&'a DimenReference),
    Value(AssignableValue),
    Ext(Rc<dyn ExternalCommand>),
    Prim(&'static PrimitiveAssignment)
}

use crate::interpreter::state::{StateChange,RegisterStateChange};

impl Assignment {
    pub fn assign(&self,int:&Interpreter,global:bool) -> Result<(),TeXError> {
        use crate::utils::u8toi16;
        match self {
            Assignment::Prim(p) => (p._assign)(int,global),
            Assignment::Value(av) => match av {
                AssignableValue::Int(d) => (d._assign)(int,global),
                AssignableValue::Register((i,_)) => {
                    int.read_eq();
                    let num = int.read_number()?;
                    int.change_state(StateChange::Register(RegisterStateChange {
                        index: u8toi16(*i),
                        value: num,
                        global
                    }));
                    Ok(())
                }
                AssignableValue::Dim((i,_)) => {
                    int.read_eq();
                    let num = int.read_dimension()?;
                    int.change_state(StateChange::Dimen(RegisterStateChange {
                        index: u8toi16(*i),
                        value: num,
                        global
                    }));
                    Ok(())
                }
                AssignableValue::PrimReg(r) => {
                    int.read_eq();
                    let num = int.read_number()?;
                    int.change_state(StateChange::Register(RegisterStateChange {
                        index: -u8toi16(r.index),
                        value: num,
                        global
                    }));
                    Ok(())
                },
                AssignableValue::PrimDim(r) => {
                    int.read_eq();
                    let num = int.read_dimension()?;
                    int.change_state(StateChange::Dimen(RegisterStateChange {
                        index: -u8toi16(r.index),
                        value: num,
                        global
                    }));
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
    fn get_num(&self,int:&Interpreter) -> Result<i32,TeXError>;
}

#[derive(Clone)]
pub enum ParamToken {
    Param(u8),
    Token(Token)
}
impl ParamToken {
    pub fn as_string(&self) -> String { match self {
        ParamToken::Param(0) => "##".to_owned(),
        ParamToken::Param(i) => "#".to_owned() + &i.to_string(),
        ParamToken::Token(tk) => tk.as_string()
    } }
}
impl Display for ParamToken {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        use ansi_term::Colour::*;
        match self {
            ParamToken::Param(_) => write!(f,"{}",Yellow.paint(self.as_string())),
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
impl Display for Signature {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        for e in &self.elems {
            write!(f,"{}",e);
        }
        let s = fmt::format(format_args!("hello {}", "world"));
        if self.endswithbrace {write!(f,"{}","{")} else {
            write!(f,"")
        }
    }
}
pub struct ParamList<'a>(&'a Vec<ParamToken>);

impl Display for ParamList<'_> {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        for e in self.0 {
            write!(f,"{}",e);
        }
        write!(f,"")
    }
}

#[derive(Clone)]
pub struct DefMacro {
    pub name:String,
    pub protected:bool,
    pub long:bool,
    pub sig:Signature,
    pub ret:Vec<ParamToken>
}
impl DefMacro {
    pub fn expand(&self,tk:Token,int:&Interpreter) -> Result<(),TeXError> {
        todo!()
    }
}

#[derive(Clone)]
pub enum TeXCommand {
    Primitive(&'static PrimitiveExecutable),
    AV(AssignableValue),
    /*
    Register(&'a RegisterReference),
    Dimen(&'a DimenReference),
     */
    Ext(Rc<dyn ExternalCommand>),
    Cond(&'static Conditional),
    Int(&'static IntCommand),
    Char((String,Token)),
    Ass(&'static PrimitiveAssignment),
    Def(Rc<DefMacro>)
}

impl PartialEq for TeXCommand {
    fn eq(&self, _other: &Self) -> bool {
        todo!()
    }
}

impl fmt::Display for TeXCommand {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        match self {
            TeXCommand::Primitive(p) =>
                write!(f,"{}",p.name),
            _ => todo!("commands.rs 27")
        }
    }
}
impl TeXCommand {
    pub fn defmacro(_tks : Vec<Token>,_source:Rc<Token>,_protected:bool) -> TeXCommand {
        todo!("commands.rs 33")
    }
    pub fn name(&self) -> String {
        match self {
            TeXCommand::Char((a,_)) => a.to_string(),
            TeXCommand::Ass(a) => a.name.to_string(),
            TeXCommand::Primitive(pr) => pr.name.to_string(),
            TeXCommand::AV(av) => av.name(),
            TeXCommand::Ext(jr) => jr.name(),
            TeXCommand::Cond(c) => c.name.to_string(),
            TeXCommand::Int(i) => i.name.to_string(),
            TeXCommand::Def(d) => d.name.clone()
        }
    }
    pub fn as_expandable(self) -> Result<Expandable,TeXCommand> {
        match self {
            TeXCommand::Cond(c) => Ok(Expandable::Cond(c)),
            TeXCommand::Ext(e) if e.expandable() => Ok(Expandable::Ext(e)),
            TeXCommand::Primitive(p) if p.expandable => Ok(Expandable::Primitive(p)),
            TeXCommand::Def(d) if !d.protected => todo!(),
            _ => Err(self)
        }
    }
    pub fn as_hasnum(self) -> Result<HasNum,TeXCommand> {
        match self {
            TeXCommand::AV(av) => match av {
                AssignableValue::Register((d,s)) => Ok(HasNum::Register((d,s))),//Some(HasNum::AssDim(d)),
                AssignableValue::Dim((d,s)) => Ok(HasNum::Dim((d,s))),//Some(HasNum::AssDim(d)),
                AssignableValue::Int(d) => Ok(HasNum::AssInt(d)),
                AssignableValue::PrimDim(d) => Ok(HasNum::PrimDim(d)),
                AssignableValue::PrimReg(d) => Ok(HasNum::PrimReg(d)),
            },
            TeXCommand::Ext(ext) if ext.has_num() => Ok(HasNum::Ext(ext)),
            TeXCommand::Int(i) => Ok(HasNum::Int(i)),
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