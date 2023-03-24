use std::sync::Arc;
use crate::commands::{RegisterReference, AssignableValue, NumAssValue, DefMacro, NumericCommand, ParamToken, PrimitiveAssignment, PrimitiveExecutable, ProvidesExecutableWhatsit, ProvidesWhatsit, Signature, TokenList, DimenReference, SkipReference, TokReference, PrimitiveTeXCommand, FontAssValue, ProvidesBox, TokAssValue, MathWhatsit, MuSkipReference, SimpleWhatsit};
use crate::interpreter::{Interpreter, TeXMode};
use crate::ontology::{Token, ExpansionRef};
use crate::catcodes::CategoryCode;
use crate::interpreter::state::{FontStyle, GroupType, State};
use crate::utils::{TeXError, TeXStr, TeXString};
use crate::{log,TeXErr,FileEnd};
use crate::VERSION_INFO;
use crate::stomach::whatsits::{Accent, PrintChar, SpaceChar, WhatsitTrait};

pub static SPACE: SimpleWhatsit = SimpleWhatsit {
    name:" ",
    modes:|m| match m {
        TeXMode::Horizontal | TeXMode::RestrictedHorizontal | TeXMode::Math | TeXMode::Displaymath => true,
        _ => false
    },
    _get: |tk,int| {
        match int.state.mode {
            TeXMode::Horizontal | TeXMode::RestrictedHorizontal => Ok(Whatsit::Space(
                SpaceChar {
                    font: int.state.currfont.get(&()),
                    sourceref: int.update_reference(tk),
                    nonbreaking:true
                })),
            _ => Ok(MSkip {
                skip: MuSkip {base:12 * 65536, stretch:None, shrink:None},
                sourceref: int.update_reference(tk)
            }.as_whatsit())/*Ok(Whatsit::Math(MathGroup::new(
                MathKernel::MathChar(MathChar {
                    class:0,family:0,position:32,font:int.state.textfonts.get(&0),
                    sourceref:int.update_reference(tk)
                }),int.state.displaymode.get(&()))))*/
        }
    }
};

pub static PAR : PrimitiveExecutable = PrimitiveExecutable {
    expandable:false,
    name:"par",
    _apply:|_, _| {
        Ok(())
    }
};
pub static RELAX : PrimitiveExecutable = PrimitiveExecutable {
    expandable:false,
    name:"relax",
    _apply:|_, _| {
        Ok(())
    }
};
pub static CATCODE : NumAssValue = NumAssValue {
    name: "catcode",
    _assign: |_rf,int,global| {
        let num = int.read_number()? as u8;
        int.read_eq();
        let cat = CategoryCode::fromint(int.read_number()?);
        int.state.catcodes.set(num,cat,global);
        Ok(())
    },
    _getvalue: |int| {
        let char = int.read_number()?;
        Ok(Numeric::Int(CategoryCode::toint(&int.state.catcodes.get_scheme().get_code(char as u8)) as i32))
    }
};

pub static SFCODE : NumAssValue = NumAssValue {
    name:"sfcode",
    _assign: |_rf,int,global| {
        let char = int.read_number()? as u8;
        int.read_eq();
        let val = int.read_number()?;
        int.state.sfcodes.set(char,val,global);
        Ok(())
    },
    _getvalue: |int| {
        let char = int.read_number()? as u8;
        Ok(Numeric::Int(int.state.sfcodes.get(&char)))
    }
};

use chrono::{Datelike, Timelike};
use crate::fonts::{Font, NULL_FONT};

pub static CHARDEF: PrimitiveAssignment = PrimitiveAssignment {
    name: "chardef",
    _assign: |_,int,global| {
        let c = int.read_command_token()?;
        int.read_eq();
        let num = int.read_number()?;
        let cmd = PrimitiveTeXCommand::Char(Token::new(num as u8,CategoryCode::Other,None,None,true)).as_command();
        int.change_command(c.cmdname(),Some(cmd),global);
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
        int.state.registers.set(index as i32,val,global);
        Ok(())
    },
    _getvalue: |int| {
        let index = int.read_number()? as i32;
        let num = int.state.registers.get(&index);
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
        int.state.dimensions.set(index as i32,val,global);
        Ok(())
    },
    _getvalue: |int| {
        let index = int.read_number()? as u16;
        let dim = int.state.dimensions.get(&(index as i32));
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
        int.state.skips.set(index as i32,val,global);
        Ok(())
    },
    _getvalue: |int| {
        let index = int.read_number()? as u16;
        let dim = int.state.skips.get(&(index as i32));
        log!("\\skip {} = {}",index,dim);
        Ok(Numeric::Skip(dim))
    }
};

pub static COUNTDEF: PrimitiveAssignment = PrimitiveAssignment {
    name:"countdef",
    _assign: |_,int,global| {
        let cmd = int.read_command_token()?;
        int.set_relax(&cmd);
        int.read_eq();
        let num = int.read_number()? as u16;
        let command = PrimitiveTeXCommand::AV(AssignableValue::Register(num)).as_command();

        int.change_command(cmd.cmdname(),Some(command),global);
        Ok(())
    }
};

pub static DIMENDEF: PrimitiveAssignment = PrimitiveAssignment {
    name:"dimendef",
    _assign: |_,int,global| {
        let cmd = int.read_command_token()?;
        int.set_relax(&cmd);
        int.read_eq();
        let num = int.read_number()? as u16;
        let command = PrimitiveTeXCommand::AV(AssignableValue::Dim(num)).as_command();

        int.change_command(cmd.cmdname(),Some(command),global);
        Ok(())
    }
};

pub static SKIPDEF: PrimitiveAssignment = PrimitiveAssignment {
    name:"skipdef",
    _assign: |_,int,global| {
        let cmd = int.read_command_token()?;
        int.set_relax(&cmd);
        int.read_eq();
        let num = int.read_number()? as u16;
        let command = PrimitiveTeXCommand::AV(AssignableValue::Skip(num)).as_command();
        int.change_command(cmd.cmdname(),Some(command),global);
        Ok(())
    }
};

pub static MUSKIPDEF: PrimitiveAssignment = PrimitiveAssignment {
    name:"muskipdef",
    _assign: |_,int,global| {
        let cmd = int.read_command_token()?;
        int.set_relax(&cmd);
        int.read_eq();
        let num = int.read_number()? as u16;
        let command = PrimitiveTeXCommand::AV(AssignableValue::MuSkip(num)).as_command();

        int.change_command(cmd.cmdname(),Some(command),global);
        Ok(())
    }
};

pub static TOKSDEF: PrimitiveAssignment = PrimitiveAssignment {
    name:"toksdef",
    _assign: |_,int,global| {
        let cmd = int.read_command_token()?;
        int.set_relax(&cmd);
        int.read_eq();
        let num = int.read_number()? as u16;
        let command = PrimitiveTeXCommand::AV(AssignableValue::Toks(num)).as_command();

        int.change_command(cmd.cmdname(),Some(command),global);
        Ok(())
    }
};

pub static PROTECTED : PrimitiveAssignment = PrimitiveAssignment {
    name:"protected",
    _assign: |rf,int,iglobal| {
        let mut long = false;
        let mut global = iglobal;
        while int.has_next() {
            int.expand_until(false)?;
            let next = int.next_token();
            match next.catcode {
                CategoryCode::Escape | CategoryCode::Active => {
                    match *int.get_command(&next.cmdname())?.orig {
                        PrimitiveTeXCommand::Ass(a) if *a == DEF => {
                            return do_def(int,global,true,long,false)
                        }
                        PrimitiveTeXCommand::Ass(a) if *a == EDEF => {
                            return do_def(int,global,true,long,true)
                        }
                        PrimitiveTeXCommand::Ass(a) if *a == GDEF => {
                            return do_def(int,true,true,long,false)
                        }
                        PrimitiveTeXCommand::Ass(a) if *a == XDEF => {
                            return do_def(int,true,true,long,true)
                        }
                        PrimitiveTeXCommand::Ass(a) if *a == LONG => {
                            long = true;
                        }
                        PrimitiveTeXCommand::Ass(a) if *a == GLOBAL => {
                            global = true;
                        }
                        _ => TeXErr!(next.clone() => "Expected \\def or \\edef or \\long after \\protected: {}",next)
                    }
                }
                _ => TeXErr!(next.clone() => "Expected control sequence or active character; got: {}",next)
            }
        }
        FileEnd!()
    }
};

pub static LONG: PrimitiveAssignment = PrimitiveAssignment {
    name:"long",
    _assign: |rf,int,iglobal| {
        let mut protected = false;
        let mut global = iglobal;
        while int.has_next() {
            int.expand_until(false)?;
            let next = int.next_token();
            match next.catcode {
                CategoryCode::Escape | CategoryCode::Active => {
                    match *int.get_command(&next.cmdname())?.orig {
                        PrimitiveTeXCommand::Ass(a) if *a == DEF => {
                            return do_def(int,global,protected,true,false)
                        }
                        PrimitiveTeXCommand::Ass(a) if *a == EDEF => {
                            return do_def(int,global,protected,true,true)
                        }
                        PrimitiveTeXCommand::Ass(a) if *a == GDEF => {
                            return do_def(int,true,protected,true,false)
                        }
                        PrimitiveTeXCommand::Ass(a) if *a == XDEF => {
                            return do_def(int,true,protected,true,true)
                        }
                        PrimitiveTeXCommand::Ass(a) if *a == PROTECTED => {
                            protected = true;
                        }
                        PrimitiveTeXCommand::Ass(a) if *a == GLOBAL => {
                            global = true;
                        }
                        _ => TeXErr!(next => "Expected \\def or \\edef or \\protected after \\long")
                    }
                }
                _ => TeXErr!(next.clone() => "Expected control sequence or active character; got: {}",next)
            }
        }
        FileEnd!()
    }
};


fn read_sig(int:&mut Interpreter) -> Result<Signature,TeXError> {
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
                //int.assert_has_next()?;
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
                        if inext.char < 48 {
                            TeXErr!(inext.clone() => "Expected argument #{}; got:#{}",currarg,inext)
                        }
                        let arg = inext.char - 48;
                        if currarg == arg {
                            retsig.push(ParamToken::Param(arg));
                            currarg += 1
                        } else {
                            TeXErr!(inext.clone() => "Expected argument #{}; got:#{}",currarg,inext)
                        }
                    }
                }
            }
            _ => retsig.push(ParamToken::Token(next.clean()))
        }
    }
    FileEnd!()
}

fn do_def(int:&mut Interpreter, global:bool, protected:bool, long:bool,edef:bool) -> Result<(),TeXError> {
    let command = int.next_token();
    match command.catcode {
        CategoryCode::Escape | CategoryCode::Active => {}
        _ => TeXErr!(command.clone() => "\\def expected control sequence or active character; got: {}",command)
    }
    let sig = read_sig(int)?;
    let ret = int.read_token_list(edef,true,edef,false)?;
    log!("\\def {}{}{}{}{}",command,sig,"{",TokenList(&ret),"}");
    let dm = PrimitiveTeXCommand::Def(DefMacro {
        protected,
        long,
        sig,
        ret
    }).as_command();
    int.change_command(command.cmdname(),Some(dm),global);
    Ok(())
}

use crate::interpreter::dimensions::{dimtostr, MuSkip, Numeric, round_f, Skip};
use crate::stomach::whatsits::{ExecutableWhatsit, Whatsit};
use crate::stomach::math::{Above, Delimiter, MathAccent, MathBin, MathChar, MathClose, MathGroup, MathInner, MathKernel, MathOp, MathOpen, MathOrd, MathPunct, MathRel, MKern, Overline, Radical, Underline};
use crate::stomach::boxes::{BoxMode, TeXBox, HBox, VBox, VBoxType};
use crate::stomach::groups::FontChange;
use crate::stomach::simple::{AlignBlock, HAlign, HFil, HFill, HKern, HRule, HSkip, Hss, Indent, Leaders, Left, Mark, Middle, MoveRight, MSkip, Penalty, Raise, Right, SimpleWI, VAlign, VFil, VFill, VKern, VRule, VSkip, Vss};

pub static GLOBAL : PrimitiveAssignment = PrimitiveAssignment {
    name:"global",
    _assign: |_rf,int,_global| {
        let mut last : Option<Token> = None;
        'a: loop {
            int.expand_until(true)?;
            int.eat_relax();
            let next = int.read_command_token()?;
            match last {
                Some(n) if n == next =>
                    TeXErr!(next.clone() => "Assignment expected after \\global; found: {}",next),
                _ => ()
            }
            let cmd = int.get_command(&next.cmdname())?;
            if cmd.assignable() {
                cmd.assign(next,int,true)?;
                return Ok(())
            } else {
                last = Some(next.clone());
                int.requeue(next);
            }
        }
    }
};

pub static DEF: PrimitiveAssignment = PrimitiveAssignment {
    name:"def",
    _assign: |_,int,global| do_def(int, global, false, false,false)
};

pub static GDEF: PrimitiveAssignment = PrimitiveAssignment {
    name:"gdef",
    _assign: |_,int,_global| do_def(int, true, false, false,false)
};

pub static XDEF: PrimitiveAssignment = PrimitiveAssignment {
    name:"xdef",
    _assign: |_,int,_global| do_def(int, true, false, false,true)
};

pub static EDEF: PrimitiveAssignment = PrimitiveAssignment {
    name:"edef",
    _assign: |_,int,global| do_def(int,global,false,false,true)
};

pub static LET: PrimitiveAssignment = PrimitiveAssignment {
    name:"let",
    _assign: |_,int,global| {
        let cmd = int.next_token();
        if cmd.catcode != CategoryCode::Escape && cmd.catcode != CategoryCode::Active {
            TeXErr!(cmd.clone() => "Control sequence or active character expected; found {} of catcode {}",cmd,cmd.catcode)
        }
        int.read_eq();
        let def = int.next_token();
        log!("\\let {}={}",cmd,def);
        let ch = match def.catcode {
            CategoryCode::Escape | CategoryCode::Active => {
                int.state.commands.get(&def.cmdname())//.map(|x| x)
            }
            _ => Some(PrimitiveTeXCommand::Char(def).as_command())
        };
        int.change_command(cmd.cmdname(),ch,global);
        Ok(())
    }
};

pub static FUTURELET: PrimitiveAssignment = PrimitiveAssignment {
    name:"futurelet",
    _assign: |_,int,global| {
        let newcmd = int.next_token();
        match newcmd.catcode {
            CategoryCode::Escape | CategoryCode::Active => {}
            _ => TeXErr!(newcmd => "Expected command after \\futurelet")
        }
        let first = int.next_token();
        let second = int.next_token();
        let p = match second.catcode {
            CategoryCode::Escape | CategoryCode::Active => {
                int.state.commands.get(&second.cmdname())//.map(|x| x.as_command())
            }
            _ => Some(PrimitiveTeXCommand::Char(second.clone()).as_command())
        };
        int.change_command(newcmd.cmdname(),p,global);
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
        int.state.catcodes.set_newline(num,global);
        Ok(())
    },
    _getvalue: |int| {
        Ok(Numeric::Int(int.state.catcodes.get_scheme().newlinechar as i32))
    }
};

pub static ENDLINECHAR : NumAssValue = NumAssValue {
    name: "endlinechar",
    _assign: |_,int,global| {
        int.read_eq();
        let num = int.read_number()? as u8;
        log!("\\endlinechar: {}",num);
        int.state.catcodes.set_endline(num,global);
        Ok(())
    },
    _getvalue: |int| {
        Ok(Numeric::Int(int.state.catcodes.get_scheme().endlinechar as i32))
    }
};

pub static ESCAPECHAR: NumAssValue = NumAssValue {
    name:"escapechar",
    _assign: |_,int,global| {
        int.read_eq();
        let num = int.read_number()? as u8;
        log!("\\escapechar: {}",num);
        int.state.catcodes.set_escape(num,global);
        Ok(())
    },
    _getvalue: |int| {
        Ok(Numeric::Int(int.state.catcodes.get_scheme().escapechar as i32))
    }
};

