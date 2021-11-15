use std::ops::Deref;
use crate::commands::{AssignableValue, AssValue, DefMacro, IntCommand, ParamList, ParamToken, PrimitiveAssignment, PrimitiveExecutable, ProvidesExecutableWhatsit, ProvidesWhatsit, RegisterReference, Signature, TeXCommand};
use crate::interpreter::Interpreter;
use crate::ontology::{Token, Expansion};
use crate::catcodes::CategoryCode;
use crate::interpreter::state::{CategoryCodeChange, CommandChange, GroupType, CharChange, RegisterStateChange, StateChange};
use crate::utils::{kpsewhich, TeXError};
use crate::{log,TeXErr,FileEnd};

pub static PAR : PrimitiveExecutable = PrimitiveExecutable {
    expandable:false,
    name:"par",
    _apply:|cs: Token, _int: &Interpreter| {
        Ok(None)
    }
};
pub static RELAX : PrimitiveExecutable = PrimitiveExecutable {
    expandable:false,
    name:"relax",
    _apply:|cs: Token, _int: &Interpreter| {
        Ok(None)
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
use chrono::{Datelike, Timelike};
use crate::commands::etex::UNEXPANDED;

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

pub static COUNT : AssValue<i32> = AssValue {
    name: "count",
    _assign: |int,global| {
        let index = u8toi16(int.read_number()? as u8);
        int.read_eq();
        let val = int.read_number()?;
        log!("\\count sets {} to {}",index,val);
        int.change_state(StateChange::Register(RegisterStateChange {
            index: index,
            value: val,
            global
        }));
        Ok(())
    },
    _getvalue: |int| {
        let index = int.read_number()? as u8;
        let num = int.state_register(u8toi16(index));
        log!("\\count {} = {}",index,num);
        Ok(num)
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
                        _ => TeXErr!(int,"Expected \\def or \\edef or \\protected after \\long")
                    }
                }
                _ => TeXErr!(int,"Expected control sequence or active character; got: {}",next)
            }
        }
        return FileEnd!(int)
    }
};


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
                int.assert_has_next()?;
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
                    _ => TeXErr!(int,"Expected argument {}; got:{}",currarg,next)
                }
            }
            _ => retsig.push(ParamToken::Token(next))
        }
    }
    FileEnd!(int)
}

fn do_def(int:&Interpreter, global:bool, protected:bool, long:bool) -> Result<(),TeXError> {
    use std::str::from_utf8;
    let command = int.next_token();
    match command.catcode {
        CategoryCode::Escape | CategoryCode::Active => {}
        _ => TeXErr!(int,"\\def expected control sequence or active character; got: {}",command)
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
                int.assert_has_next()?;
                let next = int.next_token();
                match next.catcode {
                    CategoryCode::Parameter => ret.push(ParamToken::Param(0)),
                    _ => {
                        let num = match from_utf8(&[next.char]) {
                            Ok(n) => match n.parse::<u8>() {
                                Ok(u) => u,
                                Err(_) => TeXErr!(int,"Expected digit between 1 and {}; got: {}",sig.arity,next)
                            }
                            Err(_) => TeXErr!(int,"Expected digit between 1 and {}; got: {}",sig.arity,next)
                        };
                        if num < 1 || num > sig.arity {
                            TeXErr!(int,"Expected digit between 1 and {}; got: {}",sig.arity,next)
                        }
                        ret.push(ParamToken::Param(num))
                    }
                }
            },
            _ => ret.push(ParamToken::Token(next))
        }
    }
    FileEnd!(int)
}

use crate::commands::Expandable;
use crate::stomach::whatsits::ExecutableWhatsit;

