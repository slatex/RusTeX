use crate::interpreter::Interpreter;
use crate::commands::{TeXCommand, Conditional, PrimitiveExecutable};
use crate::utils::TeXError;
use crate::catcodes::CategoryCode;
use crate::log;


fn dotrue(int: &Interpreter,cond:u8,unless:bool) -> Result<(),TeXError> {
    if unless {
        dofalse(int,cond,false)
    } else {
        int.setcondition(cond,true);
        Ok(())
    }
}

use crate::FileEnd;

pub fn false_loop(int:&Interpreter,initifs:u8,allowelse : bool) -> Result<(),TeXError> {
    let mut inifs = initifs;
    //log!("false loop: {}",inifs);
    while int.has_next() {
        let next = int.next_token();
        match next.catcode {
            CategoryCode::Escape | CategoryCode::Active => {
                match int.state_get_command(&next.cmdname()) {
                    None => {}
                    Some(p) => {
                        match p {
                            TeXCommand::Primitive(x) if inifs == 0 && *x == FI => {
                                int.popcondition();
                                return Ok(())
                            }
                            TeXCommand::Primitive(x) if allowelse && inifs == 0 && *x == ELSE => {
                                return Ok(())
                            }
                            TeXCommand::Primitive(x) if *x == FI => inifs -=1,
                            TeXCommand::Cond(_) => inifs += 1,
                            _ => {}
                        }
                    }
                }
            }
            _ => {}
        }
    }
    FileEnd!(int)
}

fn dofalse(int: &Interpreter,cond:u8,unless:bool) -> Result<(),TeXError> {
    if unless {
        dotrue(int,cond,false)
    } else {
        let initifs = int.setcondition(cond,false);
        let inifs = initifs;
        false_loop(int,inifs,true)
    }
}

pub static FI : PrimitiveExecutable = PrimitiveExecutable {
    _apply: |_tk,int| {
        int.popcondition();
        Ok(None)
    },
    expandable: true,
    name: "fi"
};

pub static UNLESS: PrimitiveExecutable = PrimitiveExecutable {
    name:"unless",
    _apply: |_tk,_int| {
        todo!()
    },
    expandable: true
};

pub static OR: PrimitiveExecutable = PrimitiveExecutable {
    name:"or",
    _apply: |_tk,_int| {
        todo!()
    },
    expandable: true
};

use crate::TeXErr;

pub static ELSE: PrimitiveExecutable = PrimitiveExecutable {
    _apply: |_tk,int| {
        match int.getcondition() {
            None => TeXErr!(int,"extra \\else"),
            Some((_,None)) => {
                Ok(None)
            }
            Some((_,_)) => {
                false_loop(int,0,false)?;
                Ok(None)
            }
        }
    },
    expandable: true,
    name: "else"
};

pub static IFX : Conditional = Conditional {
    name:"ifx",
    _apply: |int,cond,unless| {
        use CategoryCode::*;
        let tka = int.next_token();
        let tkb = int.next_token();
        let istrue = match (tka.catcode,tkb.catcode) {
            (Active|Escape,Active|Escape) => {
               match (int.state_get_command(&tka.cmdname()),int.state_get_command(&tkb.cmdname())) {
                   (None,None) => true,
                   (None,_) => false,
                   (_,None) => false,
                   (Some(cmd1),Some(cmd2)) => cmd1 == cmd2
               }
            }
            (_a,_b) if matches!(_a,_b) => tka.char == tkb.char,
            _ => false
        };
        log!("\\ifx {}{}: {}",tka,tkb,istrue);
        if istrue {dotrue(int,cond,unless)} else {dofalse(int,cond,unless)}
    }
};

pub static IFNUM : Conditional = Conditional {
    _apply: |int,cond,unless| {
        let i1 = int.read_number()?;
        let rel = int.read_keyword(vec!["<","=",">"])?;
        let i2 = int.read_number()?;
        let istrue = match rel {
            Some(ref s) if s == "<" => i1 < i2,
            Some(ref s) if s == "=" => i1 == i2,
            Some(ref s) if s == ">" => i1 > i2,
            _ =>  TeXErr!(int,"Expected '<','=' or '>' in \\ifnum")
        };
        log!("\\ifnum {}{}{}: {}",i1,rel.as_ref().unwrap(),i2,istrue);
        if istrue {dotrue(int,cond,unless)} else {dofalse(int,cond,unless)}
    },
    name:"ifnum"
};

pub static IFEOF : Conditional = Conditional {
    name:"ifeof",
    _apply: |int,cond,unless| {
        match int.read_number()? as u8 {
            18 => dofalse(int,cond,unless),
            i => {
                if int.file_eof(i)? {dotrue(int,cond,unless)} else {dofalse(int,cond,unless)}
            }
        }
    }
};

use crate::ontology::Token;

