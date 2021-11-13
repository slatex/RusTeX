pub mod primitives;
pub mod etex;
pub mod pdftex;
pub mod conditionals;

use crate::ontology::{Expansion, Token};
use crate::interpreter::Interpreter;
use std::rc::Rc;
use std::fmt;
use std::fmt::Formatter;
use crate::utils::TeXError;

pub struct PrimitiveExecutable {
    pub (in crate) _apply:fn(cs:Token,itp:&Interpreter) -> Result<Expansion,TeXError>,
    pub expandable : bool,
    pub name: &'static str
}
impl PrimitiveExecutable {
    pub fn apply(&self,cs:Token,itp:&Interpreter) -> Result<Expansion,TeXError> {
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

pub struct AssValue<'a, T> {
    pub name: &'a str,
    pub _assign: fn(int: &Interpreter,global: bool) -> Result<(),TeXError>,
    pub _getvalue: fn(int: &Interpreter) -> Result<T,TeXError>
}

#[derive(Clone)]
pub enum AssignableValue<'a> {
    Dim(&'a AssValue<'a,i32>),
    Int(&'a AssValue<'a,i32>),
    Register(&'a RegisterReference),
    Dimen(&'a DimenReference)
}

impl AssignableValue<'_> {
    pub fn name(&self) -> String {
        use AssignableValue::*;
        match self {
            Dim(d) => d.name.to_string(),
            Int(i) => i.name.to_string(),
            Register(r) => r.name.to_string(),
            Dimen(d) => d.name.to_string()
        }
    }
}

pub struct IntCommand<'a> {
    pub _getvalue: fn(int: &Interpreter) -> Result<i32,TeXError>,
    pub name : &'a str
}

pub enum HasNum<'a> {
    AssDim(&'a AssValue<'a,i32>),
    AssInt(&'a AssValue<'a,i32>),
    Int(&'a IntCommand<'a>),
    Register(&'a RegisterReference),
    Dimen(&'a DimenReference),
    Ext(Rc<dyn ExternalCommand + 'a>)
}

impl HasNum<'_> {
    pub fn get(&self,int:&Interpreter) -> Result<i32,TeXError> {
        use HasNum::*;
        match self {
            AssDim(d) => (d._getvalue)(int),
            AssInt(i) => (i._getvalue)(int),
            Register(r) => Ok(int.state_register(r.index)),
            Dimen(r) => Ok(int.state_dimension(r.index)),
            Ext(r) => r.get_num(int),
            Int(i) => (i._getvalue)(int)
        }
    }
}

#[derive(PartialEq)]
pub struct RegisterReference {
    pub index: i8,
    pub name: &'static str
}

#[derive(PartialEq)]
pub struct DimenReference {
    pub index: i8,
    pub name: &'static str
}

pub enum Expandable<'a> {
    Cond(&'a Conditional),
    Primitive(&'a PrimitiveExecutable),
    Ext(Rc<dyn ExternalCommand + 'a>),
    Def
}

impl Expandable<'_> {
    pub fn expand(&self,tk:Token,int:&Interpreter) -> Result<(),TeXError> {
        use Expandable::*;
        match self {
            Cond(c) => c.expand(tk,int),
            Primitive(p) => Ok(int.push_expansion((p._apply)(tk,int)?)),
            Ext(p) => Ok(int.push_expansion(p.expand(int)?)),
            Def => todo!()
        }
    }
}

pub struct PrimitiveAssignment<'a> {
    pub name: &'a str,
    pub _assign: fn(int: &Interpreter,global: bool) -> Result<(),TeXError>
}

pub enum Assignment<'a> {
    //Register(&'a RegisterReference),
    //Dimen(&'a DimenReference),
    Value(&'a AssignableValue<'a>),
    Ext(Rc<dyn ExternalCommand + 'a>),
    Prim(&'a PrimitiveAssignment<'a>)
}

use crate::interpreter::state::{StateChange,RegisterStateChange};

impl Assignment<'_> {
    pub fn assign(&self,int:&Interpreter,global:bool) -> Result<(),TeXError> {
        match self {
            Assignment::Prim(p) => (p._assign)(int,global),
            Assignment::Value(av) => match av {
                AssignableValue::Int(d) => (d._assign)(int,global),
                AssignableValue::Dim(d) => (d._assign)(int,global),
                AssignableValue::Register(r) => {
                    int.read_eq();
                    let num = int.read_number()?;
                    int.change_state(StateChange::Register(RegisterStateChange {
                        index: r.index,
                        value: num,
                        global
                    }));
                    Ok(())
                },
                AssignableValue::Dimen(r) => {
                    int.read_eq();
                    let num = int.read_dimension()?;
                    int.change_state(StateChange::Dimen(RegisterStateChange {
                        index: r.index,
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
pub enum TeXCommand<'a> {
    Primitive(&'a PrimitiveExecutable),
    AV(AssignableValue<'a>),
    /*
    Register(&'a RegisterReference),
    Dimen(&'a DimenReference),
     */
    Ext(Rc<dyn ExternalCommand + 'a>),
    Cond(&'a Conditional),
    Int(&'a IntCommand<'a>),
    Char((String,Token)),
    Ass(&'a PrimitiveAssignment<'a>),
    Def
}

impl PartialEq for TeXCommand<'_> {
    fn eq(&self, _other: &Self) -> bool {
        todo!()
    }
}

impl<'a> fmt::Display for TeXCommand<'a> {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        match self {
            TeXCommand::Primitive(p) =>
                write!(f,"{}",p.name),
            _ => todo!("commands.rs 27")
        }
    }
}
impl<'b> TeXCommand<'b> {
    pub fn defmacro<'a>(_tks : Vec<Token>,_source:Rc<Token>,_protected:bool) -> TeXCommand<'a> {
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
            TeXCommand::Def => todo!()
        }
    }
    pub fn as_expandable(&self) -> Option<Expandable<'_>> {
        match self {
            TeXCommand::Cond(c) => Some(Expandable::Cond(c)),
            TeXCommand::Ext(e) if e.expandable() => Some(Expandable::Ext(Rc::clone(e))),
            TeXCommand::Primitive(p) if p.expandable => Some(Expandable::Primitive(p)),
            TeXCommand::Def => todo!(),
            _ => None
        }
    }
    pub fn as_hasnum(&self) -> Option<HasNum<'_>> {
        match self {
            TeXCommand::AV(av) => match av {
                AssignableValue::Dim(d) => Some(HasNum::AssDim(d)),
                AssignableValue::Int(d) => Some(HasNum::AssInt(d)),
                AssignableValue::Dimen(d) => Some(HasNum::Dimen(d)),
                AssignableValue::Register(d) => Some(HasNum::Register(d)),
            },
            TeXCommand::Ext(ext) if ext.has_num() => Some(HasNum::Ext(Rc::clone(&ext))),
            TeXCommand::Int(i) => Some(HasNum::Int(i)),
            _ => None
        }
    }
    pub fn as_assignment(&self) -> Option<Assignment<'_>> {
        match self {
            TeXCommand::Ass(a) => Some(Assignment::Prim(a)),
            TeXCommand::AV(av) => Some(Assignment::Value(av)),
            TeXCommand::Ext(ext) if ext.assignable() => Some(Assignment::Ext(Rc::clone(&ext))),
            _ => None
        }
    }
}