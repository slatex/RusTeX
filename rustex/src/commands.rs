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
    pub apply:fn(cs:Token,itp:&mut Interpreter) -> Result<Expansion,TeXError>,
    pub expandable : bool,
    pub name: &'static str
}
impl PartialEq for PrimitiveExecutable {
    fn eq(&self, other: &Self) -> bool {
        self.name == other.name
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


pub trait ExternalCommand {
    fn name(&self) -> String;
    fn execute(&self,int : &mut Interpreter) -> bool;
}

#[derive(Clone)]
pub enum TeXCommand<'a> {
    Primitive(&'a PrimitiveExecutable),
    Register(&'a RegisterReference),
    Dimen(&'a DimenReference),
    Ext(Rc<dyn ExternalCommand + 'a>),
    Def
}

impl PartialEq for TeXCommand<'_> {
    fn eq(&self, other: &Self) -> bool {
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
            TeXCommand::Primitive(pr) => pr.name.to_string(),
            TeXCommand::Register(reg) => reg.name.to_string(),
            TeXCommand::Dimen(dr) => dr.name.to_string(),
            TeXCommand::Ext(jr) => jr.name(),
            TeXCommand::Def => todo!()
        }
    }
}