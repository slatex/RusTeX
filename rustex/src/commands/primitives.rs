use std::cell::RefCell;
use std::ops::Deref;
use std::rc::Rc;
use crate::commands::{RegisterReference, AssignableValue, NumAssValue, DefMacro, NumericCommand, ParamToken, PrimitiveAssignment, PrimitiveExecutable, ProvidesExecutableWhatsit, ProvidesWhatsit, Signature, TokenList, DimenReference, SkipReference, TokReference, PrimitiveTeXCommand, FontAssValue, TeXCommand, ProvidesBox, TokAssValue, MathWhatsit, MuSkipReference, SimpleWhatsit};
use crate::interpreter::{Interpreter, TeXMode};
use crate::ontology::{Token, Expansion, ExpansionRef};
use crate::catcodes::CategoryCode;
use crate::interpreter::state::{FontStyle, GroupType, StateChange};
use crate::utils::{TeXError, TeXStr, TeXString};
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
pub static CATCODE : NumAssValue = NumAssValue {
    name: "catcode",
    _assign: |_rf,int,global| {
        let num = int.read_number()? as u8;
        int.read_eq();
        let cat = CategoryCode::fromint(int.read_number()?);
        int.change_state(StateChange::Cat(num,cat,global));
        Ok(())
    },
    _getvalue: |int| {
        let char = int.read_number()?;
        Ok(Numeric::Int(CategoryCode::toint(&int.state_catcodes().get_code(char as u8)) as i64))
    }
};

pub static SFCODE : NumAssValue = NumAssValue {
    name:"sfcode",
    _assign: |_rf,int,global| {
        let char = int.read_number()? as u8;
        int.read_eq();
        let val = int.read_number()?;
        int.change_state(StateChange::Sfcode(char,val,global));
        Ok(())
    },
    _getvalue: |int| {
        let char = int.read_number()? as u8;
        Ok(Numeric::Int(int.state_sfcode(char)))
    }
};

use crate::references::{SourceFileReference, SourceReference};
use chrono::{Datelike, Timelike};
use crate::commands::AssignableValue::FontRef;
use crate::fonts::{Font, Nullfont};

pub static CHARDEF: PrimitiveAssignment = PrimitiveAssignment {
    name: "chardef",
    _assign: |rf,int,global| {
        let c = int.read_command_token()?;
        int.read_eq();
        let num = int.read_number()?;
        let cmd = PrimitiveTeXCommand::Char(Token::new(num as u8,CategoryCode::Other,None,SourceReference::None,true)).as_ref(rf);
        int.change_state(StateChange::Cs(c.cmdname().clone(),Some(cmd),global));
        Ok(())
    }
};

pub static COUNT : NumAssValue = NumAssValue {
    name: "count",
    _assign: |_,int,global| {
        let index = int.read_number()? as u16;
        int.read_eq();
        let val = int.read_number()?;
        log!("\\count sets {} to {}",index,val);
        int.change_state(StateChange::Register(index as i32,val,global));
        Ok(())
    },
    _getvalue: |int| {
        let index = int.read_number()? as i32;
        let num = int.state_register(index);
        log!("\\count {} = {}",index,num);
        Ok(Numeric::Int(num))
    }
};

pub static DIMEN : NumAssValue = NumAssValue {
    name: "dimen",
    _assign: |_,int,global| {
        let index = int.read_number()? as u16;
        int.read_eq();
        let val = int.read_dimension()?;
        log!("\\dimen sets {} to {}",index,val);
        int.change_state(StateChange::Dimen(index as i32,val,global));
        Ok(())
    },
    _getvalue: |int| {
        let index = int.read_number()? as u16;
        let dim = int.state_dimension(index as i32);
        log!("\\dimen {} = {}",index,dim);
        Ok(Numeric::Dim(dim))
    }
};

pub static SKIP : NumAssValue = NumAssValue {
    name: "skip",
    _assign: |_,int,global| {
        let index = int.read_number()? as u16;
        int.read_eq();
        let val = int.read_skip()?;
        log!("\\skip sets {} to {}",index,val);
        int.change_state(StateChange::Skip(index as i32,val,global));
        Ok(())
    },
    _getvalue: |int| {
        let index = int.read_number()? as u16;
        let dim = int.state_skip(index as i32);
        log!("\\skip {} = {}",index,dim);
        Ok(Numeric::Skip(dim))
    }
};

pub static COUNTDEF: PrimitiveAssignment = PrimitiveAssignment {
    name:"countdef",
    _assign: |rf,int,global| {
        let cmd = int.read_command_token()?;
        int.set_relax(&cmd);
        int.read_eq();
        let num = int.read_number()? as u16;
        let command = PrimitiveTeXCommand::AV(AssignableValue::Register(num)).as_ref(rf);

        int.change_state(StateChange::Cs(cmd.cmdname().clone(),
                                         Some(command),
                                         global));
        Ok(())
    }
};

pub static DIMENDEF: PrimitiveAssignment = PrimitiveAssignment {
    name:"dimendef",
    _assign: |rf,int,global| {
        let cmd = int.read_command_token()?;
        int.set_relax(&cmd);
        int.read_eq();
        let num = int.read_number()? as u16;
        let command = PrimitiveTeXCommand::AV(AssignableValue::Dim(num)).as_ref(rf);

        int.change_state(StateChange::Cs(cmd.cmdname().clone(),
                                         Some(command),
            global));
        Ok(())
    }
};

pub static SKIPDEF: PrimitiveAssignment = PrimitiveAssignment {
    name:"skipdef",
    _assign: |rf,int,global| {
        let cmd = int.read_command_token()?;
        int.set_relax(&cmd);
        int.read_eq();
        let num = int.read_number()? as u16;
        let command = PrimitiveTeXCommand::AV(AssignableValue::Skip(num)).as_ref(rf);

        int.change_state(StateChange::Cs(cmd.cmdname().clone(),
                                         Some(command),
            global));
        Ok(())
    }
};

pub static MUSKIPDEF: PrimitiveAssignment = PrimitiveAssignment {
    name:"muskipdef",
    _assign: |rf,int,global| {
        let cmd = int.read_command_token()?;
        int.set_relax(&cmd);
        int.read_eq();
        let num = int.read_number()? as u16;
        let command = PrimitiveTeXCommand::AV(AssignableValue::MuSkip(num)).as_ref(rf);

        int.change_state(StateChange::Cs(cmd.cmdname().clone(),
                                         Some(command),
                                         global));
        Ok(())
    }
};

pub static TOKSDEF: PrimitiveAssignment = PrimitiveAssignment {
    name:"toksdef",
    _assign: |rf,int,global| {
        let cmd = int.read_command_token()?;
        int.set_relax(&cmd);
        int.read_eq();
        let num = int.read_number()? as u16;
        let command = PrimitiveTeXCommand::AV(AssignableValue::Toks(num)).as_ref(rf);

        int.change_state(StateChange::Cs(cmd.cmdname().clone(),
                                         Some(command),
            global));
        Ok(())
    }
};

pub static PROTECTED : PrimitiveAssignment = PrimitiveAssignment {
    name:"protected",
    _assign: |rf,int,iglobal| {
        let mut long = false;
        let mut global = iglobal;
        while int.has_next() {
            int.expand_until(false);
            let next = int.next_token();
            match next.catcode {
                CategoryCode::Escape | CategoryCode::Active => {
                    match *int.get_command(&next.cmdname())?.orig {
                        PrimitiveTeXCommand::Ass(a) if *a == DEF => {
                            return do_def(rf,int,global,true,long,false)
                        }
                        PrimitiveTeXCommand::Ass(a) if *a == EDEF => {
                            return do_def(rf,int,global,true,long,true)
                        }
                        PrimitiveTeXCommand::Ass(a) if *a == GDEF => {
                            return do_def(rf,int,true,true,long,false)
                        }
                        PrimitiveTeXCommand::Ass(a) if *a == XDEF => {
                            return do_def(rf,int,true,true,long,true)
                        }
                        PrimitiveTeXCommand::Ass(a) if *a == LONG => {
                            long = true;
                        }
                        PrimitiveTeXCommand::Ass(a) if *a == GLOBAL => {
                            global = true;
                        }
                        _ => TeXErr!((int,Some(next.clone())),"Expected \\def or \\edef or \\long after \\protected: {}",next)
                    }
                }
                _ => TeXErr!((int,Some(next.clone())),"Expected control sequence or active character; got: {}",next)
            }
        }
        FileEnd!(int)
    }
};

