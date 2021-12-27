pub mod primitives;
pub mod pdftex;
pub mod conditionals;
pub mod pgfsvg;

use std::cell::RefCell;
use crate::ontology::{Expansion, ExpansionRef, Token};
use crate::interpreter::{Interpreter, TeXMode};
use std::rc::Rc;
use std::fmt;
use std::fmt::{Display, Formatter, Pointer};
use crate::catcodes::{CategoryCode, CategoryCodeScheme};
use crate::interpreter::dimensions::{dimtostr, Numeric};
use crate::utils::{TeXError, TeXString,TeXStr};
use crate::{COPY_COMMANDS_FULL, log};

pub struct PrimitiveExecutable {
    pub (in crate) _apply:fn(tk:&mut Expansion,itp:&Interpreter) -> Result<(),TeXError>,
    pub expandable : bool,
    pub name: &'static str
}
impl PrimitiveExecutable {
    pub fn apply(&self,tk:&mut Expansion,itp:&Interpreter) -> Result<(),TeXError> {
        (self._apply)(tk,itp)
    }
}
pub struct Conditional {
    pub name: &'static str,
    _apply:fn(int:&Interpreter,cond:usize,unless:bool) -> Result<(),TeXError>
}
impl Conditional {
    pub fn expand(&self,int:&Interpreter) -> Result<(),TeXError> {
        (self._apply)(int,int.pushcondition(),false)
    }
}

impl PartialEq for PrimitiveExecutable {
    fn eq(&self, other: &Self) -> bool {
        self.name == other.name
    }
}

pub struct NumAssValue {
    pub name: &'static str,
    pub _assign: fn(rf:ExpansionRef,int: &Interpreter,global: bool) -> Result<(),TeXError>,
    pub _getvalue: fn(int: &Interpreter) -> Result<Numeric,TeXError>
}
impl PartialEq for NumAssValue {
    fn eq(&self, other: &Self) -> bool {
        self.name == other.name
    }
}

use crate::fonts::Font;

pub struct FontAssValue {
    pub name: &'static str,
    pub _assign: fn(rf:ExpansionRef,int: &Interpreter,global: bool) -> Result<(),TeXError>,
    pub _getvalue: fn(int: &Interpreter) -> Result<Rc<Font>,TeXError>
}
impl PartialEq for FontAssValue {
    fn eq(&self, other: &Self) -> bool {
        self.name == other.name
    }
}

pub struct TokAssValue {
    pub name: &'static str,
    pub _assign: fn(rf:ExpansionRef,int: &Interpreter,global: bool) -> Result<(),TeXError>,
    pub _getvalue: fn(int: &Interpreter) -> Result<Vec<Token>,TeXError>
}
impl PartialEq for TokAssValue {
    fn eq(&self, other: &Self) -> bool {
        self.name == other.name
    }
}

pub struct NumericCommand {
    pub _getvalue: fn(int: &Interpreter) -> Result<Numeric,TeXError>,
    pub name : &'static str
}

#[derive(PartialEq)]
pub struct RegisterReference {
    pub index: u8,
    pub name: &'static str
}

#[derive(PartialEq)]
pub struct DimenReference {
    pub index: u8,
    pub name: &'static str
}

#[derive(PartialEq)]
pub struct SkipReference {
    pub index: u8,
    pub name: &'static str
}

#[derive(PartialEq)]
pub struct MuSkipReference {
    pub index: u8,
    pub name: &'static str
}

#[derive(PartialEq)]
pub struct TokReference {
    pub index: u8,
    pub name: &'static str
}

pub struct PrimitiveAssignment {
    pub name: &'static str,
    pub _assign: fn(rf:ExpansionRef,int: &Interpreter,global: bool) -> Result<(),TeXError>
}
impl PartialEq for PrimitiveAssignment {
    fn eq(&self, other: &Self) -> bool {
        self.name == other.name
    }
}

#[derive(Clone)]
pub struct DefMacro {
    pub protected:bool,
    pub long:bool,
    pub sig:Signature,
    pub ret:Vec<Token>
}
impl Display for DefMacro {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        write!(f,"{}->{}{}{}",self.sig,"{",TokenList(&self.ret),"}")
    }
}
impl PartialEq for DefMacro {
    fn eq(&self, other: &Self) -> bool {
        self.long == other.long &&
            self.protected == other.protected &&
            self.sig == other.sig &&
            self.ret.len() == other.ret.len() &&
            {
                for i in 0..self.ret.len() {
                    if self.ret.get(i) != other.ret.get(i) {return false}
                }
                true
            }
    }
}

use crate::stomach::whatsits::{ExecutableWhatsit, MathGroup, MathKernel, SimpleWI, TeXBox, Whatsit, WIGroup};

