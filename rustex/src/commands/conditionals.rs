use std::ops::Deref;
use crate::interpreter::Interpreter;
use crate::ontology::{Expansion, Token};
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
    Err(TeXError::new("File ended unexpectedly".to_string()))
}

fn dofalse(int: &Interpreter,cond:u8,unless:bool) -> Result<(),TeXError> {
    if unless {
        dotrue(int,cond,false)
    } else {
        let initifs = int.setcondition(cond,false);
        let mut inifs = initifs;
        false_loop(int,inifs,true)
    }
}

pub static FI : PrimitiveExecutable = PrimitiveExecutable {
    _apply: |tk,int| {
        int.popcondition();
        Ok(())
    },
    expandable: true,
    name: "fi"
};

pub static UNLESS: PrimitiveExecutable = PrimitiveExecutable {
    name:"unless",
    _apply: |tk,int| {
        todo!()
    },
    expandable: true
};

pub static OR: PrimitiveExecutable = PrimitiveExecutable {
    name:"or",
    _apply: |tk,int| {
        todo!()
    },
    expandable: true
};

pub static ELSE: PrimitiveExecutable = PrimitiveExecutable {
    _apply: |tk,int| {
        match int.getcondition() {
            None => Err(TeXError::new("extra \\else".to_string())),
            Some((_,None)) => {
                Ok(())
            }
            Some((i,_)) => {
                false_loop(int,0,false)?;
                Ok(())
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
                   _ => todo!()
               }
            }
            (a,b) if matches!(a,b) => tka.char == tkb.char,
            _ => false
        };
        log!("\\ifx {}{}: {}",tka,tkb,istrue);
        if istrue {dotrue(int,cond,unless)} else {dofalse(int,cond,unless)}
    }
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
        log!("\\ifnum {}{}{}: {}",i1,rel.as_ref().unwrap(),i2,istrue);
        if istrue {dotrue(int,cond,unless)} else {dofalse(int,cond,unless)}
    },
    name:"ifnum"
};

pub static IFTRUE : Conditional = Conditional {
    name:"iftrue",
    _apply: |int,cond,unless| {
        todo!()
    }
};

pub static IFFALSE : Conditional = Conditional {
    name:"iffalse",
    _apply: |int,cond,unless| {
        todo!()
    }
};

pub static IF : Conditional = Conditional {
    name:"if",
    _apply: |int,cond,unless| {
        todo!()
    }
};

pub static IFEOF : Conditional = Conditional {
    name:"ifeof",
    _apply: |int,cond,unless| {
        todo!()
    }
};

pub static IFDEFINED : Conditional = Conditional {
    name:"ifdefined",
    _apply: |int,cond,unless| {
        todo!()
    }
};

pub static IFODD : Conditional = Conditional {
    name:"ifodd",
    _apply: |int,cond,unless| {
        todo!()
    }
};

pub static IFDIM : Conditional = Conditional {
    name:"ifdim",
    _apply: |int,cond,unless| {
        todo!()
    }
};

pub static IFCSNAME : Conditional = Conditional {
    name:"ifcsname",
    _apply: |int,cond,unless| {
        todo!()
    }
};

pub static IFCAT : Conditional = Conditional {
    name:"ifcat",
    _apply: |int,cond,unless| {
        todo!()
    }
};

pub static IFCASE : Conditional = Conditional {
    name:"ifcase",
    _apply: |int,cond,unless| {
        todo!()
    }
};

pub static IFMMODE : Conditional = Conditional {
    name:"ifmmode",
    _apply: |int,cond,unless| {
        todo!()
    }
};

pub static IFVMODE : Conditional = Conditional {
    name:"ifvmode",
    _apply: |int,cond,unless| {
        todo!()
    }
};

pub static IFHMODE : Conditional = Conditional {
    name:"ifhmode",
    _apply: |int,cond,unless| {
        todo!()
    }
};

pub static IFVOID : Conditional = Conditional {
    name:"ifvoid",
    _apply: |int,cond,unless| {
        todo!()
    }
};

pub static IFVBOX : Conditional = Conditional {
    name:"ifvbox",
    _apply: |int,cond,unless| {
        todo!()
    }
};

pub static IFHBOX : Conditional = Conditional {
    name:"ifhbox",
    _apply: |int,cond,unless| {
        todo!()
    }
};

pub static IFINNER : Conditional = Conditional {
    name:"ifinner",
    _apply: |int,cond,unless| {
        todo!()
    }
};

pub static IFINCSNAME : Conditional = Conditional {
    name:"ifincsname",
    _apply: |int,cond,unless| {
        todo!()
    }
};

pub static IFFONTCHAR : Conditional = Conditional {
    name:"iffontchar",
    _apply: |int,cond,unless| {
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