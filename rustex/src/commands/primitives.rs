use std::ops::Deref;
use crate::commands::{AssignableValue, AssValue, DefMacro, IntCommand, ParamList, ParamToken, PrimitiveAssignment, PrimitiveExecutable, RegisterReference, Signature, TeXCommand};
use crate::interpreter::Interpreter;
use crate::ontology::{Token, Expansion};
use crate::catcodes::CategoryCode;
use crate::interpreter::state::{CategoryCodeChange, CommandChange, GroupType, NewlineChange, RegisterStateChange, StateChange};
use crate::utils::{kpsewhich, TeXError};

pub static PAR : PrimitiveExecutable = PrimitiveExecutable {
    expandable:false,
    name:"par",
    _apply:|cs: Token, _int: &Interpreter| {
        Ok(())
    }
};
pub static RELAX : PrimitiveExecutable = PrimitiveExecutable {
    expandable:false,
    name:"relax",
    _apply:|cs: Token, _int: &Interpreter| {
        Ok(())
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
use chrono::Timelike;

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
            cmd: Some(TeXCommand::AV(AssignableValue::Register(( num as u8, cmd.cmdname())))),
            global
        }));
        Ok(())
    }
};

pub static PROTECTED : PrimitiveAssignment = PrimitiveAssignment {
    name:"protected",
    _assign: |int,global| todo!()
};

pub static LONG: PrimitiveAssignment = PrimitiveAssignment {
    name:"long",
    _assign: |int,global| {
        let mut protected = false;
        while int.has_next() {
            let next = int.next_token();
            match next.catcode {
                CategoryCode::Escape | CategoryCode::Active => {
                    match int.get_command(&next.cmdname())? {
                        TeXCommand::Ass(a) if *a == DEF => {
                            return do_def(int,global,protected,true)
                        }
                        TeXCommand::Ass(a) if *a == EDEF => {
                            todo!()
                        }
                        TeXCommand::Ass(a) if *a == PROTECTED => {
                            protected = true;
                        }
                        _ => return Err(TeXError::new("Expected \\def or \\edef or \\protected after \\long".to_owned()))
                    }
                }
                _ => return Err(TeXError::new("Expected control sequence or active character; got: ".to_owned() + &next.as_string()))
            }
        }
        return Err(TeXError::new("File ended unexpectedly".to_string()))
    }
};

use crate::log;

fn readSig(int:&Interpreter) -> Result<Signature,TeXError> {
    let mut retsig : Vec<ParamToken> = Vec::new();
    let mut currarg = 1 as u8;
    while int.has_next() {
        let next = int.next_token();
        match next.catcode {
            CategoryCode::BeginGroup => {
                return Ok(Signature {
                    elems: retsig,
                    endswithbrace: false,
                    arity:currarg-1
                })
            }
            CategoryCode::Parameter => {
                if !int.has_next() {
                    return Err(TeXError::new("File ended unexpectedly".to_string()))
                }
                let next = int.next_token();
                match next.catcode {
                    CategoryCode::BeginGroup => {
                        return Ok(Signature {
                            elems: retsig,
                            endswithbrace: true,
                            arity:currarg-1
                        })
                    }
                    _ if currarg == 1 && next.char == 49 => {
                        currarg += 1;
                        retsig.push(ParamToken::Param(1))
                    }
                    _ if currarg == 2 && next.char == 50 => {
                        currarg += 1;
                        retsig.push(ParamToken::Param(2))
                    }
                    _ if currarg == 3 && next.char == 51 => {
                        currarg += 1;
                        retsig.push(ParamToken::Param(3))
                    }
                    _ if currarg == 4 && next.char == 52 => {
                        currarg += 1;
                        retsig.push(ParamToken::Param(4))
                    }
                    _ if currarg == 5 && next.char == 53 => {
                        currarg += 1;
                        retsig.push(ParamToken::Param(5))
                    }
                    _ if currarg == 6 && next.char == 54 => {
                        currarg += 1;
                        retsig.push(ParamToken::Param(6))
                    }
                    _ if currarg == 7 && next.char == 55 => {
                        currarg += 1;
                        retsig.push(ParamToken::Param(7))
                    }
                    _ if currarg == 8 && next.char == 56 => {
                        currarg += 1;
                        retsig.push(ParamToken::Param(8))
                    }
                    _ if currarg == 9 && next.char == 57 => {
                        currarg += 1;
                        retsig.push(ParamToken::Param(9))
                    }
                    _ => return Err(TeXError::new("Expected argument ".to_owned() + &currarg.to_string() + "; got: " + &next.as_string()))
                }
            }
            _ => retsig.push(ParamToken::Token(next))
        }
    }
    Err(TeXError::new("File ended unexpectedly".to_string()))
}