pub struct ProvidesExecutableWhatsit {
    pub name: &'static str,
    pub _get: fn(tk:&Token,int: &Interpreter) -> Result<ExecutableWhatsit,TeXError>
}

pub struct ProvidesBox {
    pub name: &'static str,
    pub _get: fn(tk:&Token,int: &Interpreter) -> Result<TeXBox,TeXError>
}

impl PartialEq for ProvidesBox {
    fn eq(&self, other: &Self) -> bool {
        self.name == other.name
    }
}

pub struct MathWhatsit {
    pub name: &'static str,
    pub _get: fn(tk:&Token,int: &Interpreter,prev:Option<&mut MathGroup>) -> Result<Option<MathKernel>,TeXError>
}

impl PartialEq for MathWhatsit {
    fn eq(&self, other: &Self) -> bool {
        self.name == other.name
    }
}

pub struct SimpleWhatsit {
    pub name: &'static str,
    pub modes: fn(TeXMode) -> bool,
    pub _get: fn(tk:&Token,int: &Interpreter) -> Result<Whatsit,TeXError>
}

impl PartialEq for SimpleWhatsit {
    fn eq(&self, other: &Self) -> bool {
        self.name == other.name
    }
}

// -------------------------------------------------------------------------------------------------

#[derive(Clone)]
pub enum AssignableValue {
    Dim(u16),
    Register(u16),
    Skip(u16),
    MuSkip(u16),
    Toks(u16),
    Int(&'static NumAssValue),
    Font(&'static FontAssValue),
    Tok(&'static TokAssValue),
    FontRef(Rc<Font>),
    PrimReg(&'static RegisterReference),
    PrimDim(&'static DimenReference),
    PrimSkip(&'static SkipReference),
    PrimMuSkip(&'static MuSkipReference),
    PrimToks(&'static TokReference)
}

impl AssignableValue {
    pub fn name(&self) -> Option<TeXStr> {
        use AssignableValue::*;
        match self {
            Dim(_) | Register(_) | Skip(_) | Toks(_) | MuSkip(_) => None,
            Int(i) => Some(i.name.into()),
            Font(f) => Some(f.name.into()),
            Tok(r) => Some(r.name.into()),
            PrimReg(r) => Some(r.name.into()),
            PrimDim(d) => Some(d.name.into()),
            PrimSkip(d) => Some(d.name.into()),
            PrimMuSkip(d) => Some(d.name.into()),
            PrimToks(d) => Some(d.name.into()),
            FontRef(_) => None
        }
    }
}

use crate::TeXErr;

use crate::interpreter::state::StateChange;

pub trait ExternalCommand {
    fn expandable(&self) -> bool;
    fn assignable(&self) -> bool;
    fn has_num(&self) -> bool;
    fn has_whatsit(&self) -> bool;
    fn name(&self) -> String;
    fn execute(&self,int : &Interpreter) -> Result<(),TeXError>;
    fn expand(&self,exp:&mut Expansion,int:&Interpreter) -> Result<(),TeXError>;
    fn assign(&self,int:&Interpreter,global:bool) -> Result<(),TeXError>;
    fn get_num(&self,int:&Interpreter) -> Result<Numeric,TeXError>;
}

#[derive(Clone)]
pub enum ParamToken {
    Param(u8,Token),
    Token(Token)
}
impl PartialEq for ParamToken {
    fn eq(&self, other: &Self) -> bool {
        match (self,other) {
            (ParamToken::Param(a1,_),ParamToken::Param(b1,_)) => a1 == b1,
            (ParamToken::Token(a),ParamToken::Token(b)) => a == b,
            _ => false
        }
    }
}
impl Display for ParamToken {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        use ansi_term::Colour::*;
        match self {
            ParamToken::Param(0,u) => write!(f,"{}",Yellow.paint(TeXString(vec!(u.char,u.char)).to_string())),
            ParamToken::Param(i,u) => write!(f,"{}{}",Yellow.paint(TeXString(vec!(u.char)).to_string()),Yellow.paint(i.to_string())),
            ParamToken::Token(t) => write!(f,"{}",t)
        }
    }
}

#[derive(Clone)]
pub struct Signature {
    pub(in crate) elems:Vec<ParamToken>,
    pub(in crate) endswithbrace:bool,
    pub(in crate) arity:u8
}
impl PartialEq for Signature {
    fn eq(&self, other: &Self) -> bool {
        self.arity == other.arity &&
            self.endswithbrace == other.endswithbrace &&
            self.elems.len() == other.elems.len() && {
                for i in 0..self.elems.len() {
                    if self.elems.get(i) != other.elems.get(i) {return false}
                }
                return true
            }
    }
}
impl Display for Signature {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        for e in &self.elems {
            write!(f,"{}",e)?;
        }
        if self.endswithbrace {write!(f,"{}","{")} else {
            write!(f,"")
        }
    }
}
pub struct ParamList<'a>(&'a Vec<ParamToken>);

impl Display for ParamList<'_> {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        for e in self.0 {
            write!(f,"{}",e)?;
        }
        write!(f,"")
    }
}

pub struct TokenList<'a>(pub &'a Vec<Token>);
impl Display for TokenList<'_> {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        for e in self.0 {
            match e.catcode {
                CategoryCode::Escape => write!(f,"\\{}",e.name().to_string())?,
                _ => write!(f,"{}",TeXString(vec!(e.char)).to_string())?
            }
        }
        write!(f,"")
    }
}

