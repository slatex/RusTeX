use crate::commands::{RegisterReference, AssignableValue, AssValue, DefMacro, IntCommand, ParamList, ParamToken, PrimitiveAssignment, PrimitiveExecutable, ProvidesExecutableWhatsit, ProvidesWhatsit, Signature, TeXCommand, TokenList, DimenReference, SkipReference, TokReference, PrimitiveTeXCommand};
use crate::interpreter::Interpreter;
use crate::ontology::{Token, Expansion, ExpansionRef};
use crate::catcodes::CategoryCode;
use crate::interpreter::state::{GroupType, StateChange};
use crate::utils::{TeXError, TeXString};
use crate::{log,TeXErr,FileEnd};
use crate::VERSION_INFO;

pub static PAR : PrimitiveExecutable = PrimitiveExecutable {
    expandable:false,
    name:"par",
    _apply:|_cs: &mut Expansion, _int: &Interpreter| {
        Ok(())
    }
};
pub static RELAX : PrimitiveExecutable = PrimitiveExecutable {
    expandable:false,
    name:"relax",
    _apply:|_cs: &mut Expansion, _int: &Interpreter| {
        Ok(())
    }
};
pub static CATCODE : AssValue<i32> = AssValue {
    name: "catcode",
    _assign: |rf,int,global| {
        let num = int.read_number()? as u8;
        int.read_eq();
        let cat = CategoryCode::fromint(int.read_number()?);
        int.change_state(StateChange::Cat(num,cat,global));
        Ok(())
    },
    _getvalue: |int| {
        let char = int.read_number()?;
        Ok(CategoryCode::toint(&int.state_catcodes().get_code(char as u8)) as i32)
    }
};

pub static SFCODE : AssValue<i32> = AssValue {
    name:"sfcode",
    _assign: |rf,int,global| {
        let char = int.read_number()? as u8;
        int.read_eq();
        let val = int.read_number()?;
        int.change_state(StateChange::Sfcode(char,val,global));
        Ok(())
    },
    _getvalue: |int| {
        let char = int.read_number()? as u8;
        Ok(int.state_sfcode(char))
    }
};

use crate::references::SourceReference;
use std::rc::Rc;
use std::str::from_utf8;
use chrono::{Datelike, Timelike};

pub static CHARDEF: PrimitiveAssignment = PrimitiveAssignment {
    name: "chardef",
    _assign: |rf,int,global| {
        let c = int.read_command_token()?;
        int.read_eq();
        let num = int.read_number()?;
        let cmd = PrimitiveTeXCommand::Char(Token {
            char: num as u8,
            catcode: CategoryCode::Other,
            name_opt: None,
            reference: Box::new(SourceReference::None),
            expand:true
        }).as_ref(&rf);
        int.change_state(StateChange::Cs(c.cmdname(),Some(cmd),global));
        Ok(())
    }
};

pub static COUNT : AssValue<i32> = AssValue {
    name: "count",
    _assign: |_,int,global| {
        let index = u8toi16(int.read_number()? as u8);
        int.read_eq();
        let val = int.read_number()?;
        log!("\\count sets {} to {}",index,val);
        int.change_state(StateChange::Register(index,val,global));
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
    _assign: |rf,int,global| {
        let cmd = int.read_command_token()?;
        int.read_eq();
        let num = int.read_number()?;
        let command = PrimitiveTeXCommand::AV(AssignableValue::Register(num as u8)).as_ref(&rf);

        int.change_state(StateChange::Cs(cmd.cmdname(),
                                         Some(command),
                                         global));
        Ok(())
    }
};

pub static DIMENDEF: PrimitiveAssignment = PrimitiveAssignment {
    name:"dimendef",
    _assign: |rf,int,global| {
        let cmd = int.read_command_token()?;
        int.read_eq();
        let num = int.read_number()?;
        let command = PrimitiveTeXCommand::AV(AssignableValue::Dim(num as u8)).as_ref(&rf);

        int.change_state(StateChange::Cs(cmd.cmdname(),
                                         Some(command),
            global));
        Ok(())
    }
};

pub static SKIPDEF: PrimitiveAssignment = PrimitiveAssignment {
    name:"skipdef",
    _assign: |rf,int,global| {
        let cmd = int.read_command_token()?;
        int.read_eq();
        let num = int.read_number()?;
        let command = PrimitiveTeXCommand::AV(AssignableValue::Skip(num as u8)).as_ref(&rf);

        int.change_state(StateChange::Cs(cmd.cmdname(),
                                         Some(command),
            global));
        Ok(())
    }
};

pub static TOKSDEF: PrimitiveAssignment = PrimitiveAssignment {
    name:"toksdef",
    _assign: |rf,int,global| {
        let cmd = int.read_command_token()?;
        int.read_eq();
        let num = int.read_number()?;
        let command = PrimitiveTeXCommand::AV(AssignableValue::Toks(num as u8)).as_ref(&rf);

        int.change_state(StateChange::Cs(cmd.cmdname(),
                                         Some(command),
            global));
        Ok(())
    }
};

pub static PROTECTED : PrimitiveAssignment = PrimitiveAssignment {
    name:"protected",
    _assign: |rf,int,global| {
        let mut long = false;
        while int.has_next() {
            let next = int.next_token();
            match next.catcode {
                CategoryCode::Escape | CategoryCode::Active => {
                    match int.get_command(&next.cmdname())?.get_orig() {
                        PrimitiveTeXCommand::Ass(a) if *a == DEF => {
                            return do_def(rf,int,global,true,long,false)
                        }
                        PrimitiveTeXCommand::Ass(a) if *a == EDEF => {
                            return do_def(rf,int,global,true,long,true)
                        }
                        PrimitiveTeXCommand::Ass(a) if *a == LONG => {
                            long = true;
                        }
                        _ => TeXErr!(int,"Expected \\def or \\edef or \\long after \\protected: {}",next)
                    }
                }
                _ => TeXErr!(int,"Expected control sequence or active character; got: {}",next)
            }
        }
        FileEnd!(int)
    }
};

pub static LONG: PrimitiveAssignment = PrimitiveAssignment {
    name:"long",
    _assign: |rf,int,global| {
        let mut protected = false;
        while int.has_next() {
            let next = int.next_token();
            match next.catcode {
                CategoryCode::Escape | CategoryCode::Active => {
                    match int.get_command(&next.cmdname())?.get_orig() {
                        PrimitiveTeXCommand::Ass(a) if *a == DEF => {
                            return do_def(rf,int,global,protected,true,false)
                        }
                        PrimitiveTeXCommand::Ass(a) if *a == EDEF => {
                            return do_def(rf,int,global,protected,true,true)
                        }
                        PrimitiveTeXCommand::Ass(a) if *a == PROTECTED => {
                            protected = true;
                        }
                        _ => TeXErr!(int,"Expected \\def or \\edef or \\protected after \\long")
                    }
                }
                _ => TeXErr!(int,"Expected control sequence or active character; got: {}",next)
            }
        }
        FileEnd!(int)
    }
};


fn read_sig(int:&Interpreter) -> Result<Signature,TeXError> {
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
                let inext = int.next_token();
                match inext.catcode {
                    CategoryCode::BeginGroup => {
                        return Ok(Signature {
                            elems: retsig,
                            endswithbrace: true,
                            arity:currarg-1
                        })
                    }
                    _ => {
                        let arg = inext.char - 48;
                        if currarg == arg {
                            retsig.push(ParamToken::Param(arg,next));
                            currarg += 1
                        } else {
                            TeXErr!(int,"Expected argument {}; got:{}",currarg,next)
                        }
                    }
                }
            }
            _ => retsig.push(ParamToken::Token(next))
        }
    }
    FileEnd!(int)
}

fn do_def(rf:ExpansionRef, int:&Interpreter, global:bool, protected:bool, long:bool,edef:bool) -> Result<(),TeXError> {
    use std::str::from_utf8;
    let command = int.next_token();
    match command.catcode {
        CategoryCode::Escape | CategoryCode::Active => {}
        _ => TeXErr!(int,"\\def expected control sequence or active character; got: {}",command)
    }
    let sig = read_sig(int)?;
    let arity = sig.arity;
    let ret = int.read_token_list(edef,edef)?;/*,Box::new(|x,i| {
        match x.catcode {
            CategoryCode::Parameter => {
                i.assert_has_next()?;
                let next = i.next_token();
                match next.catcode {
                    CategoryCode::Parameter => Ok(Some(ParamToken::Param(0,x))),
                    _ => {
                        let num = match from_utf8(&[next.char]) {
                            Ok(n) => match n.parse::<u8>() {
                                Ok(u) => u,
                                Err(_) => TeXErr!(i,"Expected digit between 1 and {}; got: {}",arity,next)
                            }
                            Err(_) => TeXErr!(i,"Expected digit between 1 and {}; got: {}",arity,next)
                        };
                        if num < 1 || num > arity {
                            TeXErr!(i,"Expected digit between 1 and {}; got: {}",arity,next)
                        }
                        Ok(Some(ParamToken::Param(num,x)))
                    }
                }
            }
            _ => Ok(Some(ParamToken::Token(x)))
        }
    }))?; */
    log!("\\def {}{}{}{}{}",command,sig,"{",TokenList(&ret),"}");
    let dm = PrimitiveTeXCommand::Def(DefMacro {
        protected,
        long,
        sig,
        ret
    }).as_ref(&rf);
    int.change_state(StateChange::Cs(command.cmdname(),
                                     Some(dm),
        global));
    Ok(())
}

use crate::commands::Expandable;
use crate::interpreter::dimensions::Numeric;
use crate::stomach::whatsits::ExecutableWhatsit;

pub static GLOBAL : PrimitiveAssignment = PrimitiveAssignment {
    name:"global",
    _assign: |rf,int,global| {
        let next = int.read_command_token()?;
        match int.get_command(&next.cmdname())?.as_assignment() {
            Ok(a) => a.assign(next,int,true)?,
            Err(_) => TeXErr!(int,"Assignment expected after \\global; found: {}",next)
        }
        Ok(())
    }
};

pub static DEF: PrimitiveAssignment = PrimitiveAssignment {
    name:"def",
    _assign: |rf,int,global| do_def(rf,int, global, false, false,false)
};

pub static GDEF: PrimitiveAssignment = PrimitiveAssignment {
    name:"gdef",
    _assign: |rf,int,_global| do_def(rf,int, true, false, false,false)
};

pub static XDEF: PrimitiveAssignment = PrimitiveAssignment {
    name:"xdef",
    _assign: |rf,int,_global| do_def(rf,int, true, false, false,true)
};

pub static EDEF: PrimitiveAssignment = PrimitiveAssignment {
    name:"edef",
    _assign: |rf,int,global| do_def(rf,int,global,false,false,true)
};

pub static LET: PrimitiveAssignment = PrimitiveAssignment {
    name:"let",
    _assign: |rf,int,global| {
        let cmd = int.next_token();
        if !matches!(cmd.catcode,CategoryCode::Escape) && !matches!(cmd.catcode,CategoryCode::Active) {
            TeXErr!(int,"Control sequence or active character expected; found {}",cmd)
        }
        int.read_eq();
        let def = int.next_token();
        log!("\\let {}={}",cmd,def);
        let ch = match def.catcode {
            CategoryCode::Escape | CategoryCode::Active => {
                int.state_get_command(&def.cmdname()).map(|x| x.as_ref(rf.0))
            }
            _ => Some(PrimitiveTeXCommand::Char(def).as_ref(&rf))
        };
        int.change_state(StateChange::Cs(cmd.cmdname(),ch,global));
        Ok(())
    }
};