fn do_def(int:&Interpreter, global:bool, protected:bool, long:bool) -> Result<(),TeXError> {
    use std::str::from_utf8;
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
                log!("\\def {}{}{}{}{}",command,sig,"{",ParamList(&ret),"}");
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
            CategoryCode::Parameter => {
                if !int.has_next() {
                    return Err(TeXError::new("File ended unexpectedly".to_string()))
                }
                let next = int.next_token();
                match next.catcode {
                    CategoryCode::Parameter => ret.push(ParamToken::Param(0)),
                    _ => {
                        let num = match from_utf8(&[next.char]) {
                            Ok(n) => match n.parse::<u8>() {
                                Ok(u) => u,
                                Err(_) => return Err(TeXError::new("Expected digit between 1 and ".to_string() + &sig.arity.to_string() + "; got: " + &next.as_string()))
                            }
                            Err(_) => return Err(TeXError::new("Expected digit between 1 and ".to_string() + &sig.arity.to_string() + "; got: " + &next.as_string()))
                        };
                        if num < 1 || num > sig.arity {
                            return Err(TeXError::new("Expected digit between 1 and ".to_string() + &sig.arity.to_string() + "; got: " + &next.as_string()))
                        }
                        ret.push(ParamToken::Param(num))
                    }
                }
            },
            _ => ret.push(ParamToken::Token(next))
        }
    }
    Err(TeXError::new("File ended unexpectedly".to_string()))
}

pub static DEF: PrimitiveAssignment = PrimitiveAssignment {
    name:"def",
    _assign: |int,global| do_def(int, global, false, false)
};

pub static EDEF: PrimitiveAssignment = PrimitiveAssignment {
    name:"edef",
    _assign: |int,global| todo!()
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
        log!("\\let {}={}",cmd,def);
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
    _apply:|tk,int| {
        let filename = int.read_string()?;
        if filename.starts_with("|kpsewhich ") {
            todo!()
        } else {
            let file = int.get_file(&filename)?;
            int.push_file(file);
            Ok(())
        }
    }
};

pub static BEGINGROUP : PrimitiveExecutable = PrimitiveExecutable {
    name:"begingroup",
    expandable:false,
    _apply:|tk,int| {
        int.new_group(GroupType::Begingroup);
        Ok(())
    }
};

pub static TIME : IntCommand = IntCommand {
    _getvalue: |int| {
        let time = int.jobinfo.time;
        Ok(((time.hour() * 60) + time.minute()) as i32)
    },
    name: "time"
};
use crate::utils::u8toi16;
pub static DIVIDE : PrimitiveAssignment = PrimitiveAssignment {
    name: "divide",
    _assign: |int,global| {
        let cmd = int.read_command_token()?;
        let (index,num,regdimskip) : (i16,i32,u8) = match int.get_command(&cmd.cmdname())? {
            TeXCommand::AV(AssignableValue::Register((i,_))) => (u8toi16(i),int.state_register(u8toi16(i)),0),
            TeXCommand::AV(AssignableValue::PrimReg(p)) => todo!(),
            _ => todo!()
            //_ => return Err(TeXError::new("Expected register after \\divide; got: ".to_owned() + &cmd.as_string()))
        };
        int.read_keyword(vec!("by"));
        let div = int.read_number()?;
        log!("\\divide sets {} to {}",index,num/div);
        let ch = match regdimskip {
            0 => StateChange::Register(RegisterStateChange {
                index,
                value: num / div,
                global
            }),
            _ => todo!()
        };
        int.change_state(ch);
        Ok(())
    }
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
    TeXCommand::Ass(&EDEF),
    TeXCommand::Ass(&LET),
    TeXCommand::Ass(&LONG),
    TeXCommand::Ass(&PROTECTED),
    TeXCommand::Ass(&DIVIDE),
    TeXCommand::Primitive(&INPUT),
    TeXCommand::Primitive(&END),
    TeXCommand::Primitive(&BEGINGROUP),
    TeXCommand::Int(&TIME)
]}