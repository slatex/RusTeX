use std::ops::Deref;
use crate::interpreter::Interpreter;
use crate::ontology::{Expansion, Token};
use crate::commands::{TeXCommand, Conditional, PrimitiveExecutable};
use crate::utils::TeXError;
use crate::catcodes::CategoryCode;

fn dotrue(int: &Interpreter,cond:u8,unless:bool) -> Result<(),TeXError> {
    if unless {
        dofalse(int,cond,false)
    } else {
        int.setcondition(cond,true);
        Ok(())
    }
}
fn dofalse(int: &Interpreter,cond:u8,unless:bool) -> Result<(),TeXError> {
    if unless {
        dotrue(int,cond,false)
    } else {
        let initifs = int.setcondition(cond,false);
        let mut inifs = initifs;
        while int.has_next() {
            let next = int.next_token();
            match next.catcode {
                CategoryCode::Escape | CategoryCode::Active => {
                    match int.state_get_command(&next.cmdname()) {
                        None => {}
                        Some(p) => {
                            match p.deref() {
                                TeXCommand::Primitive(x) if **x == FI && inifs == 0 => {
                                    int.popcondition();
                                    return Ok(())
                                }
                                _ => todo!()
                            }
                        }
                    }
                }
                _ => {}
            }
        }
        Err(TeXError::new("File ended unexpectedly".to_string()))
    }
}

pub static FI : PrimitiveExecutable = PrimitiveExecutable {
    _apply: |tk,int| {
        int.popcondition();
        Ok(Expansion::dummy(vec!()))
    },
    expandable: true,
    name: "fi"
};

pub static ELSE: PrimitiveExecutable = PrimitiveExecutable {
    _apply: |tk,int| {
        int.popcondition();
        Ok(Expansion::dummy(vec!()))
    },
    expandable: true,
    name: "else"
};

pub static IFNUM : Conditional = Conditional {
    _apply: |int,cond,unless| {
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
        if istrue {dotrue(int,cond,unless)} else {dofalse(int,cond,unless)}
    },
    name:"ifnum"
};

pub fn conditional_commands() -> Vec<TeXCommand<'static>> {vec![
    TeXCommand::Cond(&IFNUM),
    TeXCommand::Primitive(&ELSE),
    TeXCommand::Primitive(&FI)
]}