pub static NEWLINECHAR : AssValue<i32> = AssValue {
    name: "newlinechar",
    _assign: |_,int,global| {
        int.read_eq();
        let num = int.read_number()? as u8;
        log!("\\newlinechar: {}",num);
        int.change_state(StateChange::Newline(num,global));
        Ok(())
    },
    _getvalue: |int| {
        Ok(int.state_catcodes().newlinechar as i32)
    }
};

pub static ENDLINECHAR : AssValue<i32> = AssValue {
    name: "endlinechar",
    _assign: |_,int,global| {
        int.read_eq();
        let num = int.read_number()? as u8;
        log!("\\endlinechar: {}",num);
        int.change_state(StateChange::Endline(num,global));
        Ok(())
    },
    _getvalue: |int| {
        Ok(int.state_catcodes().newlinechar as i32)
    }
};

pub static INPUT: PrimitiveExecutable = PrimitiveExecutable {
    name:"input",
    expandable:false,
    _apply:|_rf,int| {
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
    _apply:|_rf,int| {
        int.new_group(GroupType::Begingroup);
        Ok(())
    }
};

pub static ENDGROUP : PrimitiveExecutable = PrimitiveExecutable {
    name:"endgroup",
    expandable:false,
    _apply:|_rf,int| {
        int.pop_group(GroupType::Begingroup)?;
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
    _apply: |rf,int| {
        let number = int.read_number()?;
        rf.2 = crate::interpreter::string_to_tokens(number.to_string().into());
        Ok(())
    },
    expandable: true,
    name: "number"
};

use crate::utils::u8toi16;
fn get_inrv(int:&Interpreter) -> Result<(i16,Numeric,Numeric),TeXError> {
    use crate::commands::PrimitiveTeXCommand::*;
    let cmd = int.read_command_token()?;
    int.read_keyword(vec!("by"))?;
    let (index,num,val) : (i16,Numeric,Numeric) = match int.get_command(&cmd.cmdname())?.get_orig() {
        AV(AssignableValue::Register(i)) =>
            (u8toi16(i),Numeric::Int(int.state_register(u8toi16(i))),int.read_number_i(false)?),
        AV(AssignableValue::PrimReg(r)) =>
            (-u8toi16(r.index),Numeric::Int(int.state_register(-u8toi16(r.index))),int.read_number_i(false)?),
        AV(AssignableValue::Int(c)) if *c == COUNT => {
            let i = u8toi16(int.read_number()? as u8);
            (i,Numeric::Int(int.state_register(i)),int.read_number_i(false)?)
        }
        _ => todo!()
        //_ => return Err(TeXError::new("Expected register after \\divide; got: ".to_owned() + &cmd.as_string()))
    };
    Ok((index,num,val))
}
pub static DIVIDE : PrimitiveAssignment = PrimitiveAssignment {
    name: "divide",
    _assign: |_,int,global| {
        let (index,num,div) = get_inrv(int)?;
        log!("\\divide sets {} to {}",index,num/div);
        let ch = match (num,div) {
            (Numeric::Int(num),Numeric::Int(div)) => StateChange::Register(index,num/ div,global),
            _ => todo!()
        };
        int.change_state(ch);
        Ok(())
    }
};
pub static MULTIPLY : PrimitiveAssignment = PrimitiveAssignment {
    name: "multiply",
    _assign: |_,int,global| {
        let (index,num,fac) = get_inrv(int)?;
        log!("\\multiply sets {} to {}",index,num*fac);
        let ch = match (num,fac) {
            (Numeric::Int(num),Numeric::Int(fac)) => StateChange::Register(index,num * fac, global),
            _ => todo!()
        };
        int.change_state(ch);
        Ok(())
    }
};
pub static ADVANCE : PrimitiveAssignment = PrimitiveAssignment {
    name: "advance",
    _assign: |_,int,global| {
        let (index,num,sum) = get_inrv(int)?;
        log!("\\advance sets {} to {}",index,num+sum);
        let ch = match (num,sum) {
            (Numeric::Int(num),Numeric::Int(sum)) => StateChange::Register(index,num + sum,global),
            _ => todo!()
        };
        int.change_state(ch);
        Ok(())
    }
};

pub static THE: PrimitiveExecutable = PrimitiveExecutable {
    name:"the",
    expandable:true,
    _apply:|rf,int| {
        use crate::interpreter::string_to_tokens as stt;
        use crate::commands::PrimitiveTeXCommand::*;
        let reg = int.read_command_token()?;
        log!("\\the {}",reg);
        rf.2 = match int.get_command(&reg.cmdname())?.get_orig() {
            Int(ic) => stt((ic._getvalue)(int)?.to_string().into()),
            AV(AssignableValue::Int(i)) => stt((i._getvalue)(int)?.to_string().into()),
            AV(AssignableValue::PrimReg(i)) => stt(int.state_register(-u8toi16(i.index)).to_string().into()),
            AV(AssignableValue::Register(i)) => stt(int.state_register(u8toi16(i)).to_string().into()),
            AV(AssignableValue::Toks(i)) => int.state_tokens(u8toi16(i)),
            AV(AssignableValue::PrimToks(r)) => int.state_tokens(-u8toi16(r.index)),
            p => todo!("{}",p)
        };
        Ok(())
    }
};

pub static IMMEDIATE : PrimitiveExecutable = PrimitiveExecutable {
    name:"immediate",
    expandable:false,
    _apply:|_,int| {
        let next = int.read_command_token()?;
        match int.get_command(&next.cmdname())?.get_orig() {
            PrimitiveTeXCommand::Whatsit(ProvidesWhatsit::Exec(e)) => {
                let wi = (e._get)(&next,int)?;
                (wi._apply)(int)?;
                Ok(())
            }
            _ => todo!()
        }
    }
};

pub static OPENOUT: ProvidesExecutableWhatsit = ProvidesExecutableWhatsit {
    name:"openout",
    _get: |_tk,int| {
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

pub static OPENIN: PrimitiveExecutable = PrimitiveExecutable {
    _apply: |_,int| {
        let num = int.read_number()? as u8;
        int.read_eq();
        let filename = int.read_string()?;
        let file = int.get_file(&filename)?;
        int.file_openin(num,file)?;
        Ok(())
    },
    name:"openin",
    expandable:false,
};

pub static CLOSEOUT: ProvidesExecutableWhatsit = ProvidesExecutableWhatsit {
    name:"closeout",
    _get: |_tk,int| {
        let num = int.read_number()? as u8;

        Ok(ExecutableWhatsit {
            _apply: Box::new(move |nint: &Interpreter| {
                nint.file_closeout(num)
            })
        })
    }
};

pub static CLOSEIN: PrimitiveExecutable = PrimitiveExecutable {
    _apply: |_,int| {
        let num = int.read_number()? as u8;
        int.file_closein(num)?;
        Ok(())
    },
    name:"closein",
    expandable:false,
};

pub static READ: PrimitiveAssignment = PrimitiveAssignment {
    name:"read",
    _assign: |rf,int,global| {
        let index = int.read_number()? as u8;
        match int.read_keyword(vec!("to"))? {
            Some(_) => (),
            None => TeXErr!(int,"\"to\" expected in \\read")
        }
        let newcmd = int.read_command_token()?;
        let toks = int.file_read(index,true)?;
        let cmd = PrimitiveTeXCommand::Def(DefMacro {
            protected: false,
            long: false,
            sig: Signature {
                elems: vec![],
                endswithbrace: false,
                arity: 0
            },
            ret: toks
        }).as_ref(&rf);
        int.change_state(StateChange::Cs(newcmd.cmdname(),
            Some(cmd),
            global));
        Ok(())
    }
};

pub static WRITE: ProvidesExecutableWhatsit = ProvidesExecutableWhatsit {
    name: "write",
    _get: |_tk, int| {
        let num = int.read_number()? as u8;
        int.assert_has_next()?;
        let next = int.next_token();
        if next.catcode != CategoryCode::BeginGroup {
            TeXErr!(int,"Begin group token expected after \\write")
        }

        let ret = int.read_token_list(true,false)?;
        let string = int.tokens_to_string(ret);
        return Ok(ExecutableWhatsit {
            _apply: Box::new(move |int| {
                int.file_write(num,string)
            })
        });
    }
};

pub static MESSAGE: PrimitiveExecutable = PrimitiveExecutable {
    name:"message",
    expandable:false,
    _apply:|_,int| {
        use ansi_term::Colour::*;
        let next = int.next_token();
        if next.catcode != CategoryCode::BeginGroup {
            TeXErr!(int,"Begin group token expected after \\message")
        }
        let ret = int.read_token_list(true,false)?;
        let string = int.tokens_to_string(ret);
        print!("{}",Yellow.paint(string.to_string()));
        Ok(())
    }
};

pub static NOEXPAND: PrimitiveExecutable = PrimitiveExecutable {
    name:"noexpand",
    expandable:true,
    _apply:|cs,int| {
        int.assert_has_next()?;
        let next = int.next_token();
        int.requeue(Token {
            char: next.char,
            catcode: next.catcode,
            name_opt: next.name_opt,
            reference: next.reference,
            expand: false
        });
        Ok(())
    }
};

pub static EXPANDAFTER: PrimitiveExecutable = PrimitiveExecutable {
    name:"expandafter",
    expandable:true,
    _apply:|rf,int| {
        int.assert_has_next()?;
        let tmp = int.next_token();
        int.assert_has_next()?;
        let next = int.next_token();
        match next.catcode {
            CategoryCode::Escape | CategoryCode::Active => {
                match int.get_command(&next.cmdname())?.as_expandable_with_protected() {
                    Ok(exp) => {
                        match exp.get_expansion(next,int)? {
                            Some(e) => rf.2 = e.2,
                            None => ()
                        }
                        rf.2.insert(0,tmp);
                        Ok(())
                    },
                    Err(_) => {
                        todo!("Maybe? {}",next);
                        rf.2.push(tmp);
                        rf.2.push(next);
                        Ok(())
                    }
                }
            },
            _ => {
                rf.2.push(tmp);
                rf.2.push(next);
                Ok(())
            }
        }
    }
};

pub static MEANING: PrimitiveExecutable = PrimitiveExecutable {
    name:"meaning",
    expandable:true,
    _apply:|rf,int| {
        int.assert_has_next()?;
        let next = int.next_token();
        let string = match next.catcode {
            CategoryCode::Active | CategoryCode::Escape => {
                match int.state_get_command(&next.cmdname()) {
                    None => "undefined".into(),
                    Some(p) => p.meaning(&int.state_catcodes())
                }
            }
            _ => PrimitiveTeXCommand::Char(next).as_ref(&rf.get_ref()).meaning(&int.state_catcodes())
        };
        rf.2 = crate::interpreter::string_to_tokens(string);
        Ok(())
    }
};

pub static STRING: PrimitiveExecutable = PrimitiveExecutable {
    name:"string",
    expandable:true,
    _apply:|rf,int| {
        int.assert_has_next()?;
        let next = int.next_token();
        log!("\\string: {}",next);
        rf.2 = match next.catcode {
            CategoryCode::Escape => {
                let mut s : TeXString = if int.state_catcodes().escapechar == 255 {"".into()} else {int.state_catcodes().escapechar.into()};
                crate::interpreter::string_to_tokens(s + next.cmdname())
            }
            CategoryCode::Space => vec!(next),
            _ => vec!(Token {
                char: next.char,
                catcode: CategoryCode::Other,
                name_opt: next.name_opt,
                reference: next.reference,
                expand: true
            })
        };
        Ok(())
    }
};

pub static MATHCHARDEF: PrimitiveAssignment = PrimitiveAssignment {
    name:"mathchardef",
    _assign: |rf,int,global| {
        let chartok = int.read_command_token()?;
        int.read_eq();
        let num = int.read_number()?;
        let cmd = PrimitiveTeXCommand::MathChar(num as u32).as_ref(&rf);
        int.change_state(StateChange::Cs(chartok.cmdname(),Some(cmd),
            global));
        Ok(())
    }
};

pub static CSNAME: PrimitiveExecutable = PrimitiveExecutable {
    name:"csname",
    expandable:true,
    _apply:|rf,int| {
        let incs = int.newincs();
        let mut cmdname : TeXString = "".into();
        while incs == int.currcs() && int.has_next() {
            let next = int.next_token();
            match next.catcode {
                CategoryCode::Escape => {
                    match int.get_command(&next.cmdname())?.as_expandable_with_protected() {
                        Ok(exp) => exp.expand(next,int)?,
                        Err(_) => {
                            if int.state_catcodes().escapechar != 255 {
                                cmdname += int.state_catcodes().escapechar.into()
                            }
                            cmdname += next.name()
                        }
                    }
                }
                CategoryCode::Active =>  {
                    match int.get_command(&next.cmdname())?.as_expandable_with_protected() {
                        Ok(exp) => exp.expand(next,int)?,
                        Err(_) => cmdname += next.name()
                    }
                }
                _ => cmdname += next.char.into()
            }
        }
        let ret = Token {
            char: int.state_catcodes().escapechar,
            catcode: CategoryCode::Escape,
            name_opt: Some(cmdname.clone()),
            reference: Box::new(SourceReference::None),
            expand: true
        };
        match int.state_get_command(&cmdname) {
            Some(_) => (),
            None => {
                let cmd = PrimitiveTeXCommand::Primitive(&RELAX).as_ref(&rf.get_ref());
                int.change_state(StateChange::Cs(cmdname,Some(cmd),false))
            }
        }
        rf.2.push(ret);
        Ok(())
    }
};

pub static ENDCSNAME: PrimitiveExecutable = PrimitiveExecutable {
    name:"endcsname",
    expandable:true,
    _apply:|_,int| {
        int.popcs()?;
        Ok(())
    }
};

pub static ERRMESSAGE: PrimitiveExecutable = PrimitiveExecutable {
    name:"errmessage",
    expandable:false,
    _apply:|_,int| {
        use ansi_term::Colour::*;
        let next = int.next_token();
        if next.catcode != CategoryCode::BeginGroup {
            TeXErr!(int,"Begin group token expected after \\message")
        }
        let ret = int.read_token_list(true,false)?;
        let string = int.tokens_to_string(ret);
        let mut eh = int.state_tokens(-u8toi16(ERRHELP.index));
        let rethelp = if !eh.is_empty() {
            eh.push(Token{
                char: 0,
                catcode: CategoryCode::EndGroup,
                name_opt: None,
                reference: Box::new(SourceReference::None),
                expand: false
            });
            eh.insert(0,Token {
                char: 0,
                catcode: CategoryCode::BeginGroup,
                name_opt: None,
                reference: Box::new(SourceReference::None),
                expand: false
            });
            int.push_tokens(eh);
            let rethelp = int.read_token_list(true,false)?;
            int.tokens_to_string(rethelp)
        } else {"".into()};
        TeXErr!(int,"\n{}\n\n{}",Red.bold().paint(string.to_string()),rethelp)
    }
};

pub static ETEXREVISION : PrimitiveExecutable = PrimitiveExecutable {
    expandable:true,
    _apply: |rf, _int| {
        rf.2 = crate::interpreter::string_to_tokens(VERSION_INFO.etexrevision.clone());
        Ok(())
    },
    name: "etexrevision"
};

pub static ETEXVERSION : IntCommand = IntCommand {
    _getvalue: |_int| {
        Ok(VERSION_INFO.etexversion.to_string().parse().unwrap())
    },
    name: "eTeXversion"
};

pub static UNEXPANDED: PrimitiveExecutable = PrimitiveExecutable {
    name:"unexpanded",
    expandable:true,
    _apply:|exp,int| {
        int.expand_until(true);
        match int.next_token().catcode {
            CategoryCode::BeginGroup => {
                exp.2 = int.read_token_list(false,false)?;
                Ok(())
            }
            _ => TeXErr!(int,"Balanced argument expected after \\unexpanded")
        }
    }
};

// REGISTERS ---------------------------------------------------------------------------------------

pub static PRETOLERANCE : RegisterReference = RegisterReference {
    name: "pretolerance",
    index:5
};

pub static TOLERANCE : RegisterReference = RegisterReference {
    name: "tolerance",
    index:6
};

pub static HBADNESS : RegisterReference = RegisterReference {
    name: "hbadness",
    index:7
};

pub static VBADNESS : RegisterReference = RegisterReference {
    name: "vbadness",
    index:8
};

pub static LINEPENALTY : RegisterReference = RegisterReference {
    name: "linepenalty",
    index:9
};

pub static HYPHENPENALTY : RegisterReference = RegisterReference {
    name: "hyphenpenalty",
    index:10
};

pub static EXHYPHENPENALTY : RegisterReference = RegisterReference {
    name: "exhyphenpenalty",
    index:11
};

pub static BINOPPENALTY : RegisterReference = RegisterReference {
    name: "binoppenalty",
    index:12
};

pub static RELPENALTY : RegisterReference = RegisterReference {
    name: "relpenalty",
    index:13
};

pub static CLUBPENALTY : RegisterReference = RegisterReference {
    name: "clubpenalty",
    index:14
};

pub static WIDOWPENALTY : RegisterReference = RegisterReference {
    name: "widowpenalty",
    index:15
};

pub static DISPLAYWIDOWPENALTY : RegisterReference = RegisterReference {
    name: "displaywidowpenalty",
    index:16
};

pub static BROKENPENALTY : RegisterReference = RegisterReference {
    name: "brokenpenalty",
    index:17
};

pub static PREDISPLAYPENALTY : RegisterReference = RegisterReference {
    name: "predisplaypenalty",
    index:18
};

pub static DOUBLEHYPHENDEMERITS : RegisterReference = RegisterReference {
    name: "doublehyphendemerits",
    index:19
};

pub static FINALHYPHENDEMERITS : RegisterReference = RegisterReference {
    name: "finalhyphendemerits",
    index:20
};

pub static ADJDEMERITS : RegisterReference = RegisterReference {
    name: "adjdemerits",
    index:21
};

pub static TRACINGLOSTCHARS : RegisterReference = RegisterReference {
    name: "tracinglostchars",
    index:22
};

pub static UCHYPH : RegisterReference = RegisterReference {
    name: "uchyph",
    index:23
};

pub static DEFAULTHYPHENCHAR : RegisterReference = RegisterReference {
    name: "defaulthyphenchar",
    index:24
};

pub static DEFAULTSKEWCHAR : RegisterReference = RegisterReference {
    name: "defaultskewchar",
    index:25
};

pub static DELIMITERFACTOR : RegisterReference = RegisterReference {
    name: "delimiterfactor",
    index:26
};

pub static SHOWBOXBREADTH : RegisterReference = RegisterReference {
    name: "showboxbreadth",
    index:27
};

pub static SHOWBOXDEPTH : RegisterReference = RegisterReference {
    name: "showboxdepth",
    index:28
};

pub static ERRORCONTEXTLINES : RegisterReference = RegisterReference {
    name: "errorcontextlines",
    index:29
};

pub static MAXDEADCYCLES : RegisterReference = RegisterReference {
    name: "maxdeadcycles",
    index:30
};

pub static TRACINGSTATS : RegisterReference = RegisterReference {
    name: "tracingstats",
    index:31
};

pub static LEFTHYPHENMIN : RegisterReference = RegisterReference {
    name: "lefthyphenmin",
    index:32
};

pub static RIGHTHYPHENMIN : RegisterReference = RegisterReference {
    name: "righhyphenmin",
    index:33
};

pub static SAVINGHYPHCODES : RegisterReference = RegisterReference {
    name: "savinghyphcodes",
    index:34
};

// -----------

pub static FAM : RegisterReference = RegisterReference {
    name: "fam",
    index:41
};

pub static SPACEFACTOR : RegisterReference = RegisterReference {
    name: "spacefactor",
    index:42
};

// -----------

pub static GLOBALDEFS : RegisterReference = RegisterReference {
    name: "globaldefs",
    index:45
};

// -----------

pub static TRACINGNESTING : RegisterReference = RegisterReference {
    name: "tracingnesting",
    index:47
};

// -----------

pub static MAG : RegisterReference = RegisterReference {
    name: "mag",
    index:53
};

pub static LANGUAGE : RegisterReference = RegisterReference {
    name: "language",
    index:54
};

pub static HANGAFTER : RegisterReference = RegisterReference {
    name: "hangafter",
    index:55
};

pub static INTERLINEPENALTY : RegisterReference = RegisterReference {
    name: "interlinepenalty",
    index:56
};

pub static FLOATINGPENALTY : RegisterReference = RegisterReference {
    name: "floatingpenalty",
    index:57
};

pub static LASTNODETYPE : RegisterReference = RegisterReference {
    name: "lastnodetype",
    index:58
};

pub static INSERTPENALTIES : RegisterReference = RegisterReference {
    name: "insertpenalties",
    index:59
};

// -----

pub static BADNESS : RegisterReference = RegisterReference {
    name: "badness",
    index:61
};

pub static DEADCYCLES : RegisterReference = RegisterReference {
    name: "deadcycles",
    index:62
};

pub static INTERLINEPENALTIES : RegisterReference = RegisterReference {
    name: "interlinepenalties",
    index:63
};

pub static CLUBPENALTIES : RegisterReference = RegisterReference {
    name: "clubpenalties",
    index:64
};

pub static WIDOWPENALTIES : RegisterReference = RegisterReference {
    name: "widowpenalties",
    index:65
};

pub static DISPLAYWIDOWPENALTIES : RegisterReference = RegisterReference {
    name: "displaywidowpenalties",
    index:66
};

pub static OUTPUTPENALTY : RegisterReference = RegisterReference {
    name: "outputpenalty",
    index:67
};

pub static SAVINGVDISCARDS : RegisterReference = RegisterReference {
    name: "savingvdiscards",
    index:68
};

pub static DISPLAYINDENT : RegisterReference = RegisterReference {
    name: "displayindent",
    index:69
};

pub static SYNCTEX : RegisterReference = RegisterReference {
    name: "synctex",
    index:70
};

pub static POSTDISPLAYPENALTY : RegisterReference = RegisterReference {
    name: "postdisplaypenalty",
    index:71
};

pub static TRACINGSCANTOKENS : RegisterReference = RegisterReference {
    name: "tracingscantokens",
    index:72
};

// Dimensions --------------------------------------------------------------------------------------

pub static HFUZZ : DimenReference = DimenReference {
    name: "hfuzz",
    index:5
};

pub static VFUZZ : DimenReference = DimenReference {
    name: "vfuzz",
    index:6
};

pub static OVERFULLRULE : DimenReference = DimenReference {
    name: "overfullrule",
    index:7
};

pub static MAXDEPTH : DimenReference = DimenReference {
    name: "maxdepth",
    index:8
};

pub static SPLITMAXDEPTH : DimenReference = DimenReference {
    name: "splitmaxdepth",
    index:9
};

pub static BOXMAXDEPTH : DimenReference = DimenReference {
    name: "boxmaxdepth",
    index:10
};

pub static DELIMITERSHORTFALL : DimenReference = DimenReference {
    name: "delimitershortfall",
    index:11
};

pub static NULLDELIMITERSPACE : DimenReference = DimenReference {
    name: "nulldelimiterspace",
    index:12
};

pub static SCRIPTSPACE : DimenReference = DimenReference {
    name: "scriptspace",
    index:13
};

pub static PARINDENT : DimenReference = DimenReference {
    name: "parindent",
    index:14
};

pub static VSIZE : DimenReference = DimenReference {
    name: "vsize",
    index:15
};

pub static HSIZE : DimenReference = DimenReference {
    name: "hsize",
    index:16
};

// -----------------

pub static LINESKIPLIMIT : DimenReference = DimenReference {
    name: "lineskiplimit",
    index:21
};

pub static MATHSURROUND : DimenReference = DimenReference {
    name: "mathsurround",
    index:22
};

// -----------------

pub static HANGINDENT : DimenReference = DimenReference {
    name: "hangindent",
    index:25
};

// ----------------

pub static PAGETOTAL : DimenReference = DimenReference {
    name: "pagetotal",
    index:27
};

pub static PAGESTRETCH : DimenReference = DimenReference {
    name: "pagestretch",
    index:28
};

pub static PAGEFILSTRETCH : DimenReference = DimenReference {
    name: "pagefilstretch",
    index:29
};

pub static PAGEFILLSTRETCH : DimenReference = DimenReference {
    name: "pagefillstretch",
    index:30
};

pub static PAGEFILLLSTRETCH : DimenReference = DimenReference {
    name: "pagefilllstretch",
    index:31
};

pub static PAGESHRINK : DimenReference = DimenReference {
    name: "pageshrink",
    index:32
};

pub static PAGEDEPTH : DimenReference = DimenReference {
    name: "pagedepth",
    index:33
};

// -------------

pub static EMERGENCYSTRETCH : DimenReference = DimenReference {
    name: "emergencystretch",
    index:63
};

pub static VOFFSET : DimenReference = DimenReference {
    name: "voffset",
    index:64
};

pub static HOFFSET : DimenReference = DimenReference {
    name: "hoffset",
    index:65
};

pub static DISPLAYWIDTH : DimenReference = DimenReference {
    name: "displaywidth",
    index:66
};

pub static PREDISPLAYSIZE : DimenReference = DimenReference {
    name: "predisplaysize",
    index:67
};

// Skips -------------------------------------------------------------------------------------------

pub static PARSKIP : SkipReference = SkipReference {
    name: "parskip",
    index:5
};

pub static ABOVEDISPLAYSKIP : SkipReference = SkipReference {
    name: "abovedisplayskip",
    index:6
};

pub static ABOVEDISPLAYSHORTSKIP : SkipReference = SkipReference {
    name: "abovedisplayshortskip",
    index:7
};

pub static BELOWDISPLAYSKIP : SkipReference = SkipReference {
    name: "belowdisplayskip",
    index:8
};

pub static BELOWDISPLAYSHORTSKIP : SkipReference = SkipReference {
    name: "belowdisplayshortskip",
    index:9
};

pub static TOPSKIP : SkipReference = SkipReference {
    name: "topskip",
    index:10
};

pub static SPLITTOPSKIP : SkipReference = SkipReference {
    name: "splittopskip",
    index:11
};

pub static PARFILLSKIP : SkipReference = SkipReference {
    name: "parfillskip",
    index:12
};

pub static BASELINESKIP : SkipReference = SkipReference {
    name: "baselineskip",
    index:13
};

pub static LINESKIP : SkipReference = SkipReference {
    name: "lineskip",
    index:14
};

pub static PREVDEPTH : SkipReference = SkipReference {
    name: "prevdepth",
    index:15
};

// -----------

pub static LEFTSKIP : SkipReference = SkipReference {
    name: "leftskip",
    index:17
};

pub static RIGHTSKIP : SkipReference = SkipReference {
    name: "rightskip",
    index:18
};

// ----------

pub static TABSKIP : SkipReference = SkipReference {
    name: "tabskip",
    index:20
};

pub static SPACESKIP : SkipReference = SkipReference {
    name: "spaceskip",
    index:21
};

pub static XSPACESKIP : SkipReference = SkipReference {
    name: "xspaceskip",
    index:22
};

pub static BIGSKIPAMOUNT : SkipReference = SkipReference {
    name: "bigskipamount",
    index:23
};

// Tokens ------------------------------------------------------------------------------------------

pub static EVERYJOB : TokReference = TokReference {
    name:"everyjob",
    index:5
};

pub static EVERYPAR : TokReference = TokReference {
    name:"everypar",
    index:6
};

pub static EVERYMATH : TokReference = TokReference {
    name:"everymath",
    index:7
};

pub static EVERYDISPLAY : TokReference = TokReference {
    name:"everydisplay",
    index:8
};

pub static EVERYHBOX : TokReference = TokReference {
    name:"everyhbox",
    index:9
};

pub static EVERYVBOX : TokReference = TokReference {
    name:"everyvbox",
    index:10
};

pub static EVERYCR : TokReference = TokReference {
    name:"everycr",
    index:11
};

pub static ERRHELP : TokReference = TokReference {
    name:"errhelp",
    index:12
};

pub static OUTPUT : TokReference = TokReference {
    name:"output",
    index:13
};

pub static EVERYEOF : TokReference = TokReference {
    name:"everyeof",
    index:14
};


// TODO --------------------------------------------------------------------------------------------

pub static END: PrimitiveExecutable = PrimitiveExecutable {
    name:"end",
    expandable:false,
    _apply:|_tk,_int| {todo!()}
};

pub static BATCHMODE: PrimitiveExecutable = PrimitiveExecutable {
    name:"batchmode",
    expandable:false,
    _apply:|_tk,_int| {todo!()}
};

pub static BYE: PrimitiveExecutable = PrimitiveExecutable {
    name:"bye",
    expandable:false,
    _apply:|_tk,_int| {todo!()}
};

pub static CHAR: PrimitiveExecutable = PrimitiveExecutable {
    name:"char",
    expandable:false,
    _apply:|_tk,_int| {todo!()}
};

pub static CR: PrimitiveExecutable = PrimitiveExecutable {
    name:"cr",
    expandable:false,
    _apply:|_tk,_int| {todo!()}
};

pub static CRCR: PrimitiveExecutable = PrimitiveExecutable {
    name:"crcr",
    expandable:false,
    _apply:|_tk,_int| {todo!()}
};

pub static CURRENTGROUPLEVEL: PrimitiveExecutable = PrimitiveExecutable {
    name:"currentgrouplevel",
    expandable:false,
    _apply:|_tk,_int| {todo!()}
};

pub static DETOKENIZE: PrimitiveExecutable = PrimitiveExecutable {
    name:"detokenize",
    expandable:false,
    _apply:|_tk,_int| {todo!()}
};

pub static DIMEXPR: PrimitiveExecutable = PrimitiveExecutable {
    name:"dimexpr",
    expandable:false,
    _apply:|_tk,_int| {todo!()}
};

pub static DUMP: PrimitiveExecutable = PrimitiveExecutable {
    name:"dump",
    expandable:false,
    _apply:|_tk,_int| {todo!()}
};

pub static ENDINPUT: PrimitiveExecutable = PrimitiveExecutable {
    name:"endinput",
    expandable:false,
    _apply:|_tk,_int| {todo!()}
};

pub static EQNO: PrimitiveExecutable = PrimitiveExecutable {
    name:"eqno",
    expandable:false,
    _apply:|_tk,_int| {todo!()}
};

pub static ERRORSTOPMODE: PrimitiveExecutable = PrimitiveExecutable {
    name:"errorstopmode",
    expandable:false,
    _apply:|_tk,_int| {todo!()}
};

pub static EXPANDED: PrimitiveExecutable = PrimitiveExecutable {
    name:"expanded",
    expandable:false,
    _apply:|_tk,_int| {todo!()}
};

pub static FONTNAME: PrimitiveExecutable = PrimitiveExecutable {
    name:"fontname",
    expandable:false,
    _apply:|_tk,_int| {todo!()}
};

pub static FONTCHARWD: PrimitiveExecutable = PrimitiveExecutable {
    name:"fontcharwd",
    expandable:false,
    _apply:|_tk,_int| {todo!()}
};

pub static FONTCHARHT: PrimitiveExecutable = PrimitiveExecutable {
    name:"fontcharht",
    expandable:false,
    _apply:|_tk,_int| {todo!()}
};

pub static FONTCHARDP: PrimitiveExecutable = PrimitiveExecutable {
    name:"fontchardp",
    expandable:false,
    _apply:|_tk,_int| {todo!()}
};

pub static FONTCHARIC: PrimitiveExecutable = PrimitiveExecutable {
    name:"fontcharic",
    expandable:false,
    _apply:|_tk,_int| {todo!()}
};

pub static GLUEEXPR: PrimitiveExecutable = PrimitiveExecutable {
    name:"glueexpr",
    expandable:false,
    _apply:|_tk,_int| {todo!()}
};

pub static IGNORESPACES: PrimitiveExecutable = PrimitiveExecutable {
    name:"end",
    expandable:false,
    _apply:|_tk,_int| {todo!()}
};

pub static INPUTLINENO: PrimitiveExecutable = PrimitiveExecutable {
    name:"inputlineno",
    expandable:false,
    _apply:|_tk,_int| {todo!()}
};

pub static JOBNAME: PrimitiveExecutable = PrimitiveExecutable {
    name:"jobname",
    expandable:false,
    _apply:|_tk,_int| {todo!()}
};

pub static LOWERCASE: PrimitiveExecutable = PrimitiveExecutable {
    name:"lowercase",
    expandable:false,
    _apply:|_tk,_int| {todo!()}
};

pub static MUEXPR: PrimitiveExecutable = PrimitiveExecutable {
    name:"muexpr",
    expandable:false,
    _apply:|_tk,_int| {todo!()}
};

pub static NULLFONT: PrimitiveExecutable = PrimitiveExecutable {
    name:"nullfont",
    expandable:false,
    _apply:|_tk,_int| {todo!()}
};

pub static NUMEXPR: PrimitiveExecutable = PrimitiveExecutable {
    name:"end",
    expandable:false,
    _apply:|_tk,_int| {todo!()}
};

pub static ROMANNUMERAL: PrimitiveExecutable = PrimitiveExecutable {
    name:"romannumeral",
    expandable:true,
    _apply:|rf,int| {
        let mut num = int.read_number()?;
        if num <= 0 {
            return Ok(())
        }
        let mut ret : Vec<u8> = vec!();
        while num >= 1000 {
            num -= 1000;
            ret.push(109); // m
        }
        if num >= 900 {
            num -= 900;
            ret.push(99); // c
            ret.push(109); // m
        }
        if num >= 500 {
            num -= 500;
            ret.push(100); // d
        }
        if num >= 400 {
            num -= 400;
            ret.push(99); // c
            ret.push(100); // d
        }
        while num >= 100 {
            num -= 100;
            ret.push(99); // c
        }
        if num >= 90 {
            num -= 90;
            ret.push(120); // x
            ret.push(99); // c
        }
        if num >= 50 {
            num -= 50;
            ret.push(108); // l
        }
        if num >= 40 {
            num -= 40;
            ret.push(120); // x
            ret.push(108); // l
        }
        while num >= 10 {
            num -= 10;
            ret.push(120); // x
        }
        if num >= 9 {
            num -= 9;
            ret.push(105); // i
            ret.push(120); // x
        }
        if num >= 5 {
            num -= 5;
            ret.push(118); // v
        }
        if num >= 4 {
            num -= 4;
            ret.push(105); // i
            ret.push(118); // v
        }
        while num >= 1 {
            num -= 1;
            ret.push(105); // i
        }
        rf.2 = crate::interpreter::string_to_tokens(ret.into());
        Ok(())
    }
};

pub static SCANTOKENS: PrimitiveExecutable = PrimitiveExecutable {
    name:"scantokens",
    expandable:false,
    _apply:|_tk,_int| {todo!()}
};

pub static SHIPOUT: PrimitiveExecutable = PrimitiveExecutable {
    name:"shipout",
    expandable:false,
    _apply:|_tk,_int| {todo!()}
};

pub static UPPERCASE: PrimitiveExecutable = PrimitiveExecutable {
    name:"uppercase",
    expandable:false,
    _apply:|_tk,_int| {todo!()}
};

pub static TEXTSTYLE: PrimitiveExecutable = PrimitiveExecutable {
    name:"textstyle",
    expandable:false,
    _apply:|_tk,_int| {todo!()}
};

pub static SCRIPTSTYLE: PrimitiveExecutable = PrimitiveExecutable {
    name:"scriptstyle",
    expandable:false,
    _apply:|_tk,_int| {todo!()}
};

pub static SCRIPTSCRIPTSTYLE: PrimitiveExecutable = PrimitiveExecutable {
    name:"scriptscriptstyle",
    expandable:false,
    _apply:|_tk,_int| {todo!()}
};

pub static SPECIAL: PrimitiveExecutable = PrimitiveExecutable {
    name:"special",
    expandable:false,
    _apply:|_tk,_int| {todo!()}
};

pub static NONSCRIPT: PrimitiveExecutable = PrimitiveExecutable {
    name:"nonscript",
    expandable:false,
    _apply:|_tk,_int| {todo!()}
};

pub static HOLDINGINSERTS: PrimitiveExecutable = PrimitiveExecutable {
    name:"holdinginserts",
    expandable:false,
    _apply:|_tk,_int| {todo!()}
};

pub static LEQNO: PrimitiveExecutable = PrimitiveExecutable {
    name:"leqno",
    expandable:false,
    _apply:|_tk,_int| {todo!()}
};

pub static LOOSENESS: PrimitiveExecutable = PrimitiveExecutable {
    name:"looseness",
    expandable:false,
    _apply:|_tk,_int| {todo!()}
};

pub static NOBOUNDARY: PrimitiveExecutable = PrimitiveExecutable {
    name:"noboundary",
    expandable:false,
    _apply:|_tk,_int| {todo!()}
};

pub static SCROLLMODE: PrimitiveExecutable = PrimitiveExecutable {
    name:"scrollmode",
    expandable:false,
    _apply:|_tk,_int| {todo!()}
};

pub static NONSTOPMODE: PrimitiveExecutable = PrimitiveExecutable {
    name:"nonstopmode",
    expandable:false,
    _apply:|_tk,_int| {todo!()}
};

pub static OMIT: PrimitiveExecutable = PrimitiveExecutable {
    name:"omit",
    expandable:false,
    _apply:|_tk,_int| {todo!()}
};

pub static PAUSING: PrimitiveExecutable = PrimitiveExecutable {
    name:"pausing",
    expandable:false,
    _apply:|_tk,_int| {todo!()}
};

pub static PREVGRAF: PrimitiveExecutable = PrimitiveExecutable {
    name:"prevgraf",
    expandable:false,
    _apply:|_tk,_int| {todo!()}
};

pub static SETLANGUAGE: PrimitiveExecutable = PrimitiveExecutable {
    name:"setlanguage",
    expandable:false,
    _apply:|_tk,_int| {todo!()}
};

pub static SHOW: PrimitiveExecutable = PrimitiveExecutable {
    name:"show",
    expandable:false,
    _apply:|_tk,_int| {todo!()}
};

pub static SHOWBOX: PrimitiveExecutable = PrimitiveExecutable {
    name:"showbox",
    expandable:false,
    _apply:|_tk,_int| {todo!()}
};

pub static SHOWLISTS: PrimitiveExecutable = PrimitiveExecutable {
    name:"showlists",
    expandable:false,
    _apply:|_tk,_int| {todo!()}
};

pub static SHOWTHE: PrimitiveExecutable = PrimitiveExecutable {
    name:"showthe",
    expandable:false,
    _apply:|_tk,_int| {todo!()}
};

pub static SPAN: PrimitiveExecutable = PrimitiveExecutable {
    name:"span",
    expandable:false,
    _apply:|_tk,_int| {todo!()}
};

pub static TRACINGCOMMANDS: PrimitiveExecutable = PrimitiveExecutable {
    name:"tracingcommands",
    expandable:false,
    _apply:|_tk,_int| {todo!()}
};

pub static TRACINGMACROS: PrimitiveExecutable = PrimitiveExecutable {
    name:"tracingmacros",
    expandable:false,
    _apply:|_tk,_int| {todo!()}
};

pub static TRACINGONLINE: PrimitiveExecutable = PrimitiveExecutable {
    name:"tracingonline",
    expandable:false,
    _apply:|_tk,_int| {todo!()}
};

pub static TRACINGOUTPUT: PrimitiveExecutable = PrimitiveExecutable {
    name:"tracingoutput",
    expandable:false,
    _apply:|_tk,_int| {todo!()}
};

pub static TRACINGPAGES: PrimitiveExecutable = PrimitiveExecutable {
    name:"tracingpages",
    expandable:false,
    _apply:|_tk,_int| {todo!()}
};

pub static TRACINGPARAGRAPHS: PrimitiveExecutable = PrimitiveExecutable {
    name:"tracingparagraphs",
    expandable:false,
    _apply:|_tk,_int| {todo!()}
};

pub static TRACINGRESTORES: PrimitiveExecutable = PrimitiveExecutable {
    name:"tracingrestores",
    expandable:false,
    _apply:|_tk,_int| {todo!()}
};

pub static VALIGN: PrimitiveExecutable = PrimitiveExecutable {
    name:"valign",
    expandable:false,
    _apply:|_tk,_int| {todo!()}
};

pub static BEGINL: PrimitiveExecutable = PrimitiveExecutable {
    name:"beginL",
    expandable:false,
    _apply:|_tk,_int| {todo!()}
};

pub static BEGINR: PrimitiveExecutable = PrimitiveExecutable {
    name:"beginR",
    expandable:false,
    _apply:|_tk,_int| {todo!()}
};

pub static BOTMARKS: PrimitiveExecutable = PrimitiveExecutable {
    name:"botmarks",
    expandable:false,
    _apply:|_tk,_int| {todo!()}
};

pub static CURRENTGROUPTYPE: PrimitiveExecutable = PrimitiveExecutable {
    name:"currentgrouptype",
    expandable:false,
    _apply:|_tk,_int| {todo!()}
};

pub static CURRENTIFBRANCH: PrimitiveExecutable = PrimitiveExecutable {
    name:"currentifbranch",
    expandable:false,
    _apply:|_tk,_int| {todo!()}
};

pub static CURRENTIFLEVEL: PrimitiveExecutable = PrimitiveExecutable {
    name:"currentiflevel",
    expandable:false,
    _apply:|_tk,_int| {todo!()}
};

pub static CURRENTIFTYPE: PrimitiveExecutable = PrimitiveExecutable {
    name:"currentiftype",
    expandable:false,
    _apply:|_tk,_int| {todo!()}
};

pub static ENDL: PrimitiveExecutable = PrimitiveExecutable {
    name:"endL",
    expandable:false,
    _apply:|_tk,_int| {todo!()}
};

pub static ENDR: PrimitiveExecutable = PrimitiveExecutable {
    name:"endR",
    expandable:false,
    _apply:|_tk,_int| {todo!()}
};

pub static FIRSTMARKS: PrimitiveExecutable = PrimitiveExecutable {
    name:"firstmarks",
    expandable:false,
    _apply:|_tk,_int| {todo!()}
};

pub static GLUESHRINK: PrimitiveExecutable = PrimitiveExecutable {
    name:"glueshrink",
    expandable:false,
    _apply:|_tk,_int| {todo!()}
};

pub static GLUESHRINKORDER: PrimitiveExecutable = PrimitiveExecutable {
    name:"glueshrinkorder",
    expandable:false,
    _apply:|_tk,_int| {todo!()}
};

pub static GLUESTRETCH: PrimitiveExecutable = PrimitiveExecutable {
    name:"gluestretch",
    expandable:false,
    _apply:|_tk,_int| {todo!()}
};

pub static GLUESTRETCHORDER: PrimitiveExecutable = PrimitiveExecutable {
    name:"gluestretchorder",
    expandable:false,
    _apply:|_tk,_int| {todo!()}
};

pub static GLUETOMU: PrimitiveExecutable = PrimitiveExecutable {
    name:"gluetomu",
    expandable:false,
    _apply:|_tk,_int| {todo!()}
};

pub static INTERACTIONMODE: PrimitiveExecutable = PrimitiveExecutable {
    name:"interactionmode",
    expandable:false,
    _apply:|_tk,_int| {todo!()}
};

pub static LASTLINEFIT: PrimitiveExecutable = PrimitiveExecutable {
    name:"lastlinefit",
    expandable:false,
    _apply:|_tk,_int| {todo!()}
};

pub static MARKS: PrimitiveExecutable = PrimitiveExecutable {
    name:"marks",
    expandable:false,
    _apply:|_tk,_int| {todo!()}
};

pub static MUTOGLUE: PrimitiveExecutable = PrimitiveExecutable {
    name:"mutoglue",
    expandable:false,
    _apply:|_tk,_int| {todo!()}
};

pub static PAGEDISCARDS: PrimitiveExecutable = PrimitiveExecutable {
    name:"pagediscards",
    expandable:false,
    _apply:|_tk,_int| {todo!()}
};

pub static PARSHAPEDIMEN: PrimitiveExecutable = PrimitiveExecutable {
    name:"parshapedimen",
    expandable:false,
    _apply:|_tk,_int| {todo!()}
};

pub static PARSHAPEINDENT: PrimitiveExecutable = PrimitiveExecutable {
    name:"parshapeindent",
    expandable:false,
    _apply:|_tk,_int| {todo!()}
};

pub static PARSHAPELENGTH: PrimitiveExecutable = PrimitiveExecutable {
    name:"parshapelength",
    expandable:false,
    _apply:|_tk,_int| {todo!()}
};

pub static PREDISPLAYDIRECTION: PrimitiveExecutable = PrimitiveExecutable {
    name:"predisplaydirection",
    expandable:false,
    _apply:|_tk,_int| {todo!()}
};

pub static SHOWGROUPS: PrimitiveExecutable = PrimitiveExecutable {
    name:"showgroups",
    expandable:false,
    _apply:|_tk,_int| {todo!()}
};

pub static SHOWIFS: PrimitiveExecutable = PrimitiveExecutable {
    name:"showifs",
    expandable:false,
    _apply:|_tk,_int| {todo!()}
};

pub static SHOWTOKENS: PrimitiveExecutable = PrimitiveExecutable {
    name:"showtokens",
    expandable:false,
    _apply:|_tk,_int| {todo!()}
};

pub static SPLITBOTMARKS: PrimitiveExecutable = PrimitiveExecutable {
    name:"splitbotmarks",
    expandable:false,
    _apply:|_tk,_int| {todo!()}
};

pub static SPLITDISCARDS: PrimitiveExecutable = PrimitiveExecutable {
    name:"splitdiscards",
    expandable:false,
    _apply:|_tk,_int| {todo!()}
};

pub static SPLITFIRSTMARKS: PrimitiveExecutable = PrimitiveExecutable {
    name:"splitfirstmarks",
    expandable:false,
    _apply:|_tk,_int| {todo!()}
};

pub static TEXXETSTATE: PrimitiveExecutable = PrimitiveExecutable {
    name:"TeXXeTstate",
    expandable:false,
    _apply:|_tk,_int| {todo!()}
};

pub static TOPMARKS: PrimitiveExecutable = PrimitiveExecutable {
    name:"topmarks",
    expandable:false,
    _apply:|_tk,_int| {todo!()}
};

pub static TRACINGASSIGNS: PrimitiveExecutable = PrimitiveExecutable {
    name:"tracingassigns",
    expandable:false,
    _apply:|_tk,_int| {todo!()}
};

pub static TRACINGGROUPS: PrimitiveExecutable = PrimitiveExecutable {
    name:"tracinggroups",
    expandable:false,
    _apply:|_tk,_int| {todo!()}
};

pub static TRACINGIFS: PrimitiveExecutable = PrimitiveExecutable {
    name:"tracingifs",
    expandable:false,
    _apply:|_tk,_int| {todo!()}
};


pub static EFCODE: PrimitiveExecutable = PrimitiveExecutable {
    name:"efcode",
    expandable:false,
    _apply:|_tk,_int| {todo!()}
};

pub static LEFTMARGINKERN: PrimitiveExecutable = PrimitiveExecutable {
    name:"leftmarginkern",
    expandable:false,
    _apply:|_tk,_int| {todo!()}
};

pub static LETTERSPACEFONT: PrimitiveExecutable = PrimitiveExecutable {
    name:"letterspacefont",
    expandable:false,
    _apply:|_tk,_int| {todo!()}
};

pub static QUITVMODE: PrimitiveExecutable = PrimitiveExecutable {
    name:"quitvmode",
    expandable:false,
    _apply:|_tk,_int| {todo!()}
};

pub static RIGHTMARGINKERN: PrimitiveExecutable = PrimitiveExecutable {
    name:"rightmarginkern",
    expandable:false,
    _apply:|_tk,_int| {todo!()}
};

pub static TAGCODE: PrimitiveExecutable = PrimitiveExecutable {
    name:"tagcode",
    expandable:false,
    _apply:|_tk,_int| {todo!()}
};

pub static AFTERASSIGNMENT: PrimitiveExecutable = PrimitiveExecutable {
    name:"afterassignment",
    expandable:false,
    _apply:|_tk,_int| {todo!()}
};

pub static AFTERGROUP: PrimitiveExecutable = PrimitiveExecutable {
    name:"aftergroup",
    expandable:false,
    _apply:|_tk,_int| {todo!()}
};

pub static DELCODE: PrimitiveExecutable = PrimitiveExecutable {
    name:"delcode",
    expandable:false,
    _apply:|_tk,_int| {todo!()}
};

pub static DIMEN: PrimitiveExecutable = PrimitiveExecutable {
    name:"dimen",
    expandable:false,
    _apply:|_tk,_int| {todo!()}
};

pub static ESCAPECHAR: PrimitiveExecutable = PrimitiveExecutable {
    name:"escapechar",
    expandable:false,
    _apply:|_tk,_int| {todo!()}
};

pub static FONT: PrimitiveExecutable = PrimitiveExecutable {
    name:"font",
    expandable:false,
    _apply:|_tk,_int| {todo!()}
};

pub static FONTDIMEN: PrimitiveExecutable = PrimitiveExecutable {
    name:"fontdimen",
    expandable:false,
    _apply:|_tk,_int| {todo!()}
};

pub static FUTURELET: PrimitiveExecutable = PrimitiveExecutable {
    name:"futurelet",
    expandable:false,
    _apply:|_tk,_int| {todo!()}
};

pub static HYPHENATION: PrimitiveExecutable = PrimitiveExecutable {
    name:"hyphenation",
    expandable:false,
    _apply:|_tk,_int| {todo!()}
};

pub static HYPHENCHAR: PrimitiveExecutable = PrimitiveExecutable {
    name:"hyphenchar",
    expandable:false,
    _apply:|_tk,_int| {todo!()}
};

pub static LCCODE: PrimitiveExecutable = PrimitiveExecutable {
    name:"lccode",
    expandable:false,
    _apply:|_tk,_int| {todo!()}
};

pub static LPCODE: PrimitiveExecutable = PrimitiveExecutable {
    name:"lpcode",
    expandable:false,
    _apply:|_tk,_int| {todo!()}
};

pub static RPCODE: PrimitiveExecutable = PrimitiveExecutable {
    name:"rpcode",
    expandable:false,
    _apply:|_tk,_int| {todo!()}
};

pub static SETBOX: PrimitiveExecutable = PrimitiveExecutable {
    name:"setbox",
    expandable:false,
    _apply:|_tk,_int| {todo!()}
};

pub static MATHCODE: PrimitiveExecutable = PrimitiveExecutable {
    name:"mathcode",
    expandable:false,
    _apply:|_tk,_int| {todo!()}
};

pub static MUSKIP: PrimitiveExecutable = PrimitiveExecutable {
    name:"muskip",
    expandable:false,
    _apply:|_tk,_int| {todo!()}
};

pub static MUSKIPDEF: PrimitiveExecutable = PrimitiveExecutable {
    name:"muskipdef",
    expandable:false,
    _apply:|_tk,_int| {todo!()}
};

pub static OUTER: PrimitiveExecutable = PrimitiveExecutable {
    name:"outer",
    expandable:false,
    _apply:|_tk,_int| {todo!()}
};

pub static PAGEGOAL: PrimitiveExecutable = PrimitiveExecutable {
    name:"pagegoal",
    expandable:false,
    _apply:|_tk,_int| {todo!()}
};

pub static PARSHAPE: PrimitiveExecutable = PrimitiveExecutable {
    name:"parshape",
    expandable:false,
    _apply:|_tk,_int| {todo!()}
};

pub static PATTERNS: PrimitiveExecutable = PrimitiveExecutable {
    name:"patterns",
    expandable:false,
    _apply:|_tk,_int| {todo!()}
};

pub static READLINE: PrimitiveExecutable = PrimitiveExecutable {
    name:"readline",
    expandable:false,
    _apply:|_tk,_int| {todo!()}
};

pub static SCRIPTFONT: PrimitiveExecutable = PrimitiveExecutable {
    name:"scriptfont",
    expandable:false,
    _apply:|_tk,_int| {todo!()}
};

pub static SCRIPTSCRIPTFONT: PrimitiveExecutable = PrimitiveExecutable {
    name:"scriptscriptfont",
    expandable:false,
    _apply:|_tk,_int| {todo!()}
};

pub static SKEWCHAR: PrimitiveExecutable = PrimitiveExecutable {
    name:"skewchar",
    expandable:false,
    _apply:|_tk,_int| {todo!()}
};

pub static SKIP: PrimitiveExecutable = PrimitiveExecutable {
    name:"skip",
    expandable:false,
    _apply:|_tk,_int| {todo!()}
};

pub static TEXTFONT: PrimitiveExecutable = PrimitiveExecutable {
    name:"textfont",
    expandable:false,
    _apply:|_tk,_int| {todo!()}
};

pub static TOKS: PrimitiveExecutable = PrimitiveExecutable {
    name:"toks",
    expandable:false,
    _apply:|_tk,_int| {todo!()}
};

pub static UCCODE: PrimitiveExecutable = PrimitiveExecutable {
    name:"uccode",
    expandable:false,
    _apply:|_tk,_int| {todo!()}
};


// -------------------------------------------------------------------------------------------------

pub fn tex_commands() -> Vec<PrimitiveTeXCommand> {vec![
    PrimitiveTeXCommand::Primitive(&PAR),
    PrimitiveTeXCommand::Primitive(&RELAX),
    PrimitiveTeXCommand::AV(AssignableValue::Int(&CATCODE)),
    PrimitiveTeXCommand::AV(AssignableValue::Int(&SFCODE)),
    PrimitiveTeXCommand::AV(AssignableValue::Int(&NEWLINECHAR)),
    PrimitiveTeXCommand::AV(AssignableValue::Int(&ENDLINECHAR)),
    PrimitiveTeXCommand::AV(AssignableValue::Int(&COUNT)),
    PrimitiveTeXCommand::Ass(&CHARDEF),
    PrimitiveTeXCommand::Ass(&COUNTDEF),
    PrimitiveTeXCommand::Ass(&DIMENDEF),
    PrimitiveTeXCommand::Ass(&SKIPDEF),
    PrimitiveTeXCommand::Ass(&TOKSDEF),
    PrimitiveTeXCommand::Ass(&GLOBAL),
    PrimitiveTeXCommand::Ass(&DEF),
    PrimitiveTeXCommand::Ass(&EDEF),
    PrimitiveTeXCommand::Ass(&GDEF),
    PrimitiveTeXCommand::Ass(&XDEF),
    PrimitiveTeXCommand::Ass(&LET),
    PrimitiveTeXCommand::Ass(&LONG),
    PrimitiveTeXCommand::Ass(&PROTECTED),
    PrimitiveTeXCommand::Ass(&DIVIDE),
    PrimitiveTeXCommand::Ass(&MULTIPLY),
    PrimitiveTeXCommand::Ass(&ADVANCE),
    PrimitiveTeXCommand::Primitive(&INPUT),
    PrimitiveTeXCommand::Primitive(&BEGINGROUP),
    PrimitiveTeXCommand::Primitive(&ENDGROUP),
    PrimitiveTeXCommand::Primitive(&THE),
    PrimitiveTeXCommand::Primitive(&NUMBER),
    PrimitiveTeXCommand::Primitive(&IMMEDIATE),
    PrimitiveTeXCommand::Whatsit(ProvidesWhatsit::Exec(&OPENOUT)),
    PrimitiveTeXCommand::Primitive(&OPENIN),
    PrimitiveTeXCommand::Whatsit(ProvidesWhatsit::Exec(&CLOSEOUT)),
    PrimitiveTeXCommand::Primitive(&CLOSEIN),
    PrimitiveTeXCommand::Whatsit(ProvidesWhatsit::Exec(&WRITE)),
    PrimitiveTeXCommand::Ass(&READ),
    PrimitiveTeXCommand::Ass(&MATHCHARDEF),
    PrimitiveTeXCommand::Int(&TIME),
    PrimitiveTeXCommand::Int(&YEAR),
    PrimitiveTeXCommand::Int(&MONTH),
    PrimitiveTeXCommand::Int(&DAY),
    PrimitiveTeXCommand::Primitive(&NOEXPAND),
    PrimitiveTeXCommand::Primitive(&EXPANDAFTER),
    PrimitiveTeXCommand::Primitive(&MEANING),
    PrimitiveTeXCommand::Primitive(&ETEXREVISION),
    PrimitiveTeXCommand::Primitive(&UNEXPANDED),
    PrimitiveTeXCommand::Int(&ETEXVERSION),

    PrimitiveTeXCommand::AV(AssignableValue::PrimReg(&PRETOLERANCE)),
    PrimitiveTeXCommand::AV(AssignableValue::PrimReg(&TOLERANCE)),
    PrimitiveTeXCommand::AV(AssignableValue::PrimReg(&HBADNESS)),
    PrimitiveTeXCommand::AV(AssignableValue::PrimReg(&VBADNESS)),
    PrimitiveTeXCommand::AV(AssignableValue::PrimReg(&LINEPENALTY)),
    PrimitiveTeXCommand::AV(AssignableValue::PrimReg(&HYPHENPENALTY)),
    PrimitiveTeXCommand::AV(AssignableValue::PrimReg(&EXHYPHENPENALTY)),
    PrimitiveTeXCommand::AV(AssignableValue::PrimReg(&BINOPPENALTY)),
    PrimitiveTeXCommand::AV(AssignableValue::PrimReg(&RELPENALTY)),
    PrimitiveTeXCommand::AV(AssignableValue::PrimReg(&CLUBPENALTY)),
    PrimitiveTeXCommand::AV(AssignableValue::PrimReg(&WIDOWPENALTY)),
    PrimitiveTeXCommand::AV(AssignableValue::PrimReg(&DISPLAYWIDOWPENALTY)),
    PrimitiveTeXCommand::AV(AssignableValue::PrimReg(&BROKENPENALTY)),
    PrimitiveTeXCommand::AV(AssignableValue::PrimReg(&PREDISPLAYPENALTY)),
    PrimitiveTeXCommand::AV(AssignableValue::PrimReg(&DOUBLEHYPHENDEMERITS)),
    PrimitiveTeXCommand::AV(AssignableValue::PrimReg(&FINALHYPHENDEMERITS)),
    PrimitiveTeXCommand::AV(AssignableValue::PrimReg(&ADJDEMERITS)),
    PrimitiveTeXCommand::AV(AssignableValue::PrimReg(&TRACINGLOSTCHARS)),
    PrimitiveTeXCommand::AV(AssignableValue::PrimReg(&UCHYPH)),
    PrimitiveTeXCommand::AV(AssignableValue::PrimReg(&DEFAULTHYPHENCHAR)),
    PrimitiveTeXCommand::AV(AssignableValue::PrimReg(&DEFAULTSKEWCHAR)),
    PrimitiveTeXCommand::AV(AssignableValue::PrimReg(&DELIMITERFACTOR)),
    PrimitiveTeXCommand::AV(AssignableValue::PrimReg(&SHOWBOXBREADTH)),
    PrimitiveTeXCommand::AV(AssignableValue::PrimReg(&SHOWBOXDEPTH)),
    PrimitiveTeXCommand::AV(AssignableValue::PrimReg(&ERRORCONTEXTLINES)),
    PrimitiveTeXCommand::AV(AssignableValue::PrimReg(&MAXDEADCYCLES)),
    PrimitiveTeXCommand::AV(AssignableValue::PrimReg(&TRACINGSTATS)),
    PrimitiveTeXCommand::AV(AssignableValue::PrimReg(&LEFTHYPHENMIN)),
    PrimitiveTeXCommand::AV(AssignableValue::PrimReg(&RIGHTHYPHENMIN)),
    PrimitiveTeXCommand::AV(AssignableValue::PrimReg(&SAVINGHYPHCODES)),
    PrimitiveTeXCommand::AV(AssignableValue::PrimReg(&FAM)),
    PrimitiveTeXCommand::AV(AssignableValue::PrimReg(&SPACEFACTOR)),
    PrimitiveTeXCommand::AV(AssignableValue::PrimReg(&GLOBALDEFS)),
    PrimitiveTeXCommand::AV(AssignableValue::PrimReg(&TRACINGNESTING)),
    PrimitiveTeXCommand::AV(AssignableValue::PrimReg(&MAG)),
    PrimitiveTeXCommand::AV(AssignableValue::PrimReg(&LANGUAGE)),
    PrimitiveTeXCommand::AV(AssignableValue::PrimReg(&HANGAFTER)),
    PrimitiveTeXCommand::AV(AssignableValue::PrimReg(&INTERLINEPENALTY)),
    PrimitiveTeXCommand::AV(AssignableValue::PrimReg(&FLOATINGPENALTY)),
    PrimitiveTeXCommand::AV(AssignableValue::PrimReg(&LASTNODETYPE)),
    PrimitiveTeXCommand::AV(AssignableValue::PrimReg(&INSERTPENALTIES)),
    PrimitiveTeXCommand::AV(AssignableValue::PrimReg(&BADNESS)),
    PrimitiveTeXCommand::AV(AssignableValue::PrimReg(&DEADCYCLES)),
    PrimitiveTeXCommand::AV(AssignableValue::PrimReg(&INTERLINEPENALTIES)),
    PrimitiveTeXCommand::AV(AssignableValue::PrimReg(&CLUBPENALTIES)),
    PrimitiveTeXCommand::AV(AssignableValue::PrimReg(&WIDOWPENALTIES)),
    PrimitiveTeXCommand::AV(AssignableValue::PrimReg(&DISPLAYWIDOWPENALTIES)),
    PrimitiveTeXCommand::AV(AssignableValue::PrimReg(&OUTPUTPENALTY)),
    PrimitiveTeXCommand::AV(AssignableValue::PrimReg(&SAVINGVDISCARDS)),
    PrimitiveTeXCommand::AV(AssignableValue::PrimReg(&DISPLAYINDENT)),
    PrimitiveTeXCommand::AV(AssignableValue::PrimReg(&SYNCTEX)),
    PrimitiveTeXCommand::AV(AssignableValue::PrimReg(&POSTDISPLAYPENALTY)),
    PrimitiveTeXCommand::AV(AssignableValue::PrimReg(&TRACINGSCANTOKENS)),

    PrimitiveTeXCommand::AV(AssignableValue::PrimDim(&HFUZZ)),
    PrimitiveTeXCommand::AV(AssignableValue::PrimDim(&VFUZZ)),
    PrimitiveTeXCommand::AV(AssignableValue::PrimDim(&OVERFULLRULE)),
    PrimitiveTeXCommand::AV(AssignableValue::PrimDim(&MAXDEPTH)),
    PrimitiveTeXCommand::AV(AssignableValue::PrimDim(&SPLITMAXDEPTH)),
    PrimitiveTeXCommand::AV(AssignableValue::PrimDim(&BOXMAXDEPTH)),
    PrimitiveTeXCommand::AV(AssignableValue::PrimDim(&DELIMITERSHORTFALL)),
    PrimitiveTeXCommand::AV(AssignableValue::PrimDim(&NULLDELIMITERSPACE)),
    PrimitiveTeXCommand::AV(AssignableValue::PrimDim(&SCRIPTSPACE)),
    PrimitiveTeXCommand::AV(AssignableValue::PrimDim(&PARINDENT)),
    PrimitiveTeXCommand::AV(AssignableValue::PrimDim(&VSIZE)),
    PrimitiveTeXCommand::AV(AssignableValue::PrimDim(&HSIZE)),
    PrimitiveTeXCommand::AV(AssignableValue::PrimDim(&LINESKIPLIMIT)),
    PrimitiveTeXCommand::AV(AssignableValue::PrimDim(&MATHSURROUND)),
    PrimitiveTeXCommand::AV(AssignableValue::PrimDim(&HANGINDENT)),
    PrimitiveTeXCommand::AV(AssignableValue::PrimDim(&PAGETOTAL)),
    PrimitiveTeXCommand::AV(AssignableValue::PrimDim(&PAGESTRETCH)),
    PrimitiveTeXCommand::AV(AssignableValue::PrimDim(&PAGEFILSTRETCH)),
    PrimitiveTeXCommand::AV(AssignableValue::PrimDim(&PAGEFILLSTRETCH)),
    PrimitiveTeXCommand::AV(AssignableValue::PrimDim(&PAGEFILLLSTRETCH)),
    PrimitiveTeXCommand::AV(AssignableValue::PrimDim(&PAGESHRINK)),
    PrimitiveTeXCommand::AV(AssignableValue::PrimDim(&PAGEDEPTH)),
    PrimitiveTeXCommand::AV(AssignableValue::PrimDim(&EMERGENCYSTRETCH)),
    PrimitiveTeXCommand::AV(AssignableValue::PrimDim(&VOFFSET)),
    PrimitiveTeXCommand::AV(AssignableValue::PrimDim(&HOFFSET)),
    PrimitiveTeXCommand::AV(AssignableValue::PrimDim(&DISPLAYWIDTH)),
    PrimitiveTeXCommand::AV(AssignableValue::PrimDim(&PREDISPLAYSIZE)),

    PrimitiveTeXCommand::AV(AssignableValue::PrimSkip(&PARSKIP)),
    PrimitiveTeXCommand::AV(AssignableValue::PrimSkip(&ABOVEDISPLAYSKIP)),
    PrimitiveTeXCommand::AV(AssignableValue::PrimSkip(&ABOVEDISPLAYSHORTSKIP)),
    PrimitiveTeXCommand::AV(AssignableValue::PrimSkip(&BELOWDISPLAYSKIP)),
    PrimitiveTeXCommand::AV(AssignableValue::PrimSkip(&BELOWDISPLAYSHORTSKIP)),
    PrimitiveTeXCommand::AV(AssignableValue::PrimSkip(&TOPSKIP)),
    PrimitiveTeXCommand::AV(AssignableValue::PrimSkip(&SPLITTOPSKIP)),
    PrimitiveTeXCommand::AV(AssignableValue::PrimSkip(&PARFILLSKIP)),
    PrimitiveTeXCommand::AV(AssignableValue::PrimSkip(&BASELINESKIP)),
    PrimitiveTeXCommand::AV(AssignableValue::PrimSkip(&LINESKIP)),
    PrimitiveTeXCommand::AV(AssignableValue::PrimSkip(&PREVDEPTH)),
    PrimitiveTeXCommand::AV(AssignableValue::PrimSkip(&LEFTSKIP)),
    PrimitiveTeXCommand::AV(AssignableValue::PrimSkip(&RIGHTSKIP)),
    PrimitiveTeXCommand::AV(AssignableValue::PrimSkip(&TABSKIP)),
    PrimitiveTeXCommand::AV(AssignableValue::PrimSkip(&SPACESKIP)),
    PrimitiveTeXCommand::AV(AssignableValue::PrimSkip(&XSPACESKIP)),
    PrimitiveTeXCommand::AV(AssignableValue::PrimSkip(&BIGSKIPAMOUNT)),

    PrimitiveTeXCommand::AV(AssignableValue::PrimToks(&EVERYJOB)),
    PrimitiveTeXCommand::AV(AssignableValue::PrimToks(&EVERYPAR)),
    PrimitiveTeXCommand::AV(AssignableValue::PrimToks(&EVERYMATH)),
    PrimitiveTeXCommand::AV(AssignableValue::PrimToks(&EVERYDISPLAY)),
    PrimitiveTeXCommand::AV(AssignableValue::PrimToks(&EVERYHBOX)),
    PrimitiveTeXCommand::AV(AssignableValue::PrimToks(&EVERYVBOX)),
    PrimitiveTeXCommand::AV(AssignableValue::PrimToks(&EVERYCR)),
    PrimitiveTeXCommand::AV(AssignableValue::PrimToks(&EVERYEOF)),
    PrimitiveTeXCommand::AV(AssignableValue::PrimToks(&ERRHELP)),
    PrimitiveTeXCommand::AV(AssignableValue::PrimToks(&OUTPUT)),

    // TODO ----------------------------------------------------------------------------------------
    PrimitiveTeXCommand::Primitive(&END),
    PrimitiveTeXCommand::Primitive(&BATCHMODE),
    PrimitiveTeXCommand::Primitive(&BYE),
    PrimitiveTeXCommand::Primitive(&CHAR),
    PrimitiveTeXCommand::Primitive(&CR),
    PrimitiveTeXCommand::Primitive(&CRCR),
    PrimitiveTeXCommand::Primitive(&CSNAME),
    PrimitiveTeXCommand::Primitive(&ENDCSNAME),
    PrimitiveTeXCommand::Primitive(&CURRENTGROUPLEVEL),
    PrimitiveTeXCommand::Primitive(&DETOKENIZE),
    PrimitiveTeXCommand::Primitive(&DIMEXPR),
    PrimitiveTeXCommand::Primitive(&DUMP),
    PrimitiveTeXCommand::Primitive(&ENDINPUT),
    PrimitiveTeXCommand::Primitive(&EQNO),
    PrimitiveTeXCommand::Primitive(&ERRMESSAGE),
    PrimitiveTeXCommand::Primitive(&ERRORSTOPMODE),
    PrimitiveTeXCommand::Primitive(&EXPANDED),
    PrimitiveTeXCommand::Primitive(&FONTNAME),
    PrimitiveTeXCommand::Primitive(&FONTCHARWD),
    PrimitiveTeXCommand::Primitive(&FONTCHARHT),
    PrimitiveTeXCommand::Primitive(&FONTCHARDP),
    PrimitiveTeXCommand::Primitive(&FONTCHARIC),
    PrimitiveTeXCommand::Primitive(&GLUEEXPR),
    PrimitiveTeXCommand::Primitive(&IGNORESPACES),
    PrimitiveTeXCommand::Primitive(&INPUTLINENO),
    PrimitiveTeXCommand::Primitive(&JOBNAME),
    PrimitiveTeXCommand::Primitive(&LOWERCASE),
    PrimitiveTeXCommand::Primitive(&MESSAGE),
    PrimitiveTeXCommand::Primitive(&MUEXPR),
    PrimitiveTeXCommand::Primitive(&NULLFONT),
    PrimitiveTeXCommand::Primitive(&NUMEXPR),
    PrimitiveTeXCommand::Primitive(&ROMANNUMERAL),
    PrimitiveTeXCommand::Primitive(&SCANTOKENS),
    PrimitiveTeXCommand::Primitive(&SHIPOUT),
    PrimitiveTeXCommand::Primitive(&STRING),
    PrimitiveTeXCommand::Primitive(&UPPERCASE),
    PrimitiveTeXCommand::Primitive(&TEXTSTYLE),
    PrimitiveTeXCommand::Primitive(&SCRIPTSTYLE),
    PrimitiveTeXCommand::Primitive(&SCRIPTSCRIPTSTYLE),
    PrimitiveTeXCommand::Primitive(&SPECIAL),
    PrimitiveTeXCommand::Primitive(&NONSCRIPT),
    PrimitiveTeXCommand::Primitive(&HOLDINGINSERTS),
    PrimitiveTeXCommand::Primitive(&LEQNO),
    PrimitiveTeXCommand::Primitive(&LOOSENESS),
    PrimitiveTeXCommand::Primitive(&NOBOUNDARY),
    PrimitiveTeXCommand::Primitive(&SCROLLMODE),
    PrimitiveTeXCommand::Primitive(&NONSTOPMODE),
    PrimitiveTeXCommand::Primitive(&OMIT),
    PrimitiveTeXCommand::Primitive(&PAUSING),
    PrimitiveTeXCommand::Primitive(&PREVGRAF),
    PrimitiveTeXCommand::Primitive(&SETLANGUAGE),
    PrimitiveTeXCommand::Primitive(&SHOW),
    PrimitiveTeXCommand::Primitive(&SHOWBOX),
    PrimitiveTeXCommand::Primitive(&SHOWLISTS),
    PrimitiveTeXCommand::Primitive(&SHOWTHE),
    PrimitiveTeXCommand::Primitive(&SPAN),
    PrimitiveTeXCommand::Primitive(&TRACINGCOMMANDS),
    PrimitiveTeXCommand::Primitive(&TRACINGMACROS),
    PrimitiveTeXCommand::Primitive(&TRACINGONLINE),
    PrimitiveTeXCommand::Primitive(&TRACINGOUTPUT),
    PrimitiveTeXCommand::Primitive(&TRACINGPAGES),
    PrimitiveTeXCommand::Primitive(&TRACINGPARAGRAPHS),
    PrimitiveTeXCommand::Primitive(&TRACINGRESTORES),
    PrimitiveTeXCommand::Primitive(&VALIGN),
    PrimitiveTeXCommand::Primitive(&BEGINL),
    PrimitiveTeXCommand::Primitive(&BEGINR),
    PrimitiveTeXCommand::Primitive(&BOTMARKS),
    PrimitiveTeXCommand::Primitive(&CURRENTGROUPTYPE),
    PrimitiveTeXCommand::Primitive(&CURRENTIFBRANCH),
    PrimitiveTeXCommand::Primitive(&CURRENTIFLEVEL),
    PrimitiveTeXCommand::Primitive(&CURRENTIFTYPE),
    PrimitiveTeXCommand::Primitive(&ENDL),
    PrimitiveTeXCommand::Primitive(&ENDR),
    PrimitiveTeXCommand::Primitive(&FIRSTMARKS),
    PrimitiveTeXCommand::Primitive(&GLUESHRINK),
    PrimitiveTeXCommand::Primitive(&GLUESHRINKORDER),
    PrimitiveTeXCommand::Primitive(&GLUESTRETCH),
    PrimitiveTeXCommand::Primitive(&GLUESTRETCHORDER),
    PrimitiveTeXCommand::Primitive(&GLUETOMU),
    PrimitiveTeXCommand::Primitive(&INTERACTIONMODE),
    PrimitiveTeXCommand::Primitive(&LASTLINEFIT),
    PrimitiveTeXCommand::Primitive(&MARKS),
    PrimitiveTeXCommand::Primitive(&MUTOGLUE),
    PrimitiveTeXCommand::Primitive(&PAGEDISCARDS),
    PrimitiveTeXCommand::Primitive(&PARSHAPEDIMEN),
    PrimitiveTeXCommand::Primitive(&PARSHAPEINDENT),
    PrimitiveTeXCommand::Primitive(&PARSHAPELENGTH),
    PrimitiveTeXCommand::Primitive(&PREDISPLAYDIRECTION),
    PrimitiveTeXCommand::Primitive(&SHOWGROUPS),
    PrimitiveTeXCommand::Primitive(&SHOWIFS),
    PrimitiveTeXCommand::Primitive(&SHOWTOKENS),
    PrimitiveTeXCommand::Primitive(&SPLITBOTMARKS),
    PrimitiveTeXCommand::Primitive(&SPLITDISCARDS),
    PrimitiveTeXCommand::Primitive(&SPLITFIRSTMARKS),
    PrimitiveTeXCommand::Primitive(&TEXXETSTATE),
    PrimitiveTeXCommand::Primitive(&TOPMARKS),
    PrimitiveTeXCommand::Primitive(&TRACINGASSIGNS),
    PrimitiveTeXCommand::Primitive(&TRACINGGROUPS),
    PrimitiveTeXCommand::Primitive(&TRACINGIFS),
    PrimitiveTeXCommand::Primitive(&EFCODE),
    PrimitiveTeXCommand::Primitive(&LEFTMARGINKERN),
    PrimitiveTeXCommand::Primitive(&LETTERSPACEFONT),
    PrimitiveTeXCommand::Primitive(&QUITVMODE),
    PrimitiveTeXCommand::Primitive(&RIGHTMARGINKERN),
    PrimitiveTeXCommand::Primitive(&TAGCODE),
    PrimitiveTeXCommand::Primitive(&AFTERASSIGNMENT),
    PrimitiveTeXCommand::Primitive(&AFTERGROUP),
    PrimitiveTeXCommand::Primitive(&DELCODE),
    PrimitiveTeXCommand::Primitive(&DIMEN),
    PrimitiveTeXCommand::Primitive(&ESCAPECHAR),
    PrimitiveTeXCommand::Primitive(&FONT),
    PrimitiveTeXCommand::Primitive(&FONTDIMEN),
    PrimitiveTeXCommand::Primitive(&FUTURELET),
    PrimitiveTeXCommand::Primitive(&HYPHENATION),
    PrimitiveTeXCommand::Primitive(&HYPHENCHAR),
    PrimitiveTeXCommand::Primitive(&LCCODE),
    PrimitiveTeXCommand::Primitive(&LPCODE),
    PrimitiveTeXCommand::Primitive(&RPCODE),
    PrimitiveTeXCommand::Primitive(&SETBOX),
    PrimitiveTeXCommand::Primitive(&MATHCODE),
    PrimitiveTeXCommand::Primitive(&MUSKIP),
    PrimitiveTeXCommand::Primitive(&MUSKIPDEF),
    PrimitiveTeXCommand::Primitive(&OUTER),
    PrimitiveTeXCommand::Primitive(&PAGEGOAL),
    PrimitiveTeXCommand::Primitive(&PARSHAPE),
    PrimitiveTeXCommand::Primitive(&PATTERNS),
    PrimitiveTeXCommand::Primitive(&READLINE),
    PrimitiveTeXCommand::Primitive(&SCRIPTFONT),
    PrimitiveTeXCommand::Primitive(&SCRIPTSCRIPTFONT),
    PrimitiveTeXCommand::Primitive(&SKEWCHAR),
    PrimitiveTeXCommand::Primitive(&SKIP),
    PrimitiveTeXCommand::Primitive(&TEXTFONT),
    PrimitiveTeXCommand::Primitive(&TOKS),
    PrimitiveTeXCommand::Primitive(&UCCODE),
]}