pub static INPUT: PrimitiveExecutable = PrimitiveExecutable {
    name:"input",
    expandable:true,
    _apply:|rf,int| {
        use std::process::Command;
        use std::str;
        let filename = int.read_string()?;
        if filename.starts_with("|kpsewhich ") {
            let pwd = int.jobinfo.in_file().display().to_string();
            let args = filename[11..].split(" ");
            let out = if cfg!(target_os = "windows") {
                Command::new("cmd").current_dir(&pwd).env("PWD",&pwd).env("CD",&pwd).args(&["/C",&filename[1..]])//args.collect::<Vec<&str>>())
                    .output().expect("kpsewhich not found!")
                    .stdout
            } else {
                Command::new("kpsewhich").current_dir(&pwd).env("PWD",&pwd).env("CD",&pwd).args(args.collect::<Vec<&str>>())
                    .output().expect("kpsewhich not found!")
                    .stdout
            };
            let ret = std::str::from_utf8(out.as_slice()).unwrap().trim();
            int.requeue(int.eof_token());
            int.insert_every(&EVERYEOF);
            rf.2 = crate::interpreter::string_to_tokens(ret.into());
            Ok(())
        } else {
            let file = int.get_file(&filename)?;
            int.push_file(file);
            Ok(())
        }
    }
};

//pub static mut LOGSOON : u8 = 0;

pub static BEGINGROUP : PrimitiveExecutable = PrimitiveExecutable {
    name:"begingroup",
    expandable:false,
    _apply:|_rf,int| {
        int.state.push(int.stomach,GroupType::Begingroup);
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
        Ok(Numeric::Int(((time.hour() * 60) + time.minute()) as i32))
    },
    name: "time"
};

pub static YEAR : NumericCommand = NumericCommand {
    name:"year",
    _getvalue: |int| {
        Ok(Numeric::Int(int.jobinfo.time.year() as i32))
    }
};

pub static MONTH : NumericCommand = NumericCommand {
    name:"month",
    _getvalue: |int| {
        Ok(Numeric::Int(int.jobinfo.time.month() as i32))
    }
};