fn get_if_token(cond:u8,int:&Interpreter) -> Result<Option<Token>,TeXError> {
    while int.has_next() {
        let next = int.next_token();
        match next.catcode {
            CategoryCode::Escape | CategoryCode::Active => {
                let currcond = match int.getcondition() {
                    Some((i, _)) => i == cond,
                    _ => unreachable!()
                };
                match int.get_command(&next.cmdname())? {
                    TeXCommand::Char(tk) => return Ok(Some(tk)),
                    TeXCommand::Primitive(e) if (*e == ELSE || *e == FI) && currcond => {
                        return Ok(None)
                    }
                    p => match p.as_expandable_with_protected() {
                        Ok(e) => e.expand(next, int)?,
                        Err(_) => return Ok(Some(next))
                    }
                }
            }
            _ => return Ok(Some(next))
        }
    }
    FileEnd!(int)
}

pub static IF : Conditional = Conditional {
    name:"if",
    _apply: |int,cond,unless| {
        let first = get_if_token(cond,int)?;
        let second = get_if_token(cond,int)?;
        let istrue = match (first,second) {
            (None,_) | (_,None) => false,
            (Some(a),Some(b)) => {
                if a.catcode == CategoryCode::Escape && b.catcode == CategoryCode::Escape { true } else {
                    a.char == b.char
                }
            }
        };
        if istrue {dotrue(int,cond,unless)} else {dofalse(int,cond,unless)}
    }
};

pub static IFTRUE : Conditional = Conditional {
    name:"iftrue",
    _apply: |_int,_cond,_unless| {
        todo!()
    }
};

pub static IFFALSE : Conditional = Conditional {
    name:"iffalse",
    _apply: |_int,_cond,_unless| {
        todo!()
    }
};

pub static IFDEFINED : Conditional = Conditional {
    name:"ifdefined",
    _apply: |_int,_cond,_unless| {
        todo!()
    }
};

pub static IFODD : Conditional = Conditional {
    name:"ifodd",
    _apply: |_int,_cond,_unless| {
        todo!()
    }
};

pub static IFDIM : Conditional = Conditional {
    name:"ifdim",
    _apply: |_int,_cond,_unless| {
        todo!()
    }
};

pub static IFCSNAME : Conditional = Conditional {
    name:"ifcsname",
    _apply: |_int,_cond,_unless| {
        todo!()
    }
};

pub static IFCAT : Conditional = Conditional {
    name:"ifcat",
    _apply: |_int,_cond,_unless| {
        todo!()
    }
};

pub static IFCASE : Conditional = Conditional {
    name:"ifcase",
    _apply: |_int,_cond,_unless| {
        todo!()
    }
};

pub static IFMMODE : Conditional = Conditional {
    name:"ifmmode",
    _apply: |_int,_cond,_unless| {
        todo!()
    }
};

pub static IFVMODE : Conditional = Conditional {
    name:"ifvmode",
    _apply: |_int,_cond,_unless| {
        todo!()
    }
};

pub static IFHMODE : Conditional = Conditional {
    name:"ifhmode",
    _apply: |_int,_cond,_unless| {
        todo!()
    }
};

pub static IFVOID : Conditional = Conditional {
    name:"ifvoid",
    _apply: |_int,_cond,_unless| {
        todo!()
    }
};

pub static IFVBOX : Conditional = Conditional {
    name:"ifvbox",
    _apply: |_int,_cond,_unless| {
        todo!()
    }
};

pub static IFHBOX : Conditional = Conditional {
    name:"ifhbox",
    _apply: |_int,_cond,_unless| {
        todo!()
    }
};

pub static IFINNER : Conditional = Conditional {
    name:"ifinner",
    _apply: |_int,_cond,_unless| {
        todo!()
    }
};

pub static IFINCSNAME : Conditional = Conditional {
    name:"ifincsname",
    _apply: |_int,_cond,_unless| {
        todo!()
    }
};

pub static IFFONTCHAR : Conditional = Conditional {
    name:"iffontchar",
    _apply: |_int,_cond,_unless| {
        todo!()
    }
};

pub fn conditional_commands() -> Vec<TeXCommand> {vec![
    TeXCommand::Primitive(&ELSE),
    TeXCommand::Primitive(&FI),
    TeXCommand::Primitive(&UNLESS),
    TeXCommand::Primitive(&OR),
    TeXCommand::Cond(&IFNUM),
    TeXCommand::Cond(&IFX),
    TeXCommand::Cond(&IFTRUE),
    TeXCommand::Cond(&IFFALSE),
    TeXCommand::Cond(&IF),
    TeXCommand::Cond(&IFEOF),
    TeXCommand::Cond(&IFDEFINED),
    TeXCommand::Cond(&IFODD),
    TeXCommand::Cond(&IFDIM),
    TeXCommand::Cond(&IFCSNAME),
    TeXCommand::Cond(&IFCAT),
    TeXCommand::Cond(&IFCASE),
    TeXCommand::Cond(&IFMMODE),
    TeXCommand::Cond(&IFHMODE),
    TeXCommand::Cond(&IFVMODE),
    TeXCommand::Cond(&IFVOID),
    TeXCommand::Cond(&IFVBOX),
    TeXCommand::Cond(&IFHBOX),
    TeXCommand::Cond(&IFINNER),
    TeXCommand::Cond(&IFINCSNAME),
    TeXCommand::Cond(&IFFONTCHAR),
]}