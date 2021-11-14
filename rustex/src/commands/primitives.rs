use std::ops::Deref;
use crate::commands::{AssignableValue, AssValue, DefMacro, ParamToken, PrimitiveAssignment, PrimitiveExecutable, RegisterReference, Signature, TeXCommand};
use crate::interpreter::Interpreter;
use crate::ontology::{Token, Expansion};
use crate::catcodes::CategoryCode;
use crate::interpreter::state::{CategoryCodeChange, CommandChange, NewlineChange, StateChange};
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
pub static CATCODE : AssValue<i32> = AssValue {
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
pub static CHARDEF: PrimitiveAssignment = PrimitiveAssignment {
    name: "chardef",
    _assign: |int,global| {
        let c = int.read_command_token()?;
        int.read_eq();
        let num = int.read_number()?;
        int.change_state(StateChange::Cs(CommandChange {
            name: c.cmdname(),
            cmd: Some(TeXCommand::Char((c.cmdname(),
                Token {
                char: num as u8,
                catcode: CategoryCode::Other,
                name_opt: None,
                reference: Box::new(SourceReference::None)
            }))),
            global
        }));
        Ok(())
    }
};
pub static COUNTDEF: PrimitiveAssignment = PrimitiveAssignment {
    name:"countdef",
    _assign: |int,global| {
        let cmd = int.read_command_token()?;
        int.read_eq();
        let num = int.read_number()?;

        int.change_state(StateChange::Cs(CommandChange {
            name: cmd.cmdname(),
            cmd: Some(TeXCommand::AV(AssignableValue::Register((num as u8, cmd.cmdname())))),
            global
        }));
        Ok(())
    }
};

use crate::log;

fn readSig(int:&Interpreter) -> Result<Signature,TeXError> {
    let mut retsig : Vec<ParamToken> = Vec::new();
    while int.has_next() {
        let next = int.next_token();
        match next.catcode {
            CategoryCode::BeginGroup => {
                return Ok(Signature {
                    elems: retsig,
                    endswithbrace: false
                })
            }
            _ => todo!()
        }
    }
    Err(TeXError::new("File ended unexpectedly".to_string()))
}

fn doDef(int:&Interpreter,global:bool,protected:bool,long:bool) -> Result<(),TeXError> {
    let command = int.next_token();
    match command.catcode {
        CategoryCode::Escape | CategoryCode::Active => {}
        _ => return Err(TeXError::new("\\def expected control sequence or active character; got: ".to_owned() + &command.as_string()))
    }
    let sig = readSig(int)?;
    let mut ingroups = 0;
    let mut ret : Vec<ParamToken> = Vec::new();
    while int.has_next() {
        let next = int.next_token();
        match next.catcode {
            CategoryCode::BeginGroup => {
                ingroups += 1;
                ret.push(ParamToken::Token(next));
            }
            CategoryCode::EndGroup if ingroups == 0 => {
                log!("\\def {} {} {}{}{}",command.as_string(),sig,"{",ret.iter().map(|x| x.as_string()).collect::<Vec<_>>().join(""),"}");
                let dm = DefMacro {
                    name: "".to_string(),
                    protected,
                    long,
                    sig,
                    ret
                };
                int.change_state(StateChange::Cs(CommandChange {
                    name: command.cmdname(),
                    cmd: Some(TeXCommand::Def(Rc::new(dm))),
                    global
                }));
                return Ok(())
            }
            CategoryCode::EndGroup => {
                ingroups -=1;
                ret.push(ParamToken::Token(next));
            },
            CategoryCode::Parameter => todo!("{}",int.current_line()),
            _ => ret.push(ParamToken::Token(next))
        }
    }
    Err(TeXError::new("File ended unexpectedly".to_string()))
}

pub static DEF: PrimitiveAssignment = PrimitiveAssignment {
    name:"def",
    _assign: |int,global| doDef(int,global,false,false)
};

pub static LET: PrimitiveAssignment = PrimitiveAssignment {
    name:"let",
    _assign: |int,global| {
        let cmd = int.next_token();
        if !matches!(cmd.catcode,CategoryCode::Escape) && !matches!(cmd.catcode,CategoryCode::Active) {
            return Err(TeXError::new("Control sequence or active character expected; found".to_owned() + &cmd.name()))
        }
        int.read_eq();
        let def = int.next_token();
        log!("\\let \\{}={}",cmd.cmdname(),def.as_string());
        let ch = match def.catcode {
            CategoryCode::Escape | CategoryCode::Active => {
                int.state_get_command(&def.cmdname())
            }
            _ => Some(TeXCommand::Char((def.name(),def)))
        };
        int.change_state(StateChange::Cs(CommandChange {
            name: cmd.cmdname(),
            cmd: ch,
            global
        }));
        Ok(())
    }
};

pub static NEWLINECHAR : AssValue<i32> = AssValue {
    name: "newlinechar",
    _assign: |int,global| {
        int.read_eq();
        let num = int.read_number()? as u8;
        log!("\\newlinechar: {}",num);
        int.change_state(StateChange::Newline(NewlineChange {
            char: num,
            global
        }));
        Ok(())
    },
    _getvalue: |int| {
        Ok(int.state_catcodes().newlinechar as i32)
    }
};



pub static INPUT: PrimitiveExecutable = PrimitiveExecutable {
    name:"input",
    expandable:false,
    _apply:|tk,int| {todo!()}
};

pub static END: PrimitiveExecutable = PrimitiveExecutable {
    name:"end",
    expandable:false,
    _apply:|tk,int| {todo!()}
};

pub fn tex_commands() -> Vec<TeXCommand> {vec![
    TeXCommand::Primitive(&PAR),
    TeXCommand::Primitive(&RELAX),
    TeXCommand::AV(AssignableValue::Int(&CATCODE)),
    TeXCommand::AV(AssignableValue::Int(&NEWLINECHAR)),
    TeXCommand::Ass(&CHARDEF),
    TeXCommand::Ass(&COUNTDEF),
    TeXCommand::Ass(&DEF),
    TeXCommand::Ass(&LET),
    TeXCommand::Primitive(&INPUT),
    TeXCommand::Primitive(&END),
]}