pub static LONG: PrimitiveAssignment = PrimitiveAssignment {
    name:"long",
    _assign: |rf,int,iglobal| {
        let mut protected = false;
        let mut global = iglobal;
        while int.has_next() {
            int.expand_until(false);
            let next = int.next_token();
            match next.catcode {
                CategoryCode::Escape | CategoryCode::Active => {
                    match *int.get_command(&next.cmdname())?.orig {
                        PrimitiveTeXCommand::Ass(a) if *a == DEF => {
                            return do_def(rf,int,global,protected,true,false)
                        }
                        PrimitiveTeXCommand::Ass(a) if *a == EDEF => {
                            return do_def(rf,int,global,protected,true,true)
                        }
                        PrimitiveTeXCommand::Ass(a) if *a == GDEF => {
                            return do_def(rf,int,true,protected,true,false)
                        }
                        PrimitiveTeXCommand::Ass(a) if *a == XDEF => {
                            return do_def(rf,int,true,protected,true,true)
                        }
                        PrimitiveTeXCommand::Ass(a) if *a == PROTECTED => {
                            protected = true;
                        }
                        PrimitiveTeXCommand::Ass(a) if *a == GLOBAL => {
                            global = true;
                        }
                        _ => TeXErr!((int,Some(next)),"Expected \\def or \\edef or \\protected after \\long")
                    }
                }
                _ => TeXErr!((int,Some(next.clone())),"Expected control sequence or active character; got: {}",next)
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
                        if (inext.char < 48) {
                            TeXErr!((int,Some(inext.clone())),"Expected argument #{}; got:#{}",currarg,inext)
                        }
                        let arg = inext.char - 48;
                        if currarg == arg {
                            retsig.push(ParamToken::Param(arg,next));
                            currarg += 1
                        } else {
                            TeXErr!((int,Some(inext.clone())),"Expected argument #{}; got:#{}",currarg,inext)
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
    let command = int.next_token();
    match command.catcode {
        CategoryCode::Escape | CategoryCode::Active => {}
        _ => TeXErr!((int,Some(command.clone())),"\\def expected control sequence or active character; got: {}",command)
    }
    let sig = read_sig(int)?;
    let ret = int.read_token_list(edef,true,edef,false)?;
    log!("\\def {}{}{}{}{}",command,sig,"{",TokenList(&ret),"}");
    let dm = PrimitiveTeXCommand::Def(DefMacro {
        protected,
        long,
        sig,
        ret
    }).as_ref(rf);
    int.change_state(StateChange::Cs(command.cmdname().clone(),
                                     Some(dm),
        global));
    Ok(())
}

use crate::interpreter::dimensions::{dimtostr, Numeric, Skip};
use crate::stomach::whatsits::{AlignBlock, BoxMode, ExecutableWhatsit, HBox, MathKernel, SimpleWI, TeXBox, VBox, Whatsit, WIGroup};

pub static GLOBAL : PrimitiveAssignment = PrimitiveAssignment {
    name:"global",
    _assign: |_rf,int,_global| {
        int.expand_until(true)?;
        let next = int.read_command_token()?;
        let cmd = int.get_command(&next.cmdname())?;
        if !cmd.assignable() {
            TeXErr!((int,Some(next.clone())),"Assignment expected after \\global; found: {}",next)
        }
        cmd.assign(next,int,true)?;
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
        if cmd.catcode != CategoryCode::Escape && cmd.catcode != CategoryCode::Active {
            TeXErr!((int,Some(cmd.clone())),"Control sequence or active character expected; found {} of catcode {}",cmd,cmd.catcode)
        }
        int.read_eq();
        let def = int.next_token();
        log!("\\let {}={}",cmd,def);
        let ch = match def.catcode {
            CategoryCode::Escape | CategoryCode::Active => {
                int.state_get_command(&def.cmdname()).map(|x| x.as_ref(rf.0))
            }
            _ => Some(PrimitiveTeXCommand::Char(def).as_ref(rf))
        };
        int.change_state(StateChange::Cs(cmd.cmdname().clone(),ch,global));
        Ok(())
    }
};

pub static FUTURELET: PrimitiveAssignment = PrimitiveAssignment {
    name:"futurelet",
    _assign: |rf,int,global| {
        let newcmd = int.next_token();
        match newcmd.catcode {
            CategoryCode::Escape | CategoryCode::Active => {}
            _ => TeXErr!((int,Some(newcmd)),"Expected command after \\futurelet")
        }
        let first = int.next_token();
        let second = int.next_token();
        let p = match second.catcode {
            CategoryCode::Escape | CategoryCode::Active => {
                int.state_get_command(&second.cmdname()).map(|x| x.as_ref(rf.0))
            }
            _ => Some(PrimitiveTeXCommand::Char(second.clone()).as_command())
        };
        int.change_state(StateChange::Cs(newcmd.cmdname().clone(),p,global));
        int.push_tokens(vec!(first,second));
        Ok(())
    }
};

pub static NEWLINECHAR : NumAssValue = NumAssValue {
    name: "newlinechar",
    _assign: |_,int,global| {
        int.read_eq();
        let num = int.read_number()? as u8;
        log!("\\newlinechar: {}",num);
        int.change_state(StateChange::Newline(num,global));
        Ok(())
    },
    _getvalue: |int| {
        Ok(Numeric::Int(int.state_catcodes().newlinechar as i64))
    }
};

pub static ENDLINECHAR : NumAssValue = NumAssValue {
    name: "endlinechar",
    _assign: |_,int,global| {
        int.read_eq();
        let num = int.read_number()? as u8;
        log!("\\endlinechar: {}",num);
        int.change_state(StateChange::Endline(num,global));
        Ok(())
    },
    _getvalue: |int| {
        Ok(Numeric::Int(int.state_catcodes().endlinechar as i64))
    }
};

pub static ESCAPECHAR: NumAssValue = NumAssValue {
    name:"escapechar",
    _assign: |_,int,global| {
        int.read_eq();
        let num = int.read_number()? as u8;
        log!("\\escapechar: {}",num);
        int.change_state(StateChange::Escapechar(num,global));
        Ok(())
    },
    _getvalue: |int| {
        Ok(Numeric::Int(int.state_catcodes().escapechar as i64))
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

pub static TIME : NumericCommand = NumericCommand {
    _getvalue: |int| {
        let time = int.jobinfo.time;
        Ok(Numeric::Int(((time.hour() * 60) + time.minute()) as i64))
    },
    name: "time"
};

pub static YEAR : NumericCommand = NumericCommand {
    name:"year",
    _getvalue: |int| {
        Ok(Numeric::Int(int.jobinfo.time.year() as i64))
    }
};

pub static MONTH : NumericCommand = NumericCommand {
    name:"month",
    _getvalue: |int| {
        Ok(Numeric::Int(int.jobinfo.time.month() as i64))
    }
};

pub static DAY : NumericCommand = NumericCommand {
    name:"day",
    _getvalue: |int| {
        Ok(Numeric::Int(int.jobinfo.time.day() as i64))
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
fn get_inrv(int:&Interpreter,withint:bool) -> Result<(i32,Numeric,Numeric),TeXError> {
    use crate::commands::PrimitiveTeXCommand::*;
    let cmd = int.read_command_token()?;
    let (index,num,val) : (i32,Numeric,Numeric) = match *int.get_command(&cmd.cmdname())?.orig {
        AV(AssignableValue::Register(i)) => {
            int.read_keyword(vec!("by"))?;
            ((i as i32),Numeric::Int(int.state_register((i as i32))),int.read_number_i(false)?)
        }
        AV(AssignableValue::PrimReg(r)) => {
            int.read_keyword(vec!("by"))?;
            (-(r.index as i32), Numeric::Int(int.state_register(-(r.index as i32))), int.read_number_i(false)?)
        }
        AV(AssignableValue::Int(c)) if *c == COUNT => {
            let i = int.read_number()? as u16;
            int.read_keyword(vec!("by"))?;
            (i as i32,Numeric::Int(int.state_register(i as i32)),int.read_number_i(false)?)
        }
        AV(AssignableValue::Dim(i)) => {
            int.read_keyword(vec!("by"))?;
            (i as i32,Numeric::Dim(int.state_dimension(i as i32)), if withint {int.read_number_i(false)?} else {Numeric::Dim(int.read_dimension()?)})
        }
        AV(AssignableValue::PrimDim(r)) => {
            int.read_keyword(vec!("by"))?;
            (-(r.index as i32), Numeric::Dim(int.state_register(-(r.index as i32))),if withint {int.read_number_i(false)?} else {Numeric::Dim(int.read_dimension()?)})
        }
        AV(AssignableValue::Skip(i)) => {
            int.read_keyword(vec!("by"))?;
            (i as i32, Numeric::Skip(int.state_skip(i as i32)),if withint {int.read_number_i(false)?} else {Numeric::Skip(int.read_skip()?)})
        }
        ref p =>{
            todo!("{}",p)
        }
        //_ => return Err(TeXError::new("Expected register after \\divide; got: ".to_owned() + &cmd.as_string()))
    };
    Ok((index,num,val))
}
pub static DIVIDE : PrimitiveAssignment = PrimitiveAssignment {
    name: "divide",
    _assign: |_,int,global| {
        let (index,num,div) = get_inrv(int,true)?;
        log!("\\divide sets {} to {}",index,num/div);
        let ch = match num {
            Numeric::Int(i) => StateChange::Register(index,i / div.get_i64(),global),
            Numeric::Dim(_) => StateChange::Dimen(index,match (num / div.as_int()) {
                Numeric::Dim(i) => i,
                _ => unreachable!()
            },global),
            _ => todo!()
        };
        int.change_state(ch);
        Ok(())
    }
};
pub static MULTIPLY : PrimitiveAssignment = PrimitiveAssignment {
    name: "multiply",
    _assign: |_,int,global| {
        let (index,num,fac) = get_inrv(int,true)?;
        log!("\\multiply sets {} to {}",index,num*fac);
        let ch = match num {
            Numeric::Int(_) => StateChange::Register(index,match (num * fac.as_int()) {
                Numeric::Int(i) => i,
                _ => unreachable!()
            }, global),
            Numeric::Dim(_) => StateChange::Dimen(index,match (num * fac.as_int()) {
                Numeric::Dim(i) => i,
                _ => unreachable!()
            },global),
            _ => todo!()
        };
        int.change_state(ch);
        Ok(())
    }
};
pub static ADVANCE : PrimitiveAssignment = PrimitiveAssignment {
    name: "advance",
    _assign: |_,int,global| {
        let (index,num,sum) = get_inrv(int,false)?;
        log!("\\advance sets {} to {}",index,num+sum);
        let ch = match (num,sum) {
            (Numeric::Int(num),Numeric::Int(sum)) => StateChange::Register(index,num + sum,global),
            (Numeric::Int(num),Numeric::Dim(sum)) => StateChange::Register(index,num+sum,global),
            (Numeric::Dim(num),Numeric::Dim(sum)) => StateChange::Dimen(index,num + sum,global),
            (Numeric::Skip(num),Numeric::Skip(sum)) => StateChange::Skip(index,num + sum,global),
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
        int.expand_until(false)?;
        let reg = int.read_command_token()?;
        log!("\\the {}",reg);
        rf.2 = match &*int.get_command(&reg.cmdname())?.orig {
            Num(ic) => {
                let ret = (ic._getvalue)(int)?;
                log!("\\the{} = {}",reg,ret);
                stt(ret.to_string().into())
            },
            AV(AssignableValue::Int(i)) => stt((i._getvalue)(int)?.to_string().into()),
            AV(AssignableValue::PrimReg(i)) => stt(int.state_register(-(i.index as i32)).to_string().into()),
            AV(AssignableValue::Register(i)) => stt(int.state_register(*i as i32).to_string().into()),
            AV(AssignableValue::Toks(i)) => int.state_tokens(*i as i32),
            AV(AssignableValue::PrimToks(r)) => int.state_tokens(-(r.index as i32)),
            AV(AssignableValue::Tok(r)) => (r._getvalue)(int)?,
            Char(tk) => stt(tk.char.to_string().into()),
            MathChar(i) => stt(i.to_string().into()),
            AV(AssignableValue::Dim(i)) => stt(dimtostr(int.state_dimension((*i as i32))).into()),
            AV(AssignableValue::PrimDim(r)) => stt(dimtostr(int.state_dimension((-(r.index as i32)))).into()),
            AV(AssignableValue::Skip(i)) => stt(int.state_skip((*i as i32)).to_string().into()),
            AV(AssignableValue::PrimSkip(r)) => stt(int.state_skip(-(r.index as i32)).to_string().into()),
            AV(AssignableValue::FontRef(f)) => vec!(Token::new(0,CategoryCode::Escape,Some(f.name.clone()),SourceReference::None,true)),
            AV(AssignableValue::Font(f)) if **f == FONT =>
                vec!(Token::new(0,CategoryCode::Escape,Some(int.get_font().name.clone()),SourceReference::None,true)),
            AV(AssignableValue::Font(f)) => {
                let font = (f._getvalue)(int)?;
                vec!(Token::new(0,CategoryCode::Escape,Some(font.name.clone()),SourceReference::None,true))
            }
            p => {
                todo!("{}",p)
            }
        };
        Ok(())
    }
};

pub static IMMEDIATE : PrimitiveExecutable = PrimitiveExecutable {
    name:"immediate",
    expandable:false,
    _apply:|_,int| {
        use crate::commands::pdftex::*;
        let next = int.read_command_token()?;
        match *int.get_command(&next.cmdname())?.orig {
            PrimitiveTeXCommand::Whatsit(ProvidesWhatsit::Exec(e)) => {
                let wi = (e._get)(&next,int)?;
                (wi._apply)(int)?;
                Ok(())
            }
            PrimitiveTeXCommand::Primitive(x) if *x == PDFXFORM => {
                int.requeue(next);
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
        log!("\\openin {}",num);
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
        log!("\\closein {}",num);
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
            None => TeXErr!((int,None),"\"to\" expected in \\read")
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
        }).as_ref(rf);
        int.change_state(StateChange::Cs(newcmd.cmdname().clone(),
            Some(cmd),
            global));
        Ok(())
    }
};

pub static READLINE: PrimitiveAssignment = PrimitiveAssignment {
    name:"readline",
    _assign: |rf,int,global| {
        let index = int.read_number()? as u8;
        match int.read_keyword(vec!("to"))? {
            Some(_) => (),
            None => TeXErr!((int,None),"\"to\" expected in \\read")
        }
        let newcmd = int.read_command_token()?;
        let toks = int.file_read_line(index)?;
        let cmd = PrimitiveTeXCommand::Def(DefMacro {
            protected: false,
            long: false,
            sig: Signature {
                elems: vec![],
                endswithbrace: false,
                arity: 0
            },
            ret: toks
        }).as_ref(rf);
        int.change_state(StateChange::Cs(newcmd.cmdname().clone(),
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
            TeXErr!((int,Some(next)),"Begin group token expected after \\write")
        }

        let ret = int.read_token_list(true,true,false,true)?;
        let string = int.tokens_to_string(&ret);
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
            TeXErr!((int,Some(next)),"Begin group token expected after \\message")
        }
        let ret = int.read_token_list(true,false,false,true)?;
        let string = int.tokens_to_string(&ret);
        print!("{}",Yellow.paint(string.to_string()));
        Ok(())
    }
};

pub static NOEXPAND: PrimitiveExecutable = PrimitiveExecutable {
    name:"noexpand",
    expandable:true,
    _apply:|_cs,int| {
        int.assert_has_next()?;
        let next = int.next_token();
        int.requeue(next.deexpand());
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
                let cmd = match int.state_get_command(&next.cmdname()) {
                    None => {
                        rf.2.push(tmp);
                        rf.2.push(next);
                        return Ok(())
                    }
                    Some(p) => p
                };
                if cmd.expandable(true) {
                    match cmd.get_expansion(next, int)? {
                        Some(e) => rf.2 = e.2,
                        None => ()
                    }
                    rf.2.insert(0, tmp);
                    Ok(())
                } else {
                    rf.2.push(tmp);
                    rf.2.push(next);
                    Ok(())
                }
            }
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
            _ => PrimitiveTeXCommand::Char(next).as_ref(rf.get_ref()).meaning(&int.state_catcodes())
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
                let s : TeXString = if int.state_catcodes().escapechar == 255 {"".into()} else {int.state_catcodes().escapechar.into()};
                crate::interpreter::string_to_tokens(s + next.cmdname().into())
            }
            CategoryCode::Space => vec!(next),
            _ => vec!(Token::new(next.char,CategoryCode::Other,Some(next.name().clone()),next.reference.deref().clone(),true))
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
        let cmd = PrimitiveTeXCommand::MathChar(num as u32).as_ref(rf);
        int.change_state(StateChange::Cs(chartok.cmdname().clone(),Some(cmd),
            global));
        Ok(())
    }
};

pub fn csname(int : &Interpreter) -> Result<TeXString,TeXError> {
    let incs = int.newincs();
    let mut cmdname : TeXString = "".into();
    log!("\\csname: {}",int.preview());
    while incs == int.currcs() && int.has_next() {
        let next = int.next_token();
        match next.catcode {
            CategoryCode::Escape | CategoryCode::Active => {
                let cmd = int.get_command(&next.cmdname())?;
                match *cmd.orig {
                    PrimitiveTeXCommand::Primitive(ec) if *ec == ENDCSNAME => {
                        int.popcs()?
                    }
                    PrimitiveTeXCommand::Primitive(ec) if *ec == CSNAME => {
                        cmd.expand(next,int)?;
                    }
                    _ if next.expand && cmd.expandable(true) =>
                        cmd.expand(next,int)?,
                    _ if next.catcode == CategoryCode::Escape => {
                        if int.state_catcodes().escapechar != 255 {
                            cmdname += int.state_catcodes().escapechar
                        }
                        cmdname += next.name()
                    }
                    _ => cmdname += next.char
                }
            }
            _ => cmdname += next.char
        }
    }
    log!("\\csname return: {}",cmdname);
    return Ok(cmdname)
}

pub static CSNAME: PrimitiveExecutable = PrimitiveExecutable {
    name:"csname",
    expandable:true,
    _apply:|rf,int| {
        let cmdname : TeXStr = csname(int)?.into();
        let ret = Token::new(int.state_catcodes().escapechar,CategoryCode::Escape,Some(cmdname.clone()),SourceReference::None,true);
        match int.state_get_command(&cmdname) {
            Some(_) => (),
            None => {
                let cmd = PrimitiveTeXCommand::Primitive(&RELAX).as_ref(rf.get_ref());
                int.change_state(StateChange::Cs(cmdname,Some(cmd),false))
            }
        }
        rf.2.push(ret);
        Ok(())
    }
};

pub static ENDCSNAME: PrimitiveExecutable = PrimitiveExecutable {
    name:"endcsname",
    expandable:false,
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
        println!("Error: {}",int.preview());
        let next = int.next_token();
        if next.catcode != CategoryCode::BeginGroup {
            TeXErr!((int,Some(next)),"Begin group token expected after \\message")
        }
        let ret = int.read_token_list(true,false,false,true)?;
        let string = int.tokens_to_string(&ret);
        let mut eh = int.state_tokens(-(ERRHELP.index as i32));
        println!("Errhelp: {}",TokenList(&eh));
        let rethelp = if !eh.is_empty() {
            eh.push(Token::new(0,CategoryCode::EndGroup,None,SourceReference::None,false));
            //eh.insert(0,Token::new(0,CategoryCode::BeginGroup,None,SourceReference::None,false));
            int.push_tokens(eh);
            let rethelp = int.read_token_list(true,false,false,true)?;
            int.tokens_to_string(&rethelp)
        } else {"".into()};
        TeXErr!((int,None),"{}\n{}",Red.bold().paint(string.to_string()),rethelp)
    }
};

pub static ETEXREVISION : PrimitiveExecutable = PrimitiveExecutable {
    expandable:true,
    _apply: |rf, _int| {
        rf.2 = crate::interpreter::string_to_tokens(VERSION_INFO.etexrevision.clone());
        Ok(())
    },
    name: "eTeXrevision"
};

pub static ETEXVERSION : NumericCommand = NumericCommand {
    _getvalue: |_int| {
        Ok(Numeric::Int(VERSION_INFO.etexversion.to_string().parse().unwrap()))
    },
    name: "eTeXversion"
};

fn expr_loop(int: &Interpreter,getnum : fn(&Interpreter) -> Result<Numeric,TeXError>,cont:Option<(Box<dyn Fn(Numeric) -> Numeric>,u8)>) -> Result<Numeric,TeXError> {
    int.skip_ws();
    let mut first = match int.read_keyword(vec!("("))? {
        Some(_) => {
            let ret = expr_loop(int, getnum,None)?;
            match int.read_keyword(vec!(")"))? {
                Some(_) => ret,
                None => TeXErr!((int,None),"Expected ) in expression")
            }
        }
        _ => (getnum)(int)?
    };
    match int.read_keyword(vec!("-","+","*","/"))? {
        Some(p) if p == "+" => {
            first = match cont {
                Some((f,_)) => (f.deref())(first),
                _ => first
            };
            let ncont : Box<dyn Fn(Numeric) -> Numeric> = Box::new(move |x:Numeric| {first + x});
            expr_loop(int,getnum,Some((ncont,1 as u8)))
        }
        Some(p) if p == "-" => {
            match cont {
                Some((f,i)) if i >= 2 => {
                    first = (f.deref())(first);
                    let ncont : Box<dyn Fn(Numeric) -> Numeric> = Box::new(move |x:Numeric| {first - x});
                    expr_loop(int,getnum,Some((ncont,2 as u8)))
                }
                Some((f,i)) => {
                    let ncont : Box<dyn Fn(Numeric) -> Numeric> = Box::new(move |x:Numeric| {(f.deref())(first - x)});
                    expr_loop(int,getnum,Some((ncont,i as u8)))
                }
                None => {
                    let ncont : Box<dyn Fn(Numeric) -> Numeric> = Box::new(move |x:Numeric| {first - x});
                    expr_loop(int,getnum,Some((ncont,2)))
                }
            }
        }
        Some(p) if p == "*" => {
            match cont {
                Some((f,i)) if i >= 3 => {
                    first = (f.deref())(first);
                    let ncont : Box<dyn Fn(Numeric) -> Numeric> = Box::new(move |x:Numeric| {first * x});
                    expr_loop(int,getnum,Some((ncont,3 as u8)))
                }
                Some((f,i)) => {
                    let ncont : Box<dyn Fn(Numeric) -> Numeric> = Box::new(move |x:Numeric| {(f.deref())(first * x)});
                    expr_loop(int,getnum,Some((ncont,i as u8)))
                }
                None => {
                    let ncont : Box<dyn Fn(Numeric) -> Numeric> = Box::new(move |x:Numeric| {first * x});
                    expr_loop(int,getnum,Some((ncont,3)))
                }
            }
        }
        Some(p) if p == "/" => {
            match cont {
                Some((f,i)) => {
                    let ncont : Box<dyn Fn(Numeric) -> Numeric> = Box::new(move |x:Numeric| {(f.deref())(first / x)});
                    expr_loop(int,getnum,Some((ncont,i as u8)))
                }
                None => {
                    let ncont : Box<dyn Fn(Numeric) -> Numeric> = Box::new(move |x:Numeric| {first / x});
                    expr_loop(int,getnum,Some((ncont,3)))
                }
            }
        }
        //Some(p) if p == "/" => {} //Ok(first / int.read_number_i(true)?),
        //Some(p) if p == "*" => {} //Ok(first * int.read_number_i(true)?),
        _ => match cont {
            Some((f,_)) => {
                Ok((f.deref())(first))
            }
            None => Ok(first)
        } //Ok(first)
    }

}

/*
fn expr_loop(int : &Interpreter,getnum : fn(&Interpreter) -> Result<Numeric,TeXError>) -> Result<Numeric,TeXError> {
    int.skip_ws();
    let first = match int.read_keyword(vec!("("))? {
        Some(_) => {
            let ret = expr_loop(int, getnum)?;
            match int.read_keyword(vec!(")"))? {
                Some(_) => ret,
                None => TeXErr!((int,None),"Expected ) in expression")
            }
        }
        _ => (getnum)(int)?
    };
    match int.read_keyword(vec!("-","+","*","/"))? {
        Some(p) if p == "+" => Ok(first + expr_loop(int,getnum)?),
        Some(p) if p == "*" => Ok(first * int.read_number_i(true)?),
        Some(p) if p == "/" => Ok(first / int.read_number_i(true)?),
        Some(p) if p == "-" => Ok(first - expr_loop(int,getnum)?),
        Some(o) => TeXErr!((int,None),"TODO: {}",o),
        None => Ok(first)
    }
}

 */

fn eatrelax(int : &Interpreter) {
    if int.has_next() {
        let next = int.next_token();
        match next.catcode {
            CategoryCode::Escape | CategoryCode::Active => {
                match int.state_get_command(&next.cmdname()).map(|x| x.orig) {
                    Some(p)  => match &*p {
                        PrimitiveTeXCommand::Primitive(r) if **r == RELAX => (),
                        _ => int.requeue(next)
                    }
                    _ => int.requeue(next)
                }
            }
            _ => {
                int.requeue(next)
            }
        }
    }
}

pub static NUMEXPR: NumericCommand = NumericCommand {
    name:"numexpr",
    _getvalue: |int| {
        log!("\\numexpr starts: >{}",int.preview());
        let ret =expr_loop(int,|i| i.read_number_i(false),None)?;
        eatrelax(int);
        log!("\\numexpr: {}",ret);
        Ok(ret)
    }
};

pub static DIMEXPR: NumericCommand = NumericCommand {
    name:"dimexpr",
    _getvalue: |int| {
        log!("\\dimexpr starts: >{}",int.preview());
        let ret =expr_loop(int,|i| Ok(Numeric::Dim(i.read_dimension()?)),None)?;
        eatrelax(int);
        log!("\\dimexpr: {}",ret);
        Ok(ret)
    }
};

pub static GLUEEXPR: NumericCommand = NumericCommand {
    name:"glueexpr",
    _getvalue: |int| {
        log!("\\glueexpr starts: >{}",int.preview());
        let ret =expr_loop(int,|i| Ok(Numeric::Skip(i.read_skip()?)),None)?;
        eatrelax(int);
        log!("\\glueexpr: {}",ret);
        Ok(ret)
    }
};

pub static MUEXPR: NumericCommand = NumericCommand {
    name:"muexpr",
    _getvalue: |int| {
        log!("\\muexpr starts: >{}",int.preview());
        let ret =expr_loop(int,|i| Ok(Numeric::MuSkip(i.read_muskip()?)),None)?;
        eatrelax(int);
        log!("\\muexpr: {}",ret);
        Ok(ret)
    }
};


pub static UNEXPANDED: PrimitiveExecutable = PrimitiveExecutable {
    name:"unexpanded",
    expandable:true,
    _apply:|exp,int| {
        exp.2 = int.read_balanced_argument(false,false,false,true)?;
        Ok(())
    }
};

pub static ROMANNUMERAL: PrimitiveExecutable = PrimitiveExecutable {
    name:"romannumeral",
    expandable:true,
    _apply:|rf,int| {
        log!("\\romannumeral: {}",int.preview());
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

pub static DETOKENIZE: PrimitiveExecutable = PrimitiveExecutable {
    name:"detokenize",
    expandable:true,
    _apply:|exp,int| {
        let tkl = int.read_balanced_argument(false,false,false,true)?;
        let space = Token::new(32,CategoryCode::Space,None,SourceReference::None,false);
        let escape = match int.state_catcodes().escapechar {
            255 => None,
            _ => Some(Token::new(int.state_catcodes().escapechar, CategoryCode::Other, None, SourceReference::None, false))
        };
        for t in tkl {
            match t.catcode {
                CategoryCode::Space | CategoryCode::EOL => exp.2.push(space.clone()),
                CategoryCode::Escape => {
                    for tk in &escape { exp.2.push(tk.clone()) }
                    for t in t.name().iter() {
                        exp.2.push(Token::new(*t,CategoryCode::Other,None,SourceReference::None,false));
                    }
                    if t.name().len() > 1 { exp.2.push(space.clone()) }
                    else if t.name().len() == 1 {
                        let c = t.name().iter().first().unwrap();
                        match int.state_catcodes().get_code(*c) {
                            CategoryCode::Letter => exp.2.push(space.clone()),
                            _ => ()
                        }
                    }
                }
                _ => {
                    exp.2.push(Token::new(t.char,CategoryCode::Other,None,SourceReference::None,false));
                }
            }
        }
        log!("\\detokenize: {}",TokenList(&exp.2));
        Ok(())
    }
};

pub static LCCODE: NumAssValue = NumAssValue {
    name:"lccode",
    _assign: |_,int,global| {
        let num1 = int.read_number()? as u8;
        int.read_eq();
        let num2 = int.read_number()? as u8;
        int.change_state(StateChange::Lccode(num1,num2,global));
        Ok(())
    },
    _getvalue: |int| {
        let char = int.read_number()? as u8;
        Ok(Numeric::Int(int.state_lccode(char) as i64))
    }
};

pub static UCCODE: NumAssValue = NumAssValue {
    name: "uccode",
    _assign: |_, int, global| {
        let num1 = int.read_number()? as u8;
        int.read_eq();
        let num2 = int.read_number()? as u8;
        int.change_state(StateChange::Uccode(num1, num2, global));
        Ok(())
    },
    _getvalue: |int| {
        let char = int.read_number()? as u8;
        Ok(Numeric::Int(int.state_uccode(char) as i64))
    }
};

pub static LOWERCASE: PrimitiveExecutable = PrimitiveExecutable {
    name:"lowercase",
    expandable:false,
    _apply:|rf,int| {
        let erf = rf.get_ref();
        for t in int.read_balanced_argument(false,false,false,true)? {
            match t.catcode {
                CategoryCode::Escape => rf.2.push(t.copied(erf.clone())),
                o => {
                    let lc = int.state_lccode(t.char);
                    rf.2.push(Token::new(lc,o,None,SourceReference::Exp(erf.clone()),true))
                }
            }
        }
        Ok(())
    }
};

pub static UPPERCASE: PrimitiveExecutable = PrimitiveExecutable {
    name:"uppercase",
    expandable:false,
    _apply:|rf,int| {
        let erf = rf.get_ref();
        for t in int.read_balanced_argument(false,false,false,true)? {
            match t.catcode {
                CategoryCode::Escape => rf.2.push(t.copied(erf.clone())),
                o => {
                    let uc = int.state_uccode(t.char);
                    rf.2.push(Token::new(uc,o,None,SourceReference::Exp(erf.clone()),true))
                }
            }
        }
        Ok(())
    }
};

pub static FONT: FontAssValue = FontAssValue {
    name:"font",
    _assign: |_rf,int,global| {
        let cmd = int.read_command_token()?;
        int.read_eq();
        let mut name = int.read_string()?;
        if !name.ends_with(".tfm") {name += ".tfm"}
        let ff = int.state_get_font(&name)?;
        let at = match int.read_keyword(vec!("at","scaled"))? {
            Some(s) if s == "at" => Some(int.read_dimension()?),
            Some(s) if s == "scaled" => Some(((ff.as_ref().size as f64) * match int.read_number_i(true)? {
                Numeric::Float(f) => f,
                Numeric::Dim(i) => (i as f64) / 65536.0,
                _ => todo!()
            }).round() as i64),
            _ => None
        };
        let font = Font::new(ff,at,cmd.cmdname().clone());
        int.change_state(StateChange::Cs(cmd.cmdname().clone(),Some(PrimitiveTeXCommand::AV(AssignableValue::FontRef(font)).as_command()),global));
        Ok(())
    },
    _getvalue: |_int| {
        todo!()
    }
};

pub static TEXTFONT: FontAssValue = FontAssValue {
    name:"textfont",
    _assign: |_rf,int,global| {
        let ind = int.read_number()?;
        if ind < 0 || ind > 15 {
            TeXErr!((int,None),"\\textfont expected 0 <= n <= 15; got: {}",ind)
        }
        let f = read_font(int)?;
        int.change_state(StateChange::Textfont(ind as usize,f,global));
        Ok(())
    },
    _getvalue: |int| {
        let ind = int.read_number()?;
        if ind < 0 || ind > 15 {
            TeXErr!((int,None),"\\textfont expected 0 <= n <= 15; got: {}",ind)
        }
        Ok(int.state.borrow().getTextFont(ind as u8))
    }
};

pub static SCRIPTFONT: FontAssValue = FontAssValue {
    name:"scriptfont",
    _assign: |_rf,int,global| {
        let ind = int.read_number()?;
        if ind < 0 || ind > 15 {
            TeXErr!((int,None),"\\scriptfont expected 0 <= n <= 15; got: {}",ind)
        }
        let f = read_font(int)?;
        int.change_state(StateChange::Scriptfont(ind as usize,f,global));
        Ok(())
    },
    _getvalue: |int| {
        let ind = int.read_number()?;
        if ind < 0 || ind > 15 {
            TeXErr!((int,None),"\\scriptfont expected 0 <= n <= 15; got: {}",ind)
        }
        Ok(int.state.borrow().getScriptFont(ind as u8))
    }
};
pub static SCRIPTSCRIPTFONT: FontAssValue = FontAssValue {
    name:"scriptscriptfont",
    _assign: |_rf,int,global| {
        let ind = int.read_number()?;
        if ind < 0 || ind > 15 {
            TeXErr!((int,None),"\\scriptscriptfont expected 0 <= n <= 15; got: {}",ind)
        }
        let f = read_font(int)?;
        int.change_state(StateChange::Scriptscriptfont(ind as usize,f,global));
        Ok(())
    },
    _getvalue: |int| {
        let ind = int.read_number()?;
        if ind < 0 || ind > 15 {
            TeXErr!((int,None),"\\scriptscriptfont expected 0 <= n <= 15; got: {}",ind)
        }
        Ok(int.state.borrow().getScriptScriptFont(ind as u8))
    }
};


pub fn read_font<'a>(int : &Interpreter) -> Result<Rc<Font>,TeXError> {
    int.expand_until(true)?;
    let tk = int.read_command_token()?;
    let cmd = int.get_command(tk.cmdname())?;
    match &*cmd.orig {
        PrimitiveTeXCommand::AV(AssignableValue::FontRef(f)) =>
            Ok(f.clone()),
        PrimitiveTeXCommand::AV(AssignableValue::Font(_)) => Ok(int.get_font()),
        PrimitiveTeXCommand::Ass(p) if **p == NULLFONT =>
        Ok(Nullfont.try_with(|x| x.clone()).unwrap()),
        _ => TeXErr!((int, Some(tk)),"Font expected!")
    }
}

pub static FONTDIMEN: NumAssValue = NumAssValue {
    name:"fontdimen",
    _assign: |_rf,int,_global| {
        let i = int.read_number()? as u16;
        let f = read_font(int)?;
        int.read_eq();
        let d = int.read_dimension()?;
        f.set_dimen(i,d);
        Ok(())
    },
    _getvalue: |int| {
        let i = int.read_number()? as u16;
        let f = read_font(int)?;
        Ok(Numeric::Dim(f.get_dimen(i)))
    }
};

pub static LPCODE: NumAssValue = NumAssValue {
    name:"lpcode",
    _assign: |_rf,int,_global| {
        let f = read_font(int)?;
        let i = int.read_number()? as u16;
        int.read_eq();
        let d = int.read_number()? as u8;
        f.set_lp(i,d);
        Ok(())
    },
    _getvalue: |int| {
        let f = read_font(int)?;
        let i = int.read_number()? as u16;
        Ok(Numeric::Int(f.get_lp(i)))
    }
};

pub static RPCODE: NumAssValue = NumAssValue {
    name:"rpcode",
    _assign: |_rf,int,_global| {
        let f = read_font(int)?;
        let i = int.read_number()? as u16;
        int.read_eq();
        let d = int.read_number()? as u8;
        f.set_rp(i,d);
        Ok(())
    },
    _getvalue: |int| {
        let f = read_font(int)?;
        let i = int.read_number()? as u16;
        Ok(Numeric::Int(f.get_rp(i)))
    }
};

pub static HYPHENCHAR: NumAssValue = NumAssValue {
    name:"hyphenchar",
    _assign: |_rf,int,_global| {
        let f = read_font(int)?;
        int.read_eq();
        let d = int.read_number()?;
        f.inner.borrow_mut().hyphenchar = d as u16;
        Ok(())
    },
    _getvalue: |int| {
        let f = read_font(int)?;
        let x = f.inner.borrow().hyphenchar as i64;
        Ok(Numeric::Int(x))
    }
};

pub static SKEWCHAR: NumAssValue = NumAssValue {
    name:"skewchar",
    _assign: |_rf,int,_global| {
        let f = read_font(int)?;
        int.read_eq();
        let d = int.read_number()?;
        f.inner.borrow_mut().skewchar = d as u16;
        Ok(())
    },
    _getvalue: |int| {
        let f = read_font(int)?;
        let x = f.inner.borrow().skewchar as i64;
        Ok(Numeric::Int(x))
    }
};

pub static EXPANDED: PrimitiveExecutable = PrimitiveExecutable {
    name:"expanded",
    expandable:true,
    _apply:|rf,int| {
        rf.2 = int.read_balanced_argument(true,true,false,true)?.iter().map(|x| x.cloned()).collect();
        Ok(())
    }
};

pub static INPUTLINENO: NumericCommand = NumericCommand {
    _getvalue: |int| {
        Ok(Numeric::Int(int.line_no() as i64))
    },
    name:"inputlineno",
};

pub static LASTSKIP: NumericCommand = NumericCommand {
    name:"lastskip",
    _getvalue: |int| {
        match int.stomach.borrow().last_whatsit() {
            Some(Whatsit::Simple(SimpleWI::VSkip(s,_))) => Ok(Numeric::Skip(s)),
            Some(Whatsit::Simple(SimpleWI::HSkip(s,_))) => Ok(Numeric::Skip(s)),
            _ => Ok(Numeric::Skip(Skip {
                base:0,stretch:None,shrink:None
            }))
        }
    },
};

pub static SETBOX: PrimitiveAssignment = PrimitiveAssignment {
    name:"setbox",
    _assign: |_rf,int,global| {
        let index = int.read_number()? as u16;
        int.read_eq();
        int.state.borrow_mut().insetbox = true;
        let wi = int.read_box()?;
        int.change_state(StateChange::Box(index as i32,wi,global));
        Ok(())
    }
};

pub static HBOX: ProvidesBox = ProvidesBox {
    name:"hbox",
    _get: |tk,int| {
        let (spread,width) = match int.read_keyword(vec!("to","spread"))? {
            Some(s) if s == "to" => (0 as i64,Some(int.read_dimension()?)),
            Some(s) if s == "spread" => (int.read_dimension()?,None),
            _ => (0 as i64,None)
        };
        let ret = int.read_whatsit_group(BoxMode::H,true)?;
        /*if ret.is_empty() {Ok(TeXBox::Void)} else*/ {
            Ok(TeXBox::H(HBox {
                children: ret,
                spread,
                _width: width,
                _height: None,
                _depth: None,
                rf : int.update_reference(tk)
            }))
        }
    }
};

pub static VBOX: ProvidesBox = ProvidesBox {
    name:"vbox",
    _get: |tk,int| {
        let (spread,height) = match int.read_keyword(vec!("to","spread"))? {
            Some(s) if s == "to" => (0 as i64,Some(int.read_dimension()?)),
            Some(s) if s == "spread" => (int.read_dimension()?,None),
            _ => (0 as i64,None)
        };
        let ret = int.read_whatsit_group(BoxMode::V,true)?;
        /*if ret.is_empty() {Ok(TeXBox::Void)} else*/ {
            Ok(TeXBox::V(VBox {
                children: ret,
                center:false,
                spread,
                _width: None,
                _height: height,
                _depth: None,
                rf : int.update_reference(tk)
            }))
        }
    }
};

pub static VCENTER: ProvidesBox = ProvidesBox {
    name:"vcenter",
    _get: |tk,int| {
        let bx = (VBOX._get)(tk,int)?;
        match bx {
            TeXBox::V(mut vb) => {
                vb.center = true;
                Ok(TeXBox::V(vb))
            }
            _ => unreachable!()
        }
    }
};

pub static LASTBOX: ProvidesBox = ProvidesBox {
    _get: |_tk,int| {
        match int.stomach.borrow().last_whatsit() {
            Some(Whatsit::Box(tb)) => Ok(tb),
            _ => Ok(TeXBox::Void)
        }
    },
    name:"lastbox",
};

pub static UNSKIP: PrimitiveExecutable = PrimitiveExecutable {
    name:"unskip",
    expandable:false,
    _apply:|_tk,int| {
        let lw = int.stomach.borrow().last_whatsit();
        match lw {
            Some(Whatsit::Simple(SimpleWI::HSkip(_,_) | SimpleWI::VSkip(_,_))) => {
                int.stomach.borrow_mut().drop_last()
            },
            _ => ()
        }
        Ok(())
    }
};

pub static COPY: ProvidesBox = ProvidesBox {
    name:"copy",
    _get: |_tk,int| {
        let ind = int.read_number()?;
        Ok(int.state_copy_box(ind as i32))
    }
};

pub static BOX: ProvidesBox = ProvidesBox {
    name:"box",
    _get: |_tk,int| {
        let ind = int.read_number()?;
        Ok(int.state_get_box(ind as i32))
    }
};

pub static AFTERASSIGNMENT: PrimitiveExecutable = PrimitiveExecutable {
    name:"afterassignment",
    expandable:false,
    _apply:|_tk,int| {
        let next = int.next_token();
        int.state_set_afterassignment(next);
        Ok(())
    }
};

pub static ENDINPUT: PrimitiveExecutable = PrimitiveExecutable {
    name:"endinput",
    expandable:true,
    _apply:|_tk,int| {
        int.end_input();
        Ok(())
    }
};

pub static TOKS: TokAssValue = TokAssValue {
    name:"toks",
    _assign: |_rf,int,global| {
        let num = int.read_number()? as u16;
        int.read_eq();
        let r = int.read_balanced_argument(false,false,false,true)?;
        int.change_state(StateChange::Tokens(num as i32,r.iter().map(|x| x.cloned()).collect(),global));
        Ok(())
    },
    _getvalue: |int| {
        let num = int.read_number()? as u16;
        Ok(int.state_tokens(num as i32))
    }
};

pub static MATHCODE: NumAssValue = NumAssValue {
    name:"mathcode",
    _getvalue: |int| {Ok(Numeric::Int(int.state_get_mathcode( int.read_number()? as u8)))},
    _assign: |rf,int,global| {
        let i = int.read_number()? as u8;
        int.read_eq();
        let v = int.read_number()?;
        int.change_state(StateChange::Mathcode(i,v,global));
        Ok(())
    }
};

pub static DELCODE: NumAssValue = NumAssValue {
    name:"delcode",
    _getvalue: |int| {Ok(Numeric::Int(int.state_get_delcode( int.read_number()? as u8)))},
    _assign: |rf,int,global| {
        let i = int.read_number()? as u8;
        int.read_eq();
        let v = int.read_number()?;
        int.change_state(StateChange::Delcode(i,v,global));
        Ok(())
    }
};

pub static NULLFONT: PrimitiveAssignment = PrimitiveAssignment {
    name:"nullfont",
    _assign: |rf,int,global| {
        int.change_state(StateChange::Font(Nullfont.try_with(|x| x.clone()).unwrap(),global));
        int.stomach.borrow_mut().add(int,Whatsit::GroupOpen(
            WIGroup::FontChange(Nullfont.try_with(|x| x.clone()).unwrap(),int.update_reference(&rf.0),global,vec!())
        ))
    }
};

pub static JOBNAME: PrimitiveExecutable = PrimitiveExecutable {
    name:"jobname",
    expandable:true,
    _apply:|rf,int| {
        let jobname = int.jobinfo.path.file_stem().unwrap().to_str().unwrap();
        rf.2 = crate::interpreter::string_to_tokens(jobname.into());
        Ok(())
    }
};

pub static PATTERNS: PrimitiveExecutable = PrimitiveExecutable {
    name:"patterns",
    expandable:false,
    _apply:|_tk,int| {
        int.read_argument();
        Ok(())
        // TODO ?
    }
};

pub static HYPHENATION: PrimitiveExecutable = PrimitiveExecutable {
    name:"hyphenation",
    expandable:false,
    _apply:|_tk,int| {
        int.read_argument();
        Ok(())
        // TODO ?
    }
};

pub static IGNORESPACES: PrimitiveExecutable = PrimitiveExecutable {
    name:"ignorespaces",
    expandable:false,
    _apply:|_tk,int| {
        while int.has_next() {
            let next = int.next_token();
            match next.catcode {
                CategoryCode::Space | CategoryCode::EOL => (),
                CategoryCode::Escape | CategoryCode::Active => {
                    let p = int.state_get_command(next.cmdname());
                    match p {
                        Some(p) if p.expandable(true) => {
                            p.expand(next,int)?;
                        }
                        _ => {
                            int.requeue(next);
                            return Ok(())
                        }
                    }
                }
                _ => {
                    int.requeue(next);
                    return Ok(())
                }
            }
        }
        Ok(())
    }
};

pub static ERRORSTOPMODE: PrimitiveExecutable = PrimitiveExecutable {
    name:"errorstopmode",
    expandable:false,
    _apply:|_tk,_int| { Ok(()) }
};

pub static DUMP: PrimitiveExecutable = PrimitiveExecutable {
    name:"dump",
    expandable:true,
    _apply:|_tk,_int| { Ok(()) }
};

pub static VRULE: SimpleWhatsit = SimpleWhatsit {
    name:"vrule",
    modes:|m| match m {
        TeXMode::Horizontal | TeXMode::RestrictedHorizontal => true,
        _ => false
    },
    _get: |tk,int| {
        let mut height : Option<i64> = None;
        let mut width : Option<i64> = None;
        let mut depth : Option<i64> = None;
        loop {
            match int.read_keyword(vec!("height","width","depth"))? {
                Some(s) if s == "height" => height = Some(int.read_dimension()?),
                Some(s) if s == "width" => width = Some(int.read_dimension()?),
                Some(s) if s == "depth" => depth = Some(int.read_dimension()?),
                _ => break
            }
        }
        let rf = int.update_reference(tk);
        Ok(Whatsit::Simple(SimpleWI::VRule(rf,height,width,depth)))
    }
};

pub static HRULE: SimpleWhatsit = SimpleWhatsit {
    name:"hrule",
    modes:|m| match m {
        TeXMode::Vertical | TeXMode::InternalVertical => true,
        _ => false
    },
    _get: |tk,int| {
        let mut height : Option<i64> = None;
        let mut width : Option<i64> = None;
        let mut depth : Option<i64> = None;
        loop {
            match int.read_keyword(vec!("height","width","depth"))? {
                Some(s) if s == "height" => height = Some(int.read_dimension()?),
                Some(s) if s == "width" => width = Some(int.read_dimension()?),
                Some(s) if s == "depth" => depth = Some(int.read_dimension()?),
                _ => break
            }
        }
        let rf = int.update_reference(tk);
        Ok(Whatsit::Simple(SimpleWI::HRule(rf,height,width,depth)))
    }
};


pub static VFIL: SimpleWhatsit = SimpleWhatsit {
    name:"vfil",
    modes:|m| match m {
        TeXMode::Vertical | TeXMode::InternalVertical => true,
        _ => false
    },
    _get: |tk,int| {
        Ok(Whatsit::Simple(SimpleWI::VFil(int.update_reference(tk))))
    }
};

pub static VFILL: SimpleWhatsit = SimpleWhatsit {
    name:"vfill",
    modes:|m| match m {
        TeXMode::Vertical | TeXMode::InternalVertical => true,
        _ => false
    },
    _get: |tk,int| {
        Ok(Whatsit::Simple(SimpleWI::VFill(int.update_reference(tk))))
    }
};

pub static VSKIP: SimpleWhatsit = SimpleWhatsit {
    name:"vskip",
    modes:|m| match m {
        TeXMode::Vertical | TeXMode::InternalVertical => true,
        _ => false
    },
    _get: |tk,int| {
        let sk = int.read_skip()?;
        Ok(Whatsit::Simple(SimpleWI::VSkip(sk,int.update_reference(tk))))
    }
};

pub static HSKIP: SimpleWhatsit = SimpleWhatsit {
    name:"hskip",
    modes:|m| match m {
        TeXMode::Horizontal | TeXMode::RestrictedHorizontal => true,
        _ => false
    },
    _get: |tk,int| {
        let sk = int.read_skip()?;
        Ok(Whatsit::Simple(SimpleWI::HSkip(sk,int.update_reference(tk))))
    }
};

pub static HFIL: SimpleWhatsit = SimpleWhatsit {
    name:"hfil",
    modes:|m| match m {
        TeXMode::Horizontal | TeXMode::RestrictedHorizontal => true,
        _ => false
    },
    _get: |tk,int| {
        Ok(Whatsit::Simple(SimpleWI::HFil(int.update_reference(tk))))
    }
};

pub static HFILL: SimpleWhatsit = SimpleWhatsit {
    name:"hfill",
    modes:|m| match m {
        TeXMode::Horizontal | TeXMode::RestrictedHorizontal => true,
        _ => false
    },
    _get: |tk,int| {
        Ok(Whatsit::Simple(SimpleWI::HFill(int.update_reference(tk))))
    }
};

pub static PENALTY: SimpleWhatsit = SimpleWhatsit {
    name:"penalty",
    modes:|_| true,
    _get: |tk,int| {
        Ok(Whatsit::Simple(SimpleWI::Penalty(int.read_number()?)))
    }
};

pub static LOWER: SimpleWhatsit = SimpleWhatsit {
    name:"lower",
    modes:|m| {match m {
        TeXMode::Horizontal | TeXMode::RestrictedHorizontal => true,
        _ => false
    }},
    _get: |tk,int| {
        let dim = int.read_dimension()?;
        let bx = int.read_box()?;
        let rf = int.update_reference(tk);
        Ok(Whatsit::Simple(SimpleWI::Raise(-dim,bx,rf)))
    }
};

pub static RAISE: SimpleWhatsit = SimpleWhatsit {
    name:"raise",
    modes:|m| {match m {
        TeXMode::Horizontal | TeXMode::RestrictedHorizontal => true,
        _ => false
    }},
    _get: |tk,int| {
        let dim = int.read_dimension()?;
        let bx = int.read_box()?;
        let rf = int.update_reference(tk);
        Ok(Whatsit::Simple(SimpleWI::Raise(dim,bx,rf)))
    }
};

pub static KERN: SimpleWhatsit = SimpleWhatsit {
    name:"kern",
    modes:|m| { true },
    _get: |tk,int| {
        let dim = int.read_dimension()?;
        let rf = int.update_reference(tk);
        match int.get_mode() {
            TeXMode::Vertical | TeXMode::InternalVertical =>
                Ok(Whatsit::Simple(SimpleWI::VKern(dim,rf))),
            _ =>
                Ok(Whatsit::Simple(SimpleWI::HKern(dim,rf)))
        }
    }
};

pub static UNVBOX: SimpleWhatsit = SimpleWhatsit {
    name:"unvbox",
    modes:|m| { m == TeXMode::Vertical || m == TeXMode::InternalVertical },
    _get: |tk,int| {
        let ind = int.read_number()?;
        let bx = int.state_get_box(ind as i32);
        match bx {
            TeXBox::V(v) => Ok(Whatsit::Ls(v.children)),
            TeXBox::Void => Ok(Whatsit::Ls(vec!())),
            _ => TeXErr!((int,None),"incompatible list can't be unboxed")
        }
    }
};

pub static UNVCOPY: SimpleWhatsit = SimpleWhatsit {
    name:"unvcopy",
    modes:|m| { m == TeXMode::Vertical || m == TeXMode::InternalVertical },
    _get: |tk,int| {
        let ind = int.read_number()?;
        let bx = int.state_copy_box(ind as i32);
        match bx {
            TeXBox::V(v) => Ok(Whatsit::Ls(v.children)),
            TeXBox::Void => Ok(Whatsit::Ls(vec!())),
            _ => TeXErr!((int,None),"incompatible list can't be unboxed")
        }
    }
};

pub static UNHBOX: SimpleWhatsit = SimpleWhatsit {
    name:"unhbox",
    modes:|m| { m == TeXMode::Horizontal || m == TeXMode::RestrictedHorizontal || m == TeXMode::Math || m == TeXMode::Displaymath },
    _get: |tk,int| {
        let ind = int.read_number()?;
        let bx = int.state_get_box(ind as i32);
        let mode = int.get_mode();
        match (bx,mode) {
            (TeXBox::H(h),TeXMode::Horizontal | TeXMode::RestrictedHorizontal) => Ok(Whatsit::Ls(h.children)),
            (TeXBox::Void,_) => Ok(Whatsit::Ls(vec!())),
            _ => TeXErr!((int,None),"incompatible list can't be unboxed")
        }
    }
};

pub static UNHCOPY: SimpleWhatsit = SimpleWhatsit {
    name:"unhcopy",
    modes:|m| { m == TeXMode::Horizontal || m == TeXMode::RestrictedHorizontal || m == TeXMode::Math || m == TeXMode::Displaymath },
    _get: |tk,int| {
        let ind = int.read_number()?;
        let bx = int.state_copy_box(ind as i32);
        let mode = int.get_mode();
        match (bx,mode) {
            (TeXBox::H(h),TeXMode::Horizontal | TeXMode::RestrictedHorizontal) => Ok(Whatsit::Ls(h.children)),
            (TeXBox::Void,_) => Ok(Whatsit::Ls(vec!())),
            _ => TeXErr!((int,None),"incompatible list can't be unboxed")
        }
    }
};

pub static AFTERGROUP: PrimitiveExecutable = PrimitiveExecutable {
    name:"aftergroup",
    expandable:false,
    _apply:|_,int| {
        let next = int.next_token();
        int.change_state(StateChange::Aftergroup(next));
        Ok(())
    }
};

pub static TEXTSTYLE: PrimitiveExecutable = PrimitiveExecutable {
    name:"textstyle",
    expandable:false,
    _apply:|_,int| {
        int.change_state(StateChange::Fontstyle(FontStyle::Text));
        Ok(())
    }
};

pub static SCRIPTSTYLE: PrimitiveExecutable = PrimitiveExecutable {
    name:"scriptstyle",
    expandable:false,
    _apply:|_,int| {
        int.change_state(StateChange::Fontstyle(FontStyle::Script));
        Ok(())
    }
};

pub static SCRIPTSCRIPTSTYLE: PrimitiveExecutable = PrimitiveExecutable {
    name:"scriptscriptstyle",
    expandable:false,
    _apply:|_,int| {
        int.change_state(StateChange::Fontstyle(FontStyle::Scriptscript));
        Ok(())
    }
};

pub static SCANTOKENS: PrimitiveExecutable = PrimitiveExecutable {
    name:"scantokens",
    expandable:false,
    _apply:|tk,int| {
        let tks = int.read_balanced_argument(false,false,false,true)?;
        let str = int.tokens_to_string(&tks);
        int.push_string(tk.clone(),str);
        Ok(())
    }
};

pub static WD: NumAssValue = NumAssValue {
    name:"wd",
    _assign: |rf,int,global| {
        let index = int.read_number()? as i32;
        int.read_eq();
        let dim = int.read_dimension()?;
        let mut bx = int.state_get_box(index);
        match bx {
            TeXBox::Void => (),
            TeXBox::H(ref mut hb) => hb._width = Some(dim),
            TeXBox::V(ref mut hb) => hb._width = Some(dim),
        }
        int.change_state(StateChange::Box(index,bx,global));
        Ok(())
    },
    _getvalue: |int| {
        let index = int.read_number()?;
        Ok(Numeric::Dim(int.state_copy_box(index as i32).width()))
    }
};

pub static HT: NumAssValue = NumAssValue {
    name:"ht",
    _assign: |rf,int,global| {
        let index = int.read_number()? as i32;
        int.read_eq();
        let dim = int.read_dimension()?;
        let mut bx = int.state_get_box(index);
        match bx {
            TeXBox::Void => (),
            TeXBox::H(ref mut hb) => hb._height = Some(dim),
            TeXBox::V(ref mut hb) => hb._height = Some(dim),
        }
        int.change_state(StateChange::Box(index,bx,global));
        Ok(())
    },
    _getvalue: |int| {
        let index = int.read_number()?;
        Ok(Numeric::Dim(int.state_copy_box(index as i32).height()))
    }
};

pub static DP: NumAssValue = NumAssValue {
    name:"dp",
    _assign: |rf,int,global| {
        let index = int.read_number()? as i32;
        int.read_eq();
        let dim = int.read_dimension()?;
        let mut bx = int.state_get_box(index);
        match bx {
            TeXBox::Void => (),
            TeXBox::H(ref mut hb) => hb._depth = Some(dim),
            TeXBox::V(ref mut hb) => hb._depth = Some(dim),
        }
        int.change_state(StateChange::Box(index,bx,global));
        Ok(())
    },
    _getvalue: |int| {
        let index = int.read_number()?;
        Ok(Numeric::Dim(int.state_copy_box(index as i32).depth()))
    }
};

pub static FONTCHARWD: NumericCommand = NumericCommand {
    name:"fontcharwd",
    _getvalue: |int| {
        let font = read_font(int)?;
        let char = int.read_number()? as u16;
        Ok(Numeric::Dim(font.get_width(char)))
    }
};

pub static FONTCHARHT: NumericCommand = NumericCommand {
    name:"fontcharht",
    _getvalue: |int| {
        let font = read_font(int)?;
        let char = int.read_number()? as u16;
        Ok(Numeric::Dim(font.get_height(char)))
    }
};

pub static FONTCHARDP: NumericCommand = NumericCommand {
    name:"fontchardp",
    _getvalue: |int| {
        let font = read_font(int)?;
        let char = int.read_number()? as u16;
        Ok(Numeric::Dim(font.get_depth(char)))
    }
};

pub static FONTCHARIC: NumericCommand = NumericCommand {
    name:"fontchardp",
    _getvalue: |int| {
        let font = read_font(int)?;
        let char = int.read_number()? as u16;
        Ok(Numeric::Dim(font.get_ic(char)))
    }
};

pub static VADJUST: PrimitiveExecutable = PrimitiveExecutable {
    name:"vadjust",
    expandable:false,
    _apply:|_tk,int| {
        let mut ret = int.read_whatsit_group(BoxMode::V,false)?;
        int.state.borrow_mut().vadjust.append(&mut ret);
        Ok(())
    }
};

pub static CHAR: PrimitiveExecutable = PrimitiveExecutable {
    name:"char",
    expandable:false,
    _apply:|rf,int| {
        let num = int.read_number()? as u8;
        rf.2 = vec!(Token::new(num,CategoryCode::Other,None,SourceReference::Exp(rf.get_ref()),true));
        Ok(())
    }
};

pub static OMIT: PrimitiveExecutable = PrimitiveExecutable {
    name:"omit",
    expandable:false,
    _apply:|tk,int| {TeXErr!((int,Some(tk.0.clone())),"Unexpected \\omit")}
};

pub static CR: PrimitiveExecutable = PrimitiveExecutable {
    name:"cr",
    expandable:false,
    _apply:|tk,int| {TeXErr!((int,Some(tk.0.clone())),"Unexpected \\cr")}
};

pub static CRCR: PrimitiveExecutable = PrimitiveExecutable {
    name:"crcr",
    expandable:false,
    _apply:|_tk,_int| {Ok(())}
};

pub static NOALIGN: PrimitiveExecutable = PrimitiveExecutable {
    name:"noalign",
    expandable:false,
    _apply:|tk,int| {TeXErr!((int,Some(tk.0.clone())),"Unexpected \\noalign")}
};

fn do_align(int:&Interpreter,tabmode:BoxMode,betweenmode:BoxMode) -> Result<
        (Skip,Vec<(Vec<Token>,Vec<Token>,Skip)>,Vec<AlignBlock>),TeXError> {
    int.expand_until(false)?;
    let bg = int.next_token();
    match bg.catcode {
        CategoryCode::BeginGroup => (),
        CategoryCode::Escape | CategoryCode::Active => {
            let cmd = int.get_command(bg.cmdname())?;
            match &*cmd.orig {
                PrimitiveTeXCommand::Char(tk) if tk.catcode == CategoryCode::BeginGroup => (),
                _ => TeXErr!((int,Some(bg.clone())),"Expected begin group token; found: {}",bg)
            }
        }
        _ => TeXErr!((int,Some(bg.clone())),"Expected begin group token; found: {}",bg)
    }

    let mut endtemplate = Token::new(38,CategoryCode::Escape,Some("endtemplate".into()),SourceReference::None,false);
    endtemplate.cmdname = "relax".into();
    let mut endrow = Token::new(250,CategoryCode::Escape,Some("endtemplate".into()),SourceReference::None,false);
    endrow.cmdname = "relax".into();

    int.new_group(GroupType::Box(betweenmode));

    let mut tabskip = int.state_skip(-(TABSKIP.index as i32));
    let firsttabskip = tabskip;

    let mut inV = false;
    let mut columns: Vec<(Vec<Token>,Vec<Token>,Skip)> = vec!((vec!(),vec!(),tabskip));
    let mut recindex: Option<usize> = None;

    loop {
        let next = int.next_token();
        match next.catcode {
            CategoryCode::AlignmentTab if !inV && columns.last().unwrap().0.is_empty() => recindex = Some(columns.len() - 1),
            CategoryCode::AlignmentTab => {
                columns.push((vec!(),vec!(),tabskip));
                inV = false
            }
            CategoryCode::Parameter if !inV => inV = true,
            CategoryCode::Parameter => TeXErr!((int,Some(next)),"Misplaced # in alignment"),
            CategoryCode::Escape | CategoryCode::Active => {
                let proc = int.state_get_command(next.cmdname());
                match proc {
                    None => if inV { columns.last_mut().unwrap().1.push(next) } else { columns.last_mut().unwrap().0.push(next) }
                    Some(p) => match &*p.orig {
                        PrimitiveTeXCommand::Primitive(p) if **p == CR || **p == CRCR => {
                            int.insert_every(&EVERYCR);
                            break
                        }
                        PrimitiveTeXCommand::AV(AssignableValue::PrimSkip(p)) if **p == TABSKIP => {
                            tabskip = int.read_skip()?;
                            columns.last_mut().unwrap().2 = tabskip;
                        }
                        PrimitiveTeXCommand::Char(tk) if tk.catcode == CategoryCode::Parameter || tk.catcode == CategoryCode::AlignmentTab => {
                            int.requeue(tk.clone())
                        }
                        PrimitiveTeXCommand::Primitive(p) if **p == SPAN => {
                            let next = int.next_token();
                            match next.catcode {
                                CategoryCode::Escape | CategoryCode::Active => {
                                    let p = int.get_command(next.cmdname())?;
                                    if p.expandable(true) {
                                        p.expand(next,int)?
                                    } else {
                                        TeXErr!((int,Some(next)),"Expandable command expected after \\span")
                                    }
                                }
                                _ => TeXErr!((int,Some(next)),"Expandable command expected after \\span")
                            }
                        }
                        _ => if inV { columns.last_mut().unwrap().1.push(next) } else { columns.last_mut().unwrap().0.push(next) }
                    }
                }
            }
            _ => if inV { columns.last_mut().unwrap().1.push(next) } else { columns.last_mut().unwrap().0.push(next) }
        }
    }

    let mut boxes : Vec<AlignBlock> = vec!();

    'table: loop {
        'prelude: loop {
            let next = int.next_token();
            match next.catcode {
                CategoryCode::EndGroup => break 'table,
                CategoryCode::Space => (),
                CategoryCode::Active | CategoryCode::Escape => {
                    let cmd = int.state_get_command(next.cmdname());
                    match cmd {
                        None => {
                            int.requeue(next);
                            break 'prelude
                        },
                        Some(cmd) => {
                            if cmd.expandable(true) { cmd.expand(next,int)?} else {
                                match &*cmd.orig {
                                    PrimitiveTeXCommand::Char(tk) if tk.catcode == CategoryCode::EndGroup => break 'table,
                                    PrimitiveTeXCommand::Char(tk) if tk.catcode == CategoryCode::Space => (),
                                    PrimitiveTeXCommand::Primitive(c) if **c == CRCR => (),
                                    _ => {
                                        int.requeue(next);
                                        break 'prelude
                                    }
                                }
                            }
                        }
                    }
                }
                _ => {
                    int.requeue(next);
                    break 'prelude
                }
            }
        }

        let mut columnindex : usize = 0;
        let mut row:Vec<(Vec<Whatsit>,Skip)> = vec!();

        'row: loop {
            let mut doheader = true;
            inV = false;
            'prelude: loop {
                let next = int.next_token();
                match next.catcode {
                    CategoryCode::Space => (),
                    CategoryCode::Active | CategoryCode::Escape => {
                        let cmd = int.state_get_command(next.cmdname());
                        match cmd {
                            None => {
                                int.requeue(next);
                                break 'prelude
                            },
                            Some(cmd) => {
                                if cmd.expandable(true) { cmd.expand(next,int)?} else {
                                    match &*cmd.orig {
                                        PrimitiveTeXCommand::Char(tk) if tk.catcode == CategoryCode::Space => (),
                                        PrimitiveTeXCommand::Primitive(c) if **c == OMIT => {
                                            doheader = false;
                                            break 'prelude
                                        }
                                        _ => {
                                            int.requeue(next);
                                            break 'prelude
                                        }
                                    }
                                }
                            }
                        }
                    }
                    _ => {
                        int.requeue(next);
                        break 'prelude
                    }
                }
            }
            if columns.len() <= columnindex {
                match recindex {
                    Some(i) => columnindex = i,
                    None => TeXErr!((int,None),"Invalid column index in align")
                }
            }
            if doheader {
                int.push_tokens(columns.get(columnindex).unwrap().0.clone())
            }
            let _oldmode = int.get_mode();
            int.new_group(GroupType::Box(tabmode));
            'cell: loop {
                let next = int.next_token();
                match next.catcode {
                    CategoryCode::AlignmentTab if !doheader => break 'cell,
                    CategoryCode::AlignmentTab if !inV => {
                        inV = true;
                        int.requeue(endtemplate.clone());
                        int.push_tokens(columns.get(columnindex).unwrap().1.clone())
                    }
                    CategoryCode::Escape if next.char == endtemplate.char && next == endtemplate => {
                        break 'cell
                    }
                    CategoryCode::Escape if next.char == endrow.char && next == endrow => {
                        let ret = int.get_whatsit_group(GroupType::Box(tabmode))?;
                        row.push((ret,columns.get(columnindex).unwrap().2));
                        int.set_mode(_oldmode);
                        int.insert_every(&EVERYCR);
                        break 'row
                    }
                    CategoryCode::Escape | CategoryCode::Active => {
                        let p = int.get_command(next.cmdname())?;
                        match &*p.orig {
                            PrimitiveTeXCommand::Primitive(p) if (**p == CR || **p == CRCR) && !doheader => {
                                let ret = int.get_whatsit_group(GroupType::Box(tabmode))?;
                                row.push((ret,columns.get(columnindex).unwrap().2));
                                int.set_mode(_oldmode);
                                int.insert_every(&EVERYCR);
                                break 'row
                            }
                            PrimitiveTeXCommand::Primitive(p) if (**p == CR || **p == CRCR) && !inV => {
                                inV = true;
                                int.requeue(endrow.clone());
                                int.push_tokens(columns.get(columnindex).unwrap().1.clone())
                            }
                            _ => int.do_top(next,true)?
                        }
                    }
                    _ => int.do_top(next,true)?
                }
            }
            let ret = int.get_whatsit_group(GroupType::Box(tabmode))?;
            row.push((ret,columns.get(columnindex).unwrap().2));
            int.set_mode(_oldmode);
            columnindex += 1
        }
        boxes.push(AlignBlock::Block(row))
    }

    int.pop_group(GroupType::Box(betweenmode))?;
    Ok((firsttabskip, columns,boxes))
}

pub static HALIGN: SimpleWhatsit = SimpleWhatsit {
    name:"halign",
    modes: |x|  {x == TeXMode::Vertical || x == TeXMode::InternalVertical },
    _get:|tk,int| {
        let width = match int.read_keyword(vec!("to"))? {
            Some(_) => Some(int.read_dimension()?),
            None => None
        };
        let (skip,template,rows) = do_align(int,BoxMode::H,BoxMode::V)?;
        Ok(Whatsit::Simple(SimpleWI::Halign(skip,template,rows,int.update_reference(tk))))
    }
};

pub static VALIGN: SimpleWhatsit = SimpleWhatsit {
    name:"valign",
    modes: |x|  {x == TeXMode::Horizontal || x == TeXMode::RestrictedHorizontal },
    _get:|tk,int| {
        let height = match int.read_keyword(vec!("to"))? {
            Some(_) => Some(int.read_dimension()?),
            None => None
        };
        let (skip,template,columns) = do_align(int,BoxMode::V,BoxMode::H)?;
        Ok(Whatsit::Simple(SimpleWI::Valign(skip,template,columns,int.update_reference(tk))))
    }
};

pub static HSS: SimpleWhatsit = SimpleWhatsit {
    name:"hss",
    modes: |x|  {x == TeXMode::Horizontal || x == TeXMode::RestrictedHorizontal },
    _get:|tk,int| {Ok(Whatsit::Simple(SimpleWI::Hss(int.update_reference(tk))))}
};

pub static VSS: SimpleWhatsit = SimpleWhatsit {
    name:"vss",
    modes: |x|  {x == TeXMode::Vertical || x == TeXMode::InternalVertical },
    _get:|tk,int| {Ok(Whatsit::Simple(SimpleWI::Vss(int.update_reference(tk))))}
};

pub static MSKIP: SimpleWhatsit = SimpleWhatsit {
    name:"mskip",
    modes: |x|  {x == TeXMode::Math || x == TeXMode::Displaymath },
    _get:|tk,int| {
        let ms = int.read_muskip()?;
        Ok(Whatsit::Simple(SimpleWI::MSkip(ms,int.update_reference(tk))))
    }
};

pub static HANGINDENT : PrimitiveExecutable = PrimitiveExecutable {
    name: "hangindent",
    expandable:false,
    _apply: |rf,int| {
        todo!()
    }
};

pub static HANGAFTER : PrimitiveExecutable = PrimitiveExecutable {
    name: "hangafter",
    expandable:false,
    _apply: |rf,int| {
        todo!()
    }
};

pub static PARSHAPE: PrimitiveExecutable = PrimitiveExecutable {
    name:"parshape",
    expandable:false,
    _apply:|tk,int| {
        //unsafe { crate::LOG = true }
        let num = int.read_number()?;
        log!("\\parshape: Reading 2*{} dimensions:",num);
        let mut vals : Vec<(i64,i64)> = vec!();
        for i in 1..(num+1) {
            let f = int.read_dimension()?;
            log!("\\parshape: i{}={}",i,f);
            let s = int.read_dimension()?;
            log!("\\parshape: l{}={}",i,s);
            vals.push((f,s))
        }
        int.stomach.borrow_mut().base_mut().parshape = vals;
        Ok(())
    }
};

pub static INDENT: PrimitiveExecutable = PrimitiveExecutable {
    name:"indent",
    expandable:false,
    _apply:|tk,int| {
        int.stomach.borrow_mut().add(int,Whatsit::Simple(SimpleWI::Indent(
            int.state_dimension(-(crate::commands::primitives::PARINDENT.index as i32)),int.update_reference(&tk.0)
        )))?;
        Ok(())
    }
};

pub static NOINDENT: PrimitiveExecutable = PrimitiveExecutable {
    name:"noindent",
    expandable:false,
    _apply:|_tk,_int| {Ok(())}
};

pub static MATHCLOSE: MathWhatsit = MathWhatsit {
    name:"mathclose",
    _get: |tk,int| {todo!()}
};

pub static MATHBIN: MathWhatsit = MathWhatsit {
    name:"mathbin",
    _get: |tk,int| {todo!()}
};
pub static MATHINNER: MathWhatsit = MathWhatsit {
    name:"mathinner",
    _get: |tk,int| {todo!()}
};

pub static MATHOP: MathWhatsit = MathWhatsit {
    name:"mathop",
    _get: |tk,int| {todo!()}
};

pub static MATHOPEN: MathWhatsit = MathWhatsit {
    name:"mathopen",
    _get: |tk,int| {todo!()}
};

pub static MATHORD: MathWhatsit = MathWhatsit {
    name:"mathord",
    _get: |tk,int| {todo!()}
};

pub static MATHPUNCT: MathWhatsit = MathWhatsit {
    name:"mathpunct",
    _get: |tk,int| {todo!()}
};

pub static MATHREL: MathWhatsit = MathWhatsit {
    name:"mathrel",
    _get: |tk,int| {todo!()}
};

pub static MATHACCENT: MathWhatsit = MathWhatsit {
    name:"mathaccent",
    _get: |tk,int| {todo!()}
};

pub static RADICAL: MathWhatsit = MathWhatsit {
    name:"radical",
    _get: |tk,int| {todo!()}
};

pub static DELIMITER: MathWhatsit = MathWhatsit {
    name:"delimiter",
    _get: |tk,int| {
        let ret = int.read_math_whatsit()?;
        match ret {
            Some(w) => Ok(MathKernel::Delimiter(Box::new(w),int.update_reference(tk))),
            None => TeXErr!((int,None),"unfinished \\delimiter")
        }
    }
};

pub static MATHCHAR: MathWhatsit = MathWhatsit {
    name:"mathchar",
    _get: |tk,int| {todo!()}
};

pub static MIDDLE: MathWhatsit = MathWhatsit {
    name:"middle",
    _get: |tk,int| {todo!()}
};

pub static MKERN: MathWhatsit = MathWhatsit {
    name:"mkern",
    _get: |tk,int| {todo!()}
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
    name: "righthyphenmin",
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

pub static TRACINGPAGES : RegisterReference = RegisterReference {
    name: "tracingpages",
    index:75
};

pub static TRACINGCOMMANDS : RegisterReference = RegisterReference {
    name: "tracingcommands",
    index:76
};

pub static TRACINGMACROS : RegisterReference = RegisterReference {
    name: "tracingmacros",
    index:77
};

pub static TRACINGONLINE : RegisterReference = RegisterReference {
    name: "tracingonline",
    index:78
};

pub static TRACINGOUTPUT : RegisterReference = RegisterReference {
    name: "tracingoutput",
    index:79
};

pub static TRACINGPARAGRAPHS : RegisterReference = RegisterReference {
    name: "tracingparagraphs",
    index:80
};

pub static TRACINGRESTORES : RegisterReference = RegisterReference {
    name: "tracingrestores",
    index:81
};

pub static TRACINGASSIGNS : RegisterReference = RegisterReference {
    name: "tracingassigns",
    index:82
};

pub static TRACINGGROUPS : RegisterReference = RegisterReference {
    name: "tracinggroups",
    index:83
};

pub static TRACINGIFS : RegisterReference = RegisterReference {
    name: "tracingifs",
    index:84
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

// -------------------------

pub static THINMUSKIP : MuSkipReference = MuSkipReference {
    name: "thinmuskip",
    index:5
};

pub static MEDMUSKIP : MuSkipReference = MuSkipReference {
    name: "medmuskip",
    index:6
};

pub static THICKMUSKIP : MuSkipReference = MuSkipReference {
    name: "thickmuskip",
    index:7
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
    expandable:true,
    _apply:|_tk,_int| {todo!()}
};

pub static BATCHMODE: PrimitiveExecutable = PrimitiveExecutable {
    name:"batchmode",
    expandable:true,
    _apply:|_tk,int| {todo!("{} >>{}",int.current_line(),int.preview())}
};

pub static BYE: PrimitiveExecutable = PrimitiveExecutable {
    name:"bye",
    expandable:true,
    _apply:|_tk,_int| {todo!()}
};

pub static CURRENTGROUPLEVEL: PrimitiveExecutable = PrimitiveExecutable {
    name:"currentgrouplevel",
    expandable:true,
    _apply:|_tk,_int| {todo!()}
};

pub static EQNO: PrimitiveExecutable = PrimitiveExecutable {
    name:"eqno",
    expandable:true,
    _apply:|_tk,_int| {todo!()}
};

pub static FONTNAME: PrimitiveExecutable = PrimitiveExecutable {
    name:"fontname",
    expandable:true,
    _apply:|_tk,_int| {todo!()}
};

pub static SHIPOUT: PrimitiveExecutable = PrimitiveExecutable {
    name:"shipout",
    expandable:true,
    _apply:|_tk,_int| {todo!()}
};

pub static SPECIAL: PrimitiveExecutable = PrimitiveExecutable {
    name:"special",
    expandable:true,
    _apply:|_tk,_int| {todo!()}
};

pub static NONSCRIPT: PrimitiveExecutable = PrimitiveExecutable {
    name:"nonscript",
    expandable:true,
    _apply:|_tk,_int| {todo!()}
};

pub static HOLDINGINSERTS: PrimitiveExecutable = PrimitiveExecutable {
    name:"holdinginserts",
    expandable:true,
    _apply:|_tk,_int| {todo!()}
};

pub static LEQNO: PrimitiveExecutable = PrimitiveExecutable {
    name:"leqno",
    expandable:true,
    _apply:|_tk,_int| {todo!()}
};

pub static LOOSENESS: PrimitiveExecutable = PrimitiveExecutable {
    name:"looseness",
    expandable:true,
    _apply:|_tk,_int| {todo!()}
};

pub static NOBOUNDARY: PrimitiveExecutable = PrimitiveExecutable {
    name:"noboundary",
    expandable:true,
    _apply:|_tk,_int| {todo!()}
};

pub static SCROLLMODE: PrimitiveExecutable = PrimitiveExecutable {
    name:"scrollmode",
    expandable:true,
    _apply:|_tk,_int| {todo!()}
};

pub static NONSTOPMODE: PrimitiveExecutable = PrimitiveExecutable {
    name:"nonstopmode",
    expandable:true,
    _apply:|_tk,_int| {todo!()}
};

pub static PAUSING: PrimitiveExecutable = PrimitiveExecutable {
    name:"pausing",
    expandable:true,
    _apply:|_tk,_int| {todo!()}
};

pub static PREVGRAF: PrimitiveExecutable = PrimitiveExecutable {
    name:"prevgraf",
    expandable:true,
    _apply:|_tk,_int| {todo!()}
};

pub static SETLANGUAGE: PrimitiveExecutable = PrimitiveExecutable {
    name:"setlanguage",
    expandable:true,
    _apply:|_tk,_int| {todo!()}
};

pub static SHOW: PrimitiveExecutable = PrimitiveExecutable {
    name:"show",
    expandable:true,
    _apply:|_tk,_int| {todo!()}
};

pub static SHOWBOX: PrimitiveExecutable = PrimitiveExecutable {
    name:"showbox",
    expandable:true,
    _apply:|_tk,_int| {todo!()}
};

pub static SHOWLISTS: PrimitiveExecutable = PrimitiveExecutable {
    name:"showlists",
    expandable:true,
    _apply:|_tk,_int| {todo!()}
};

pub static SHOWTHE: PrimitiveExecutable = PrimitiveExecutable {
    name:"showthe",
    expandable:true,
    _apply:|_tk,_int| {todo!()}
};

pub static SPAN: PrimitiveExecutable = PrimitiveExecutable {
    name:"span",
    expandable:true,
    _apply:|_tk,_int| {todo!()}
};


pub static BEGINL: PrimitiveExecutable = PrimitiveExecutable {
    name:"beginL",
    expandable:true,
    _apply:|_tk,_int| {todo!()}
};

pub static BEGINR: PrimitiveExecutable = PrimitiveExecutable {
    name:"beginR",
    expandable:true,
    _apply:|_tk,_int| {todo!()}
};

pub static BOTMARKS: PrimitiveExecutable = PrimitiveExecutable {
    name:"botmarks",
    expandable:true,
    _apply:|_tk,_int| {todo!()}
};

pub static CURRENTGROUPTYPE: PrimitiveExecutable = PrimitiveExecutable {
    name:"currentgrouptype",
    expandable:true,
    _apply:|_tk,_int| {todo!()}
};

pub static CURRENTIFBRANCH: PrimitiveExecutable = PrimitiveExecutable {
    name:"currentifbranch",
    expandable:true,
    _apply:|_tk,_int| {todo!()}
};

pub static CURRENTIFLEVEL: PrimitiveExecutable = PrimitiveExecutable {
    name:"currentiflevel",
    expandable:true,
    _apply:|_tk,_int| {todo!()}
};

pub static CURRENTIFTYPE: PrimitiveExecutable = PrimitiveExecutable {
    name:"currentiftype",
    expandable:true,
    _apply:|_tk,_int| {todo!()}
};

pub static ENDL: PrimitiveExecutable = PrimitiveExecutable {
    name:"endL",
    expandable:true,
    _apply:|_tk,_int| {todo!()}
};

pub static ENDR: PrimitiveExecutable = PrimitiveExecutable {
    name:"endR",
    expandable:true,
    _apply:|_tk,_int| {todo!()}
};

pub static FIRSTMARKS: PrimitiveExecutable = PrimitiveExecutable {
    name:"firstmarks",
    expandable:true,
    _apply:|_tk,_int| {todo!()}
};

pub static GLUESHRINK: PrimitiveExecutable = PrimitiveExecutable {
    name:"glueshrink",
    expandable:true,
    _apply:|_tk,_int| {todo!()}
};

pub static GLUESHRINKORDER: PrimitiveExecutable = PrimitiveExecutable {
    name:"glueshrinkorder",
    expandable:true,
    _apply:|_tk,_int| {todo!()}
};

pub static GLUESTRETCH: PrimitiveExecutable = PrimitiveExecutable {
    name:"gluestretch",
    expandable:true,
    _apply:|_tk,_int| {todo!()}
};

pub static GLUESTRETCHORDER: PrimitiveExecutable = PrimitiveExecutable {
    name:"gluestretchorder",
    expandable:true,
    _apply:|_tk,_int| {todo!()}
};

pub static GLUETOMU: PrimitiveExecutable = PrimitiveExecutable {
    name:"gluetomu",
    expandable:true,
    _apply:|_tk,_int| {todo!()}
};

pub static INTERACTIONMODE: PrimitiveExecutable = PrimitiveExecutable {
    name:"interactionmode",
    expandable:true,
    _apply:|_tk,_int| {todo!()}
};

pub static LASTLINEFIT: PrimitiveExecutable = PrimitiveExecutable {
    name:"lastlinefit",
    expandable:true,
    _apply:|_tk,_int| {todo!()}
};

pub static MARKS: PrimitiveExecutable = PrimitiveExecutable {
    name:"marks",
    expandable:true,
    _apply:|_tk,_int| {todo!()}
};

pub static MUTOGLUE: PrimitiveExecutable = PrimitiveExecutable {
    name:"mutoglue",
    expandable:true,
    _apply:|_tk,_int| {todo!()}
};

pub static PAGEDISCARDS: PrimitiveExecutable = PrimitiveExecutable {
    name:"pagediscards",
    expandable:true,
    _apply:|_tk,_int| {todo!()}
};

pub static PARSHAPEDIMEN: PrimitiveExecutable = PrimitiveExecutable {
    name:"parshapedimen",
    expandable:true,
    _apply:|_tk,_int| {todo!()}
};

pub static PARSHAPEINDENT: PrimitiveExecutable = PrimitiveExecutable {
    name:"parshapeindent",
    expandable:true,
    _apply:|_tk,_int| {todo!()}
};

pub static PARSHAPELENGTH: PrimitiveExecutable = PrimitiveExecutable {
    name:"parshapelength",
    expandable:true,
    _apply:|_tk,_int| {todo!()}
};

pub static PREDISPLAYDIRECTION: PrimitiveExecutable = PrimitiveExecutable {
    name:"predisplaydirection",
    expandable:true,
    _apply:|_tk,_int| {todo!()}
};

pub static SHOWGROUPS: PrimitiveExecutable = PrimitiveExecutable {
    name:"showgroups",
    expandable:true,
    _apply:|_tk,_int| {todo!()}
};

pub static SHOWIFS: PrimitiveExecutable = PrimitiveExecutable {
    name:"showifs",
    expandable:true,
    _apply:|_tk,_int| {todo!()}
};

pub static SHOWTOKENS: PrimitiveExecutable = PrimitiveExecutable {
    name:"showtokens",
    expandable:true,
    _apply:|_tk,_int| {todo!()}
};

pub static SPLITBOTMARKS: PrimitiveExecutable = PrimitiveExecutable {
    name:"splitbotmarks",
    expandable:true,
    _apply:|_tk,_int| {todo!()}
};

pub static SPLITDISCARDS: PrimitiveExecutable = PrimitiveExecutable {
    name:"splitdiscards",
    expandable:true,
    _apply:|_tk,_int| {todo!()}
};

pub static SPLITFIRSTMARKS: PrimitiveExecutable = PrimitiveExecutable {
    name:"splitfirstmarks",
    expandable:true,
    _apply:|_tk,_int| {todo!()}
};

pub static TEXXETSTATE: PrimitiveExecutable = PrimitiveExecutable {
    name:"TeXXeTstate",
    expandable:true,
    _apply:|_tk,_int| {todo!()}
};

pub static TOPMARKS: PrimitiveExecutable = PrimitiveExecutable {
    name:"topmarks",
    expandable:true,
    _apply:|_tk,_int| {todo!()}
};

pub static EFCODE: PrimitiveExecutable = PrimitiveExecutable {
    name:"efcode",
    expandable:true,
    _apply:|_tk,_int| {todo!()}
};

pub static LEFTMARGINKERN: PrimitiveExecutable = PrimitiveExecutable {
    name:"leftmarginkern",
    expandable:true,
    _apply:|_tk,_int| {todo!()}
};

pub static LETTERSPACEFONT: PrimitiveExecutable = PrimitiveExecutable {
    name:"letterspacefont",
    expandable:true,
    _apply:|_tk,_int| {todo!()}
};

pub static QUITVMODE: PrimitiveExecutable = PrimitiveExecutable {
    name:"quitvmode",
    expandable:true,
    _apply:|_tk,_int| {todo!()}
};

pub static RIGHTMARGINKERN: PrimitiveExecutable = PrimitiveExecutable {
    name:"rightmarginkern",
    expandable:true,
    _apply:|_tk,_int| {todo!()}
};

pub static TAGCODE: PrimitiveExecutable = PrimitiveExecutable {
    name:"tagcode",
    expandable:true,
    _apply:|_tk,_int| {todo!()}
};

pub static MUSKIP: PrimitiveExecutable = PrimitiveExecutable {
    name:"muskip",
    expandable:true,
    _apply:|_tk,_int| {todo!()}
};

pub static OUTER: PrimitiveExecutable = PrimitiveExecutable {
    name:"outer",
    expandable:true,
    _apply:|_tk,_int| {todo!()}
};

pub static PAGEGOAL: PrimitiveExecutable = PrimitiveExecutable {
    name:"pagegoal",
    expandable:true,
    _apply:|_tk,_int| {todo!()}
};

pub static ABOVE: PrimitiveExecutable = PrimitiveExecutable {
    name:"above",
    expandable:true,
    _apply:|_tk,_int| {todo!()}
};

pub static ABOVEWITHDELIMS: PrimitiveExecutable = PrimitiveExecutable {
    name:"abovewithdelims",
    expandable:true,
    _apply:|_tk,_int| {todo!()}
};

pub static ACCENT: PrimitiveExecutable = PrimitiveExecutable {
    name:"accent",
    expandable:true,
    _apply:|_tk,_int| {todo!()}
};

pub static ATOP: PrimitiveExecutable = PrimitiveExecutable {
    name:"atop",
    expandable:true,
    _apply:|_tk,_int| {todo!()}
};

pub static ATOPWITHDELIMS: PrimitiveExecutable = PrimitiveExecutable {
    name:"atopwithdelims",
    expandable:true,
    _apply:|_tk,_int| {todo!()}
};

pub static BIGSKIP: PrimitiveExecutable = PrimitiveExecutable {
    name:"bigskip",
    expandable:true,
    _apply:|_tk,_int| {todo!()}
};

pub static DISCRETIONARY: PrimitiveExecutable = PrimitiveExecutable {
    name:"discretionary",
    expandable:true,
    _apply:|_tk,_int| {todo!()}
};

pub static DISPLAYSTYLE: PrimitiveExecutable = PrimitiveExecutable {
    name:"displaystyle",
    expandable:true,
    _apply:|_tk,_int| {todo!()}
};

pub static LIMITS: PrimitiveExecutable = PrimitiveExecutable {
    name:"limits",
    expandable:true,
    _apply:|_tk,_int| {todo!()}
};

pub static NOLIMITS: PrimitiveExecutable = PrimitiveExecutable {
    name:"nolimits",
    expandable:true,
    _apply:|_tk,_int| {todo!()}
};

pub static DISPLAYLIMITS: PrimitiveExecutable = PrimitiveExecutable {
    name:"displaylimits",
    expandable:true,
    _apply:|_tk,_int| {todo!()}
};

pub static MARK: PrimitiveExecutable = PrimitiveExecutable {
    name:"mark",
    expandable:true,
    _apply:|_tk,_int| {todo!()}
};

pub static TOPMARK: PrimitiveExecutable = PrimitiveExecutable {
    name:"topmark",
    expandable:true,
    _apply:|_tk,_int| {todo!()}
};

pub static FIRSTMARK: PrimitiveExecutable = PrimitiveExecutable {
    name:"firstmark",
    expandable:true,
    _apply:|_tk,_int| {todo!()}
};

pub static BOTMARK: PrimitiveExecutable = PrimitiveExecutable {
    name:"botmark",
    expandable:true,
    _apply:|_tk,_int| {todo!()}
};

pub static SPLITFIRSTMARK: PrimitiveExecutable = PrimitiveExecutable {
    name:"splitfirstmark",
    expandable:true,
    _apply:|_tk,_int| {todo!()}
};

pub static SPLITBOTMARK: PrimitiveExecutable = PrimitiveExecutable {
    name:"splitbotmark",
    expandable:true,
    _apply:|_tk,_int| {todo!()}
};

pub static HFILNEG: PrimitiveExecutable = PrimitiveExecutable {
    name:"hfilneg",
    expandable:true,
    _apply:|_tk,_int| {todo!()}
};

pub static INSERT: PrimitiveExecutable = PrimitiveExecutable {
    name:"insert",
    expandable:true,
    _apply:|_tk,_int| {todo!()}
};

pub static ITALICCORR: PrimitiveExecutable = PrimitiveExecutable {
    name:"italiccorr",
    expandable:true,
    _apply:|_tk,_int| {todo!()}
};

pub static LASTPENALTY: PrimitiveExecutable = PrimitiveExecutable {
    name:"lastpenalty",
    expandable:true,
    _apply:|_tk,_int| {todo!()}
};

pub static LASTKERN: PrimitiveExecutable = PrimitiveExecutable {
    name:"lastkern",
    expandable:true,
    _apply:|_tk,_int| {todo!()}
};

pub static LEADERS: PrimitiveExecutable = PrimitiveExecutable {
    name:"leaders",
    expandable:true,
    _apply:|_tk,_int| {todo!()}
};

pub static CLEADERS: PrimitiveExecutable = PrimitiveExecutable {
    name:"cleaders",
    expandable:true,
    _apply:|_tk,_int| {todo!()}
};

pub static XLEADERS: PrimitiveExecutable = PrimitiveExecutable {
    name:"xleaders",
    expandable:true,
    _apply:|_tk,_int| {todo!()}
};

pub static LEFT: PrimitiveExecutable = PrimitiveExecutable {
    name:"left",
    expandable:true,
    _apply:|_tk,_int| {todo!()}
};

pub static MATHCHOICE: PrimitiveExecutable = PrimitiveExecutable {
    name:"mathchoice",
    expandable:true,
    _apply:|_tk,_int| {todo!()}
};

pub static MEDSKIP: PrimitiveExecutable = PrimitiveExecutable {
    name:"medskip",
    expandable:true,
    _apply:|_tk,_int| {todo!()}
};

pub static MOVELEFT: PrimitiveExecutable = PrimitiveExecutable {
    name:"moveleft",
    expandable:true,
    _apply:|_tk,_int| {todo!()}
};

pub static MOVERIGHT: PrimitiveExecutable = PrimitiveExecutable {
    name:"moveright",
    expandable:true,
    _apply:|_tk,_int| {todo!()}
};

pub static OVER: PrimitiveExecutable = PrimitiveExecutable {
    name:"over",
    expandable:true,
    _apply:|_tk,_int| {todo!()}
};

pub static OVERLINE: PrimitiveExecutable = PrimitiveExecutable {
    name:"overline",
    expandable:true,
    _apply:|_tk,_int| {todo!()}
};

pub static OVERWITHDELIMS: PrimitiveExecutable = PrimitiveExecutable {
    name:"overwithdelims",
    expandable:true,
    _apply:|_tk,_int| {todo!()}
};

pub static RIGHT: PrimitiveExecutable = PrimitiveExecutable {
    name:"right",
    expandable:true,
    _apply:|_tk,_int| {todo!()}
};

pub static SMALLSKIP: PrimitiveExecutable = PrimitiveExecutable {
    name:"smallskip",
    expandable:true,
    _apply:|_tk,_int| {todo!()}
};

pub static UNDERLINE: PrimitiveExecutable = PrimitiveExecutable {
    name:"underline",
    expandable:true,
    _apply:|_tk,_int| {todo!()}
};

pub static UNKERN: PrimitiveExecutable = PrimitiveExecutable {
    name:"unkern",
    expandable:true,
    _apply:|_tk,_int| {todo!()}
};

pub static UNPENALTY: PrimitiveExecutable = PrimitiveExecutable {
    name:"unpenalty",
    expandable:true,
    _apply:|_tk,_int| {todo!()}
};

pub static VFILNEG: PrimitiveExecutable = PrimitiveExecutable {
    name:"vfilneg",
    expandable:true,
    _apply:|_tk,_int| {todo!()}
};

pub static VSPLIT: PrimitiveExecutable = PrimitiveExecutable {
    name:"vsplit",
    expandable:true,
    _apply:|_tk,_int| {todo!()}
};

pub static VTOP: PrimitiveExecutable = PrimitiveExecutable {
    name:"vtop",
    expandable:true,
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
    PrimitiveTeXCommand::AV(AssignableValue::Int(&ESCAPECHAR)),
    PrimitiveTeXCommand::AV(AssignableValue::Int(&COUNT)),
    PrimitiveTeXCommand::AV(AssignableValue::Int(&DIMEN)),
    PrimitiveTeXCommand::AV(AssignableValue::Int(&SKIP)),
    PrimitiveTeXCommand::AV(AssignableValue::Int(&HT)),
    PrimitiveTeXCommand::AV(AssignableValue::Int(&WD)),
    PrimitiveTeXCommand::AV(AssignableValue::Int(&DP)),
    PrimitiveTeXCommand::Ass(&CHARDEF),
    PrimitiveTeXCommand::Ass(&COUNTDEF),
    PrimitiveTeXCommand::Ass(&DIMENDEF),
    PrimitiveTeXCommand::Ass(&SKIPDEF),
    PrimitiveTeXCommand::Ass(&MUSKIPDEF),
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
    PrimitiveTeXCommand::Ass(&SETBOX),
    PrimitiveTeXCommand::AV(AssignableValue::Font(&FONT)),
    PrimitiveTeXCommand::AV(AssignableValue::Font(&TEXTFONT)),
    PrimitiveTeXCommand::AV(AssignableValue::Font(&SCRIPTFONT)),
    PrimitiveTeXCommand::AV(AssignableValue::Font(&SCRIPTSCRIPTFONT)),
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
    PrimitiveTeXCommand::Whatsit(ProvidesWhatsit::Simple(&VRULE)),
    PrimitiveTeXCommand::Whatsit(ProvidesWhatsit::Simple(&HRULE)),
    PrimitiveTeXCommand::Whatsit(ProvidesWhatsit::Simple(&VFIL)),
    PrimitiveTeXCommand::Whatsit(ProvidesWhatsit::Simple(&VFILL)),
    PrimitiveTeXCommand::Whatsit(ProvidesWhatsit::Simple(&VSKIP)),
    PrimitiveTeXCommand::Whatsit(ProvidesWhatsit::Simple(&HSKIP)),
    PrimitiveTeXCommand::Whatsit(ProvidesWhatsit::Simple(&HFIL)),
    PrimitiveTeXCommand::Whatsit(ProvidesWhatsit::Simple(&HFILL)),
    PrimitiveTeXCommand::Whatsit(ProvidesWhatsit::Simple(&PENALTY)),
    PrimitiveTeXCommand::Whatsit(ProvidesWhatsit::Simple(&LOWER)),
    PrimitiveTeXCommand::Whatsit(ProvidesWhatsit::Simple(&RAISE)),
    PrimitiveTeXCommand::Whatsit(ProvidesWhatsit::Simple(&KERN)),
    PrimitiveTeXCommand::Whatsit(ProvidesWhatsit::Simple(&UNHBOX)),
    PrimitiveTeXCommand::Whatsit(ProvidesWhatsit::Simple(&UNHCOPY)),
    PrimitiveTeXCommand::Whatsit(ProvidesWhatsit::Simple(&UNVBOX)),
    PrimitiveTeXCommand::Whatsit(ProvidesWhatsit::Simple(&UNVCOPY)),
    PrimitiveTeXCommand::Whatsit(ProvidesWhatsit::Simple(&HALIGN)),
    PrimitiveTeXCommand::Whatsit(ProvidesWhatsit::Simple(&VALIGN)),
    PrimitiveTeXCommand::Whatsit(ProvidesWhatsit::Simple(&HSS)),
    PrimitiveTeXCommand::Whatsit(ProvidesWhatsit::Simple(&VSS)),
    PrimitiveTeXCommand::Whatsit(ProvidesWhatsit::Simple(&MSKIP)),
    PrimitiveTeXCommand::Ass(&READ),
    PrimitiveTeXCommand::Ass(&READLINE),
    PrimitiveTeXCommand::Ass(&NULLFONT),
    PrimitiveTeXCommand::Ass(&MATHCHARDEF),
    PrimitiveTeXCommand::Ass(&FUTURELET),
    PrimitiveTeXCommand::AV(AssignableValue::Tok(&TOKS)),
    PrimitiveTeXCommand::Num(&TIME),
    PrimitiveTeXCommand::Num(&YEAR),
    PrimitiveTeXCommand::Num(&MONTH),
    PrimitiveTeXCommand::Num(&DAY),
    PrimitiveTeXCommand::Num(&NUMEXPR),
    PrimitiveTeXCommand::Num(&DIMEXPR),
    PrimitiveTeXCommand::Num(&GLUEEXPR),
    PrimitiveTeXCommand::Num(&MUEXPR),
    PrimitiveTeXCommand::Num(&INPUTLINENO),
    PrimitiveTeXCommand::Num(&FONTCHARWD),
    PrimitiveTeXCommand::Num(&FONTCHARHT),
    PrimitiveTeXCommand::Num(&FONTCHARDP),
    PrimitiveTeXCommand::Num(&FONTCHARIC),
    PrimitiveTeXCommand::Num(&LASTSKIP),
    PrimitiveTeXCommand::AV(AssignableValue::Int(&MATHCODE)),
    PrimitiveTeXCommand::Primitive(&ROMANNUMERAL),
    PrimitiveTeXCommand::Primitive(&NOEXPAND),
    PrimitiveTeXCommand::Primitive(&EXPANDAFTER),
    PrimitiveTeXCommand::Primitive(&MEANING),
    PrimitiveTeXCommand::Primitive(&ETEXREVISION),
    PrimitiveTeXCommand::Primitive(&UNEXPANDED),
    PrimitiveTeXCommand::Primitive(&PARSHAPE),
    PrimitiveTeXCommand::Primitive(&HANGINDENT),
    PrimitiveTeXCommand::Primitive(&HANGAFTER),
    PrimitiveTeXCommand::Num(&ETEXVERSION),
    PrimitiveTeXCommand::AV(AssignableValue::Int(&LCCODE)),
    PrimitiveTeXCommand::AV(AssignableValue::Int(&UCCODE)),
    PrimitiveTeXCommand::AV(AssignableValue::Int(&FONTDIMEN)),
    PrimitiveTeXCommand::AV(AssignableValue::Int(&LPCODE)),
    PrimitiveTeXCommand::AV(AssignableValue::Int(&RPCODE)),
    PrimitiveTeXCommand::AV(AssignableValue::Int(&HYPHENCHAR)),
    PrimitiveTeXCommand::AV(AssignableValue::Int(&SKEWCHAR)),
    PrimitiveTeXCommand::AV(AssignableValue::Int(&DELCODE)),
    PrimitiveTeXCommand::Whatsit(ProvidesWhatsit::Box(&HBOX)),
    PrimitiveTeXCommand::Whatsit(ProvidesWhatsit::Box(&VBOX)),
    PrimitiveTeXCommand::Whatsit(ProvidesWhatsit::Box(&VCENTER)),
    PrimitiveTeXCommand::Whatsit(ProvidesWhatsit::Box(&LASTBOX)),
    PrimitiveTeXCommand::Whatsit(ProvidesWhatsit::Box(&BOX)),
    PrimitiveTeXCommand::Whatsit(ProvidesWhatsit::Box(&COPY)),
    PrimitiveTeXCommand::Whatsit(ProvidesWhatsit::Math(&MATHCLOSE)),
    PrimitiveTeXCommand::Whatsit(ProvidesWhatsit::Math(&MATHBIN)),
    PrimitiveTeXCommand::Whatsit(ProvidesWhatsit::Math(&MATHINNER)),
    PrimitiveTeXCommand::Whatsit(ProvidesWhatsit::Math(&MATHOP)),
    PrimitiveTeXCommand::Whatsit(ProvidesWhatsit::Math(&MATHOPEN)),
    PrimitiveTeXCommand::Whatsit(ProvidesWhatsit::Math(&MATHORD)),
    PrimitiveTeXCommand::Whatsit(ProvidesWhatsit::Math(&MATHPUNCT)),
    PrimitiveTeXCommand::Whatsit(ProvidesWhatsit::Math(&MATHREL)),
    PrimitiveTeXCommand::Whatsit(ProvidesWhatsit::Math(&MATHACCENT)),
    PrimitiveTeXCommand::Whatsit(ProvidesWhatsit::Math(&RADICAL)),
    PrimitiveTeXCommand::Whatsit(ProvidesWhatsit::Math(&DELIMITER)),
    PrimitiveTeXCommand::Whatsit(ProvidesWhatsit::Math(&MATHCHAR)),
    PrimitiveTeXCommand::Whatsit(ProvidesWhatsit::Math(&MIDDLE)),
    PrimitiveTeXCommand::Whatsit(ProvidesWhatsit::Math(&MKERN)),

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
    PrimitiveTeXCommand::AV(AssignableValue::PrimReg(&TRACINGPAGES)),
    PrimitiveTeXCommand::AV(AssignableValue::PrimReg(&TRACINGCOMMANDS)),
    PrimitiveTeXCommand::AV(AssignableValue::PrimReg(&TRACINGMACROS)),
    PrimitiveTeXCommand::AV(AssignableValue::PrimReg(&TRACINGONLINE)),
    PrimitiveTeXCommand::AV(AssignableValue::PrimReg(&TRACINGOUTPUT)),
    PrimitiveTeXCommand::AV(AssignableValue::PrimReg(&TRACINGPARAGRAPHS)),
    PrimitiveTeXCommand::AV(AssignableValue::PrimReg(&TRACINGRESTORES)),
    PrimitiveTeXCommand::AV(AssignableValue::PrimReg(&TRACINGASSIGNS)),
    PrimitiveTeXCommand::AV(AssignableValue::PrimReg(&TRACINGGROUPS)),
    PrimitiveTeXCommand::AV(AssignableValue::PrimReg(&TRACINGIFS)),

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

    PrimitiveTeXCommand::AV(AssignableValue::PrimMuSkip(&THINMUSKIP)),
    PrimitiveTeXCommand::AV(AssignableValue::PrimMuSkip(&MEDMUSKIP)),
    PrimitiveTeXCommand::AV(AssignableValue::PrimMuSkip(&THICKMUSKIP)),

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
    PrimitiveTeXCommand::Primitive(&DUMP),
    PrimitiveTeXCommand::Primitive(&ENDINPUT),
    PrimitiveTeXCommand::Primitive(&EQNO),
    PrimitiveTeXCommand::Primitive(&ERRMESSAGE),
    PrimitiveTeXCommand::Primitive(&ERRORSTOPMODE),
    PrimitiveTeXCommand::Primitive(&EXPANDED),
    PrimitiveTeXCommand::Primitive(&FONTNAME),
    PrimitiveTeXCommand::Primitive(&IGNORESPACES),
    PrimitiveTeXCommand::Primitive(&JOBNAME),
    PrimitiveTeXCommand::Primitive(&LOWERCASE),
    PrimitiveTeXCommand::Primitive(&MESSAGE),
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
    PrimitiveTeXCommand::Primitive(&EFCODE),
    PrimitiveTeXCommand::Primitive(&LEFTMARGINKERN),
    PrimitiveTeXCommand::Primitive(&LETTERSPACEFONT),
    PrimitiveTeXCommand::Primitive(&QUITVMODE),
    PrimitiveTeXCommand::Primitive(&RIGHTMARGINKERN),
    PrimitiveTeXCommand::Primitive(&TAGCODE),
    PrimitiveTeXCommand::Primitive(&AFTERASSIGNMENT),
    PrimitiveTeXCommand::Primitive(&AFTERGROUP),
    PrimitiveTeXCommand::Primitive(&HYPHENATION),
    PrimitiveTeXCommand::Primitive(&MUSKIP),
    PrimitiveTeXCommand::Primitive(&OUTER),
    PrimitiveTeXCommand::Primitive(&PAGEGOAL),
    PrimitiveTeXCommand::Primitive(&PATTERNS),
    PrimitiveTeXCommand::Primitive(&ABOVE),
    PrimitiveTeXCommand::Primitive(&ABOVEWITHDELIMS),
    PrimitiveTeXCommand::Primitive(&ACCENT),
    PrimitiveTeXCommand::Primitive(&ATOP),
    PrimitiveTeXCommand::Primitive(&ATOPWITHDELIMS),
    PrimitiveTeXCommand::Primitive(&BIGSKIP),
    PrimitiveTeXCommand::Primitive(&DISCRETIONARY),
    PrimitiveTeXCommand::Primitive(&DISPLAYSTYLE),
    PrimitiveTeXCommand::Primitive(&LIMITS),
    PrimitiveTeXCommand::Primitive(&NOLIMITS),
    PrimitiveTeXCommand::Primitive(&DISPLAYLIMITS),
    PrimitiveTeXCommand::Primitive(&MARK),
    PrimitiveTeXCommand::Primitive(&TOPMARK),
    PrimitiveTeXCommand::Primitive(&FIRSTMARK),
    PrimitiveTeXCommand::Primitive(&BOTMARK),
    PrimitiveTeXCommand::Primitive(&SPLITFIRSTMARK),
    PrimitiveTeXCommand::Primitive(&SPLITBOTMARK),
    PrimitiveTeXCommand::Primitive(&HFILNEG),
    PrimitiveTeXCommand::Primitive(&INDENT),
    PrimitiveTeXCommand::Primitive(&INSERT),
    PrimitiveTeXCommand::Primitive(&ITALICCORR),
    PrimitiveTeXCommand::Primitive(&LASTPENALTY),
    PrimitiveTeXCommand::Primitive(&LASTKERN),
    PrimitiveTeXCommand::Primitive(&LEADERS),
    PrimitiveTeXCommand::Primitive(&CLEADERS),
    PrimitiveTeXCommand::Primitive(&XLEADERS),
    PrimitiveTeXCommand::Primitive(&LEFT),
    PrimitiveTeXCommand::Primitive(&MATHCHOICE),
    PrimitiveTeXCommand::Primitive(&MEDSKIP),
    PrimitiveTeXCommand::Primitive(&MOVELEFT),
    PrimitiveTeXCommand::Primitive(&MOVERIGHT),
    PrimitiveTeXCommand::Primitive(&NOALIGN),
    PrimitiveTeXCommand::Primitive(&NOINDENT),
    PrimitiveTeXCommand::Primitive(&OVER),
    PrimitiveTeXCommand::Primitive(&OVERLINE),
    PrimitiveTeXCommand::Primitive(&OVERWITHDELIMS),
    PrimitiveTeXCommand::Primitive(&RIGHT),
    PrimitiveTeXCommand::Primitive(&SMALLSKIP),
    PrimitiveTeXCommand::Primitive(&UNDERLINE),
    PrimitiveTeXCommand::Primitive(&UNSKIP),
    PrimitiveTeXCommand::Primitive(&UNKERN),
    PrimitiveTeXCommand::Primitive(&UNPENALTY),
    PrimitiveTeXCommand::Primitive(&VADJUST),
    PrimitiveTeXCommand::Primitive(&VFILNEG),
    PrimitiveTeXCommand::Primitive(&VSPLIT),
    PrimitiveTeXCommand::Primitive(&VTOP),
]}