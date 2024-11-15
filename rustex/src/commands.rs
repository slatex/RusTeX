pub mod primitives;
pub mod pdftex;
pub mod conditionals;
pub mod pgfsvg; pub mod rustex_specials;
pub mod latex_bindings;
pub(crate) mod registers;

use crate::ontology::{Expansion, ExpansionRef, Token};
use crate::interpreter::{Interpreter, TeXMode};
use std::fmt;
use std::fmt::{Display, Formatter, Pointer};
use std::sync::Arc;
use crate::catcodes::{CategoryCode, CategoryCodeScheme};
use crate::interpreter::dimensions::{dimtostr, Numeric};
use crate::utils::{TeXError, TeXString,TeXStr};
use crate::{FileEnd, log};
use crate::commands::ProvidesWhatsit::Exec;

pub struct PrimitiveExecutable {
    pub (in crate) _apply:fn(tk:&mut Expansion,itp:&mut Interpreter) -> Result<(),TeXError>,
    pub expandable : bool,
    pub name: &'static str
}
impl PrimitiveExecutable {
    pub fn apply(&self,tk:&mut Expansion,itp:&mut Interpreter) -> Result<(),TeXError> {
        (self._apply)(tk,itp)
    }
}
pub struct Conditional {
    pub name: &'static str,
    _apply:fn(int:&mut Interpreter,cond:usize,unless:bool) -> Result<(),TeXError>
}
impl Conditional {
    pub fn expand(&self,int:&mut Interpreter) -> Result<(),TeXError> {
        let i = int.state.conditions.len();
        int.push_condition(None);
        (self._apply)(int,i,false)
    }
}

impl PartialEq for PrimitiveExecutable {
    fn eq(&self, other: &Self) -> bool {
        self.name == other.name
    }
}

pub struct NumAssValue {
    pub name: &'static str,
    pub _assign: fn(rf:ExpansionRef,int: &mut Interpreter,global: bool) -> Result<(),TeXError>,
    pub _getvalue: fn(int: &mut Interpreter) -> Result<Numeric,TeXError>
}
impl PartialEq for NumAssValue {
    fn eq(&self, other: &Self) -> bool {
        self.name == other.name
    }
}

use crate::fonts::{ArcFont, Font};

pub struct FontAssValue {
    pub name: &'static str,
    pub _assign: fn(rf:ExpansionRef,int: &mut Interpreter,global: bool) -> Result<(),TeXError>,
    pub _getvalue: fn(int: &mut Interpreter) -> Result<ArcFont,TeXError>
}
impl PartialEq for FontAssValue {
    fn eq(&self, other: &Self) -> bool {
        self.name == other.name
    }
}

pub struct TokAssValue {
    pub name: &'static str,
    pub _assign: fn(rf:ExpansionRef,int: &mut Interpreter,global: bool) -> Result<(),TeXError>,
    pub _getvalue: fn(int: &mut Interpreter) -> Result<Vec<Token>,TeXError>
}
impl PartialEq for TokAssValue {
    fn eq(&self, other: &Self) -> bool {
        self.name == other.name
    }
}

pub struct NumericCommand {
    pub _getvalue: fn(int: &mut Interpreter) -> Result<Numeric,TeXError>,
    pub name : &'static str
}

#[derive(PartialEq)]
pub struct RegisterReference {
    pub index: usize,
    pub name: &'static str
}

#[derive(PartialEq)]
pub struct DimenReference {
    pub index: usize,
    pub name: &'static str
}

#[derive(PartialEq)]
pub struct SkipReference {
    pub index: usize,
    pub name: &'static str
}

#[derive(PartialEq)]
pub struct MuSkipReference {
    pub index: usize,
    pub name: &'static str
}

#[derive(PartialEq)]
pub struct TokReference {
    pub index: usize,
    pub name: &'static str
}