fn do_edef(int:&Interpreter, global:bool, protected:bool, long:bool) -> Result<(),TeXError> {
    use std::str::from_utf8;
    let command = int.next_token();
    match command.catcode {
        CategoryCode::Escape | CategoryCode::Active => {}
        _ => TeXErr!(int,"\\def expected control sequence or active character; got: {}",command)
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
                int.assert_has_next()?;
                let next = int.next_token();
                match next.catcode {
                    CategoryCode::Parameter => ret.push(ParamToken::Param(0)),
                    _ => {
                        let num = match from_utf8(&[next.char]) {
                            Ok(n) => match n.parse::<u8>() {
                                Ok(u) => u,
                                Err(_) => TeXErr!(int,"Expected digit between 1 and {}; got: {}",sig.arity,next)
                            }
                            Err(_) => TeXErr!(int,"Expected digit between 1 and {}; got: {}",sig.arity,next)
                        };
                        if num < 1 || num > sig.arity {
                            TeXErr!(int,"Expected digit between 1 and {}; got: {}",sig.arity,next)
                        }
                        ret.push(ParamToken::Param(num))
                    }
                }
            },
            CategoryCode::Active | CategoryCode::Escape => {
                let cmd = int.get_command(&next.cmdname())?.as_expandable();
                match cmd {
                    Ok(Expandable::Primitive(x)) if *x == THE || *x == UNEXPANDED => {
                        match (x._apply)(next,int)? {
                            Some(e) => {
                                let rc = Rc::new(e);
                                for tk in &rc.exp {
                                    ret.push(ParamToken::Token(tk.copied(Rc::clone(&rc))))
                                }
                            }
                            None => ()
                        }
                    }
                    Ok(e) => e.expand(next,int)?,
                    Err(_) => ret.push(ParamToken::Token(next))
                }
            }
            _ => ret.push(ParamToken::Token(next))
        }
    }
    FileEnd!(int)
}

pub static DEF: PrimitiveAssignment = PrimitiveAssignment {
    name:"def",
    _assign: |int,global| do_def(int, global, false, false)
};

pub static EDEF: PrimitiveAssignment = PrimitiveAssignment {
    name:"edef",
    _assign: |int,global| do_edef(int,global,false,false)
};

