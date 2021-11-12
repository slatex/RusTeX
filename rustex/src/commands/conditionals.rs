use crate::interpreter::Interpreter;
use crate::ontology::Token;
use crate::commands::{TeXCommand,PrimitiveExecutable};
use crate::ontology::Expansion;
use crate::utils::TeXError;

#[derive(Clone)]
pub(in crate) struct Condition {
    pub cond:Option<bool>,
    pub unless:bool
}
fn expand<'a>(cs: Token, int: &'a mut Interpreter) -> &'a mut Condition {
    let cond = Condition {
        cond:None,
        unless:false
    };
    int.state.conditions.push(cond);
    int.state.conditions.last_mut().unwrap()
}
fn dotrue(int: &mut Interpreter,cond:&mut Condition,allow_unless:bool) {
    if cond.unless && allow_unless {
        dofalse(int,cond,false)
    } else {
        cond.cond = Some(true)
    }
}
fn dofalse(int: &mut Interpreter,cond:&mut Condition,allow_unless:bool) {
    if cond.unless && allow_unless {
        dotrue(int,cond,false)
    } else {
        todo!()
    }
}

pub static IFNUM : PrimitiveExecutable = PrimitiveExecutable {
    expandable:true,
    apply: |cs: Token, int: &mut Interpreter| {
        let cond = expand(cs.clone(),int);
        let i1 = int.read_number()?;
        let rel = int.read_keyword(vec!["<","=",">"]);
        let i2 = int.read_number()?;
        let istrue = match rel {
            Some(s) if s == "<" => i1 < i2,
            Some(s) if s == "=" => i1 == i2,
            Some(s) if s == ">" => i1 > i2,
            _ => return Err(TeXError::new("Expected '<','=' or '>' in \\ifnum".to_string()))
        };
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