pub struct PrimitiveAssignment {
    pub name: &'static str,
    pub _assign: fn(rf:ExpansionRef,int: &mut Interpreter,global: bool) -> Result<(),TeXError>
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
impl DefMacro {
    fn clean(&self) -> DefMacro {
        let mut nret : Vec<Token> = vec!();
        for x in &self.ret {
            nret.push(x.clean())
        }
        DefMacro {
            protected:self.protected,
            long:self.long,
            sig:self.sig.clean(),
            ret:nret
        }
    }
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

use crate::stomach::whatsits::{ExecutableWhatsit, Whatsit, WhatsitTrait};
use crate::stomach::math::{MathGroup,MathKernel};
use crate::stomach::boxes::TeXBox;

pub struct ProvidesExecutableWhatsit {
    pub name: &'static str,
    pub _get: fn(tk:&Token,int: &mut Interpreter) -> Result<ExecutableWhatsit,TeXError>
}

pub struct ProvidesBox {
    pub name: &'static str,
    pub _get: fn(tk:&Token,int: &mut Interpreter) -> Result<TeXBox,TeXError>
}

impl PartialEq for ProvidesBox {
    fn eq(&self, other: &Self) -> bool {
        self.name == other.name
    }
}

pub struct MathWhatsit {
    pub name: &'static str,
    pub _get: fn(tk:&Token,int: &mut Interpreter) -> Result<Option<MathKernel>,TeXError>
}

impl PartialEq for MathWhatsit {
    fn eq(&self, other: &Self) -> bool {
        self.name == other.name
    }
}

pub struct SimpleWhatsit {
    pub name: &'static str,
    pub modes: fn(TeXMode) -> bool,
    pub _get: fn(tk:&Token,int: &mut Interpreter) -> Result<Whatsit,TeXError>
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
    FontRef(ArcFont),
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

use crate::stomach::groups::FontChange;

pub trait ExternalCommand : Send + Sync {
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
    Param(u8),
    Token(Token)
}
impl PartialEq for ParamToken {
    fn eq(&self, other: &Self) -> bool {
        match (self,other) {
            (ParamToken::Param(a1),ParamToken::Param(b1)) => a1 == b1,
            (ParamToken::Token(a),ParamToken::Token(b)) => a == b,
            _ => false
        }
    }
}
impl Display for ParamToken {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        use ansi_term::Colour::*;
        match self {
            ParamToken::Param(0) => write!(f,"{}",Yellow.paint("##")),
            ParamToken::Param(i) => write!(f,"{}{}",Yellow.paint("#"),Yellow.paint(i.to_string())),
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
impl Signature {
    fn clean(&self) -> Signature {
        let mut nelems : Vec<ParamToken> = vec!();
        for x in &self.elems {
            match x {
                p@ParamToken::Param(_) => nelems.push(p.clone()),
                ParamToken::Token(t) => nelems.push(ParamToken::Token(t.clean()))
            }
        }
        Signature {
            elems:nelems,
            endswithbrace:self.endswithbrace,
            arity:self.arity.clone()
        }
    }
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
        }
    }
    pub fn get(&self,tk:&Token,int:&mut Interpreter) -> Result<Whatsit,TeXError> {
        use ProvidesWhatsit::*;
        match self {
            Box(b) => Ok(Whatsit::Box((b._get)(tk,int)?)),
            Exec(e) => Ok(Whatsit::Exec(Arc::new((e._get)(tk,int)?))),
            Math(_) => {
                TeXErr!("Should be unreachable!")
            },//Ok(Whatsit::Math((m._get)(tk,int)?)),
            Simple(s) => Ok((s._get)(tk,int)?)
        }
    }
}

pub enum PrimitiveTeXCommand {
    Primitive(&'static PrimitiveExecutable),
    AV(AssignableValue),
    Ext(Arc<dyn ExternalCommand>),
    Cond(&'static Conditional),
    Num(&'static NumericCommand),
    Char(Token),
    Ass(&'static PrimitiveAssignment),
    Def(DefMacro),
    Whatsit(ProvidesWhatsit),
    MathChar(u32)
}

impl PrimitiveTeXCommand {
    /*pub fn as_ref(self,rf:ExpansionRef) -> TeXCommand {
        TeXCommand {
            orig: Arc::new(self),
            //rf:Some(rf)
        }
    }*/
    pub fn as_command(self) -> TeXCommand {
        TeXCommand {
            orig: Arc::new(self),
            //rf: None
        }
    }
    pub fn meaning(&self,catcodes:&CategoryCodeScheme) -> TeXString {
        use PrimitiveTeXCommand::*;
        let ret = match self {
            Char(c) => match c.catcode {
                CategoryCode::Space => {
                    let s : TeXString = "blank space ".into();
                    s + c.char.into()
                },
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
                CategoryCode::MathShift => {
                    let s : TeXString = "math shift character ".into();
                    s + c.char.into()
                }
                CategoryCode::AlignmentTab => {
                    let s : TeXString = "alignment tab character ".into();
                    s + c.char.into()
                }
                CategoryCode::Subscript => {
                    let s : TeXString = "subscript character ".into();
                    s + c.char.into()
                }
                CategoryCode::Superscript => {
                    let s : TeXString = "superscript character ".into();
                    s + c.char.into()
                }
                CategoryCode::Parameter => {
                    let s : TeXString = "macro parameter character ".into();
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
                        ParamToken::Param(0) => {
                            meaning += "##"
                        },
                        ParamToken::Param(i) => {
                            meaning += "#";
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
                let ret : TeXString = if catcodes.escapechar != 255 {catcodes.escapechar.into()} else {"".into()};
                ret + p.name.into()
            },
            AV(AssignableValue::Dim(i)) => {
                let ret : TeXString = if catcodes.escapechar != 255 {catcodes.escapechar.into()} else {"".into()};
                ret + "dimen".into() + i.to_string().into()
            },
            AV(AssignableValue::Skip(i)) => {
                let ret : TeXString = if catcodes.escapechar != 255 {catcodes.escapechar.into()} else {"".into()};
                ret + "skip".into() + i.to_string().into()
            },
            AV(AssignableValue::Register(i)) => {
                let ret : TeXString = if catcodes.escapechar != 255 {catcodes.escapechar.into()} else {"".into()};
                ret + "count".into() + i.to_string().into()
            },
            AV(AssignableValue::Toks(i)) => {
                let ret : TeXString = if catcodes.escapechar != 255 {catcodes.escapechar.into()} else {"".into()};
                ret + "toks".into() + i.to_string().into()
            },
            AV(AssignableValue::PrimReg(p)) => {
                let ret : TeXString = if catcodes.escapechar != 255 {catcodes.escapechar.into()} else {"".into()};
                ret + p.name.into()
            },
            AV(AssignableValue::PrimToks(p)) => {
                let ret : TeXString = if catcodes.escapechar != 255 {catcodes.escapechar.into()} else {"".into()};
                ret + p.name.into()
            },
            AV(AssignableValue::PrimDim(p)) => {
                let ret : TeXString = if catcodes.escapechar != 255 {catcodes.escapechar.into()} else {"".into()};
                ret + p.name.into()
            },
            AV(AssignableValue::PrimSkip(p)) => {
                let ret : TeXString = if catcodes.escapechar != 255 {catcodes.escapechar.into()} else {"".into()};
                ret + p.name.into()
            },
            Cond(c) => {
                let ret : TeXString = if catcodes.escapechar != 255 {catcodes.escapechar.into()} else {"".into()};
                ret + c.name.into()
            },
            Whatsit(ProvidesWhatsit::Math(m)) => {
                let ret : TeXString = if catcodes.escapechar != 255 {catcodes.escapechar.into()} else {"".into()};
                ret + m.name.into()
            },
            Ass(p) => {
                let ret : TeXString = if catcodes.escapechar != 255 {catcodes.escapechar.into()} else {"".into()};
                ret + p.name.into()
            },
            MathChar(i) => {
                let mut ret : TeXString = if catcodes.escapechar != 255 {catcodes.escapechar.into()} else {"".into()};
                ret += "mathchar\"".to_owned() + &format!("{:X}", i);
                ret
            }
            Whatsit(ProvidesWhatsit::Simple(s)) => {
                let ret : TeXString = if catcodes.escapechar != 255 {catcodes.escapechar.into()} else {"".into()};
                ret + s.name.into()
            },
            Whatsit(ProvidesWhatsit::Box(b)) => {
                let ret : TeXString = if catcodes.escapechar != 255 {catcodes.escapechar.into()} else {"".into()};
                ret + b.name.into()
            }
            Whatsit(Exec(e)) => {
                let ret : TeXString = if catcodes.escapechar != 255 {catcodes.escapechar.into()} else {"".into()};
                ret + e.name.into()
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
    pub fn get_num(&self,int:&mut Interpreter) -> Result<Numeric,TeXError> {
        use PrimitiveTeXCommand::*;
        use AssignableValue::*;
        match self {
            AV(Dim(i)) => Ok(Numeric::Dim(int.state.dimensions.get(i))),
            AV(Register(i)) => Ok(Numeric::Int(int.state.registers.get(i))),
            AV(Skip(i)) => Ok(Numeric::Skip(int.state.skips.get(i))),
            AV(MuSkip(i)) => Ok(Numeric::MuSkip(int.state.muskips.get(i))),
            AV(AssignableValue::Int(i)) => Ok((i._getvalue)(int)?),
            PrimitiveTeXCommand::Num(i) => Ok((i._getvalue)(int)?),
            AV(PrimReg(r)) => Ok(Numeric::Int(int.state.registers_prim.get(&(r.index - 1)))),
            AV(PrimDim(r)) => Ok(Numeric::Dim(int.state.dimensions_prim.get(&(r.index - 1)))),
            AV(PrimSkip(r)) => Ok(Numeric::Skip(int.state.skips_prim.get(&(r.index - 1)))),
            AV(PrimMuSkip(r)) => Ok(Numeric::MuSkip(int.state.muskips_prim.get(&(r.index - 1)))),
            Ext(r) => r.get_num(int),
            Char(u) => Ok(Numeric::Int(u.char as i32)),
            MathChar(u) => Ok(Numeric::Int(*u as i32)),
            _ => unreachable!("{}",self)
        }
    }
    pub fn get_expansion(&self,tk:Token,int:&mut Interpreter,cmd:Arc<TeXCommand>) -> Result<Option<Expansion>,TeXError> {
        use PrimitiveTeXCommand::*;
        log!("Expanding {}",tk);
        match self {
            Cond(c) => {c.expand(int)?; Ok(None)},
            Primitive(p) => {
                let mut exp = Expansion::new(tk,cmd.orig.clone());
                (p._apply)(&mut exp,int)?;
                Ok(Some(exp))
            },
            Ext(p) => {
                let mut exp = Expansion::new(tk,cmd.orig.clone());
                p.expand(&mut exp,int)?;
                Ok(Some(exp))
            },
            Def(d) => Ok(Some(self.do_def(tk, int, d,cmd)?)),
            _ => TeXErr!("Should be unreachable!")
        }
    }
    pub fn expand(&self,tk:Token,int:&mut Interpreter,cmd:Arc<TeXCommand>) -> Result<(),TeXError> {
        match self.get_expansion(tk,int,cmd)? {
            Some(exp) => {
                int.push_expansion(exp);
                Ok(())
            }
            None => Ok(())
        }
    }
    fn do_def(&self, tk:Token, int:&mut Interpreter, d:&DefMacro,cmd:Arc<TeXCommand>) -> Result<Expansion,TeXError> {
        /*if tk.cmdname().to_string() == "@kernel@make@file@csname"
            {
            println!("Here! {} {}",int.current_line(),int.preview());
            for p in crate::utils::tex_stacktrace(int,Some(tk.clone())) {
                println!("{} - {}",p.0,p.1);
            }
            print!("");
            //TeXErr!(tk => "temp");
            //unsafe { crate::LOG = true }
        }*/
        log!("{}",d);
        //if unsafe{crate::LOG} {
            log!("    >>{}",int.preview());
        //}
        let in_halign = int.in_halign();
        let mut args : Vec<Vec<Token>> = Vec::with_capacity(d.sig.arity as usize);
        let mut iter = d.sig.elems.iter().peekable();
        loop {
            match iter.next() {
                None => break,
                Some(ParamToken::Token(tk)) => {
                    let next = int.next_token();
                    if *tk != next {
                        TeXErr!(next.clone() => "Expected >{}<; found >{}< (in {})\n{}  >>{}",tk,next,d,int.current_line(),int.preview())
                    }
                }
                Some(ParamToken::Param(_)) => {
                    match iter.peek() {
                        None if d.sig.endswithbrace => {
                            let mut retarg : Vec<Token> = Vec::with_capacity(50);
                            'A: loop {
                                while int.has_next() {
                                    //int.assert_has_next()?;
                                    let next = if in_halign {int.next_token_halign()} else {int.next_token()};
                                    match next.catcode {
                                        CategoryCode::BeginGroup => {
                                            int.requeue(next);
                                            break 'A
                                        }
                                        _ => retarg.push(next)
                                    }
                                }
                                FileEnd!()
                            }
                            args.push(retarg)
                        },
                        None | Some(ParamToken::Param(_)) => {
                            int.skip_ws();
                            let next = int.read_argument()?;
                            args.push(next);
                        },
                        Some(ParamToken::Token(itk)) => {
                            iter.next();
                            let mut delim : Vec<Token> = vec!(itk.clone());
                            loop {
                                match iter.peek() {
                                    Some(ParamToken::Token(t)) => {
                                        iter.next();
                                        delim.push(t.clone());
                                    }
                                    _ => break
                                }
                            }
                            let mut retarg : Vec<Token> = Vec::with_capacity(50);
                            let mut groups = 0;
                            let mut totalgroups = 0;
                            while int.has_next() {
                                let next = if in_halign {int.next_token_halign()} else {int.next_token()};
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
                                    TeXErr!(next => "Missing begin group token somewhere: {}\nsofar: {}\nin: {}:{}",TokenList(&delim),TokenList(&retarg),tk.name(),d)
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
        let mut exp = Expansion::with_capacity(tk,cmd.orig.clone(),50);
        let mut rf = exp.get_ref();
        let mut iter = d.ret.iter();
        loop {
            match iter.next() {
                None => break,
                Some(tk) => match tk.catcode {
                    CategoryCode::Parameter => match iter.next().unwrap() {
                        tk if tk.catcode == CategoryCode::Parameter =>
                            exp.2.push(tk.copied(&mut rf)),
                        next => {
                            let arg = next.char - 49;
                            if arg >= d.sig.arity {
                                TeXErr!(next.clone() => "Expected argument number; got:{}",next)
                            }
                            for tk in args.get(arg as usize).unwrap() { exp.2.push(tk.cloned()) }
                        }
                    }
                    _ => exp.2.push(tk.copied(&mut rf))
                }
            }
        }
        Ok(exp)
    }
    pub fn assign(&self,tk:Token,int:&mut Interpreter,globally:bool,cmd:Arc<TeXCommand>) -> Result<(),TeXError> {
        use crate::commands::registers::GLOBALDEFS;
        use PrimitiveTeXCommand::*;

        let globals = int.state.registers_prim.get(&(GLOBALDEFS.index - 1));
        let global = !(globals < 0) && ( globally || globals > 0 );
        let rf = ExpansionRef(tk,cmd.orig.clone(),None);
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
                    int.state.registers.set(*i, num, global);
                    Ok(())
                }
                AssignableValue::Font(f) => {
                    (f._assign)(rf,int,global)?;
                    Ok(())
                }
                AssignableValue::Tok(t) => (t._assign)(rf,int,global),
                AssignableValue::FontRef(f) => {
                    int.state.currfont.set(f.clone(),global);
                    int.stomach_add(FontChange {
                        font: f.clone(),
                        closes_with_group: !global,
                        children: vec!(),
                        sourceref: None
                    }.as_whatsit())
                },
                AssignableValue::Dim(i) => {
                    int.read_eq();
                    log!("Assigning dimen {}",i);
                    let num = int.read_dimension()?;
                    log!("Assign dimen register {} to {}",i,dimtostr(num));
                    int.state.dimensions.set(*i, num, global);
                    Ok(())
                }
                AssignableValue::Skip(i) => {
                    int.read_eq();
                    let num = int.read_skip()?;
                    log!("Assign skip register {} to {}",i,num);
                    int.state.skips.set(*i, num, global);
                    Ok(())
                }
                AssignableValue::MuSkip(i) => {
                    int.read_eq();
                    let num = int.read_muskip()?;
                    log!("Assign muskip register {} to {}",i,num);
                    int.state.muskips.set(*i, num, global);
                    Ok(())
                },
                AssignableValue::PrimSkip(r) => {
                    int.read_eq();
                    let num = int.read_skip()?;
                    log!("Assign {} to {}",r.name,num);
                    int.state.skips_prim.set(r.index - 1, num, global);
                    Ok(())
                },
                AssignableValue::PrimMuSkip(r) => {
                    int.read_eq();
                    let num = int.read_muskip()?;
                    log!("Assign {} to {}",r.name,num);
                    int.state.muskips_prim.set(r.index - 1, num, global);
                    Ok(())
                },
                AssignableValue::Toks(i) => {
                    int.read_eq();
                    int.expand_until(false)?;
                    let next = int.next_token();
                    let toks = match next.catcode {
                        CategoryCode::BeginGroup => {
                            int.requeue(next);
                            int.read_balanced_argument(false,false,false,true)?
                        }
                        CategoryCode::Escape | CategoryCode::Active => {
                            let cmd = int.get_command(&next.cmdname())?;
                            match &*cmd.orig {
                                PrimitiveTeXCommand::AV(AssignableValue::Toks(j)) => int.state.toks.get(j),
                                PrimitiveTeXCommand::AV(AssignableValue::PrimToks(j)) => int.state.toks_prim.get(&(j.index - 1)),
                                _ => TeXErr!("Expected balanced argument or token register in token assignment")
                            }
                        }
                        _ => TeXErr!("Expected balanced argument or token register in token assignment")
                    };
                    int.state.toks.set(*i, toks.iter().map(|x| x.cloned()).collect(), global);
                    Ok(())
                },
                AssignableValue::PrimToks(r) => {
                    int.read_eq();
                    int.expand_until(false)?;
                    let next = int.next_token();
                    let toks = match next.catcode {
                        CategoryCode::BeginGroup => {
                            int.requeue(next);
                            int.read_balanced_argument(false,false,false,true)?
                        }
                        CategoryCode::Escape | CategoryCode::Active => {
                            let cmd = int.get_command(&next.cmdname())?;
                            match &*cmd.orig {
                                PrimitiveTeXCommand::AV(AssignableValue::Toks(j)) => int.state.toks.get(j),
                                PrimitiveTeXCommand::AV(AssignableValue::PrimToks(j)) => int.state.toks_prim.get(&(j.index - 1)),
                                _ => TeXErr!("Expected balanced argument or token register in token assignment")
                            }
                        }
                        _ => TeXErr!("Expected balanced argument or token register in token assignment")
                    };
                    int.state.toks_prim.set(r.index - 1, toks.iter().map(|x| x.cloned()).collect(), global);
                    Ok(())
                },
                AssignableValue::PrimReg(r) => {
                    int.read_eq();
                    let num = int.read_number()?;
                    log!("Assign {} to {}",r.name,num);
                    int.state.registers_prim.set(r.index - 1, num, global);
                    Ok(())
                },
                AssignableValue::PrimDim(r) => {
                    int.read_eq();
                    let num = int.read_dimension()?;
                    log!("Assign {} to {}",r.name,dimtostr(num));
                    int.state.dimensions_prim.set(r.index - 1, num, global);
                    Ok(())
                }
            },
            Ext(ext) => ext.assign(int, global),
            _ => TeXErr!("Should be unreachable!")
        }?;
        int.insert_afterassignment();
        Ok(())
    }
}

#[derive(Clone)]
pub struct TeXCommand {
    pub orig:Arc<PrimitiveTeXCommand>,
    //pub rf:Option<ExpansionRef>
}

impl TeXCommand {
    pub fn clean(&self) -> TeXCommand {
        TeXCommand {
            //rf:None,
            orig:match *self.orig {
                PrimitiveTeXCommand::Def(ref d) => Arc::new(PrimitiveTeXCommand::Def(d.clean())),
                _ => self.orig.clone()
            }
        }
    }
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
    /*pub fn as_ref(self,tk:Token) -> TeXCommand {
        if COPY_COMMANDS_FULL {
            TeXCommand {
                orig:Arc::clone(&self.orig),
                //rf:Some(ExpansionRef(tk,self.orig.clone(),None))
            }
        } else { self }
    }*/
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
    pub fn get_num(&self,int:&mut Interpreter) -> Result<Numeric,TeXError> { self.orig.get_num(int) }
    pub fn get_expansion(self,tk:Token,int:&mut Interpreter) -> Result<Option<Expansion>,TeXError> {
        let o = self.orig.clone();
        o.get_expansion(tk,int,Arc::new(self))
    }
    pub fn expand(self,tk:Token,int:&mut Interpreter) -> Result<(),TeXError> {
        self.orig.clone().expand(tk,int,Arc::new(self))
    }
    pub fn assign(self,tk:Token,int:&mut Interpreter,globally:bool) -> Result<(),TeXError> {
        self.orig.clone().assign(tk,int,globally,Arc::new(self))
    }
}