pub static LET: PrimitiveAssignment = PrimitiveAssignment {
    name:"let",
    _assign: |int,global| {
        let cmd = int.next_token();
        if !matches!(cmd.catcode,CategoryCode::Escape) && !matches!(cmd.catcode,CategoryCode::Active) {
            TeXErr!(int,"Control sequence or active character expected; found {}",cmd)
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
        int.change_state(StateChange::Newline(CharChange {
            char: num,
            global
        }));
        Ok(())
    },
    _getvalue: |int| {
        Ok(int.state_catcodes().newlinechar as i32)
    }
};

pub static ENDLINECHAR : AssValue<i32> = AssValue {
    name: "endlinechar",
    _assign: |int,global| {
        int.read_eq();
        let num = int.read_number()? as u8;
        log!("\\endlinechar: {}",num);
        int.change_state(StateChange::Endline(CharChange {
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
            Ok(None)
        }
    }
};

pub static BEGINGROUP : PrimitiveExecutable = PrimitiveExecutable {
    name:"begingroup",
    expandable:false,
    _apply:|tk,int| {
        int.new_group(GroupType::Begingroup);
        Ok(None)
    }
};

pub static TIME : IntCommand = IntCommand {
    _getvalue: |int| {
        let time = int.jobinfo.time;
        Ok(((time.hour() * 60) + time.minute()) as i32)
    },
    name: "time"
};

pub static YEAR : IntCommand = IntCommand {
    name:"year",
    _getvalue: |int| {
        Ok(int.jobinfo.time.year())
    }
};

pub static MONTH : IntCommand = IntCommand {
    name:"month",
    _getvalue: |int| {
        Ok(int.jobinfo.time.month() as i32)
    }
};

pub static DAY : IntCommand = IntCommand {
    name:"day",
    _getvalue: |int| {
        Ok(int.jobinfo.time.day() as i32)
    }
};

pub static NUMBER : PrimitiveExecutable = PrimitiveExecutable {
    _apply: |tk,int| {
        let number = int.read_number()?;
        Ok(Some(Expansion {
            cs: tk,
            exp: Interpreter::string_to_tokens(&number.to_string())
        }))
    },
    expandable: true,
    name: "number"
};

use crate::utils::u8toi16;
fn get_inrv(int:&Interpreter) -> Result<(i16,i32,u8,i32),TeXError> {
    let cmd = int.read_command_token()?;
    let (index,num,regdimskip) : (i16,i32,u8) = match int.get_command(&cmd.cmdname())? {
        TeXCommand::AV(AssignableValue::Register((i,_))) => (u8toi16(i),int.state_register(u8toi16(i)),0),
        TeXCommand::AV(AssignableValue::PrimReg(p)) => todo!(),
        TeXCommand::AV(AssignableValue::Int(c)) if *c == COUNT => {
            let i = u8toi16(int.read_number()? as u8);
            (i,int.state_register(i),0)
        }
        _ => todo!()
        //_ => return Err(TeXError::new("Expected register after \\divide; got: ".to_owned() + &cmd.as_string()))
    };
    int.read_keyword(vec!("by"));
    let val = int.read_number()?;
    Ok((index,num,regdimskip,val))
}
pub static DIVIDE : PrimitiveAssignment = PrimitiveAssignment {
    name: "divide",
    _assign: |int,global| {
        let (index,num,regdimskip,div) = get_inrv(int)?;
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
pub static MULTIPLY : PrimitiveAssignment = PrimitiveAssignment {
    name: "multiply",
    _assign: |int,global| {
        let (index,num,regdimskip,fac) = get_inrv(int)?;
        log!("\\multiply sets {} to {}",index,num*fac);
        let ch = match regdimskip {
            0 => StateChange::Register(RegisterStateChange {
                index,
                value: num * fac,
                global
            }),
            _ => todo!()
        };
        int.change_state(ch);
        Ok(())
    }
};
pub static ADVANCE : PrimitiveAssignment = PrimitiveAssignment {
    name: "advance",
    _assign: |int,global| {
        let (index,num,regdimskip,sum) = get_inrv(int)?;
        log!("\\advance sets {} to {}",index,num+sum);
        let ch = match regdimskip {
            0 => StateChange::Register(RegisterStateChange {
                index,
                value: num + sum,
                global
            }),
            _ => todo!()
        };
        int.change_state(ch);
        Ok(())
    }
};

pub static THE: PrimitiveExecutable = PrimitiveExecutable {
    name:"the",
    expandable:true,
    _apply:|tk,int| {
        let reg = int.read_command_token()?;
        log!("\\the {}",reg);
        match int.get_command(&reg.cmdname())? {
            TeXCommand::Int(ic) => Ok(Some(Expansion {
                cs: reg,
                exp: Interpreter::string_to_tokens(&(ic._getvalue)(int)?.to_string())
            })),
            TeXCommand::AV(AssignableValue::Int(i)) => Ok(Some(Expansion {
                cs: reg,
                exp: Interpreter::string_to_tokens(&(i._getvalue)(int)?.to_string())
            })),
            TeXCommand::AV(AssignableValue::PrimReg(i)) => Ok(Some(Expansion {
                cs: reg,
                exp: Interpreter::string_to_tokens(&int.state_register(-u8toi16(i.index)).to_string())
            })),
            TeXCommand::AV(AssignableValue::Register((i,_))) => Ok(Some(Expansion {
                cs: reg,
                exp: Interpreter::string_to_tokens(&int.state_register(u8toi16(i)).to_string())
            })),
            p => todo!("{}",p)
        }
    }
};

pub static IMMEDIATE : PrimitiveExecutable = PrimitiveExecutable {
    name:"immediate",
    expandable:false,
    _apply:|tk,int| {
        let next = int.read_command_token()?;
        match int.get_command(&next.cmdname())? {
            TeXCommand::Whatsit(ProvidesWhatsit::Exec(e)) => {
                let wi = (e._get)(next,int)?;
                (wi._apply)(int);
                Ok(None)
            }
            _ => todo!()
        }
    }
};

pub static OPENOUT: ProvidesExecutableWhatsit = ProvidesExecutableWhatsit {
    name:"openout",
    _get: |tk,int| {
        let num = int.read_number()? as u8;
        int.read_eq();
        let filename = int.read_string()?;
        let file = int.get_file(&filename)?;

        Ok(ExecutableWhatsit {
            _apply: Box::new(move |nint: &Interpreter| {
                nint.file_openout(num,file)
            })
        })
    }
};

pub static CLOSEOUT: ProvidesExecutableWhatsit = ProvidesExecutableWhatsit {
    name:"closeout",
    _get: |tk,int| {
        let num = int.read_number()? as u8;

        Ok(ExecutableWhatsit {
            _apply: Box::new(move |nint: &Interpreter| {
                nint.file_closeout(num)
            })
        })
    }
};

pub static WRITE: ProvidesExecutableWhatsit = ProvidesExecutableWhatsit {
    name: "write",
    _get: |tk, int| {
        let num = int.read_number()? as u8;
        int.assert_has_next()?;
        let next = int.next_token();
        if next.catcode != CategoryCode::BeginGroup {
            TeXErr!(int,"Begin group token expected after \\write")
        }
        let mut ingroups = 0;
        let mut ret : Vec<Token> = Vec::new();
        while int.has_next() {
            let next = int.next_token();
            match next.catcode {
                CategoryCode::BeginGroup => {
                    ingroups += 1;
                    ret.push(next);
                }
                CategoryCode::EndGroup if ingroups == 0 => {
                    let string = int.tokens_to_string(ret);
                    return Ok(ExecutableWhatsit {
                        _apply: Box::new(move |int| {
                            int.file_write(num,string)
                        })
                    })
                }
                CategoryCode::EndGroup => {
                    ingroups -=1;
                    ret.push(next);
                },
                CategoryCode::Active | CategoryCode::Escape => {
                    match int.state_get_command(&next.cmdname()) {
                        None => ret.push(next),
                        Some(cmd) => match cmd.as_expandable() {
                            Ok(Expandable::Primitive(x)) if *x == THE || *x == UNEXPANDED => {
                                match (x._apply)(next,int)? {
                                    Some(e) => {
                                        let rc = Rc::new(e);
                                        for tk in &rc.exp {
                                            ret.push(tk.copied(Rc::clone(&rc)))
                                        }
                                    }
                                    None => ()
                                }
                            }
                            Ok(e) => e.expand(next,int)?,
                            Err(_) => ret.push(next)
                        }
                    }
                }
                _ => ret.push(next)
            }
        }
        FileEnd!(int)
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
    TeXCommand::AV(AssignableValue::Int(&ENDLINECHAR)),
    TeXCommand::AV(AssignableValue::Int(&COUNT)),
    TeXCommand::Ass(&CHARDEF),
    TeXCommand::Ass(&COUNTDEF),
    TeXCommand::Ass(&DEF),
    TeXCommand::Ass(&EDEF),
    TeXCommand::Ass(&LET),
    TeXCommand::Ass(&LONG),
    TeXCommand::Ass(&PROTECTED),
    TeXCommand::Ass(&DIVIDE),
    TeXCommand::Ass(&MULTIPLY),
    TeXCommand::Ass(&ADVANCE),
    TeXCommand::Primitive(&INPUT),
    TeXCommand::Primitive(&END),
    TeXCommand::Primitive(&BEGINGROUP),
    TeXCommand::Primitive(&THE),
    TeXCommand::Primitive(&NUMBER),
    TeXCommand::Primitive(&IMMEDIATE),
    TeXCommand::Whatsit(ProvidesWhatsit::Exec(&OPENOUT)),
    TeXCommand::Whatsit(ProvidesWhatsit::Exec(&CLOSEOUT)),
    TeXCommand::Whatsit(ProvidesWhatsit::Exec(&WRITE)),
    TeXCommand::Int(&TIME),
    TeXCommand::Int(&YEAR),
    TeXCommand::Int(&MONTH),
    TeXCommand::Int(&DAY),
]}