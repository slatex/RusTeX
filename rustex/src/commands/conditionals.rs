use crate::interpreter::Interpreter;
use crate::commands::{Conditional, PrimitiveExecutable, PrimitiveTeXCommand};
use crate::utils::TeXError;
use crate::catcodes::CategoryCode;
use crate::log;


pub fn dotrue(int: &Interpreter,cond:u8,unless:bool) -> Result<(),TeXError> {
    if unless {
        dofalse(int,cond,false)
    } else {
        int.setcondition(cond,true);
        Ok(())
    }
}

use crate::FileEnd;

pub fn false_loop(int:&Interpreter,initifs:u8,allowelse : bool) -> Result<(),TeXError> {
    use PrimitiveTeXCommand::*;
    let mut inifs = initifs;
    //log!("false loop: {}",inifs);
    while int.has_next() {
        let next = int.next_token();
        match next.catcode {
            CategoryCode::Escape | CategoryCode::Active => {
                match int.state_get_command(&next.cmdname()) {
                    None => {}
                    Some(p) => {
                        match *p.orig {
                            Primitive(x) if inifs == 0 && *x == FI => {
                                int.popcondition();
                                return Ok(())
                            }
                            Primitive(x) if allowelse && inifs == 0 && *x == ELSE => {
                                return Ok(())
                            }
                            Primitive(x) if *x == FI => inifs -=1,
                            Cond(_) => inifs += 1,
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

pub fn dofalse(int: &Interpreter,cond:u8,unless:bool) -> Result<(),TeXError> {
    if unless {
        dotrue(int,cond,false)
    } else {
        let inifs = int.setcondition(cond,false);
        false_loop(int,inifs,true)
    }
}

pub static FI : PrimitiveExecutable = PrimitiveExecutable {
    _apply: |_,int| {
        int.popcondition();
        Ok(())
    },
    expandable: true,
    name: "fi"
};

pub static UNLESS: PrimitiveExecutable = PrimitiveExecutable {
    name:"unless",
    _apply: |_rf,int| {
        let cnd = int.next_token();
        match cnd.catcode {
            CategoryCode::Escape | CategoryCode::Active => {
                let cmd = int.get_command(&cnd.cmdname())?;
                match *cmd.orig {
                    PrimitiveTeXCommand::Cond(c) => {
                        (c._apply)(int,int.pushcondition(),true)
                    }
                    _ => TeXErr!((int,Some(cnd)),"Expected conditional after \\unless")
                }
            }
            _ => TeXErr!((int,Some(cnd)),"Expected conditional after \\unless")
        }
    },
    expandable: true
};

pub static OR: PrimitiveExecutable = PrimitiveExecutable {
    name:"or",
    _apply: |_,int| {
        match int.getcondition() {
            None => TeXErr!((int,None),"extra \\or"),
            Some((_,None)) => {
                Ok(())
            }
            Some((_,_)) => {
                false_loop(int,0,false)?;
                Ok(())
            }
        }
    },
    expandable: true
};

use crate::TeXErr;

pub static ELSE: PrimitiveExecutable = PrimitiveExecutable {
    _apply: |_,int| {
        match int.getcondition() {
            None => TeXErr!((int,None),"extra \\else"),
            Some((_,None)) => {
                Ok(())
            }
            Some((_,_)) => {
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
            //(Active|Escape,Active|Escape) if !tka.expand || !tkb.expand => todo!(),
            (Active|Escape,Active|Escape) => {
                let cmd1 = int.state_get_command(&tka.cmdname());
                let cmd2 = int.state_get_command(&tkb.cmdname());
               match (cmd1,cmd2) {
                   (None,None) => true,
                   (None,_) => false,
                   (_,None) => false,
                   (Some(c1),Some(c2)) => {
                       log!("     Compare:\n    {}\n    {}",c1.meaning(&int.state_catcodes()),c2.meaning(&int.state_catcodes()));
                       match (tka.expand,tkb.expand) {
                           (true,true) | (false,false) => c1 == c2,
                           (true,false) | (false,true) => {
                               if !c1.expandable(true) && !c2.expandable(true) {
                                   c1 == c2
                               } else {false}
                           }
                       }
                   }
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
            _ =>  TeXErr!((int,None),"Expected '<','=' or '>' in \\ifnum")
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
                let ret = int.file_eof(i)?;
                log!("\\ifeof {}: {}",i,ret);
                if ret {dotrue(int,cond,unless)} else {dofalse(int,cond,unless)}
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
                let p = int.get_command(&next.cmdname())?;
                match &*p.orig {
                    PrimitiveTeXCommand::Char(tk) => return Ok(Some(tk.clone())),
                    PrimitiveTeXCommand::Primitive(e) if (**e == ELSE || **e == FI) && currcond => {
                        return Ok(None)
                    }
                    _ if p.expandable(true) && next.expand => {p.expand(next, int)?;}
                    _ => return Ok(Some(next))
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
        if unsafe{crate::LOG} {
            match (&first,&second) {
                (Some(ref a),Some(ref b)) => log!("   {}=={}",a,b),
                (None,Some(ref b)) => log!("   NONE=={}",b),
                (Some(ref a),None) => log!("   {}==NONE",a),
                (None,None) => log!("   NONE==NONE"),
            }
        }
        let istrue = match (first,second) {
            (None,_) | (_,None) => false,
            (Some(a),Some(b)) => {
                if a.catcode == CategoryCode::Escape && b.catcode == CategoryCode::Escape { true } else
                if a.catcode == CategoryCode::Escape || b.catcode == CategoryCode::Escape { false } else
                {
                    a.char == b.char
                }
            }
        };
        if istrue {dotrue(int,cond,unless)} else {dofalse(int,cond,unless)}
    }
};

pub static IFTRUE : Conditional = Conditional {
    name:"iftrue",
    _apply: |int,cond,unless| {
        dotrue(int,cond,unless)
    }
};

pub static IFFALSE : Conditional = Conditional {
    name:"iffalse",
    _apply: |int,cond,unless| {
        dofalse(int,cond,unless)
    }
};

pub static IFDEFINED : Conditional = Conditional {
    name:"ifdefined",
    _apply: |int,cond,unless| {
        let next = int.next_token();
        let istrue = match next.catcode {
            CategoryCode::Escape | CategoryCode::Active =>
                int.state_get_command(&next.cmdname()).is_some(),
            _ => TeXErr!((int,Some(next.clone())),"Expected command after \\ifdefined; got: {}",next)
        };
        if istrue { dotrue(int,cond,unless) } else { dofalse(int,cond,unless) }
    }
};

pub static IFCSNAME : Conditional = Conditional {
    name:"ifcsname",
    _apply: |int,cond,unless| {
        use crate::commands::primitives::csname;
        let cmdname = csname(int)?.into();
        let istrue = int.state_get_command(&cmdname).is_some();
        if istrue { dotrue(int,cond,unless) } else { dofalse(int,cond,unless) }
    }
};

pub static IFCAT : Conditional = Conditional {
    name:"ifcat",
    _apply: |int,cond,unless| {
        let first = match get_if_token(cond,int)? {
            Some(tk) => tk,
            None => return dofalse(int,cond,unless)
        };
        let second = match get_if_token(cond,int)? {
            Some(tk) => tk,
            None => return dofalse(int,cond,unless)
        };
        let cc1 = match first.catcode {
            CategoryCode::Escape | CategoryCode::Active => {
                let cmd = int.state_get_command(&first.cmdname());
                match cmd {
                    None => 255,
                    Some(cmd) => match &*cmd.orig {
                        PrimitiveTeXCommand::Char(tk) => tk.catcode.toint(),
                        _ => 255
                    }
                }
            }
            o => o.toint()
        };
        let cc2 = match second.catcode {
            CategoryCode::Escape | CategoryCode::Active => {
                let cmd = int.state_get_command(&first.cmdname());
                match cmd {
                    None => 255,
                    Some(cmd) => match &*cmd.orig {
                        PrimitiveTeXCommand::Char(tk) => tk.catcode.toint(),
                        _ => 255
                    }
                }
            }
            o => o.toint()
        };
        if cc1 == cc2 { dotrue(int,cond,unless) } else { dofalse(int,cond,unless) }
    }
};

pub static IFODD : Conditional = Conditional {
    name:"ifodd",
    _apply: |int,cond,unless| {
        if int.read_number()? % 2 == 1 { dotrue(int,cond,unless) } else { dofalse(int,cond,unless) }
    }
};

pub static IFCASE : Conditional = Conditional {
    name:"ifcase",
    _apply: |int,cond,unless| {
        let num = int.read_number()? as u8;
        if num == 0 {dotrue(int,cond,unless)} else {
            use PrimitiveTeXCommand::*;
            int.setcondition(cond,false);
            let mut inifs = 0 as u8;
            let mut currnum = 1 as u8;
            //log!("false loop: {}",inifs);
            while int.has_next() {
                let next = int.next_token();
                match next.catcode {
                    CategoryCode::Escape | CategoryCode::Active => {
                        match int.state_get_command(&next.cmdname()) {
                            None => {}
                            Some(p) => {
                                match *p.orig {
                                    Primitive(x) if inifs == 0 && *x == FI => {
                                        int.popcondition();
                                        return Ok(())
                                    }
                                    Primitive(x) if inifs == 0 && *x == ELSE => {
                                        return Ok(())
                                    }
                                    Primitive(x) if inifs == 0 && *x == OR => {
                                        if num == currnum { return Ok(()) } else { currnum += 1 }
                                    }
                                    Primitive(x) if *x == FI => inifs -=1,
                                    Cond(_) => inifs += 1,
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
    }
};

pub static IFDIM : Conditional = Conditional {
    name:"ifdim",
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

pub fn conditional_commands() -> Vec<PrimitiveTeXCommand> {vec![
    PrimitiveTeXCommand::Primitive(&ELSE),
    PrimitiveTeXCommand::Primitive(&FI),
    PrimitiveTeXCommand::Primitive(&UNLESS),
    PrimitiveTeXCommand::Primitive(&OR),
    PrimitiveTeXCommand::Cond(&IFNUM),
    PrimitiveTeXCommand::Cond(&IFX),
    PrimitiveTeXCommand::Cond(&IFTRUE),
    PrimitiveTeXCommand::Cond(&IFFALSE),
    PrimitiveTeXCommand::Cond(&IF),
    PrimitiveTeXCommand::Cond(&IFEOF),
    PrimitiveTeXCommand::Cond(&IFDEFINED),
    PrimitiveTeXCommand::Cond(&IFODD),
    PrimitiveTeXCommand::Cond(&IFDIM),
    PrimitiveTeXCommand::Cond(&IFCSNAME),
    PrimitiveTeXCommand::Cond(&IFCAT),
    PrimitiveTeXCommand::Cond(&IFCASE),
    PrimitiveTeXCommand::Cond(&IFMMODE),
    PrimitiveTeXCommand::Cond(&IFHMODE),
    PrimitiveTeXCommand::Cond(&IFVMODE),
    PrimitiveTeXCommand::Cond(&IFVOID),
    PrimitiveTeXCommand::Cond(&IFVBOX),
    PrimitiveTeXCommand::Cond(&IFHBOX),
    PrimitiveTeXCommand::Cond(&IFINNER),
    PrimitiveTeXCommand::Cond(&IFINCSNAME),
    PrimitiveTeXCommand::Cond(&IFFONTCHAR),
]}