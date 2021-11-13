use std::ops::Deref;
use crate::commands::{AssignableValue, AssValue, PrimitiveAssignment, PrimitiveExecutable, TeXCommand};
use crate::interpreter::Interpreter;
use crate::ontology::{Token, Expansion};
use crate::catcodes::CategoryCode;
use crate::interpreter::state::{CategoryCodeChange, CommandChange, StateChange};
use crate::utils::TeXError;

pub static PAR : PrimitiveExecutable = PrimitiveExecutable {
    expandable:false,
    name:"par",
    _apply:|cs: Token, _int: &Interpreter| {
        Ok(Expansion {
            cs,
            exp: vec![]
        })
    }
};
pub static RELAX : PrimitiveExecutable = PrimitiveExecutable {
    expandable:false,
    name:"relax",
    _apply:|cs: Token, _int: &Interpreter| {
        Ok(Expansion {
            cs,
            exp: vec![]
        })
    }
};
pub static CATCODE : AssValue<'static,i32> = AssValue {
    name: "catcode",
    _assign: |int,global| {
        let num = int.read_number()? as u8;
        int.read_eq();
        let cat = CategoryCode::fromint(int.read_number()?);
        int.change_state(StateChange::Cat(CategoryCodeChange {
            char: num,
            catcode: cat,
            global
        }));
        Ok(())
    },
    _getvalue: |int| {
        let char = int.read_number()?;
        Ok(CategoryCode::toint(&int.state_catcodes().get_code(char as u8)) as i32)
    }
};
use crate::references::SourceReference;
use std::rc::Rc;
pub static CHARDEF: PrimitiveAssignment<'static> = PrimitiveAssignment {
    name: "chardef",
    _assign: |int,global| {
        let mut cmd: Option<Token> = None;
        while int.has_next() {
            int.skip_ws();
            let next = int.next_token();
            match next.catcode {
                CategoryCode::Escape | CategoryCode::Active => {
                    let p = int.state_get_command(&next.cmdname());
                    match p {
                        None =>{ cmd = Some(next); break }
                        Some(p) => match p.deref() {
                            TeXCommand::Cond(c) => { c.expand(next, int); },
                            TeXCommand::Primitive(p) if p.expandable =>
                                { int.push_expansion((p._apply)(next, int)?); }
                            _ => { cmd = Some(next); break }
                        }
                    }
                }
                _ => return Err(TeXError::new("Command expected; found: ".to_owned() + &next.as_string()))
            }
        };
        match cmd {
            None => Err(TeXError::new("File ended unexpectedly".to_string())),
            Some(c) => {
                int.read_eq();
                let num = int.read_number()?;
                int.change_state(StateChange::Cs(CommandChange {
                    name: c.cmdname(),
                    cmd: Some(Rc::new(TeXCommand::Char((c.cmdname(),
                        Token {
                        char: num as u8,
                        catcode: CategoryCode::Other,
                        name_opt: None,
                        reference: Box::new(SourceReference::None)
                    })))),
                    global
                }));
                Ok(())
            }
        }
    }
};

pub fn tex_commands() -> Vec<TeXCommand<'static>> {vec![
    TeXCommand::Primitive(&PAR),
    TeXCommand::Primitive(&RELAX),
    TeXCommand::AV(AssignableValue::Int(&CATCODE)),
    TeXCommand::Ass(&CHARDEF),
]}