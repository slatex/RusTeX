use crate::interpreter::Interpreter;
use crate::ontology::Token;
use crate::commands::{TeXCommand,PrimitiveExecutable};
use crate::ontology::Expansion;
use crate::utils::TeXError;

#[derive(Clone)]
pub(in crate) struct Condition {
    cond:Option<bool>,
    pub unless:bool,
    index:u8
}
impl Condition {
    pub fn new(int:&Interpreter) -> Condition {
        Condition {
            cond:None,
            unless:false,
            index:int.pushcondition()
        }
    }
}
fn expand(cs: Token, int: &Interpreter) -> Condition {
    Condition::new(int)
}
fn dotrue(int: &Interpreter,cond:&mut Condition,allow_unless:bool) {
    if cond.unless && allow_unless {
        dofalse(int,cond,false)
    } else {
        cond.cond = Some(true)
    }
}
fn dofalse(int: &Interpreter,cond:&mut Condition,allow_unless:bool) {
    if cond.unless && allow_unless {
        dotrue(int,cond,false)
    } else {
        todo!()
    }
}

pub static IFNUM : PrimitiveExecutable = PrimitiveExecutable {
    expandable:true,
    _apply: |cs: Token, int: &Interpreter| {
        let cond = expand(cs.clone(),int);
        let i1 = int.read_number()?;
        let rel = int.read_keyword(vec!["<","=",">"]);
        let i2 = int.read_number()?;
        let istrue = match rel {
            Some(ref s) if s == "<" => i1 < i2,
            Some(ref s) if s == "=" => i1 == i2,
            Some(ref s) if s == ">" => i1 > i2,
            _ => return Err(TeXError::new("Expected '<','=' or '>' in \\ifnum".to_string()))
        };
        println!("\\ifnum: {}{}{}",i1,rel.unwrap(),i2);
        //if istrue {dotrue(int,cond,true)} else {dofalse(int,cond,true)}
        Ok(Expansion {
            cs: cs,
            exp:vec![]
        })
    },
    name: "ifnum"
};

pub fn conditional_commands() -> Vec<TeXCommand<'static>> {vec![
    TeXCommand::Primitive(&IFNUM)
]}