pub static DAY : NumericCommand = NumericCommand {
    name:"day",
    _getvalue: |int| {
        Ok(Numeric::Int(int.jobinfo.time.day() as i32))
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

fn get_inrv(int:&mut Interpreter,withint:bool) -> Result<(i32,Numeric,Numeric),TeXError> {
    use crate::commands::PrimitiveTeXCommand::*;
    int.expand_until(true)?;
    let cmd = int.read_command_token()?;
    let (index,num,val) : (i32,Numeric,Numeric) = match *int.get_command(&cmd.cmdname())?.orig {
        AV(AssignableValue::Register(i)) => {
            int.read_keyword(vec!("by"))?;
            ((i as i32),Numeric::Int(int.state.registers.get(&(i as i32))),int.read_number_i(false)?)
        }
        AV(AssignableValue::PrimReg(r)) => {
            int.read_keyword(vec!("by"))?;
            (-(r.index as i32), Numeric::Int(int.state.registers.get(&-(r.index as i32))), int.read_number_i(false)?)
        }
        AV(AssignableValue::Int(c)) if *c == COUNT => {
            let i = int.read_number()? as u16;
            int.read_keyword(vec!("by"))?;
            (i as i32,Numeric::Int(int.state.registers.get(&(i as i32))),int.read_number_i(false)?)
        }
        AV(AssignableValue::Int(c)) if *c == DIMEN => {
            let i = int.read_number()? as u16;
            int.read_keyword(vec!("by"))?;
            (i as i32,Numeric::Dim(int.state.dimensions.get(&(i as i32))), if withint {int.read_number_i(false)?} else {Numeric::Dim(int.read_dimension()?)})
        }
        AV(AssignableValue::Int(c)) if *c == SKIP => {
            let i = int.read_number()? as u16;
            int.read_keyword(vec!("by"))?;
            (i as i32,Numeric::Skip(int.state.skips.get(&(i as i32))), if withint {int.read_number_i(false)?} else {Numeric::Skip(int.read_skip()?)})
        }
        AV(AssignableValue::Dim(i)) => {
            int.read_keyword(vec!("by"))?;
            (i as i32,Numeric::Dim(int.state.dimensions.get(&(i as i32))), if withint {int.read_number_i(false)?} else {Numeric::Dim(int.read_dimension()?)})
        }
        AV(AssignableValue::PrimDim(r)) => {
            int.read_keyword(vec!("by"))?;
            (-(r.index as i32), Numeric::Dim(int.state.dimensions.get(&-(r.index as i32))),if withint {int.read_number_i(false)?} else {Numeric::Dim(int.read_dimension()?)})
        }
        AV(AssignableValue::Skip(i)) => {
            int.read_keyword(vec!("by"))?;
            (i as i32, Numeric::Skip(int.state.skips.get(&(i as i32))),if withint {int.read_number_i(false)?} else {Numeric::Skip(int.read_skip()?)})
        }
        AV(AssignableValue::PrimSkip(r)) => {
            int.read_keyword(vec!("by"))?;
            (-(r.index as i32), Numeric::Skip(int.state.skips.get(&-(r.index as i32))),if withint {int.read_number_i(false)?} else {Numeric::Skip(int.read_skip()?)})
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
        match num {
            Numeric::Int(i) => int.state.registers.set(index, i / div.get_i32(), global),
            Numeric::Dim(i) => int.state.dimensions.set(index, i / div.get_i32(),global),
            Numeric::Skip(i) => int.state.skips.set(index, i / div.get_i32(),global),
            Numeric::MuSkip(i) => int.state.muskips.set(index, i / div.get_i32(),global),
            _ => TeXErr!("Should be unreachable!")
        };
        Ok(())
    }
};
pub static MULTIPLY : PrimitiveAssignment = PrimitiveAssignment {
    name: "multiply",
    _assign: |_,int,global| {
        let (index,num,fac) = get_inrv(int,true)?;
        log!("\\multiply sets {} to {}",index,num*fac);
        match num {
            Numeric::Int(_) => int.state.registers.set(index,match num * fac.as_int() {
                Numeric::Int(i) => i,
                _ => TeXErr!("Should be unreachable!")
            }, global),
            Numeric::Dim(_) => int.state.dimensions.set(index,match num * fac.as_int() {
                Numeric::Dim(i) => i,
                _ => TeXErr!("Should be unreachable!")
            },global),
            Numeric::Skip(_) => int.state.skips.set(index, match num * fac.as_int() {
                Numeric::Skip(i) => i,
                _ => TeXErr!("Should be unreachable!")
            },global),
            Numeric::MuSkip(_) => int.state.muskips.set(index, match num * fac.as_int() {
                Numeric::MuSkip(i) => i,
                _ => TeXErr!("Should be unreachable!")
            },global),
            _ => TeXErr!("Should be unreachable!")
        };
        Ok(())
    }
};
pub static ADVANCE : PrimitiveAssignment = PrimitiveAssignment {
    name: "advance",
    _assign: |_,int,global| {
        let (index,num,sum) = get_inrv(int,false)?;
        log!("\\advance sets {} to {}",index,num + sum);
        match (num,sum) {
            (Numeric::Int(num),Numeric::Int(sum)) => int.state.registers.set(index,num + sum,global),
            (Numeric::Int(num),Numeric::Dim(sum)) => int.state.registers.set(index,num+sum,global),
            (Numeric::Dim(num),Numeric::Dim(sum)) => int.state.dimensions.set(index,num + sum,global),
            (Numeric::Skip(num),Numeric::Skip(sum)) => int.state.skips.set(index,num + sum,global),
            (Numeric::MuSkip(num),Numeric::MuSkip(sum)) => int.state.muskips.set(index,num + sum,global),
            _ => TeXErr!("Should be unreachable!")
        };
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
            AV(AssignableValue::PrimReg(i)) => stt(int.state.registers.get(&-(i.index as i32)).to_string().into()),
            AV(AssignableValue::Register(i)) => stt(int.state.registers.get(&(*i as i32)).to_string().into()),
            AV(AssignableValue::Toks(i)) => int.state.toks.get(&(*i as i32)),
            AV(AssignableValue::PrimToks(r)) => int.state.toks.get(&-(r.index as i32)),
            AV(AssignableValue::Tok(r)) => (r._getvalue)(int)?,
            Char(tk) => stt(tk.char.to_string().into()),
            MathChar(i) => stt(i.to_string().into()),
            AV(AssignableValue::Dim(i)) => stt(dimtostr(int.state.dimensions.get(&(*i as i32))).into()),
            AV(AssignableValue::PrimDim(r)) => stt(dimtostr(int.state.dimensions.get(&-(r.index as i32))).into()),
            AV(AssignableValue::Skip(i)) => stt(int.state.skips.get(&(*i as i32)).to_string().into()),
            AV(AssignableValue::PrimSkip(r)) => stt(int.state.skips.get(&-(r.index as i32)).to_string().into()),
            AV(AssignableValue::FontRef(f)) => vec!(Token::new(0,CategoryCode::Escape,Some(f.name.clone()),None,true)),
            AV(AssignableValue::Font(f)) if **f == FONT =>
                vec!(Token::new(0,CategoryCode::Escape,Some(int.state.currfont.get(&()).name.clone()),None,true)),
            AV(AssignableValue::Font(f)) => {
                let font = (f._getvalue)(int)?;
                vec!(Token::new(0,CategoryCode::Escape,Some(font.name.clone()),None,true))
            }
            Primitive(p) if **p == PARSHAPE => {
                stt(int.state.parshape.get(&()).len().to_string().into())
            }
            Primitive(p) if **p == HANGINDENT => {
                stt(dimtostr(int.state.hangindent.get(&())).into())
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
                (wi._apply)(&mut int.state,int.params)?;
                Ok(())
            }
            PrimitiveTeXCommand::Primitive(x) if *x == PDFXFORM || *x==PDFOBJ || *x == OPENIN || *x == CLOSEIN => {
                int.requeue(next);
                Ok(())
            }
            _ => TeXErr!("TODO: \\immediate ...")
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
            _apply: Box::new(move |state: &mut State,_| {
                state.file_openout(num,file.clone())
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
        int.state.file_openin(num,file)?;
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
            _apply: Box::new(move |state: &mut State,_| {
                state.file_closeout(num);Ok(())
            })
        })
    }
};

pub static CLOSEIN: PrimitiveExecutable = PrimitiveExecutable {
    _apply: |_,int| {
        let num = int.read_number()? as u8;
        log!("\\closein {}",num);
        int.state.file_closein(num)?;
        Ok(())
    },
    name:"closein",
    expandable:false,
};

pub static READ: PrimitiveAssignment = PrimitiveAssignment {
    name:"read",
    _assign: |_,int,global| {
        let index = int.read_number()? as u8;
        match int.read_keyword(vec!("to"))? {
            Some(_) => (),
            None => TeXErr!("\"to\" expected in \\read")
        }
        let newcmd = int.read_command_token()?;
        let toks = int.state.file_read(index,true)?;
        let cmd = PrimitiveTeXCommand::Def(DefMacro {
            protected: false,
            long: false,
            sig: Signature {
                elems: vec![],
                endswithbrace: false,
                arity: 0
            },
            ret: toks
        }).as_command();
        int.change_command(newcmd.cmdname(),Some(cmd),global);
        Ok(())
    }
};

pub static READLINE: PrimitiveAssignment = PrimitiveAssignment {
    name:"readline",
    _assign: |_,int,global| {
        let index = int.read_number()? as u8;
        match int.read_keyword(vec!("to"))? {
            Some(_) => (),
            None => TeXErr!("\"to\" expected in \\read")
        }
        let newcmd = int.read_command_token()?;
        /*if int.current_line() == "/home/jazzpirate/work/Software/ext/sTeX/doc/manual.tex (157, 1)" {
            unsafe { crate::LOG = true }
        }*/
        let toks = int.state.file_read_line(index)?;
        let cmd = PrimitiveTeXCommand::Def(DefMacro {
            protected: false,
            long: false,
            sig: Signature {
                elems: vec![],
                endswithbrace: false,
                arity: 0
            },
            ret: toks
        }).as_command();
        int.change_command(newcmd.cmdname(),Some(cmd),global);
        Ok(())
    }
};


pub static WRITE: ProvidesExecutableWhatsit = ProvidesExecutableWhatsit {
    name: "write",
    _get: |_tk, int| {
        let num = int.read_number()? as u8;
        let next = int.next_token();
        if next.catcode != CategoryCode::BeginGroup {
            TeXErr!(next => "Begin group token expected after \\write")
        }
        let ret = int.read_token_list(true,true,true,true)?;
        let string = int.tokens_to_string(&ret) + "\n".into();
        return Ok(ExecutableWhatsit {
            _apply: Box::new(move |state,params| {
                state.file_write(num,string.clone(),params)
            })
        });
    }
};

pub static MESSAGE: PrimitiveExecutable = PrimitiveExecutable {
    name:"message",
    expandable:false,
    _apply:|_,int| {
        let next = int.next_token();
        if next.catcode != CategoryCode::BeginGroup {
            TeXErr!(next => "Begin group token expected after \\message")
        }
        let ret = int.read_token_list(true,false,false,true)?;
        let string = int.tokens_to_string(&ret);
        int.params.message(string.to_string().as_str());
        Ok(())
    }
};

pub static NOEXPAND: PrimitiveExecutable = PrimitiveExecutable {
    name:"noexpand",
    expandable:true,
    _apply:|_cs,int| {
        //int.assert_has_next()?;
        let next = int.next_token();
        int.requeue(next.deexpand());
        Ok(())
    }
};

pub static EXPANDAFTER: PrimitiveExecutable = PrimitiveExecutable {
    name:"expandafter",
    expandable:true,
    _apply:|rf,int| {
        //int.assert_has_next()?;
        let tmp = int.next_token();
        //int.assert_has_next()?;
        let next = int.next_token();
        match next.catcode {
            CategoryCode::Escape | CategoryCode::Active => {
                let cmd = match int.state.commands.get(&next.cmdname()) {
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
        //int.assert_has_next()?;
        let next = int.next_token();
        let string = match next.catcode {
            CategoryCode::Active | CategoryCode::Escape => {
                match int.state.commands.get(&next.cmdname()) {
                    None => "undefined".into(),
                    Some(p) => p.meaning(&int.state.catcodes.get_scheme())
                }
            }
            _ => PrimitiveTeXCommand::Char(next).as_command().meaning(&int.state.catcodes.get_scheme())
        };
        rf.2 = crate::interpreter::string_to_tokens(string);
        Ok(())
    }
};

pub static STRING: PrimitiveExecutable = PrimitiveExecutable {
    name:"string",
    expandable:true,
    _apply:|rf,int| {
        //int.assert_has_next()?;
        let next = int.next_token();
        log!("\\string: {}",next);
        rf.2 = match next.catcode {
            CategoryCode::Escape => {
                let esc = int.state.catcodes.get_scheme().escapechar;
                let s : TeXString = if esc == 255 {"".into()} else {esc.into()};
                crate::interpreter::string_to_tokens(s + next.cmdname().into())
            }
            CategoryCode::Space => vec!(next),
            _ => vec!(Token::new(next.char,CategoryCode::Other,Some(next.name().clone()),next.reference.clone(),true))
        };
        Ok(())
    }
};

pub static MATHCHARDEF: PrimitiveAssignment = PrimitiveAssignment {
    name:"mathchardef",
    _assign: |_,int,global| {
        let chartok = int.read_command_token()?;
        int.read_eq();
        let num = int.read_number()?;
        let cmd = PrimitiveTeXCommand::MathChar(num as u32).as_command();
        int.change_command(chartok.cmdname(),Some(cmd),global);
        Ok(())
    }
};

pub fn csname(int : &mut Interpreter) -> Result<TeXString,TeXError> {
    int.state.incs += 1;
    let incs = int.state.incs;
    let mut cmdname : TeXString = "".into();
    log!("\\csname: {}",int.preview());
    while incs == int.state.incs && int.has_next() {
        let next = int.next_token();
        match next.catcode {
            CategoryCode::Escape | CategoryCode::Active => {
                let cmd = int.get_command(&next.cmdname())?;
                match *cmd.orig {
                    PrimitiveTeXCommand::Primitive(ec) if *ec == ENDCSNAME => {
                        if int.state.incs <= 0 {
                            TeXErr!(next => "spurious \\endcsname")
                        }
                        int.state.incs -=1;
                    }
                    PrimitiveTeXCommand::Primitive(ec) if *ec == CSNAME => {
                        cmd.expand(next,int)?;
                    }
                    _ if next.expand && cmd.expandable(true) =>
                        cmd.expand(next,int)?,
                    _ if next.catcode == CategoryCode::Escape => {
                        let esc = int.state.catcodes.get_scheme().escapechar;
                        if esc != 255 {
                            cmdname += esc
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
        let mut erf = rf.get_ref();
        let ret = Token::new(int.state.catcodes.get_scheme().escapechar,CategoryCode::Escape,Some(cmdname.clone()),None,true)
            .copied(&mut erf);
        match int.state.commands.get(&cmdname) {
            Some(_) => (),
            None => {
                let cmd = PrimitiveTeXCommand::Primitive(&RELAX).as_command();
                int.change_command(cmdname,Some(cmd),false)
            }
        }
        rf.2.push(ret);
        Ok(())
    }
};

pub static ENDCSNAME: PrimitiveExecutable = PrimitiveExecutable {
    name:"endcsname",
    expandable:false,
    _apply:|tk,int| {
        if int.state.incs <= 0 {
            TeXErr!(tk.0.clone() => "spurious \\endcsname")
        }
        int.state.incs -=1;
        Ok(())
    }
};

pub static LATEX3ERROR: PrimitiveExecutable = PrimitiveExecutable {
    name:"LaTeX3 error:",
    expandable:false,
    _apply:|tk,int| { (ERRMESSAGE._apply)(tk,int) }
};

pub static ERRMESSAGE: PrimitiveExecutable = PrimitiveExecutable {
    name:"errmessage",
    expandable:false,
    _apply:|_,int| {
        //TeXErr!(tk.0.clone() => "temp {}",int.preview());
        let next = int.next_token();
        if next.catcode != CategoryCode::BeginGroup {
            TeXErr!(next => "Begin group token expected after \\errmessage")
        }
        let ret = int.read_token_list(true,false,false,true)?;
        let string = int.tokens_to_string(&ret);
        let eh = int.state.toks.get(&-(ERRHELP.index as i32));
        let rethelp : TeXString = /*if !eh.is_empty() {
           /* eh.push(Token::new(0,CategoryCode::EndGroup,None,None,false));
            int.push_tokens(eh);
            unsafe {crate::LOG = true};
            let rethelp = int.read_token_list(true,false,false,true)?;
            int.tokens_to_string(&rethelp) */ "".into()
        } else */{"".into()};
        TeXErr!("{}\n{}",string.to_string(),rethelp)
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

fn expr_loop(int: &mut Interpreter,getnum : fn(&mut Interpreter) -> Result<Numeric,TeXError>) -> Result<Numeric,TeXError> {
    match expr_loop_main(int,getnum)? {
        Numeric::BigInt(i) => Ok(Numeric::Int(i as i32)),
        o => Ok(o)
    }
}

fn expr_loop_main(int: &mut Interpreter,getnum : fn(&mut Interpreter) -> Result<Numeric,TeXError>) -> Result<Numeric,TeXError> {
    int.skip_ws();
    log!("expr_loop: >{}",int.preview());
    let mut first = expr_loop_inner(int,getnum)?;
    loop {
        match int.read_keyword(vec!("+","-","*","/"))? {
            Some(s) if s == "+" => {
                let mut second = expr_loop_inner(int,getnum)?;
                'innera: loop {
                    match int.read_keyword(vec!("*","/"))? {
                        None => {
                            first = first + second;
                            break 'innera
                        }
                        Some(s) if s == "*" => {
                            let third = expr_loop_inner(int,|int| Ok(Numeric::BigInt(int.read_number()? as i64)))?;
                            second = second * third
                        }
                        Some(_) => {
                            let third = expr_loop_inner(int,|int| Ok(Numeric::BigInt(int.read_number()? as i64)))?;
                            second = second / third
                        }
                    }
                }
            }
            Some(s) if s == "-" => {
                let mut second = expr_loop_inner(int,getnum)?;
                'innerb: loop {
                    match int.read_keyword(vec!("*","/"))? {
                        None => {
                            first = first - second;
                            break 'innerb
                        }
                        Some(s) if s == "*" => {
                            let third = expr_loop_inner(int,|int| Ok(Numeric::BigInt(int.read_number()? as i64)))?;
                            second = second * third
                        }
                        Some(_) => {
                            let third = expr_loop_inner(int,|int| Ok(Numeric::BigInt(int.read_number()? as i64)))?;
                            second = second / third
                        }
                    }
                }
            }
            Some(s) if s == "*" => {
                let second = expr_loop_inner(int,|int| Ok(Numeric::BigInt(int.read_number()? as i64)))?;
                first = first * second
            }
            Some(_) => {
                let second = expr_loop_inner(int,|int| Ok(Numeric::BigInt(int.read_number()? as i64)))?;
                first = first / second
            }
            None => {
                log!("    >{}",int.preview());
                return Ok(first)
            }
        }
    }
}

fn expr_loop_inner(int: &mut Interpreter,getnum : fn(&mut Interpreter) -> Result<Numeric,TeXError>) -> Result<Numeric,TeXError> {
    match int.read_keyword(vec!("("))? {
        Some(_) => {
            let r = expr_loop_main(int,getnum)?;
            match int.read_keyword(vec!(")"))? {
                Some(_) => Ok(r),
                None => TeXErr!("Expected ')'")
            }
        }
        None => match (getnum)(int)? {
            Numeric::Int(i) => Ok(Numeric::BigInt(i as i64)),
            o => Ok(o)
        }
    }
}

pub static NUMEXPR: NumericCommand = NumericCommand {
    name:"numexpr",
    _getvalue: |int| {
        log!("\\numexpr starts: >{}",int.preview());
        let ret =expr_loop(int,|i| i.read_number_i(false))?;
        int.eat_relax();
        log!("\\numexpr: {}",ret);
        Ok(ret)
    }
};

pub static DIMEXPR: NumericCommand = NumericCommand {
    name:"dimexpr",
    _getvalue: |int| {
        log!("\\dimexpr starts: >{}",int.preview());
        let ret =expr_loop(int,|i| Ok(Numeric::Dim(i.read_dimension()?)))?;
        int.eat_relax();
        log!("\\dimexpr: {}",ret);
        Ok(ret)
    }
};

pub static GLUEEXPR: NumericCommand = NumericCommand {
    name:"glueexpr",
    _getvalue: |int| {
        log!("\\glueexpr starts: >{}",int.preview());
        let ret =expr_loop(int,|i| Ok(Numeric::Skip(i.read_skip()?)))?;
        int.eat_relax();
        log!("\\glueexpr: {}",ret);
        Ok(ret)
    }
};

pub static MUEXPR: NumericCommand = NumericCommand {
    name:"muexpr",
    _getvalue: |int| {
        log!("\\muexpr starts: >{}",int.preview());
        let ret =expr_loop(int,|i| Ok(Numeric::MuSkip(i.read_muskip()?)))?;
        int.eat_relax();
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
        let space = Token::new(32,CategoryCode::Space,None,None,false);
        let escape = match int.state.catcodes.get_scheme().escapechar {
            255 => None,
            o => Some(Token::new(o, CategoryCode::Other, None, None, false))
        };
        for t in tkl {
            match t.catcode {
                CategoryCode::Space | CategoryCode::EOL => exp.2.push(space.clone()),
                CategoryCode::Parameter => {
                    exp.2.push(Token::new(t.char,CategoryCode::Other,None,None,false));
                    exp.2.push(Token::new(t.char,CategoryCode::Other,None,None,false))
                },
                CategoryCode::Escape => {
                    for tk in &escape { exp.2.push(tk.clone()) }
                    for t in t.name().iter() {
                        exp.2.push(Token::new(*t,CategoryCode::Other,None,None,false));
                    }
                    if t.name().len() > 1 { exp.2.push(space.clone()) }
                    else if t.name().len() == 1 {
                        let c = t.name().iter().first().unwrap();
                        match int.state.catcodes.get_scheme().get_code(*c) {
                            CategoryCode::Letter => exp.2.push(space.clone()),
                            _ => ()
                        }
                    }
                }
                _ => {
                    exp.2.push(Token::new(t.char,CategoryCode::Other,None,None,false));
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
        if num2 == 0 {
            int.state.lccodes.set(num1, num1, global);
        } else {
            int.state.lccodes.set(num1, num2, global);
        }
        Ok(())
    },
    _getvalue: |int| {
        let char = int.read_number()? as u8;
        Ok(Numeric::Int(int.state.lccodes.get(&char) as i32))
    }
};

pub static UCCODE: NumAssValue = NumAssValue {
    name: "uccode",
    _assign: |_, int, global| {
        let num1 = int.read_number()? as u8;
        int.read_eq();
        let num2 = int.read_number()? as u8;
        if num2 == 0 {
            int.state.uccodes.set(num1, num1, global);
        } else {
            int.state.uccodes.set(num1, num2, global);
        }
        Ok(())
    },
    _getvalue: |int| {
        let char = int.read_number()? as u8;
        Ok(Numeric::Int(int.state.uccodes.get(&char) as i32))
    }
};

pub static LOWERCASE: PrimitiveExecutable = PrimitiveExecutable {
    name:"lowercase",
    expandable:false,
    _apply:|rf,int| {
        let mut erf = rf.get_ref();
        for t in int.read_balanced_argument(false,false,false,true)? {
            match t.catcode {
                CategoryCode::Escape => rf.2.push(t.copied(&mut erf)),
                o => {
                    let lc = int.state.lccodes.get(&t.char);
                    rf.2.push(Token::new(lc,o,None,Some(erf.as_src_ref()),true))
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
        let mut erf = rf.get_ref();
        for t in int.read_balanced_argument(false,false,false,true)? {
            match t.catcode {
                CategoryCode::Escape => rf.2.push(t.copied(&mut erf)),
                o => {
                    let uc = int.state.uccodes.get(&t.char);
                    rf.2.push(Token::new(uc,o,None,Some(erf.as_src_ref()),true))
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
        let ff = int.state.get_font(int.jobinfo.in_file(),name.into())?;
        let at = match int.read_keyword(vec!("at","scaled"))? {
            Some(s) if s == "at" => Some(int.read_dimension()?),
            Some(s) if s == "scaled" => Some(round_f((ff.as_ref().size as f64) * match int.read_number_i(true)? {
                Numeric::Float(f) => f,
                Numeric::Dim(i) => (i as f64) / 65536.0,
                _ => TeXErr!("TODO: \\font")
            })),
            _ => None
        };
        let font = Font::new(ff,at,cmd.cmdname());
        int.change_command(cmd.cmdname(),Some(PrimitiveTeXCommand::AV(AssignableValue::FontRef(font)).as_command()),global);
        Ok(())
    },
    _getvalue: |int| {
        Ok(int.state.currfont.get(&()))
    }
};

pub static TEXTFONT: FontAssValue = FontAssValue {
    name:"textfont",
    _assign: |_rf,int,global| {
        let ind = int.read_number()?;
        if ind < 0 || ind > 15 {
            TeXErr!("\\textfont expected 0 <= n <= 15; got: {}",ind)
        }
        let f = read_font(int)?;
        int.state.textfonts.set(ind as usize,f,global);
        Ok(())
    },
    _getvalue: |int| {
        let ind = int.read_number()?;
        if ind < 0 || ind > 15 {
            TeXErr!("\\textfont expected 0 <= n <= 15; got: {}",ind)
        }
        Ok(int.state.textfonts.get(&(ind as usize)))
    }
};

pub static SCRIPTFONT: FontAssValue = FontAssValue {
    name:"scriptfont",
    _assign: |_rf,int,global| {
        let ind = int.read_number()?;
        if ind < 0 || ind > 15 {
            TeXErr!("\\scriptfont expected 0 <= n <= 15; got: {}",ind)
        }
        let f = read_font(int)?;
        int.state.scriptfonts.set(ind as usize,f,global);
        Ok(())
    },
    _getvalue: |int| {
        let ind = int.read_number()?;
        if ind < 0 || ind > 15 {
            TeXErr!("\\scriptfont expected 0 <= n <= 15; got: {}",ind)
        }
        Ok(int.state.scriptfonts.get(&(ind as usize)))
    }
};
pub static SCRIPTSCRIPTFONT: FontAssValue = FontAssValue {
    name:"scriptscriptfont",
    _assign: |_rf,int,global| {
        let ind = int.read_number()?;
        if ind < 0 || ind > 15 {
            TeXErr!("\\scriptscriptfont expected 0 <= n <= 15; got: {}",ind)
        }
        let f = read_font(int)?;
        int.state.scriptscriptfonts.set(ind as usize,f,global);
        Ok(())
    },
    _getvalue: |int| {
        let ind = int.read_number()?;
        if ind < 0 || ind > 15 {
            TeXErr!("\\scriptscriptfont expected 0 <= n <= 15; got: {}",ind)
        }
        Ok(int.state.scriptscriptfonts.get(&(ind as usize)))
    }
};


pub fn read_font<'a>(int : &mut Interpreter) -> Result<Arc<Font>,TeXError> {
    int.expand_until(true)?;
    let tk = int.read_command_token()?;
    let cmd = int.get_command(&tk.cmdname())?;
    match &*cmd.orig {
        PrimitiveTeXCommand::AV(AssignableValue::FontRef(f)) =>
            Ok(f.clone()),
        PrimitiveTeXCommand::AV(AssignableValue::Font(f)) =>
            (f._getvalue)(int),
        PrimitiveTeXCommand::Ass(p) if **p == NULLFONT =>
            Ok(NULL_FONT.try_with(|x| x.clone()).unwrap()),
        _ => TeXErr!(tk => "Font expected!")
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
        f.inner.write().unwrap().hyphenchar = d as u16;
        Ok(())
    },
    _getvalue: |int| {
        let f = read_font(int)?;
        let x = f.inner.read().unwrap().hyphenchar as i32;
        Ok(Numeric::Int(x))
    }
};

pub static SKEWCHAR: NumAssValue = NumAssValue {
    name:"skewchar",
    _assign: |_rf,int,_global| {
        let f = read_font(int)?;
        int.read_eq();
        let d = int.read_number()?;
        f.inner.write().unwrap().skewchar = d as u16;
        Ok(())
    },
    _getvalue: |int| {
        let f = read_font(int)?;
        let x = f.inner.read().unwrap().skewchar as i32;
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
        Ok(Numeric::Int(int.line_no() as i32))
    },
    name:"inputlineno",
};

pub static LASTSKIP: NumericCommand = NumericCommand {
    name:"lastskip",
    _getvalue: |int| {
        match int.stomach.last_whatsit() {
            Some(Whatsit::Simple(SimpleWI::VSkip(s))) => Ok(Numeric::Skip(s.skip)),
            Some(Whatsit::Simple(SimpleWI::HSkip(s))) => Ok(Numeric::Skip(s.skip)),
            _ => Ok(Numeric::Skip(Skip {
                base:0,stretch:None,shrink:None
            }))
        }
    },
};

pub static LASTKERN: NumericCommand = NumericCommand {
    name:"lastkern",
    _getvalue: |int| {
        match int.stomach.last_whatsit() {
            Some(Whatsit::Simple(SimpleWI::VKern(s))) => Ok(Numeric::Dim(s.dim)),
            Some(Whatsit::Simple(SimpleWI::HKern(s))) => Ok(Numeric::Dim(s.dim)),
            _ => Ok(Numeric::Dim(0))
        }
    },
};

pub static UNKERN: PrimitiveExecutable = PrimitiveExecutable {
    name:"unkern",
    expandable:false,
    _apply:|_,int| {
        let remove = match int.stomach.last_whatsit() {
            Some(Whatsit::Simple(SimpleWI::VKern(_) | SimpleWI::HKern(_))) => true,
            _ => false
        };
        if remove {int.stomach.drop_last()}
        Ok(())
    }
};

pub static UNPENALTY: PrimitiveExecutable = PrimitiveExecutable {
    name:"unpenalty",
    expandable:false,
    _apply:|_,int| {
        let remove = match int.stomach.last_whatsit() {
            Some(Whatsit::Simple(SimpleWI::Penalty(_))) => true,
            _ => false
        };
        if remove {int.stomach.drop_last()}
        Ok(())
    }
};

pub static SETBOX: PrimitiveAssignment = PrimitiveAssignment {
    name:"setbox",
    _assign: |_rf,int,global| {
        let index = int.read_number()? as u16;
        int.read_eq();
        int.state.insetbox = true;
        let wi = int.read_box()?;
        int.state.boxes.set(index as i32,wi,global);
        Ok(())
    }
};

pub static HBOX: ProvidesBox = ProvidesBox {
    name:"hbox",
    _get: |tk,int| {
        let (spread,width) = match int.read_keyword(vec!("to","spread"))? {
            Some(s) if s == "to" => (0 as i32,Some(int.read_dimension()?)),
            Some(s) if s == "spread" => (int.read_dimension()?,None),
            _ => (0 as i32,None)
        };
        int.eat_relax();
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
            Some(s) if s == "to" => (0 as i32,Some(int.read_dimension()?)),
            Some(s) if s == "spread" => (int.read_dimension()?,None),
            _ => (0 as i32,None)
        };
        int.eat_relax();
        let ret = int.read_whatsit_group(BoxMode::V,true)?;
        /*if ret.is_empty() {Ok(TeXBox::Void)} else*/ {
            Ok(TeXBox::V(VBox {
                children: ret,
                tp: VBoxType::V,
                spread,
                _width: None,
                _height: height,
                _depth: None,
                rf : int.update_reference(tk)
            }))
        }
    }
};

pub static VTOP: ProvidesBox = ProvidesBox {
    name:"vtop",
    _get: |tk,int| {
        let bx = (VBOX._get)(tk,int)?;
        match bx {
            TeXBox::V(mut vb) => {
                let lineheight = int.state.skips.get(&-(LINESKIP.index as i32)).base;
                vb.tp = VBoxType::Top(lineheight);
                Ok(TeXBox::V(vb))
            }
            _ => TeXErr!("Should be unreachable!")
        }
    }
};

pub static VCENTER: ProvidesBox = ProvidesBox {
    name:"vcenter",
    _get: |tk,int| {
        let bx = (VBOX._get)(tk,int)?;
        match bx {
            TeXBox::V(mut vb) => {
                vb.tp = VBoxType::Center;
                Ok(TeXBox::V(vb))
            }
            _ => TeXErr!("Should be unreachable!")
        }
    }
};

pub static VSPLIT: ProvidesBox = ProvidesBox {
    name:"vsplit",
    _get:|_,int| {
        let boxnum = int.read_number()?;
        match int.read_keyword(vec!("to"))? {
            Some(_) => (),
            None => TeXErr!("Expected \"to\" after \\vsplit")
        }
        let target = int.read_dimension()?;
        let vbox = match int.state.boxes.take(boxnum) {
            TeXBox::Void => return Ok(TeXBox::Void),
            TeXBox::V(vb) => vb,
            _ => TeXErr!("Cannot \\vsplit horizontal box")
        };
        let mut ret = VBox {
            children: vec!(),
            tp: vbox.tp,
            spread: vbox.spread,
            _width: vbox._width,
            _height: Some(target),
            _depth: vbox._depth,
            rf: None
        };
        let mut rest = VBox {
            children: vec!(),
            tp: vbox.tp,
            spread: vbox.spread,
            _width: vbox._width,
            _height: None,
            _depth: vbox._depth,
            rf: None
        };
        let (first,second) = crate::stomach::split_vertical(vbox.children,target,int);
        ret.children = first;
        rest.children = second;
        int.state.boxes.set(boxnum,TeXBox::V(rest),false);
        Ok(TeXBox::V(ret))
    }
};

pub static LASTBOX: ProvidesBox = ProvidesBox {
    _get: |_tk,int| {
        match int.stomach.last_box()? {
            Some(tb) => {
                Ok(tb)
            },
            _ => Ok(TeXBox::Void)
        }
    },
    name:"lastbox",
};

pub static UNSKIP: PrimitiveExecutable = PrimitiveExecutable {
    name:"unskip",
    expandable:false,
    _apply:|_tk,int| {
        let remove = match int.stomach.last_whatsit() {
            Some(Whatsit::Simple(SimpleWI::HSkip(_) | SimpleWI::VSkip(_))) => {
                true
            },
            _ => false
        };
        if remove {int.stomach.drop_last()}
        Ok(())
    }
};

pub static COPY: ProvidesBox = ProvidesBox {
    name:"copy",
    _get: |_tk,int| {
        let ind = int.read_number()?;
        Ok(int.state.boxes.get(&(ind as i32)))
    }
};

pub static BOX: ProvidesBox = ProvidesBox {
    name:"box",
    _get: |_tk,int| {
        let ind = int.read_number()?;
        Ok(int.state.boxes.take(ind as i32))
    }
};

pub static AFTERASSIGNMENT: PrimitiveExecutable = PrimitiveExecutable {
    name:"afterassignment",
    expandable:false,
    _apply:|_tk,int| {
        let next = int.next_token();
        int.state.afterassignment = Some(next);
        Ok(())
    }
};

pub static ENDINPUT: PrimitiveExecutable = PrimitiveExecutable {
    name:"endinput",
    expandable:true,
    _apply:|tk,int| {
        int.end_input(&tk.0);
        Ok(())
    }
};

pub static TOKS: TokAssValue = TokAssValue {
    name:"toks",
    _assign: |_rf,int,global| {
        let num = int.read_number()? as u16;
        int.read_eq();
        let r = int.read_balanced_argument(false,false,false,true)?;
        int.state.toks.set(num as i32,r.iter().map(|x| x.cloned()).collect(),global);
        Ok(())
    },
    _getvalue: |int| {
        let num = int.read_number()? as u16;
        Ok(int.state.toks.get(&(num as i32)))
    }
};

pub static MATHCODE: NumAssValue = NumAssValue {
    name:"mathcode",
    _getvalue: |int| {
        let num = int.read_number()? as u8;
        let mc = int.state.mathcodes.get(&num);
        Ok(Numeric::Int(mc))
    },
    _assign: |_,int,global| {
        let i = int.read_number()? as u8;
        int.read_eq();
        let v = int.read_number()?;
        int.state.mathcodes.set(i,v,global);
        Ok(())
    }
};

pub static DELCODE: NumAssValue = NumAssValue {
    name:"delcode",
    _getvalue: |int| {
        let num = int.read_number()? as u8;
        Ok(Numeric::Int(int.state.delcodes.get(&num)))
    },
    _assign: |_,int,global| {
        let i = int.read_number()? as u8;
        int.read_eq();
        let v = int.read_number()?;
        int.state.delcodes.set(i,v,global);
        Ok(())
    }
};

pub static NULLFONT: PrimitiveAssignment = PrimitiveAssignment {
    name:"nullfont",
    _assign: |rf,int,global| {
        int.state.currfont.set((),NULL_FONT.try_with(|x| x.clone()).unwrap(),global);
        int.stomach_add(FontChange {
            font: NULL_FONT.try_with(|x| x.clone()).unwrap(),
            closes_with_group: !global,
            children: vec![],
            sourceref: int.update_reference(&rf.0)
        }.as_whatsit())
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
        int.read_argument()?;
        Ok(())
        // TODO ?
    }
};

pub static HYPHENATION: PrimitiveExecutable = PrimitiveExecutable {
    name:"hyphenation",
    expandable:false,
    _apply:|_tk,int| {
        int.read_argument()?;
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
                    let p = int.state.commands.get(&next.cmdname());
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
        TeXMode::Horizontal | TeXMode::RestrictedHorizontal | TeXMode::Math | TeXMode::Displaymath => true,
        _ => false
    },
    _get: |tk,int| {
        let mut height : Option<i32> = None;
        let mut width : Option<i32> = None;
        let mut depth : Option<i32> = None;
        loop {
            match int.read_keyword(vec!("height","width","depth"))? {
                Some(s) if s == "height" => height = Some(int.read_dimension()?),
                Some(s) if s == "width" => width = Some(int.read_dimension()?),
                Some(s) if s == "depth" => depth = Some(int.read_dimension()?),
                _ => break
            }
        }
        Ok(Whatsit::Simple(SimpleWI::VRule(VRule {
            height,width,depth,sourceref:int.update_reference(tk)
        })))
    }
};

pub static HRULE: SimpleWhatsit = SimpleWhatsit {
    name:"hrule",
    modes:|m| match m {
        TeXMode::Vertical | TeXMode::InternalVertical => true,
        _ => false
    },
    _get: |tk,int| {
        let mut height : Option<i32> = None;
        let mut width : Option<i32> = None;
        let mut depth : Option<i32> = None;
        loop {
            match int.read_keyword(vec!("height","width","depth"))? {
                Some(s) if s == "height" => height = Some(int.read_dimension()?),
                Some(s) if s == "width" => width = Some(int.read_dimension()?),
                Some(s) if s == "depth" => depth = Some(int.read_dimension()?),
                _ => break
            }
        }
        Ok(Whatsit::Simple(SimpleWI::HRule(HRule {
            width,height,depth,sourceref:int.update_reference(tk)
        })))
    }
};


pub static VFIL: SimpleWhatsit = SimpleWhatsit {
    name:"vfil",
    modes:|m| match m {
        TeXMode::Vertical | TeXMode::InternalVertical => true,
        _ => false
    },
    _get: |tk,int| {
        Ok(Whatsit::Simple(SimpleWI::VFil(VFil(int.update_reference(tk)))))
    }
};

pub static VFILL: SimpleWhatsit = SimpleWhatsit {
    name:"vfill",
    modes:|m| match m {
        TeXMode::Vertical | TeXMode::InternalVertical => true,
        _ => false
    },
    _get: |tk,int| {
        Ok(Whatsit::Simple(SimpleWI::VFill(VFill(int.update_reference(tk)))))
    }
};

pub static VSKIP: SimpleWhatsit = SimpleWhatsit {
    name:"vskip",
    modes:|m| match m {
        TeXMode::Vertical | TeXMode::InternalVertical => true,
        _ => false
    },
    _get: |tk,int| {
        log!("\\vskip >{}",int.preview());
        let sk = int.read_skip()?;
        Ok(Whatsit::Simple(SimpleWI::VSkip(VSkip {
            skip:sk,
            sourceref:int.update_reference(tk)
        })))
    }
};

pub static HSKIP: SimpleWhatsit = SimpleWhatsit {
    name:"hskip",
    modes:|m| match m {
        TeXMode::Horizontal | TeXMode::RestrictedHorizontal => true,
        TeXMode::Math | TeXMode::Displaymath => true,
        _ => false
    },
    _get: |tk,int| {
        let sk = int.read_skip()?;
        Ok(Whatsit::Simple(SimpleWI::HSkip(HSkip {
            skip:sk,sourceref:int.update_reference(tk)
        })))
    }
};

pub static HFIL: SimpleWhatsit = SimpleWhatsit {
    name:"hfil",
    modes:|m| match m {
        TeXMode::Horizontal | TeXMode::RestrictedHorizontal => true,
        TeXMode::Math | TeXMode::Displaymath => true,
        _ => false
    },
    _get: |tk,int| {
        Ok(Whatsit::Simple(SimpleWI::HFil(HFil(int.update_reference(tk)))))
    }
};

pub static HFILL: SimpleWhatsit = SimpleWhatsit {
    name:"hfill",
    modes:|m| match m {
        TeXMode::Horizontal | TeXMode::RestrictedHorizontal => true,
        TeXMode::Math | TeXMode::Displaymath => true,
        _ => false
    },
    _get: |tk,int| {
        Ok(Whatsit::Simple(SimpleWI::HFill(HFill(int.update_reference(tk)))))
    }
};

pub static PENALTY: SimpleWhatsit = SimpleWhatsit {
    name:"penalty",
    modes:|_| true,
    _get: |tk,int| {
        Ok(Whatsit::Simple(SimpleWI::Penalty(Penalty {
            penalty:int.read_number()?,
            sourceref:int.update_reference(tk)
        })))
    }
};

pub static LOWER: SimpleWhatsit = SimpleWhatsit {
    name:"lower",
    modes:|m| {match m {
        TeXMode::Horizontal | TeXMode::RestrictedHorizontal | TeXMode::Math | TeXMode::Displaymath => true,
        _ => false
    }},
    _get: |tk,int| {
        let dim = int.read_dimension()?;
        let bx = int.read_box()?;
        Ok(Whatsit::Simple(SimpleWI::Raise(Raise {
            dim:-dim,
            content: bx,
            sourceref: int.update_reference(tk)
        })))
    }
};

pub static RAISE: SimpleWhatsit = SimpleWhatsit {
    name:"raise",
    modes:|m| {match m {
        TeXMode::Horizontal | TeXMode::RestrictedHorizontal | TeXMode::Math | TeXMode::Displaymath => true,
        _ => false
    }},
    _get: |tk,int| {
        let dim = int.read_dimension()?;
        let bx = int.read_box()?;
        Ok(Whatsit::Simple(SimpleWI::Raise(Raise {
            dim,
            content: bx,
            sourceref: int.update_reference(tk)
        })))
    }
};

pub static MOVELEFT: SimpleWhatsit = SimpleWhatsit {
    name:"moveleft",
    modes:|m| {match m {
        TeXMode::Vertical | TeXMode::InternalVertical => true,
        _ => false
    }},
    _get: |tk,int| {
        let dim = int.read_dimension()?;
        let bx = int.read_box()?;
        Ok(Whatsit::Simple(SimpleWI::MoveRight(MoveRight {
            dim:-dim,
            content:bx,
            sourceref:int.update_reference(tk)
        })))
    }
};

pub static MOVERIGHT: SimpleWhatsit = SimpleWhatsit {
    name:"moveright",
    modes:|m| {match m {
        TeXMode::Vertical | TeXMode::InternalVertical => true,
        _ => false
    }},
    _get: |tk,int| {
        let dim = int.read_dimension()?;
        let bx = int.read_box()?;
        let _ = int.update_reference(tk);
        Ok(Whatsit::Simple(SimpleWI::MoveRight(MoveRight {
            dim,
            content:bx,
            sourceref:int.update_reference(tk)
        })))
    }
};

pub static KERN: SimpleWhatsit = SimpleWhatsit {
    name:"kern",
    modes:|_| { true },
    _get: |tk,int| {
        let dim = int.read_dimension()?;
        let rf = int.update_reference(tk);
        match int.state.mode {
            TeXMode::Vertical | TeXMode::InternalVertical =>
                Ok(Whatsit::Simple(SimpleWI::VKern(VKern {
                    dim,
                    sourceref: rf
                }))),
            _ =>
                Ok(Whatsit::Simple(SimpleWI::HKern(HKern {
                    dim,
                    sourceref: rf
                }))),
        }
    }
};

pub static UNVBOX: SimpleWhatsit = SimpleWhatsit {
    name:"unvbox",
    modes:|m| { m == TeXMode::Vertical || m == TeXMode::InternalVertical },
    _get: |_,int| {
        let ind = int.read_number()?;
        let bx = int.state.boxes.take(ind as i32);
        match bx {
            TeXBox::V(v) => Ok(Whatsit::Ls(v.children)),
            TeXBox::Void => Ok(Whatsit::Ls(vec!())),
            _ => TeXErr!("incompatible list can't be unboxed")
        }
    }
};

pub static UNVCOPY: SimpleWhatsit = SimpleWhatsit {
    name:"unvcopy",
    modes:|m| { m == TeXMode::Vertical || m == TeXMode::InternalVertical },
    _get: |_,int| {
        let ind = int.read_number()?;
        let bx = int.state.boxes.get(&(ind as i32));
        match bx {
            TeXBox::V(v) => Ok(Whatsit::Ls(v.children)),
            TeXBox::Void => Ok(Whatsit::Ls(vec!())),
            _ => TeXErr!("incompatible list can't be unboxed")
        }
    }
};

pub static UNHBOX: SimpleWhatsit = SimpleWhatsit {
    name:"unhbox",
    modes:|m| { m == TeXMode::Horizontal || m == TeXMode::RestrictedHorizontal || m == TeXMode::Math || m == TeXMode::Displaymath },
    _get: |_,int| {
        let ind = int.read_number()?;
        let bx = int.state.boxes.take(ind as i32);
        let mode = int.state.mode;
        match (bx,mode) {
            (TeXBox::H(h),TeXMode::Horizontal | TeXMode::RestrictedHorizontal) => Ok(Whatsit::Ls(h.children)),
            (TeXBox::Void,_) => Ok(Whatsit::Ls(vec!())),
            _ => TeXErr!("incompatible list can't be unboxed")
        }
    }
};

pub static UNHCOPY: SimpleWhatsit = SimpleWhatsit {
    name:"unhcopy",
    modes:|m| { m == TeXMode::Horizontal || m == TeXMode::RestrictedHorizontal || m == TeXMode::Math || m == TeXMode::Displaymath },
    _get: |_,int| {
        let ind = int.read_number()?;
        let bx = int.state.boxes.get(&(ind as i32));
        let mode = int.state.mode;
        match (bx,mode) {
            (TeXBox::H(h),TeXMode::Horizontal | TeXMode::RestrictedHorizontal) => Ok(Whatsit::Ls(h.children)),
            (TeXBox::Void,_) => Ok(Whatsit::Ls(vec!())),
            _ => TeXErr!("incompatible list can't be unboxed")
        }
    }
};

pub static AFTERGROUP: PrimitiveExecutable = PrimitiveExecutable {
    name:"aftergroup",
    expandable:false,
    _apply:|_,int| {
        let next = int.next_token();
        int.state.aftergroups.add(next);
        Ok(())
    }
};

pub static TEXTSTYLE: PrimitiveExecutable = PrimitiveExecutable {
    name:"textstyle",
    expandable:false,
    _apply:|_,int| {
        int.state.fontstyle.set((),FontStyle::Text,false);
        Ok(())
    }
};

pub static SCRIPTSTYLE: PrimitiveExecutable = PrimitiveExecutable {
    name:"scriptstyle",
    expandable:false,
    _apply:|_,int| {
        int.state.fontstyle.set((),FontStyle::Script,false);
        Ok(())
    }
};

pub static SCRIPTSCRIPTSTYLE: PrimitiveExecutable = PrimitiveExecutable {
    name:"scriptscriptstyle",
    expandable:false,
    _apply:|_,int| {
        int.state.fontstyle.set((),FontStyle::Scriptscript,false);
        Ok(())
    }
};

pub static SCANTOKENS: PrimitiveExecutable = PrimitiveExecutable {
    name:"scantokens",
    expandable:true,
    _apply:|tk,int| {
        let tks = int.read_balanced_argument(false,false,false,true)?;
        let str = int.tokens_to_string(&tks);
        int.push_string(tk.clone(),str,true);
        Ok(())
    }
};

pub static GLUESHRINK: NumericCommand = NumericCommand {
    name:"glueshrink",
    _getvalue:|int| {
        let sk = int.read_skip()?;
        use crate::interpreter::dimensions::SkipDim;
        match sk.shrink {
            Some(SkipDim::Pt(i) | SkipDim::Fil(i) | SkipDim::Fill(i) | SkipDim::Filll(i)) => {
                Ok(Numeric::Dim(i))
            }
            _ => Ok(Numeric::Dim(0))
        }
    }
};

pub static GLUESHRINKORDER: NumericCommand = NumericCommand {
    name:"glueshrinkorder",
    _getvalue:|int| {
        let sk = int.read_skip()?;
        use crate::interpreter::dimensions::SkipDim;
        Ok(Numeric::Int(match sk.shrink {
            Some(SkipDim::Pt(_)) => 0,
            Some(SkipDim::Fil(_)) => 1,
            Some(SkipDim::Fill(_)) => 2,
            Some(SkipDim::Filll(_)) => 3,
            _ => 0
        }))
    }
};

pub static GLUESTRETCH: NumericCommand = NumericCommand {
    name:"gluestretch",
    _getvalue:|int| {
        let sk = int.read_skip()?;
        use crate::interpreter::dimensions::SkipDim;
        match sk.stretch {
            Some(SkipDim::Pt(i) | SkipDim::Fil(i) | SkipDim::Fill(i) | SkipDim::Filll(i)) => {
                Ok(Numeric::Dim(i))
            }
            _ => Ok(Numeric::Dim(0))
        }
    }
};

pub static GLUESTRETCHORDER: NumericCommand = NumericCommand {
    name:"gluestretchorder",
    _getvalue:|int| {
        let sk = int.read_skip()?;
        use crate::interpreter::dimensions::SkipDim;
        Ok(Numeric::Int(match sk.stretch {
            Some(SkipDim::Pt(_)) => 0,
            Some(SkipDim::Fil(_)) => 1,
            Some(SkipDim::Fill(_)) => 2,
            Some(SkipDim::Filll(_)) => 3,
            _ => 0
        }))
    }
};


pub static WD: NumAssValue = NumAssValue {
    name:"wd",
    _assign: |_,int,global| {
        let index = int.read_number()? as i32;
        int.read_eq();
        let dim = int.read_dimension()?;
        let mut bx = int.state.boxes.get(&index);
        match bx {
            TeXBox::Void => (),
            TeXBox::H(ref mut hb) => hb._width = Some(dim),
            TeXBox::V(ref mut hb) => hb._width = Some(dim),
        }
        int.state.boxes.set(index,bx,global);
        Ok(())
    },
    _getvalue: |int| {
        let index = int.read_number()?;
        Ok(Numeric::Dim(int.state.boxes.get_maybe(&(index as i32)).map(|x| x.width()).unwrap_or(0)))
    }
};

pub static HT: NumAssValue = NumAssValue {
    name:"ht",
    _assign: |_,int,global| {
        let index = int.read_number()? as i32;
        int.read_eq();
        let dim = int.read_dimension()?;
        let mut bx = int.state.boxes.get(&index);
        match bx {
            TeXBox::Void => (),
            TeXBox::H(ref mut hb) => hb._height = Some(dim),
            TeXBox::V(ref mut hb) => hb._height = Some(dim),
        }
        int.state.boxes.set(index,bx,global);
        Ok(())
    },
    _getvalue: |int| {
        let index = int.read_number()?;
        Ok(Numeric::Dim(int.state.boxes.get_maybe(&(index as i32)).map(|x| x.height()).unwrap_or(0)))
    }
};

pub static DP: NumAssValue = NumAssValue {
    name:"dp",
    _assign: |_,int,global| {
        let index = int.read_number()? as i32;
        int.read_eq();
        let dim = int.read_dimension()?;
        let mut bx = int.state.boxes.get(&index);
        match bx {
            TeXBox::Void => (),
            TeXBox::H(ref mut hb) => hb._depth = Some(dim),
            TeXBox::V(ref mut hb) => hb._depth = Some(dim),
        }
        int.state.boxes.set(index,bx,global);
        Ok(())
    },
    _getvalue: |int| {
        let index = int.read_number()?;
        Ok(Numeric::Dim(int.state.boxes.get_maybe(&(index as i32)).map(|x| x.depth()).unwrap_or(0)))
    }
};

pub static PAGEGOAL: NumAssValue = NumAssValue {
    name:"pagegoal",
    _assign: |_,int,_| {
        let dim = int.read_dimension()?;
        int.state.pagegoal = dim;
        Ok(())
    },
    _getvalue: |int| {
        let ph = int.stomach.page_height();
        if ph == 0 {
            Ok(Numeric::Dim(1073741823))
        } else {
            let pg = int.state.pagegoal;
            if pg == 0 {
                Ok(Numeric::Dim(int.state.dimensions.get(&-(VSIZE.index as i32))))
            } else {
                Ok(Numeric::Dim(pg))
            }
        }
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

pub static LASTPENALTY: NumericCommand = NumericCommand {
    name:"lastpenalty",
    _getvalue: |int| {
        match int.stomach.last_whatsit() {
            Some(Whatsit::Simple(SimpleWI::Penalty(p))) => Ok(Numeric::Int(p.penalty)),
            _ => Ok(Numeric::Int(0))
        }
    }
};

pub static CURRENTGROUPLEVEL: NumericCommand = NumericCommand {
    name:"currentgrouplevel",
    _getvalue:|int| {
        Ok(Numeric::Int(int.state.stack_depth() as i32))
    }
};
// https://tex.stackexchange.com/questions/24530/what-combinations-of-mode-and-currentgrouptype-exist
pub static CURRENTGROUPTYPE: NumericCommand = NumericCommand {
    name:"currentgrouptype",
    _getvalue:|int| {
        Ok(Numeric::Int(match int.state.tp.get(&()) {
            GroupType::Begingroup if int.state.stack_depth() == 0 => 0,
            GroupType::Begingroup | GroupType::Token => 1,
            GroupType::Box(BoxMode::H) => 2,
            GroupType::Box(BoxMode::V) => 4,
            GroupType::LeftRight => 16,
            GroupType::Math | GroupType::Box(BoxMode::DM) | GroupType::Box(BoxMode::M) => 9,
            GroupType::Box(BoxMode::Void) => TeXErr!("Should be unreachable!")
        }))
    }
};

pub static VADJUST: PrimitiveExecutable = PrimitiveExecutable {
    name:"vadjust",
    expandable:false,
    _apply:|_tk,int| {
        let mut ret = int.read_whatsit_group(BoxMode::V,false)?;
        int.state.vadjust.append(&mut ret);
        Ok(())
    }
};

pub static CHAR: PrimitiveExecutable = PrimitiveExecutable {
    name:"char",
    expandable:false,
    _apply:|rf,int| {
        let num = int.read_number()? as u8;
        rf.2 = vec!(Token::new(num,CategoryCode::Other,None,Some(rf.get_ref().as_src_ref()),true));
        Ok(())
    }
};

pub static OMIT: PrimitiveExecutable = PrimitiveExecutable {
    name:"omit",
    expandable:false,
    _apply:|tk,_int| {TeXErr!(tk.0.clone() => "Unexpected \\omit")}
};

pub static CR: PrimitiveExecutable = PrimitiveExecutable {
    name:"cr",
    expandable:false,
    _apply:|tk,int| {
        let align = int.state.aligns.pop();
        int.state.aligns.push(None);
        match align {
            Some(Some(ret)) => {
                int.insert_every(&EVERYCR);
                match ENDROW.try_with(|x| int.requeue(x.clone())) {
                    Ok(_) => (),
                    _ => TeXErr!(tk.0.clone() => "Error inserting \\endrow")
                };
                tk.2 = ret;
                Ok(())
            }
            _ => TeXErr!(tk.0.clone() => "Unexpected \\cr")
        }
    }
};

thread_local! {
    pub static ENDROW : Token = {
        let mut endrow = Token::new(250,CategoryCode::Escape,Some("endtemplate".into()),None,false);
        endrow.name_opt = "relax".into();
        endrow
    };
    pub static ENDTEMPLATE : Token = {
        let mut endtemplate = Token::new(38,CategoryCode::Escape,Some("endtemplate".into()),None,false);
        endtemplate.name_opt = "relax".into();
        endtemplate
    };
    pub static ENDTEMPLATESPAN : Token = {
        let mut endtemplate = Token::new(38,CategoryCode::Escape,Some("endtemplatespan".into()),None,false);
        endtemplate.name_opt = "relax".into();
        endtemplate
    };
}

pub static SPAN: PrimitiveExecutable = PrimitiveExecutable {
    name:"span",
    expandable:false,
    _apply:|tk,int| {
        let align = int.state.aligns.pop();
        int.state.aligns.push(None);
        match align {
            Some(Some(v)) => {
                int.requeue(ENDTEMPLATESPAN.try_with(|x| x.clone()).unwrap());
                int.push_tokens(v);
                Ok(())
            }
            _ => TeXErr!(tk.0.clone() => "Misplaced \\span")
        }
    }
};

pub static CRCR: PrimitiveExecutable = PrimitiveExecutable {
    name:"crcr",
    expandable:false,
    _apply:|tk,int| {
        let align = int.state.aligns.pop();
        int.state.aligns.push(None);
        match align {
            Some(Some(v)) => {
                int.insert_every(&EVERYCR);
                int.requeue(ENDROW.try_with(|x| x.clone()).unwrap());
                tk.2 = v;
                Ok(())
            }
            _ => Ok(())
        }
    }
};

pub static NOALIGN: PrimitiveExecutable = PrimitiveExecutable {
    name:"noalign",
    expandable:false,
    _apply:|tk,_int| {TeXErr!(tk.0.clone() => "Unexpected \\noalign")}
};

fn do_align(int:&mut Interpreter,tabmode:BoxMode,betweenmode:BoxMode) -> Result<
        (Skip,Vec<(Vec<Token>,Vec<Token>,Skip)>,Vec<AlignBlock>),TeXError> {
    int.expand_until(false)?;
    //unsafe {crate::LOG = true }
    let bg = int.next_token();
    match bg.catcode {
        CategoryCode::BeginGroup => (),
        CategoryCode::Escape | CategoryCode::Active => {
            let cmd = int.get_command(&bg.cmdname())?;
            match &*cmd.orig {
                PrimitiveTeXCommand::Char(tk) if tk.catcode == CategoryCode::BeginGroup => (),
                _ => TeXErr!(bg.clone() => "Expected begin group token; found: {}",bg)
            }
        }
        _ => TeXErr!(bg.clone() => "Expected begin group token; found: {}",bg)
    }

    int.state.push(int.stomach,GroupType::Box(betweenmode));

    let mut tabskip = int.state.skips.get(&-(TABSKIP.index as i32));
    let firsttabskip = tabskip;

    let mut in_v = false;
    let mut columns: Vec<(Vec<Token>,Vec<Token>,Skip)> = vec!((vec!(),vec!(),tabskip));
    let mut recindex: Option<usize> = None;

    loop {
        let next = int.next_token();
        match next.catcode {
            CategoryCode::AlignmentTab if !in_v && columns.last().unwrap().0.is_empty() => recindex = Some(columns.len() - 1),
            CategoryCode::AlignmentTab => {
                columns.push((vec!(),vec!(),tabskip));
                in_v = false
            }
            CategoryCode::Parameter if !in_v => in_v = true,
            CategoryCode::Parameter => TeXErr!(next => "Misplaced # in alignment"),
            CategoryCode::Escape | CategoryCode::Active => {
                let proc = int.state.commands.get(&next.cmdname());
                match proc {
                    None => if in_v { columns.last_mut().unwrap().1.push(next) } else { columns.last_mut().unwrap().0.push(next) }
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
                                    let p = int.get_command(&next.cmdname())?;
                                    if p.expandable(true) {
                                        p.expand(next,int)?
                                    } else {
                                        TeXErr!(next => "Expandable command expected after \\span")
                                    }
                                }
                                _ => TeXErr!(next => "Expandable command expected after \\span")
                            }
                        }
                        _ => if in_v { columns.last_mut().unwrap().1.push(next) } else { columns.last_mut().unwrap().0.push(next) }
                    }
                }
            }
            _ => if in_v { columns.last_mut().unwrap().1.push(next) } else { columns.last_mut().unwrap().0.push(next) }
        }
    }

    let mut boxes : Vec<AlignBlock> = vec!();
    let endtemplate = ENDTEMPLATE.try_with(|x| x.clone()).unwrap();
    let endtemplatespan = ENDTEMPLATESPAN.try_with(|x| x.clone()).unwrap();
    let endrow = ENDROW.try_with(|x| x.clone()).unwrap();
    let mut inspan : bool = false;

    'table: loop {
        'preludea: loop {
            let next = int.next_token();
            match next.catcode {
                CategoryCode::EndGroup => break 'table,
                CategoryCode::Space => (),
                CategoryCode::Active | CategoryCode::Escape => {
                    let cmd = int.state.commands.get(&next.cmdname());
                    match cmd {
                        None => {
                            int.requeue(next);
                            break 'preludea
                        },
                        Some(cmd) => {
                            if cmd.expandable(false) { cmd.expand(next,int)?} else {
                                match &*cmd.orig {
                                    PrimitiveTeXCommand::Char(tk) if tk.catcode == CategoryCode::EndGroup => break 'table,
                                    PrimitiveTeXCommand::Char(tk) if tk.catcode == CategoryCode::Space => (),
                                    PrimitiveTeXCommand::Primitive(c) if **c == CRCR => (),
                                    PrimitiveTeXCommand::Primitive(c) if **c == NOALIGN => {
                                        let noalign = int.read_whatsit_group(betweenmode,false)?;
                                        boxes.push(AlignBlock::Noalign(noalign))
                                    }
                                    _ => {
                                        int.requeue(next);
                                        break 'preludea
                                    }
                                }
                            }
                        }
                    }
                }
                _ => {
                    int.requeue(next);
                    break 'preludea
                }
            }
        }

        let mut columnindex : usize = 0;
        let mut row:Vec<(Vec<Whatsit>,Skip,usize)> = vec!();
        let mut cells:usize =1;

        'row: loop {
            let mut doheader = true;
            //inV = false;
            'preludeb: loop {
                let next = int.next_token();
                match next.catcode {
                    CategoryCode::Space => (),
                    CategoryCode::Active | CategoryCode::Escape => {
                        let cmd = int.state.commands.get(&next.cmdname());
                        match cmd {
                            None => {
                                int.requeue(next);
                                break 'preludeb
                            },
                            Some(cmd) => {
                                if cmd.expandable(false) { cmd.expand(next,int)?} else {
                                    match &*cmd.orig {
                                        PrimitiveTeXCommand::Char(tk) if tk.catcode == CategoryCode::Space => (),
                                        PrimitiveTeXCommand::Primitive(c) if **c == OMIT => {
                                            doheader = false;
                                            break 'preludeb
                                        }
                                        _ => {
                                            int.requeue(next);
                                            break 'preludeb
                                        }
                                    }
                                }
                            }
                        }
                    }
                    _ => {
                        int.requeue(next);
                        break 'preludeb
                    }
                }
            }
            if columns.len() <= columnindex {
                match recindex {
                    Some(i) => columnindex = i,
                    None => TeXErr!("Invalid column index in align")
                }
            }
            if doheader {
                int.push_tokens(columns.get(columnindex).unwrap().0.clone())
            }
            let _oldmode = int.state.mode;
            int.state.mode = match tabmode {
                BoxMode::H => TeXMode::RestrictedHorizontal,
                BoxMode::V => TeXMode::InternalVertical,
                _ => TeXErr!("Should be unreachable!")
            };
            if inspan { inspan = false }
            else {
                cells = 1;
                int.state.push(int.stomach,GroupType::Box(tabmode));
            }
            if doheader {
                int.state.aligns.push(Some(columns.get(columnindex).unwrap().1.clone()))
            } else {
                int.state.aligns.push(Some(vec!()))
            }
            'cell: loop {
                let next = int.next_token();
                match next.catcode {
                    CategoryCode::Escape if next.char == endtemplate.char && next == endtemplate => {
                        break 'cell
                    }
                    CategoryCode::Escape if next.char == endtemplatespan.char && next == endtemplatespan => {
                        cells += 1;
                        inspan = true;
                        break 'cell
                    }
                    CategoryCode::Escape if next.char == endrow.char && next == endrow => {
                        let ret = int.get_whatsit_group(GroupType::Box(tabmode))?;
                        row.push((ret,columns.get(columnindex).unwrap().2,cells));
                        int.state.mode = _oldmode;
                        break 'row
                    }
                    _ => int.do_top(next,true)?
                }
            }
            match int.state.aligns.pop() {
                Some(None) => (),
                _ => TeXErr!("align error")
            }
            if !inspan {
                let ret = int.get_whatsit_group(GroupType::Box(tabmode))?;
                row.push((ret, columns.get(columnindex).unwrap().2, cells));
                int.state.mode = _oldmode;
            }
            columnindex += 1
        }
        match int.state.aligns.pop() {
            Some(None) => (),
            _ => TeXErr!("align error")
        }
        boxes.push(AlignBlock::Block(row))
    }

    int.pop_group(GroupType::Box(betweenmode))?;
    Ok((firsttabskip, columns, boxes))
}

pub static HALIGN: SimpleWhatsit = SimpleWhatsit {
    name:"halign",
    modes: |x|  {x == TeXMode::Vertical || x == TeXMode::InternalVertical
        || x == TeXMode::Displaymath // only allowed if it's the only whatsit in the math list
    },
    _get:|tk,int| {
        let _width = match int.read_keyword(vec!("to"))? {
            Some(_) => Some(int.read_dimension()?),
            None => None
        };
        let (skip,template,rows) = do_align(int,BoxMode::H,BoxMode::V)?;
        Ok(Whatsit::Simple(SimpleWI::HAlign(HAlign {
            skip,template,rows,sourceref:int.update_reference(tk)
        })))
    }
};

pub static VALIGN: SimpleWhatsit = SimpleWhatsit {
    name:"valign",
    modes: |x|  {x == TeXMode::Horizontal || x == TeXMode::RestrictedHorizontal },
    _get:|tk,int| {
        let _height = match int.read_keyword(vec!("to"))? {
            Some(_) => Some(int.read_dimension()?),
            None => None
        };
        let (skip,template,columns) = do_align(int,BoxMode::V,BoxMode::H)?;
        Ok(Whatsit::Simple(SimpleWI::VAlign(VAlign {
            skip,template,columns,sourceref:int.update_reference(tk)
        })))
    }
};

pub static ITALICCORR: PrimitiveExecutable = PrimitiveExecutable {
    name:"/",
    expandable:false,
    _apply:|_tk,_int| {
        Ok(()) // TODO maybe
    }
};

pub static HSS: SimpleWhatsit = SimpleWhatsit {
    name:"hss",
    modes: |x|  {x == TeXMode::Horizontal || x == TeXMode::RestrictedHorizontal },
    _get:|tk,int| {Ok(Whatsit::Simple(SimpleWI::Hss(Hss(int.update_reference(tk)))))}
};

pub static VSS: SimpleWhatsit = SimpleWhatsit {
    name:"vss",
    modes: |x|  {x == TeXMode::Vertical || x == TeXMode::InternalVertical },
    _get:|tk,int| {Ok(Whatsit::Simple(SimpleWI::Vss(Vss(int.update_reference(tk)))))}
};

pub static MSKIP: SimpleWhatsit = SimpleWhatsit {
    name:"mskip",
    modes: |x|  {x == TeXMode::Math || x == TeXMode::Displaymath },
    _get:|tk,int| {
        let ms = int.read_muskip()?;
        Ok(Whatsit::Simple(SimpleWI::MSkip(MSkip {
            skip:ms,sourceref:int.update_reference(tk)
        })))
    }
};

pub static EQNO: SimpleWhatsit = SimpleWhatsit {
    name:"eqno",
    modes: |x|  {x == TeXMode::Math || x == TeXMode::Displaymath },
    _get:|tk,int| {Ok(Whatsit::Simple(SimpleWI::Hss(Hss(int.update_reference(tk)))))} // TODO maybe
};

pub static LEQNO: SimpleWhatsit = SimpleWhatsit {
    name:"leqno",
    modes: |x|  {x == TeXMode::Math || x == TeXMode::Displaymath },
    _get:|tk,int| {Ok(Whatsit::Simple(SimpleWI::Hss(Hss(int.update_reference(tk)))))} // TODO maybe
};

pub static MARK: SimpleWhatsit = SimpleWhatsit {
    name:"mark",
    modes: |_| { true },
    _get:|tk,int| {
        let toks = int.read_balanced_argument(true,true,true,true)?;
        Ok(Whatsit::Simple(SimpleWI::Mark(Mark {
            toks,
            sourceref: int.update_reference(tk)
        })))
    }
};

pub static LEADERS: SimpleWhatsit = SimpleWhatsit {
    name:"leaders",
    modes: |_| { true },
    _get:|tk,int| {
        match int.read_keyword(vec!("Width","Height","Depth"))? {
            Some(_) => TeXErr!("TODO: \\leaders with dimesion"),
            None => {
                let cmdtk = int.read_command_token()?;
                let cmd = int.get_command(&cmdtk.cmdname())?;
                let content = match &*cmd.orig {
                    PrimitiveTeXCommand::Whatsit(ProvidesWhatsit::Box(r)) if **r == HBOX => {
                        Whatsit::Box((HBOX._get)(&cmdtk,int)?)
                    }
                    PrimitiveTeXCommand::Whatsit(ProvidesWhatsit::Box(r)) if **r == VBOX => {
                        Whatsit::Box((VBOX._get)(&cmdtk,int)?)
                    }
                    PrimitiveTeXCommand::Whatsit(ProvidesWhatsit::Box(r)) if **r == BOX => {
                        Whatsit::Box((BOX._get)(&cmdtk,int)?)
                    }
                    PrimitiveTeXCommand::Whatsit(ProvidesWhatsit::Box(r)) if **r == COPY => {
                        Whatsit::Box((COPY._get)(&cmdtk,int)?)
                    }
                    PrimitiveTeXCommand::Whatsit(ProvidesWhatsit::Simple(r)) if **r == HRULE => {
                        (HRULE._get)(&cmdtk,int)?
                    }
                    PrimitiveTeXCommand::Whatsit(ProvidesWhatsit::Simple(r)) if **r == VRULE => {
                        (VRULE._get)(&cmdtk,int)?
                    }
                    _ => TeXErr!(cmdtk => "Expected \\hbox, \\vbox, \\box, \\copy, \\hrule or \\vrule after \\leaders")
                };
                Ok(Whatsit::Simple(SimpleWI::Leaders(Leaders {
                    bx: Box::new(content),
                    sourceref: int.update_reference(tk)
                })))
            }
        }
    }
};

pub static CLEADERS: SimpleWhatsit = SimpleWhatsit {
    name: "cleaders",
    modes: |_| { true },
    _get: |tk, int| { (LEADERS._get)(tk,int)} // TODO maybe
};

pub static XLEADERS: SimpleWhatsit = SimpleWhatsit {
    name: "xleaders",
    modes: |_| { true },
    _get: |tk, int| { (LEADERS._get)(tk,int)} // TODO maybe
};

pub static MATHCHOICE: SimpleWhatsit = SimpleWhatsit {
    name:"mathchoice",
    modes: |x| {x == TeXMode::Math || x == TeXMode::Displaymath},
    _get:|_tk,int| {
        let mode = int.state.displaymode.get(&());
        let font = int.state.fontstyle.get(&());
        let ret = match (font,mode) {
            (FontStyle::Scriptscript,_) => {
                int.skip_ws();int.read_argument()?;
                int.skip_ws();int.read_argument()?;
                int.skip_ws();int.read_argument()?;
                int.skip_ws();int.read_math_whatsit(None)?
            }
            (FontStyle::Script,_) => {
                int.skip_ws();int.read_argument()?;
                int.skip_ws();int.read_argument()?;
                int.skip_ws();
                let ret = int.read_math_whatsit(None)?;
                int.skip_ws();int.read_argument()?;
                ret
            },
            (_,false) => {
                int.skip_ws();int.read_argument()?;
                int.skip_ws();
                let ret = int.read_math_whatsit(None)?;
                int.skip_ws();int.read_argument()?;
                int.skip_ws();int.read_argument()?;
                ret
            },
            (_,_) => {
                int.skip_ws();
                let ret = int.read_math_whatsit(None)?;
                int.skip_ws();int.read_argument()?;
                int.skip_ws();int.read_argument()?;
                int.skip_ws();int.read_argument()?;
                ret
            },
        };
        match ret {
            Some(s) => Ok(s),
            _ => Ok(Whatsit::Ls(vec!()))
        }
    }
};

pub static OVER: SimpleWhatsit = SimpleWhatsit {
    name:"over",
    modes: |x| {
        x == TeXMode::Math || x == TeXMode::Displaymath
    },
    _get: |tk,int| {
        Ok(Above {
            top:vec!(),bottom:vec!(),delimiters:(None,None),
            sourceref:int.update_reference(tk),thickness:None
        }.as_whatsit())
    }
};

fn dodelim(int:&mut Interpreter) -> Result<Option<MathChar>,TeXError> {
    let wi = int.read_math_whatsit(None)?;
    match wi {
        Some(Whatsit::Math(MathGroup { kernel:
            MathKernel::MathChar(mc) |
            MathKernel::Delimiter(Delimiter { small:mc, large:_, sourceref:_}),
                               superscript:None,subscript:None,limits:_ })) =>
            if mc.class == 4 || mc.class == 5 || mc.class == 3 {
                Ok(Some(mc))
            } else if mc.class == 6 || mc.class == 0 {
                Ok(None)
            } else {
                TeXErr!("Missing delimiter after \\left")
            },
        _ => TeXErr!("Missing delimiter after \\left")
    }
}

pub static OVERWITHDELIMS: SimpleWhatsit = SimpleWhatsit {
    name:"overwithdelims",
    modes: |x| {
        x == TeXMode::Math || x == TeXMode::Displaymath
    },
    _get: |tk,int| {
        let delim1 = dodelim(int)?;
        let delim2 = dodelim(int)?;
        Ok(Above {
            top:vec!(),bottom:vec!(),delimiters:(delim1,delim2),
            sourceref:int.update_reference(tk),thickness:None
        }.as_whatsit())
    }
};

pub static ABOVE: SimpleWhatsit = SimpleWhatsit {
    name:"above",
    modes: |x| {
        x == TeXMode::Math || x == TeXMode::Displaymath
    },
    _get: |tk,int| {
        let dim = int.read_dimension()?;
        Ok(Above {
            top:vec!(),bottom:vec!(),thickness:Some(dim),delimiters:(None,None),
            sourceref:int.update_reference(tk)
        }.as_whatsit())
    }
};

pub static ABOVEWITHDELIMS: SimpleWhatsit = SimpleWhatsit {
    name:"abovewithdelims",
    modes: |x| {
        x == TeXMode::Math || x == TeXMode::Displaymath
    },
    _get: |tk,int| {
        let delim1 = dodelim(int)?;
        let delim2 = dodelim(int)?;
        let dim = int.read_dimension()?;
        Ok(Above {
            top:vec!(),bottom:vec!(),thickness:Some(dim),delimiters:(delim1,delim2),
            sourceref:int.update_reference(tk)
        }.as_whatsit())
    }
};

pub static ATOP: SimpleWhatsit = SimpleWhatsit {
    name:"atop",
    modes: |x| {
        x == TeXMode::Math || x == TeXMode::Displaymath
    },
    _get: |tk,int| {
        Ok(Above {
            top:vec!(),bottom:vec!(),thickness:Some(0),delimiters:(None,None),
            sourceref:int.update_reference(tk)
        }.as_whatsit())
    }
};

pub static ATOPWITHDELIMS: SimpleWhatsit = SimpleWhatsit {
    name:"atopwithdelims",
    modes: |x| {
        x == TeXMode::Math || x == TeXMode::Displaymath
    },
    _get: |tk,int| {
        let delim1 = dodelim(int)?;
        let delim2 = dodelim(int)?;
        Ok(Above {
            top:vec!(),bottom:vec!(),thickness:Some(0),delimiters:(delim1,delim2),
            sourceref:int.update_reference(tk)
        }.as_whatsit())
    }
};

pub static ACCENT: SimpleWhatsit = SimpleWhatsit {
    name:"accent",
    modes: |x| {
        x == TeXMode::Horizontal || x == TeXMode::RestrictedHorizontal
    },
    _get:|tk,int| {
        let num = int.read_number()?;
        int.expand_until(false)?;
        let fnt = int.state.currfont.get(&());
        let pc = loop {
            let next = int.next_token();
            match next.catcode {
                CategoryCode::Letter | CategoryCode::Other => {
                    let font = int.state.currfont.get(&());
                    let sourceref = int.update_reference(&next);
                    break PrintChar {
                        char:next.char,
                        font,sourceref
                    }
                }
                CategoryCode::BeginGroup => {
                    let sourceref = int.update_reference(&next);
                    int.requeue(next);
                    let _ret = int.read_argument()?;
                    // TODO do this properly!
                    let font = int.state.currfont.get(&());
                    break PrintChar {
                        char:32,font,sourceref
                    }
                }
                CategoryCode::EndGroup => {
                    let sourceref = int.update_reference(&next);
                    int.requeue(next);
                    let font = int.state.currfont.get(&());
                    break PrintChar {
                        char:32,font,sourceref
                    }
                }
                _ => int.do_top(next,int.state.mode == TeXMode::RestrictedHorizontal)
            }?;
        };
        Ok(Accent {
            sourceref: int.update_reference(tk),
            font: fnt,
            char: pc,
            acc: num
        }.as_whatsit())
    }
};

pub static PARSHAPE: PrimitiveExecutable = PrimitiveExecutable {
    name:"parshape",
    expandable:false,
    _apply:|_r,int| {
        //unsafe { crate::LOG = true }
        let num = int.read_number()?;
        log!("\\parshape: Reading 2*{} dimensions:",num);
        let mut vals : Vec<(i32,i32)> = vec!();
        for i in 1..(num+1) {
            let f = int.read_dimension()?;
            log!("\\parshape: i{}={}",i,f);
            let s = int.read_dimension()?;
            log!("\\parshape: l{}={}",i,s);
            vals.push((f,s))
        }
        //TeXErr!(r.0.clone() => "Here!");
        int.state.parshape.set((),vals,false);
        Ok(())
    }
};

pub static HANGINDENT : PrimitiveExecutable = PrimitiveExecutable {
    name: "hangindent",
    expandable:false,
    _apply: |_,int| {
        int.read_eq();
        let dim = int.read_dimension()?;
        int.state.hangindent.set((),dim,false);
        Ok(())
    }
};

pub static HANGAFTER : PrimitiveExecutable = PrimitiveExecutable {
    name: "hangafter",
    expandable:false,
    _apply: |_,int| {
        int.read_eq();
        let num = int.read_number()?;
        int.state.hangafter.set((),num as usize,false);
        Ok(())
    }
};

pub static INDENT: PrimitiveExecutable = PrimitiveExecutable {
    name:"indent",
    expandable:false,
    _apply:|tk,int| {
        int.stomach_add(Whatsit::Simple(SimpleWI::Indent(
            Indent {
                dim:int.state.dimensions.get(&-(crate::commands::primitives::PARINDENT.index as i32)),
                sourceref:int.update_reference(&tk.0)
            })))?;
        Ok(())
    }
};

pub static NOINDENT: PrimitiveExecutable = PrimitiveExecutable {
    name:"noindent",
    expandable:false,
    _apply:|_tk,_int| {Ok(())}
};

pub static INSERT: PrimitiveExecutable = PrimitiveExecutable {
    name:"insert",
    expandable:false,
    _apply:|_,int| {
        let num = int.read_number()? as u16;
        let mut bx = int.read_whatsit_group(BoxMode::V,true)?;
        let prev = int.state.inserts.get_mut(&num);
        match prev {
            Some(v) => v.append(&mut bx),
            None => {int.state.inserts.insert(num,bx);}
        }
        Ok(())
    }
};

pub static TOPMARK: PrimitiveExecutable = PrimitiveExecutable {
    name:"topmark",
    expandable:false,
    _apply:|_tk,_int| {TeXErr!("TODO: \\topmark")}
};

pub static FIRSTMARK: PrimitiveExecutable = PrimitiveExecutable {
    name:"firstmark",
    expandable:false,
    _apply:|_tk,_int| {TeXErr!("TODO: \\firstmark")}
};

pub static BOTMARK: PrimitiveExecutable = PrimitiveExecutable {
    name:"botmark",
    expandable:false,
    _apply:|_tk,_int| {TeXErr!("TODO: \\botmark")}
};

pub static SPLITFIRSTMARK: PrimitiveExecutable = PrimitiveExecutable {
    name:"splitfirstmark",
    expandable:false,
    _apply:|_tk,_int| {TeXErr!("TODO: \\splitfirstmark")}
};

pub static SPLITBOTMARK: PrimitiveExecutable = PrimitiveExecutable {
    name:"splitbotmark",
    expandable:false,
    _apply:|_tk,_int| {TeXErr!("TODO: \\splitbotmark")}
};

pub static DISPLAYLIMITS: MathWhatsit = MathWhatsit {
    name:"displaylimits",
    _get: |_,int,pr| {
        match pr {
            Some(p) => p.limits = int.state.displaymode.get(&()),
            _ => ()
        }
        Ok(None)
    }
};

pub static MATHCLOSE: MathWhatsit = MathWhatsit {
    name:"mathclose",
    _get: |tk, int, _| {
        let ret = int.read_math_whatsit(None)?;
        match ret {
            Some(w) => Ok(
                Some(MathKernel::MathClose(MathClose {
                    content:Box::new(w),
                    sourceref:int.update_reference(tk)
                }))),
            None => TeXErr!("unfinished \\mathclose")
        }
    }
};

pub static MATHBIN: MathWhatsit = MathWhatsit {
    name:"mathbin",
    _get: |tk, int, _| {
        let ret = int.read_math_whatsit(None)?;
        match ret {
            Some(w) => Ok(
                Some(MathKernel::MathBin(MathBin {
                    content:Box::new(w),
                    sourceref:int.update_reference(tk)
                }))),
            None => TeXErr!("unfinished \\mathbin")
        }
    }
};

pub static MATHOPEN: MathWhatsit = MathWhatsit {
    name:"mathopen",
    _get: |tk, int, _| {
        let ret = int.read_math_whatsit(None)?;
        match ret {
            Some(w) => Ok(
                Some(MathKernel::MathOpen(MathOpen {
                    content:Box::new(w),
                    sourceref:int.update_reference(tk)
                }))),
            None => TeXErr!("unfinished \\mathopen")
        }
    }
};

pub static MATHORD: MathWhatsit = MathWhatsit {
    name:"mathord",
    _get: |tk, int, _| {
        let ret = int.read_math_whatsit(None)?;
        match ret {
            Some(w) => Ok(
                Some(MathKernel::MathOrd(MathOrd {
                    content:Box::new(w),
                    sourceref:int.update_reference(tk)
                }))),
            None => TeXErr!("unfinished \\mathord")
        }
    }
};

pub static MATHPUNCT: MathWhatsit = MathWhatsit {
    name:"mathpunct",
    _get: |tk, int, _| {
        let ret = int.read_math_whatsit(None)?;
        match ret {
            Some(w) => Ok(
                Some(MathKernel::MathPunct(MathPunct {
                    content:Box::new(w),
                    sourceref:int.update_reference(tk)
                }))),
            None => TeXErr!("unfinished \\mathpunct")
        }
    }
};

pub static MATHREL: MathWhatsit = MathWhatsit {
    name:"mathrel",
    _get: |tk, int, _| {
        let ret = int.read_math_whatsit(None)?;
        match ret {
            Some(w) => Ok(
                Some(MathKernel::MathRel(MathRel {
                    content:Box::new(w),
                    sourceref:int.update_reference(tk)
                }))),
            None => TeXErr!("unfinished \\mathrel")
        }
    }
};

pub static DELIMITER: MathWhatsit = MathWhatsit {
    name:"delimiter",
    _get: |tk,int,_| {
        let num = int.read_number()?;
        let large = num % 4096;
        let small = (num - large)/4096;
        let largemc = int.do_math_char(None,large as u32);
        let smallmc = int.do_math_char(None,small as u32);
        let delim = Delimiter {
            small: smallmc,
            large: largemc,
            sourceref: int.update_reference(tk)
        };
        Ok(Some(MathKernel::Delimiter(delim)))
    }
};

pub static MATHOP : MathWhatsit = MathWhatsit {
    name: "mathop",
    _get: |tk, int, _| {
        let ret = int.read_math_whatsit(None)?;
        match ret {
            Some(w) => Ok(
                Some(MathKernel::MathOp(MathOp {
                    content:Box::new(w),
                    sourceref:int.update_reference(tk)
                }))),
            None => TeXErr!("unfinished \\mathop")
        }
    }
};

pub static MATHINNER: MathWhatsit = MathWhatsit {
    name: "mathinner",
    _get: |tk, int, _| {
        let ret = int.read_math_whatsit(None)?;
        match ret {
            Some(w) => Ok(
                Some(MathKernel::MathInner(MathInner {
                    content:Box::new(w),
                    sourceref:int.update_reference(tk)
                }))),
            None => TeXErr!("unfinished \\mathinner")
        }
    }
};

pub static UNDERLINE: MathWhatsit = MathWhatsit {
    name: "underline",
    _get: |tk, int, _| {
        let ret = int.read_math_whatsit(None)?;
        match ret {
            Some(w) => Ok(
                Some(MathKernel::Underline(Underline {
                    content:Box::new(w),
                    sourceref:int.update_reference(tk)
                }))),
            None => TeXErr!("unfinished \\underline")
        }
    }
};

pub static OVERLINE: MathWhatsit = MathWhatsit {
    name: "overline",
    _get: |tk, int, _| {
        let ret = int.read_math_whatsit(None)?;
        match ret {
            Some(w) => Ok(
                Some(MathKernel::Overline(Overline {
                    content:Box::new(w),
                    sourceref:int.update_reference(tk)
                }))),
            None => TeXErr!("unfinished \\overline")
        }
    }
};

pub static MATHACCENT: MathWhatsit = MathWhatsit {
    name:"mathaccent",
    _get: |tk,int,_| {
        let num = int.read_number()?;
        let mc = int.do_math_char(None,num as u32);
        let next = match int.read_math_whatsit(None)? {
            Some(w) => w,
            None => TeXErr!("unfinished \\mathaccent")
        };
        let accent = MathAccent {
            content:Box::new(next),
            accent:mc,
            sourceref:int.update_reference(tk)
        };
        Ok(Some(MathKernel::MathAccent(accent)))
    }
};

pub static RADICAL: MathWhatsit = MathWhatsit {
    name:"radical",
    _get: |tk,int,_| {
        let num = int.read_number()?;
        let large = num % 4096;
        let small = (num - large)/4096;
        let largemc = int.do_math_char(None,large as u32);
        let smallmc = int.do_math_char(None,small as u32);
        let body = match int.read_math_whatsit(None)? {
            None => TeXErr!(tk.clone() => "Expected Whatsit after \\radical"),
            Some(wi) => wi
        };
        let delim = Radical {
            small: smallmc,
            large: largemc,
            body:Box::new(body),
            sourceref: int.update_reference(tk)
        };
        Ok(Some(MathKernel::Radical(delim)))
    }
};

pub static MATHCHAR: MathWhatsit = MathWhatsit {
    name:"mathchar",
    _get: |_,int,_| {
        let num = int.read_number()? as u32;
        let mc = int.do_math_char(None,num);
        Ok(Some(MathKernel::MathChar(mc)))
    }
};

pub static MKERN: MathWhatsit = MathWhatsit {
    name:"mkern",
    _get: |tk,int,_| {
        let kern = int.read_muskip()?;
        Ok(Some(MathKernel::MKern(MKern {
            sk:kern,
            sourceref:int.update_reference(tk)
        })))
    }
};

pub static DISPLAYSTYLE: PrimitiveExecutable = PrimitiveExecutable {
    name:"displaystyle",
    expandable:false,
    _apply:|tk,int| {
        match int.state.mode {
            TeXMode::Math | TeXMode::Displaymath => (),
            _ => TeXErr!(tk.0.clone() => "\\displaymode only allowed in math mode")
        }
        int.state.displaymode.set((),true,false);
        Ok(())
    }
};
pub static LIMITS: MathWhatsit = MathWhatsit {
    name:"limits",
    _get: |tk,_int,last| {
        match last {
            None => TeXErr!(tk.clone() => "Nothing to \\limits here"),
            Some(s) => s.limits = true
        }
        Ok(None)
    }
};

pub static NOLIMITS: MathWhatsit = MathWhatsit {
    name:"nolimits",
    _get: |tk,_int,last| {
        match last {
            None => TeXErr!(tk.clone() => "Nothing to \\nolimits here"),
            Some(s) => s.limits = false
        }
        Ok(None)
    }
};

pub static DISCRETIONARY: PrimitiveExecutable = PrimitiveExecutable {
    name:"discretionary",
    expandable:false,
    _apply:|tk,int| {
        let _prebreak = int.read_argument()?;
        let _postbreak = int.read_argument()?;
        let nobreak = int.read_argument()?;
        tk.2 = nobreak;
        Ok(())
    }
};

pub static LEFT: MathWhatsit = MathWhatsit {
    name:"left",
    _get: |tk,int,_| {
        //int.state.push(int.stomach,GroupType::LeftRight);
        int.expand_until(true)?;
        let next = int.next_token();
        match next.char {
            46 => {
                int.stomach_add(Left { bx:None, sourceref:int.update_reference(tk)}.as_whatsit())?;
                return Ok(None)
            }
            _ => int.requeue(next)
        }
        let wi = int.read_math_whatsit(None)?;
        match wi {
            Some(Whatsit::Math(MathGroup { kernel:
                MathKernel::MathChar(mc) |
                MathKernel::Delimiter(Delimiter { small:mc, large:_, sourceref:_}),
                                   superscript:None,subscript:None,limits:_ })) => int.stomach_add(Whatsit::Simple(SimpleWI::Left(
                Left {
                    bx:Some(mc),
                    sourceref:int.update_reference(tk)
                })))?,
            _ => TeXErr!("Missing delimiter after \\left")
        }
        Ok(None)
    }
};

pub static MIDDLE: MathWhatsit = MathWhatsit {
    name:"middle",
    _get: |tk,int,_| {
        let wi = int.read_math_whatsit(None)?;
        match wi {
            Some(Whatsit::Math(MathGroup { kernel:
                MathKernel::MathChar(mc) |
                MathKernel::Delimiter(Delimiter { small:mc, large:_, sourceref:_}),
                                   superscript:None,subscript:None,limits:_ })) => int.stomach_add(Whatsit::Simple(SimpleWI::Middle(
                Middle {
                    bx:Some(mc),
                    sourceref:int.update_reference(tk)
                })))?,
            _ => TeXErr!("Missing delimiter after \\middle")
        }
        Ok(None)
    }
};

pub static RIGHT: MathWhatsit = MathWhatsit {
    name:"right",
    _get: |tk,int,_| {
        int.expand_until(true)?;
        let next = int.next_token();
        match next.char {
            46 => {
                int.stomach_add(Right { bx:None, sourceref:int.update_reference(tk)}.as_whatsit())?;
                //int.pop_group(GroupType::LeftRight)?;
                return Ok(None)
            }
            _ => int.requeue(next)
        }
        let wi = int.read_math_whatsit(None)?;
        match wi {
            Some(Whatsit::Math(MathGroup { kernel:
                MathKernel::MathChar(mc) |
                MathKernel::Delimiter(Delimiter { small:mc, large:_, sourceref:_}),
                                   superscript:None,subscript:None,limits:_ })) => int.stomach_add(Whatsit::Simple(SimpleWI::Right(
                Right {
                    bx:Some(mc),
                    sourceref:int.update_reference(tk)
                })))?,
            _ => TeXErr!("Missing delimiter after \\right")
        }
        //int.pop_group(GroupType::LeftRight)?;
        Ok(None)
    }
};

pub static END: PrimitiveExecutable = PrimitiveExecutable {
    name:"end",
    expandable:false,
    _apply:|_tk,int| {
        int.end();
        Ok(())
    }
};

pub static BATCHMODE: PrimitiveExecutable = PrimitiveExecutable {
    name:"batchmode",
    expandable:true,
    _apply:|_tk,_int| {
        Ok(())
    }
};


pub static NONSCRIPT: PrimitiveExecutable = PrimitiveExecutable {
    name:"nonscript",
    expandable:false,
    _apply:|_tk,_int| {
        Ok(()) // todo?
    }
};

pub static BEGINL: PrimitiveExecutable = PrimitiveExecutable {
    name:"beginL",
    expandable:false,
    _apply:|_tk,_int| {
        //TeXErr!("TODO") ?
        Ok(())
    }
};

pub static BEGINR: PrimitiveExecutable = PrimitiveExecutable {
    name:"beginR",
    expandable:false,
    _apply:|_tk,_int| {
        //TeXErr!("TODO") ?
        Ok(())
    }
};

pub static ENDL: PrimitiveExecutable = PrimitiveExecutable {
    name:"endL",
    expandable:false,
    _apply:|_tk,_int| {
        //TeXErr!("TODO") ?
        Ok(())
    }
};

pub static ENDR: PrimitiveExecutable = PrimitiveExecutable {
    name:"endR",
    expandable:false,
    _apply:|_tk,_int| {
        //TeXErr!("TODO") ?
        Ok(())
    }
};

pub static MARKS: PrimitiveExecutable = PrimitiveExecutable {
    name:"marks",
    expandable:true,
    _apply:|_tk,int| {
        int.read_number();int.read_argument();
        Ok(())
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

pub static PREVGRAF: RegisterReference = RegisterReference {
    name: "prevgraf",
    index:85
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

pub static BYE: PrimitiveExecutable = PrimitiveExecutable {
    name:"bye",
    expandable:true,
    _apply:|_tk,_int| {Ok(())}
};

pub static FONTNAME: PrimitiveExecutable = PrimitiveExecutable {
    name:"fontname",
    expandable:true,
    _apply:|tk,int| {
        let font = read_font(int)?;
        let name = &font.file.name;
        let str = match font.at {
            None => name.to_string(),
            Some(s) => name.to_string() + " at " + &dimtostr(s)
        };
        tk.2 = crate::interpreter::string_to_tokens(str.into());
        Ok(())
    }
};

pub static SHIPOUT: PrimitiveExecutable = PrimitiveExecutable {
    name:"shipout",
    expandable:true,
    _apply:|_tk,_int| {TeXErr!("TODO: \\shipout")}
};

pub static SPECIAL: PrimitiveExecutable = PrimitiveExecutable {
    name:"special",
    expandable:true,
    _apply:|_tk,int| {
        let _arg = int.read_string()?;
        //println!("\n{}",arg);
        Ok(())
    }
};

pub static HOLDINGINSERTS: PrimitiveExecutable = PrimitiveExecutable {
    name:"holdinginserts",
    expandable:true,
    _apply:|_tk,_int| {TeXErr!("TODO: \\holdinginserts")}
};

pub static LOOSENESS: PrimitiveExecutable = PrimitiveExecutable {
    name:"looseness",
    expandable:true,
    _apply:|_tk,_int| {TeXErr!("TODO: \\looseness")}
};

pub static NOBOUNDARY: PrimitiveExecutable = PrimitiveExecutable {
    name:"noboundary",
    expandable:true,
    _apply:|_tk,_int| {TeXErr!("TODO: \\noboundary")}
};

pub static SCROLLMODE: PrimitiveExecutable = PrimitiveExecutable {
    name:"scrollmode",
    expandable:true,
    _apply:|_tk,_int| { Ok(())}
};

pub static NONSTOPMODE: PrimitiveExecutable = PrimitiveExecutable {
    name:"nonstopmode",
    expandable:true,
    _apply:|_tk,_int| { Ok(())}
};

pub static PAUSING: PrimitiveExecutable = PrimitiveExecutable {
    name:"pausing",
    expandable:true,
    _apply:|_tk,_int| { Ok(())}
};

pub static SETLANGUAGE: PrimitiveExecutable = PrimitiveExecutable {
    name:"setlanguage",
    expandable:true,
    _apply:|_tk,_int| {TeXErr!("TODO: \\setlanguage")}
};

pub static SHOW: PrimitiveExecutable = PrimitiveExecutable {
    name:"show",
    expandable:true,
    _apply:|_tk,_int| {TeXErr!("TODO:: \\show")}
};

pub static SHOWBOX: PrimitiveExecutable = PrimitiveExecutable {
    name:"showbox",
    expandable:true,
    _apply:|_tk,_int| {TeXErr!("TODO: \\showbox")}
};

pub static SHOWLISTS: PrimitiveExecutable = PrimitiveExecutable {
    name:"showlists",
    expandable:true,
    _apply:|_tk,_int| {TeXErr!("TODO: \\showlists")}
};

pub static SHOWTHE: PrimitiveExecutable = PrimitiveExecutable {
    name:"showthe",
    expandable:true,
    _apply:|_tk,_int| {TeXErr!("TODO: \\showthe")}
};

pub static BOTMARKS: PrimitiveExecutable = PrimitiveExecutable {
    name:"botmarks",
    expandable:true,
    _apply:|_tk,_int| {TeXErr!("TODO: \\botmarks")}
};

pub static CURRENTIFBRANCH: PrimitiveExecutable = PrimitiveExecutable {
    name:"currentifbranch",
    expandable:true,
    _apply:|_tk,_int| {TeXErr!("TODO: \\currentifbranch")}
};

pub static CURRENTIFLEVEL: PrimitiveExecutable = PrimitiveExecutable {
    name:"currentiflevel",
    expandable:true,
    _apply:|_tk,_int| {TeXErr!("TODO: \\currentiflevel")}
};

pub static CURRENTIFTYPE: PrimitiveExecutable = PrimitiveExecutable {
    name:"currentiftype",
    expandable:true,
    _apply:|_tk,_int| {TeXErr!("TODO: \\currentiftype")}
};

pub static FIRSTMARKS: PrimitiveExecutable = PrimitiveExecutable {
    name:"firstmarks",
    expandable:true,
    _apply:|_tk,_int| {TeXErr!("TODO: \\firstmarks")}
};

pub static GLUETOMU: PrimitiveExecutable = PrimitiveExecutable {
    name:"gluetomu",
    expandable:true,
    _apply:|_tk,_int| {TeXErr!("TODO: \\gluetomu")}
};

pub static INTERACTIONMODE: PrimitiveExecutable = PrimitiveExecutable {
    name:"interactionmode",
    expandable:true,
    _apply:|_tk,_int| {TeXErr!("TODO: \\interactionmode")}
};

pub static LASTLINEFIT: PrimitiveExecutable = PrimitiveExecutable {
    name:"lastlinefit",
    expandable:true,
    _apply:|_tk,_int| {TeXErr!("TODO: \\lastlinefit")}
};

pub static MUTOGLUE: PrimitiveExecutable = PrimitiveExecutable {
    name:"mutoglue",
    expandable:true,
    _apply:|_tk,_int| {TeXErr!("TODO: \\mutoglue")}
};

pub static PAGEDISCARDS: PrimitiveExecutable = PrimitiveExecutable {
    name:"pagediscards",
    expandable:true,
    _apply:|_tk,_int| {TeXErr!("TODO: \\pagediscards")}
};

pub static PARSHAPEDIMEN: PrimitiveExecutable = PrimitiveExecutable {
    name:"parshapedimen",
    expandable:true,
    _apply:|_tk,_int| {TeXErr!("TODO: \\parshapedimen")}
};

pub static PARSHAPEINDENT: PrimitiveExecutable = PrimitiveExecutable {
    name:"parshapeindent",
    expandable:true,
    _apply:|_tk,_int| {TeXErr!("TODO: \\parshapeindent")}
};

pub static PARSHAPELENGTH: PrimitiveExecutable = PrimitiveExecutable {
    name:"parshapelength",
    expandable:true,
    _apply:|_tk,_int| {TeXErr!("TODO: \\parshapelength")}
};

pub static PREDISPLAYDIRECTION: PrimitiveExecutable = PrimitiveExecutable {
    name:"predisplaydirection",
    expandable:true,
    _apply:|_tk,_int| {TeXErr!("TODO: \\predisplaydirection")}
};

pub static SHOWGROUPS: PrimitiveExecutable = PrimitiveExecutable {
    name:"showgroups",
    expandable:true,
    _apply:|_tk,_int| {TeXErr!("TODO: \\showgroups")}
};

pub static SHOWIFS: PrimitiveExecutable = PrimitiveExecutable {
    name:"showifs",
    expandable:true,
    _apply:|_tk,_int| {TeXErr!("TODO: \\showifs")}
};

pub static SHOWTOKENS: PrimitiveExecutable = PrimitiveExecutable {
    name:"showtokens",
    expandable:true,
    _apply:|_tk,_int| {TeXErr!("TODO: \\showtokens")}
};

pub static SPLITBOTMARKS: PrimitiveExecutable = PrimitiveExecutable {
    name:"splitbotmarks",
    expandable:true,
    _apply:|_tk,_int| {TeXErr!("TODO: \\splitbotmarks")}
};

pub static SPLITDISCARDS: PrimitiveExecutable = PrimitiveExecutable {
    name:"splitdiscards",
    expandable:true,
    _apply:|_tk,_int| {TeXErr!("TODO: \\splitdiscards")}
};

pub static SPLITFIRSTMARKS: PrimitiveExecutable = PrimitiveExecutable {
    name:"splitfirstmarks",
    expandable:true,
    _apply:|_tk,_int| {TeXErr!("TODO: \\splitfirstmarks")}
};

pub static TEXXETSTATE: PrimitiveExecutable = PrimitiveExecutable {
    name:"TeXXeTstate",
    expandable:true,
    _apply:|_tk,_int| {TeXErr!("TODO: \\TeXXeTstate")}
};

pub static TOPMARKS: PrimitiveExecutable = PrimitiveExecutable {
    name:"topmarks",
    expandable:true,
    _apply:|_tk,_int| {TeXErr!("TODO: \\topmarks")}
};

pub static EFCODE: PrimitiveExecutable = PrimitiveExecutable {
    name:"efcode",
    expandable:true,
    _apply:|_tk,_int| {TeXErr!("TODO: \\efcode")}
};

pub static LEFTMARGINKERN: PrimitiveExecutable = PrimitiveExecutable {
    name:"leftmarginkern",
    expandable:true,
    _apply:|_tk,_int| {TeXErr!("TODO: \\leftmarginkern")}
};

pub static LETTERSPACEFONT: PrimitiveExecutable = PrimitiveExecutable {
    name:"letterspacefont",
    expandable:true,
    _apply:|_tk,_int| {TeXErr!("TODO: \\letterspacefont")}
};

pub static QUITVMODE: PrimitiveExecutable = PrimitiveExecutable {
    name:"quitvmode",
    expandable:true,
    _apply:|_tk,_int| {TeXErr!("TODO: \\quitvmode")}
};

pub static RIGHTMARGINKERN: PrimitiveExecutable = PrimitiveExecutable {
    name:"rightmarginkern",
    expandable:true,
    _apply:|_tk,_int| {TeXErr!("TODO: \\rightmarginkern")}
};

pub static TAGCODE: PrimitiveExecutable = PrimitiveExecutable {
    name:"tagcode",
    expandable:true,
    _apply:|_tk,_int| {TeXErr!("TODO: \\tagcode")}
};

pub static MUSKIP: PrimitiveExecutable = PrimitiveExecutable {
    name:"muskip",
    expandable:true,
    _apply:|_tk,_int| {TeXErr!("TODO: \\muskip")}
};

pub static OUTER: PrimitiveExecutable = PrimitiveExecutable {
    name:"outer",
    expandable:true,
    _apply:|_tk,_int| {Ok(())}
};

pub static BIGSKIP: PrimitiveExecutable = PrimitiveExecutable {
    name:"bigskip",
    expandable:true,
    _apply:|_tk,_int| {TeXErr!("TODO: \\bigskip")}
};


pub static HFILNEG: PrimitiveExecutable = PrimitiveExecutable {
    name:"hfilneg",
    expandable:true,
    _apply:|_tk,_int| {TeXErr!("TODO: \\hfilneg")}
};

pub static MEDSKIP: PrimitiveExecutable = PrimitiveExecutable {
    name:"medskip",
    expandable:true,
    _apply:|_tk,_int| {TeXErr!("TODO: \\medskip")}
};

pub static SMALLSKIP: PrimitiveExecutable = PrimitiveExecutable {
    name:"smallskip",
    expandable:true,
    _apply:|_tk,_int| {TeXErr!("TODO: \\smallskip")}
};

pub static VFILNEG: PrimitiveExecutable = PrimitiveExecutable {
    name:"vfilneg",
    expandable:true,
    _apply:|_tk,_int| {TeXErr!("TODO: \\vfilneg")}
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
    PrimitiveTeXCommand::AV(AssignableValue::Int(&PAGEGOAL)),
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
    PrimitiveTeXCommand::Whatsit(ProvidesWhatsit::Simple(&MOVELEFT)),
    PrimitiveTeXCommand::Whatsit(ProvidesWhatsit::Simple(&MOVERIGHT)),
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
    PrimitiveTeXCommand::Whatsit(ProvidesWhatsit::Simple(&MARK)),
    PrimitiveTeXCommand::Whatsit(ProvidesWhatsit::Simple(&LEADERS)),
    PrimitiveTeXCommand::Whatsit(ProvidesWhatsit::Simple(&CLEADERS)),
    PrimitiveTeXCommand::Whatsit(ProvidesWhatsit::Simple(&XLEADERS)),
    PrimitiveTeXCommand::Whatsit(ProvidesWhatsit::Simple(&MATHCHOICE)),
    PrimitiveTeXCommand::Whatsit(ProvidesWhatsit::Simple(&OVER)),
    PrimitiveTeXCommand::Whatsit(ProvidesWhatsit::Simple(&OVERWITHDELIMS)),
    PrimitiveTeXCommand::Whatsit(ProvidesWhatsit::Simple(&ATOP)),
    PrimitiveTeXCommand::Whatsit(ProvidesWhatsit::Simple(&ATOPWITHDELIMS)),
    PrimitiveTeXCommand::Whatsit(ProvidesWhatsit::Simple(&ABOVE)),
    PrimitiveTeXCommand::Whatsit(ProvidesWhatsit::Simple(&ABOVEWITHDELIMS)),
    PrimitiveTeXCommand::Whatsit(ProvidesWhatsit::Simple(&SPACE)),
    PrimitiveTeXCommand::Whatsit(ProvidesWhatsit::Simple(&EQNO)),
    PrimitiveTeXCommand::Whatsit(ProvidesWhatsit::Simple(&LEQNO)),
    PrimitiveTeXCommand::Whatsit(ProvidesWhatsit::Simple(&ACCENT)),
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
    PrimitiveTeXCommand::Num(&LASTPENALTY),
    PrimitiveTeXCommand::Num(&LASTSKIP),
    PrimitiveTeXCommand::Num(&LASTKERN),
    PrimitiveTeXCommand::Num(&CURRENTGROUPLEVEL),
    PrimitiveTeXCommand::Num(&CURRENTGROUPTYPE),
    PrimitiveTeXCommand::Num(&GLUESHRINK),
    PrimitiveTeXCommand::Num(&GLUESHRINKORDER),
    PrimitiveTeXCommand::Num(&GLUESTRETCH),
    PrimitiveTeXCommand::Num(&GLUESTRETCHORDER),
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
    PrimitiveTeXCommand::Whatsit(ProvidesWhatsit::Box(&VTOP)),
    PrimitiveTeXCommand::Whatsit(ProvidesWhatsit::Box(&VSPLIT)),
    PrimitiveTeXCommand::Whatsit(ProvidesWhatsit::Box(&VCENTER)),
    PrimitiveTeXCommand::Whatsit(ProvidesWhatsit::Box(&LASTBOX)),
    PrimitiveTeXCommand::Whatsit(ProvidesWhatsit::Box(&BOX)),
    PrimitiveTeXCommand::Whatsit(ProvidesWhatsit::Box(&COPY)),
    PrimitiveTeXCommand::Whatsit(ProvidesWhatsit::Math(&DISPLAYLIMITS)),
    PrimitiveTeXCommand::Whatsit(ProvidesWhatsit::Math(&MATHCLOSE)),
    PrimitiveTeXCommand::Whatsit(ProvidesWhatsit::Math(&MATHBIN)),
    PrimitiveTeXCommand::Whatsit(ProvidesWhatsit::Math(&MATHINNER)),
    PrimitiveTeXCommand::Whatsit(ProvidesWhatsit::Math(&UNDERLINE)),
    PrimitiveTeXCommand::Whatsit(ProvidesWhatsit::Math(&OVERLINE)),
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
    PrimitiveTeXCommand::Whatsit(ProvidesWhatsit::Math(&LEFT)),
    PrimitiveTeXCommand::Whatsit(ProvidesWhatsit::Math(&RIGHT)),
    PrimitiveTeXCommand::Whatsit(ProvidesWhatsit::Math(&MKERN)),
    PrimitiveTeXCommand::Whatsit(ProvidesWhatsit::Math(&LIMITS)),
    PrimitiveTeXCommand::Whatsit(ProvidesWhatsit::Math(&NOLIMITS)),

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
    PrimitiveTeXCommand::AV(AssignableValue::PrimReg(&PREVGRAF)),

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
    PrimitiveTeXCommand::Primitive(&DETOKENIZE),
    PrimitiveTeXCommand::Primitive(&DUMP),
    PrimitiveTeXCommand::Primitive(&ENDINPUT),
    PrimitiveTeXCommand::Primitive(&ERRMESSAGE),
    PrimitiveTeXCommand::Primitive(&LATEX3ERROR),
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
    PrimitiveTeXCommand::Primitive(&LOOSENESS),
    PrimitiveTeXCommand::Primitive(&NOBOUNDARY),
    PrimitiveTeXCommand::Primitive(&SCROLLMODE),
    PrimitiveTeXCommand::Primitive(&NONSTOPMODE),
    PrimitiveTeXCommand::Primitive(&OMIT),
    PrimitiveTeXCommand::Primitive(&PAUSING),
    PrimitiveTeXCommand::Primitive(&SETLANGUAGE),
    PrimitiveTeXCommand::Primitive(&SHOW),
    PrimitiveTeXCommand::Primitive(&SHOWBOX),
    PrimitiveTeXCommand::Primitive(&SHOWLISTS),
    PrimitiveTeXCommand::Primitive(&SHOWTHE),
    PrimitiveTeXCommand::Primitive(&SPAN),
    PrimitiveTeXCommand::Primitive(&BEGINL),
    PrimitiveTeXCommand::Primitive(&BEGINR),
    PrimitiveTeXCommand::Primitive(&BOTMARKS),
    PrimitiveTeXCommand::Primitive(&CURRENTIFBRANCH),
    PrimitiveTeXCommand::Primitive(&CURRENTIFLEVEL),
    PrimitiveTeXCommand::Primitive(&CURRENTIFTYPE),
    PrimitiveTeXCommand::Primitive(&ENDL),
    PrimitiveTeXCommand::Primitive(&ENDR),
    PrimitiveTeXCommand::Primitive(&FIRSTMARKS),
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
    PrimitiveTeXCommand::Primitive(&PATTERNS),
    PrimitiveTeXCommand::Primitive(&BIGSKIP),
    PrimitiveTeXCommand::Primitive(&DISCRETIONARY),
    PrimitiveTeXCommand::Primitive(&DISPLAYSTYLE),
    PrimitiveTeXCommand::Primitive(&TOPMARK),
    PrimitiveTeXCommand::Primitive(&FIRSTMARK),
    PrimitiveTeXCommand::Primitive(&BOTMARK),
    PrimitiveTeXCommand::Primitive(&SPLITFIRSTMARK),
    PrimitiveTeXCommand::Primitive(&SPLITBOTMARK),
    PrimitiveTeXCommand::Primitive(&HFILNEG),
    PrimitiveTeXCommand::Primitive(&INDENT),
    PrimitiveTeXCommand::Primitive(&INSERT),
    PrimitiveTeXCommand::Primitive(&ITALICCORR),
    PrimitiveTeXCommand::Primitive(&MEDSKIP),
    PrimitiveTeXCommand::Primitive(&NOALIGN),
    PrimitiveTeXCommand::Primitive(&NOINDENT),
    PrimitiveTeXCommand::Primitive(&SMALLSKIP),
    PrimitiveTeXCommand::Primitive(&UNSKIP),
    PrimitiveTeXCommand::Primitive(&UNKERN),
    PrimitiveTeXCommand::Primitive(&UNPENALTY),
    PrimitiveTeXCommand::Primitive(&VADJUST),
    PrimitiveTeXCommand::Primitive(&VFILNEG),
]}