#[derive(Clone)]
pub enum ProvidesWhatsit {
    Box(&'static ProvidesBox),
    Exec(&'static ProvidesExecutableWhatsit),
    Math(&'static MathWhatsit),
    Simple(&'static SimpleWhatsit),
}
impl ProvidesWhatsit {
    pub fn allowed_in(&self,mode:TeXMode) -> bool {
        use ProvidesWhatsit::*;
        match (mode,self) {
            (_,Box(_)) => true,
            (_,Exec(_)) => true,
            (TeXMode::Math,Math(_)) => true,
            (TeXMode::Displaymath,Math(_)) => true,
            (_,Math(_)) => false,
            (m,Simple(s))=> (s.modes)(m)
        }
    }
    pub fn name(&self) -> Option<TeXStr> {
        match self {
            ProvidesWhatsit::Exec(e) => Some(e.name.into()),
            ProvidesWhatsit::Box(b) => Some(b.name.into()),
            ProvidesWhatsit::Math(b) => Some(b.name.into()),
            ProvidesWhatsit::Simple(b) => Some(b.name.into()),
            _ => todo!()
        }
    }
    pub fn get(&self,tk:&Token,int:&Interpreter) -> Result<Whatsit,TeXError> {
        use ProvidesWhatsit::*;
        match self {
            Box(b) => Ok(Whatsit::Box((b._get)(tk,int)?)),
            Exec(e) => Ok(Whatsit::Exec(Rc::new((e._get)(tk,int)?))),
            Math(m) => {
                unreachable!()
            },//Ok(Whatsit::Math((m._get)(tk,int)?)),
            Simple(s) => Ok((s._get)(tk,int)?)
        }
    }
}

pub enum PrimitiveTeXCommand {
    Primitive(&'static PrimitiveExecutable),
    AV(AssignableValue),
    Ext(Rc<dyn ExternalCommand>),
    Cond(&'static Conditional),
    Num(&'static NumericCommand),
    Char(Token),
    Ass(&'static PrimitiveAssignment),
    Def(DefMacro),
    Whatsit(ProvidesWhatsit),
    MathChar(u32)
}

impl PrimitiveTeXCommand {
    pub fn as_ref(self,rf:ExpansionRef) -> TeXCommand {
        TeXCommand {
            orig: Rc::new(self),
            rf:Some(rf)
        }
    }
    pub fn as_command(self) -> TeXCommand {
        TeXCommand {
            orig: Rc::new(self),
            rf: None
        }
    }
    pub fn meaning(&self,catcodes:&CategoryCodeScheme) -> TeXString {
        use PrimitiveTeXCommand::*;
        let ret = match self {
            Char(c) => match c.catcode {
                CategoryCode::Space => "blank space ".into(),
                CategoryCode::Letter => {
                    let s : TeXString = "the letter ".into();
                    s + c.char.into()
                },
                CategoryCode::Other => {
                    let s : TeXString = "the character ".into();
                    s + c.char.into()
                },
                CategoryCode::BeginGroup => {
                    let s : TeXString = "begin-group character ".into();
                    s + c.char.into()
                }
                CategoryCode::EndGroup => {
                    let s : TeXString = "end-group character ".into();
                    s + c.char.into()
                }
                _ => todo!("{}",self)
            }
            Def(d) => {
                let escape : TeXString = if catcodes.escapechar != 255 {catcodes.escapechar.into()} else {"".into()};
                let mut meaning : TeXString = "".into();
                if d.protected {
                    meaning += escape.clone();
                    meaning += "protected "
                }
                if d.long {
                    meaning += escape.clone();
                    meaning += "long "
                }
                meaning += "macro:";
                for s in &d.sig.elems {
                    match s {
                        ParamToken::Token(tk) => {
                            meaning += crate::interpreter::tokens_to_string(&vec!(tk.clone()),catcodes)
                        },
                        ParamToken::Param(0,u) => {
                            meaning += u.char;
                            meaning += u.char
                        },
                        ParamToken::Param(i,u) => {
                            meaning += u.char;
                            meaning += i.to_string();
                        }
                    }
                }
                meaning += "->";
                meaning += crate::interpreter::tokens_to_string(&d.ret,catcodes);
                meaning
            }
            Num(ic) => {
                let ret : TeXString = if catcodes.escapechar != 255 {catcodes.escapechar.into()} else {"".into()};
                ret + ic.name.into()
            },
            Primitive(p) => {
                let ret : TeXString = if catcodes.escapechar != 255 {catcodes.escapechar.into()} else {"".into()};
                ret + p.name.into()
            }
            AV(AssignableValue::FontRef(f)) => TeXString::from("select font ") + TeXString::from(f.file.name.clone()) + TeXString::from(match f.at {
                Some(vl) => " at ".to_string() + &dimtostr(vl),
                None => "".to_string()
            }),
            AV(AssignableValue::Int(p)) => {
                let mut ret : TeXString = if catcodes.escapechar != 255 {catcodes.escapechar.into()} else {"".into()};
                ret + p.name.into()
            },
            AV(AssignableValue::Dim(i)) => {
                let mut ret : TeXString = if catcodes.escapechar != 255 {catcodes.escapechar.into()} else {"".into()};
                ret + "dimen".into() + i.to_string().into()
            },
            AV(AssignableValue::Skip(i)) => {
                let mut ret : TeXString = if catcodes.escapechar != 255 {catcodes.escapechar.into()} else {"".into()};
                ret + "skip".into() + i.to_string().into()
            },
            AV(AssignableValue::Register(i)) => {
                let mut ret : TeXString = if catcodes.escapechar != 255 {catcodes.escapechar.into()} else {"".into()};
                ret + "count".into() + i.to_string().into()
            },
            AV(AssignableValue::PrimReg(p)) => {
                let mut ret : TeXString = if catcodes.escapechar != 255 {catcodes.escapechar.into()} else {"".into()};
                ret + p.name.into()
            },
            AV(AssignableValue::PrimToks(p)) => {
                let mut ret : TeXString = if catcodes.escapechar != 255 {catcodes.escapechar.into()} else {"".into()};
                ret + p.name.into()
            },
            AV(AssignableValue::PrimDim(p)) => {
                let mut ret : TeXString = if catcodes.escapechar != 255 {catcodes.escapechar.into()} else {"".into()};
                ret + p.name.into()
            },
            AV(AssignableValue::PrimSkip(p)) => {
                let mut ret : TeXString = if catcodes.escapechar != 255 {catcodes.escapechar.into()} else {"".into()};
                ret + p.name.into()
            },
            Cond(c) => {
                let mut ret : TeXString = if catcodes.escapechar != 255 {catcodes.escapechar.into()} else {"".into()};
                ret + c.name.into()
            },
            Whatsit(ProvidesWhatsit::Math(m)) => {
                let mut ret : TeXString = if catcodes.escapechar != 255 {catcodes.escapechar.into()} else {"".into()};
                ret + m.name.into()
            },
            Ass(p) => {
                let mut ret : TeXString = if catcodes.escapechar != 255 {catcodes.escapechar.into()} else {"".into()};
                ret + p.name.into()
            },
            MathChar(i) => {
                let mut ret : TeXString = if catcodes.escapechar != 255 {catcodes.escapechar.into()} else {"".into()};
                ret += "mathchar\"".to_owned() + &format!("{:X}", i);
                ret
            }
            Whatsit(ProvidesWhatsit::Simple(s)) => {
                let mut ret : TeXString = if catcodes.escapechar != 255 {catcodes.escapechar.into()} else {"".into()};
                ret + s.name.into()
            },
            _ => todo!("{}",self)
        };
        ret
    }
    pub fn name(&self) -> Option<TeXStr> {
        use PrimitiveTeXCommand::*;
        match self {
            Char(_) => None,
            Ass(a) => Some(a.name.into()),
            Primitive(pr) => Some(pr.name.into()),
            AV(av) => av.name(),
            Ext(jr) => Some(jr.name().as_str().into()),
            Cond(c) => Some(c.name.into()),
            Num(i) => Some(i.name.into()),
            Def(_) => None,
            Whatsit(wi) => wi.name(),
            MathChar(_) => None
        }
    }
    pub fn expandable(&self,allowprotected:bool) -> bool {
        use PrimitiveTeXCommand::*;
        match self {
            Cond(_) => true,
            Ext(e) if e.expandable() => true,
            Primitive(p) if p.expandable => true,
            Def(d) if allowprotected || !d.protected => true,
            _ => false
        }
    }
    pub fn has_num(&self) -> bool {
        match self {
            PrimitiveTeXCommand::AV(ref av) => match av {
                AssignableValue::Register(_) => true,
                AssignableValue::Dim(_) => true,
                AssignableValue::Skip(_) => true,
                AssignableValue::MuSkip(_) => true,
                AssignableValue::Int(_) => true,
                AssignableValue::PrimDim(_) => true,
                AssignableValue::PrimReg(_) => true,
                AssignableValue::PrimSkip(_) => true,
                AssignableValue::PrimMuSkip(_) => true,
                _ => false
            },
            PrimitiveTeXCommand::Ext(ext) if ext.has_num() => true,
            PrimitiveTeXCommand::Num(_) => true,
            PrimitiveTeXCommand::MathChar(_) => true,
            PrimitiveTeXCommand::Char(_) => true,
            _ => false
        }
    }
    pub fn assignable(&self) -> bool {
        match self {
            PrimitiveTeXCommand::Ass(_) => true,
            PrimitiveTeXCommand::AV(_) => true,
            PrimitiveTeXCommand::Ext(ext) if ext.assignable() => true,
            _ => false
        }
    }
    pub fn has_whatsit(&self) -> bool {
        match self {
            PrimitiveTeXCommand::Whatsit(_) => true,
            PrimitiveTeXCommand::Ext(rc) => rc.has_whatsit(),
            _ => false
        }
    }
    pub fn get_num(&self,int:&Interpreter) -> Result<Numeric,TeXError> {
        use PrimitiveTeXCommand::*;
        use AssignableValue::*;
        use crate::utils::u8toi16;
        match self {
            AV(Dim(i)) => Ok(Numeric::Dim(int.state_dimension(*i as i32))),
            AV(Register(i)) => Ok(Numeric::Int(int.state_register(*i as i32))),
            AV(Skip(i)) => Ok(Numeric::Skip(int.state_skip(*i as i32))),
            AV(MuSkip(i)) => Ok(Numeric::MuSkip(int.state_muskip(*i as i32))),
            AV(AssignableValue::Int(i)) => Ok((i._getvalue)(int)?),
            PrimitiveTeXCommand::Num(i) => Ok((i._getvalue)(int)?),
            AV(PrimReg(r)) => Ok(Numeric::Int(int.state_register(-(r.index as i32)))),
            AV(PrimDim(r)) => Ok(Numeric::Dim(int.state_dimension(-(r.index as i32)))),
            AV(PrimSkip(r)) => Ok(Numeric::Skip(int.state_skip(-(r.index as i32)))),
            AV(PrimMuSkip(r)) => Ok(Numeric::MuSkip(int.state_muskip(-(r.index as i32)))),
            Ext(r) => r.get_num(int),
            Char(u) => Ok(Numeric::Int(u.char as i32)),
            MathChar(u) => Ok(Numeric::Int(*u as i32)),
            _ => unreachable!("{}",self)
        }
    }
    pub fn get_expansion(&self,tk:Token,int:&Interpreter,cmd:Rc<TeXCommand>) -> Result<Option<Expansion>,TeXError> {
        use PrimitiveTeXCommand::*;
        log!("Expanding {}",tk);
        match self {
            Cond(c) => {c.expand(int)?; Ok(None)},
            Primitive(p) => {
                let mut exp = Expansion(tk,cmd,vec!());
                (p._apply)(&mut exp,int)?;
                Ok(Some(exp))
            },
            Ext(p) => {
                let mut exp = Expansion(tk,cmd,vec!());
                p.expand(&mut exp,int)?;
                Ok(Some(exp))
            },
            Def(d) => Ok(Some(self.do_def(tk, int, d,cmd)?)),
            _ => unreachable!()
        }
    }
    pub fn expand(&self,tk:Token,int:&Interpreter,cmd:Rc<TeXCommand>) -> Result<(),TeXError> {
        match self.get_expansion(tk,int,cmd)? {
            Some(exp) => Ok(int.push_expansion(exp)),
            None => Ok(())
        }
    }
    fn do_def(&self, tk:Token, int:&Interpreter, d:&DefMacro,cmd:Rc<TeXCommand>) -> Result<Expansion,TeXError> {
        /*if /*int.current_line().starts_with("/usr/share/texlive/texmf-dist/tex/latex/l3kernel/expl3-code.tex (31587") &&*/ tk.cmdname().to_string() == "beamer@@decodefind" { // {
             println!("Here {}  >>{}",int.current_line(),int.preview());
             //TeXErr!((int,Some(tk)),"Have a stack trace");
             //TeXErr!((int,None),"Here!!");
             //println!("Maxdimen: {} = {}",int.state_dimension(10),Numeric::Dim(int.state_dimension(10)));
             unsafe{crate::LOG = true}
             print!("");
        }*/
        /*if unsafe{crate::LOG} && tk.name().to_string() == "__int_step:NNnnnn" {
            println!("Here! {}",int.preview());
            print!("")
        }*/
        log!("{}",d);
        if unsafe{crate::LOG} {
            log!("    >>{}",int.preview())
        }
        let mut args : Vec<Vec<Token>> = Vec::new();
        let mut i = 0;
        while i < d.sig.elems.len() {
            match d.sig.elems.get(i).unwrap() {
                ParamToken::Token(tk) => {
                    //int.assert_has_next()?;
                    let next = int.next_token();
                    if *tk != next {
                        TeXErr!((int,Some(next.clone())),"Expected >{}<; found >{}< (in {})\n{}  >>{}",tk,next,d,int.current_line(),int.preview())
                    }
                    i += 1;
                }
                ParamToken::Param(_,_) => {
                    i +=1;
                    match d.sig.elems.get(i) {
                        None if d.sig.endswithbrace => {
                            let mut retarg : Vec<Token> = vec!();
                            loop {
                                //int.assert_has_next()?;
                                let next = int.next_token();
                                match next.catcode {
                                    CategoryCode::BeginGroup => {
                                        int.requeue(next);
                                        break
                                    }
                                    _ => retarg.push(next)
                                }
                            }
                            args.push(retarg)
                        },
                        None | Some(ParamToken::Param(_,_)) => {
                            int.skip_ws();
                            let next = int.read_argument()?;
                            args.push(next);
                        },
                        Some(ParamToken::Token(itk)) => {
                            let mut delim : Vec<Token> = vec!(itk.clone());
                            i +=1;
                            while i < d.sig.elems.len() {
                                match d.sig.elems.get(i) {
                                    Some(ParamToken::Token(t)) => {
                                        delim.push(t.clone());
                                        i += 1;
                                    },
                                    _ => break
                                }
                            }
                            let mut retarg : Vec<Token> = vec!();
                            let mut groups = 0;
                            let mut totalgroups = 0;
                            while int.has_next() {
                                let next = int.next_token();
                                match next.catcode {
                                    CategoryCode::BeginGroup if groups == 0 => {
                                        groups += 1;
                                        totalgroups += 1;
                                    }
                                    CategoryCode::BeginGroup => groups += 1,
                                    CategoryCode::EndGroup => groups -=1,
                                    _ => ()
                                }
                                if groups < 0 {
                                    TeXErr!((int,Some(next)),"Missing begin group token somewhere: {}\nsofar: {}\nin: {}:{}",TokenList(&delim),TokenList(&retarg),tk.name(),d)
                                }
                                retarg.push(next);
                                if groups == 0 && retarg.ends_with(delim.as_slice()) {break}
                            }
                            //int.assert_has_next()?;
                            for _ in 0..delim.len() { retarg.pop(); }
                            if totalgroups == 1 &&
                                match retarg.first() {Some(tk) => tk.catcode == CategoryCode::BeginGroup, _ => false} &&
                                match retarg.last() {Some(tk) => tk.catcode == CategoryCode::EndGroup, _ => false} {
                                retarg = Vec::from(retarg.get(1..retarg.len()-1).unwrap());
                            }
                            args.push(retarg)
                        }
                    }
                }
            }
        }
        if unsafe{crate::LOG} {
            log!("    args:");
            for (i, a) in args.iter().enumerate() {
                log!("    {}:{}",i+1,TokenList(a));
            }
        }
        let mut exp = Expansion(tk,cmd,vec!());
        let rf = exp.get_ref();
        let mut i = 0;
        while i < d.ret.len() {
            let tk = d.ret.get(i).unwrap();
            match tk.catcode {
                CategoryCode::Parameter => {
                    i += 1;
                    let next = d.ret.get(i).unwrap();
                    match next.catcode {
                        CategoryCode::Parameter => {
                            i += 1;
                            exp.2.push(tk.copied(rf.clone()))
                        }
                        _ => {
                            i += 1;
                            let arg = next.char - 49;
                            if arg >= d.sig.arity {
                                TeXErr!((int,Some(next.clone())),"Expected argument number; got:{}",next)
                            }
                            for tk in args.get(arg as usize).unwrap() { exp.2.push(tk.cloned()) }
                        }
                    }
                }
                _ => {
                    i += 1;
                    exp.2.push(tk.copied(rf.clone()))
                },
            }
        }
        Ok(exp)
    }
    pub fn assign(&self,tk:Token,int:&Interpreter,globally:bool,cmd:Rc<TeXCommand>) -> Result<(),TeXError> {
        use crate::utils::u8toi16;
        use crate::commands::primitives::GLOBALDEFS;
        use PrimitiveTeXCommand::*;
        use crate::stomach::whatsits::Whatsit;

        let globals = int.state_register(-(GLOBALDEFS.index as i32));
        let global = !(globals < 0) && ( globally || globals > 0 );
        let rf = ExpansionRef(tk,cmd);
        match self {
            Ass(p) if **p == crate::commands::primitives::SETBOX => {
                return (p._assign)(rf,int, global)
            }
            Ass(p) => (p._assign)(rf,int, global),
            AV(av) => match av {
                AssignableValue::Int(d) => (d._assign)(rf,int, global),
                AssignableValue::Register(i) => {
                    int.read_eq();
                    let num = int.read_number()?;
                    log!("Assign register {} to {}",i,num);
                    int.change_state(StateChange::Register(*i as i32, num, global));
                    Ok(())
                }
                AssignableValue::Font(f) => {
                    (f._assign)(rf,int,global);
                    Ok(())
                }
                AssignableValue::Tok(t) => (t._assign)(rf,int,global),
                AssignableValue::FontRef(f) => {
                    int.change_state(StateChange::Font(f.clone(),global));
                    int.stomach.borrow_mut().add(int,Whatsit::GroupOpen(
                        WIGroup::FontChange(f.clone(),int.update_reference(&rf.0),global,vec!())
                    ))
                },
                AssignableValue::Dim(i) => {
                    int.read_eq();
                    log!("Assigning dimen {}",i);
                    let num = int.read_dimension()?;
                    log!("Assign dimen register {} to {}",i,dimtostr(num));
                    int.change_state(StateChange::Dimen(*i as i32, num, global));
                    Ok(())
                }
                AssignableValue::Skip(i) => {
                    int.read_eq();
                    let num = int.read_skip()?;
                    log!("Assign skip register {} to {}",i,num);
                    int.change_state(StateChange::Skip(*i as i32, num, global));
                    Ok(())
                }
                AssignableValue::MuSkip(i) => {
                    int.read_eq();
                    let num = int.read_muskip()?;
                    log!("Assign muskip register {} to {}",i,num);
                    int.change_state(StateChange::MuSkip(*i as i32, num, global));
                    Ok(())
                },
                AssignableValue::PrimSkip(r) => {
                    int.read_eq();
                    let num = int.read_skip()?;
                    log!("Assign {} to {}",r.name,num);
                    int.change_state(StateChange::Skip(-(r.index as i32), num, global));
                    Ok(())
                },
                AssignableValue::PrimMuSkip(r) => {
                    int.read_eq();
                    let num = int.read_muskip()?;
                    log!("Assign {} to {}",r.name,num);
                    int.change_state(StateChange::MuSkip(-(r.index as i32), num, global));
                    Ok(())
                },
                AssignableValue::Toks(i) => {
                    int.read_eq();
                    int.expand_until(false);
                    let next = int.next_token();
                    let toks = match next.catcode {
                        CategoryCode::BeginGroup => {
                            int.requeue(next);
                            int.read_balanced_argument(false,false,false,true)?
                        }
                        CategoryCode::Escape | CategoryCode::Active => {
                            let cmd = int.get_command(&next.cmdname())?;
                            match &*cmd.orig {
                                PrimitiveTeXCommand::AV(AssignableValue::Toks(j)) => int.state_tokens(*j as i32),
                                PrimitiveTeXCommand::AV(AssignableValue::PrimToks(j)) => int.state_tokens(-(j.index as i32)),
                                _ => TeXErr!((int,None),"Expected balanced argument or token register in token assignment")
                            }
                        }
                        _ => TeXErr!((int,None),"Expected balanced argument or token register in token assignment")
                    };
                    int.change_state(StateChange::Tokens(*i as i32, toks.iter().map(|x| x.cloned()).collect(), global));
                    Ok(())
                },
                AssignableValue::PrimToks(r) => {
                    int.read_eq();
                    int.expand_until(false);
                    let next = int.next_token();
                    let toks = match next.catcode {
                        CategoryCode::BeginGroup => {
                            int.requeue(next);
                            int.read_balanced_argument(false,false,false,true)?
                        }
                        CategoryCode::Escape | CategoryCode::Active => {
                            let cmd = int.get_command(&next.cmdname())?;
                            match &*cmd.orig {
                                PrimitiveTeXCommand::AV(AssignableValue::Toks(j)) => int.state_tokens(*j as i32),
                                PrimitiveTeXCommand::AV(AssignableValue::PrimToks(j)) => int.state_tokens(-(j.index as i32)),
                                _ => TeXErr!((int,None),"Expected balanced argument or token register in token assignment")
                            }
                        }
                        _ => TeXErr!((int,None),"Expected balanced argument or token register in token assignment")
                    };
                    int.change_state(StateChange::Tokens(-(r.index as i32), toks.iter().map(|x| x.cloned()).collect(), global));
                    Ok(())
                },
                AssignableValue::PrimReg(r) => {
                    int.read_eq();
                    let num = int.read_number()?;
                    log!("Assign {} to {}",r.name,num);
                    int.change_state(StateChange::Register(-(r.index as i32), num, global));
                    Ok(())
                },
                AssignableValue::PrimDim(r) => {
                    int.read_eq();
                    let num = int.read_dimension()?;
                    log!("Assign {} to {}",r.name,dimtostr(num));
                    int.change_state(StateChange::Dimen(-(r.index as i32), num, global));
                    Ok(())
                }
            },
            Ext(ext) => ext.assign(int, global),
            _ => unreachable!()
        }?;
        int.insert_afterassignment();
        Ok(())
    }
}

#[derive(Clone)]
pub struct TeXCommand {
    pub orig:Rc<PrimitiveTeXCommand>,
    pub rf:Option<ExpansionRef>
}

impl PartialEq for TeXCommand {
    fn eq(&self, other: &Self) -> bool {
        *self.orig == *other.orig
    }
}

impl PartialEq for PrimitiveTeXCommand {
    fn eq(&self, other: &Self) -> bool {
        use PrimitiveTeXCommand::*;
        match (self,other) {
            (Primitive(a),Primitive(b)) => a.name == b.name,
            (AV(a),AV(b)) => a.name() == b.name(),
            (Ext(a),Ext(b)) => a.name() == b.name(),
            (Cond(a),Cond(b)) => a.name == b.name,
            (Ass(a),Ass(b)) => a.name == b.name,
            (Whatsit(a),Whatsit(b)) => a.name() == b.name(),
            (MathChar(a),MathChar(b)) => a==b,
            (Char(tk),Char(tkb)) => tk.char == tkb.char && tk.catcode == tkb.catcode,
            (Def(a),Def(b)) => a == b,
            _ => false
        }
    }
}

impl fmt::Display for PrimitiveTeXCommand {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        use PrimitiveTeXCommand::*;
        match self {
            Primitive(p) =>
                write!(f,"\\{}",p.name),
            Cond(p) =>
                write!(f,"\\{}",p.name),
            Num(p) =>
                write!(f,"\\{}",p.name),
            Ext(p) =>
                write!(f,"External \\{}",p.name()),
            AV(av) => av.fmt(f),
            Def(d) => std::fmt::Display::fmt(&d, f),
            Whatsit(wi) => wi.fmt(f),
            Ass(a) =>
                write!(f,"\\{}",a.name),
            Char(tk) => write!(f,"CHAR({})",tk),
            MathChar(i) => write!(f,"MATHCHAR({})",i)
        }
    }
}

impl fmt::Display for TeXCommand {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        write!(f,"{}",*self.orig)
    }
}

impl TeXCommand {
    pub fn as_ref(self,tk:Token) -> TeXCommand {
        if COPY_COMMANDS_FULL {
            TeXCommand {
                orig:Rc::clone(&self.orig),
                rf:Some(ExpansionRef(tk,Rc::new(self)))
            }
        } else { self }
    }
    pub fn meaning(&self,catcodes:&CategoryCodeScheme) -> TeXString {
        self.orig.meaning(catcodes)
    }
    pub fn name(&self) -> Option<TeXStr> {
        self.orig.name()
    }
    pub fn expandable(&self,allowprotected:bool) -> bool {self.orig.expandable(allowprotected)}
    pub fn has_num(&self) -> bool {self.orig.has_num()}
    pub fn assignable(&self) -> bool {self.orig.assignable()}
    pub fn has_whatsit(&self) -> bool {self.orig.has_whatsit()}
    pub fn get_num(&self,int:&Interpreter) -> Result<Numeric,TeXError> { self.orig.get_num(int) }
    pub fn get_expansion(self,tk:Token,int:&Interpreter) -> Result<Option<Expansion>,TeXError> {
        let o = self.orig.clone();
        o.get_expansion(tk,int,Rc::new(self))
    }
    pub fn expand(self,tk:Token,int:&Interpreter) -> Result<(),TeXError> {
        self.orig.clone().expand(tk,int,Rc::new(self))
    }
    pub fn assign(self,tk:Token,int:&Interpreter,globally:bool) -> Result<(),TeXError> {
        self.orig.clone().assign(tk,int,globally,Rc::new(self))
    }
}