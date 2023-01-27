use std::fmt::{Display, Formatter};
use crate::references::SourceReference;
use std::sync::Arc;
use ahash::HashMap;
use ansi_term::ANSIGenericString;
use crate::catcodes::CategoryCode;
use crate::commands::PrimitiveTeXCommand;
use crate::COPY_TOKENS_FULL;
use crate::utils::{TeXString,TeXStr};

#[derive(Clone)]
pub struct Expansion(pub Token,pub Arc<PrimitiveTeXCommand>,pub Vec<Token>);

impl Expansion {
    pub fn new(tk : Token,command:Arc<PrimitiveTeXCommand>) -> Expansion {
        Expansion(tk,command,vec!())
    }
    pub fn get_ref(&mut self) -> ExpansionRef {
        ExpansionRef(self.0.clone(),Arc::clone(&self.1),None) }
}
impl Display for Expansion {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        for e in &self.2 {
            match e.catcode {
                CategoryCode::Escape => write!(f,"\\{}",e.name().to_string())?,
                _ => write!(f,"{}",TeXString(vec!(e.char)).to_string())?
            }
        }
        write!(f,"")
    }
}

#[derive(Clone)]
pub struct ExpansionRef(pub(crate) Token,pub(crate) Arc<PrimitiveTeXCommand>,pub(crate) Option<Arc<SourceReference>>);

impl ExpansionRef {
    pub fn as_src_ref(&mut self) -> Arc<SourceReference> {
        if self.2.is_none() {
            self.2 = Some(Arc::new(SourceReference::Exp(self.0.clone(),self.1.clone())))
        }
        self.2.as_ref().unwrap().clone()
    }
}

#[derive(Clone)]
pub struct Token {
    pub char : u8,
    pub catcode : CategoryCode,
    pub(in crate) name_opt: TeXStr,
    //pub(in crate) cmdname : TeXStr,
    pub reference: Option<Arc<SourceReference>>,
    pub(in crate) expand:bool
}
impl PartialEq for Token {
    fn eq(&self, other: &Self) -> bool {
        match self.catcode {
            CategoryCode::Escape => other.catcode == CategoryCode::Escape && &self.name_opt == &other.name_opt,
            _ => self.char == other.char && self.catcode == other.catcode && &self.name_opt == &other.name_opt
        }
    }
}
impl Display for Token {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        use ansi_term::Colour::*;
        let char : TeXString = self.char.into();
        let colour = match self.catcode {
            CategoryCode::Escape => Red.paint((char + self.cmdname().into()).to_string()),
            CategoryCode::BeginGroup => Green.paint(char.to_string()),
            CategoryCode::EndGroup => Green.bold().paint(char.to_string()),
            CategoryCode::Active => Red.bold().paint(char.to_string()),
            CategoryCode::Space => ANSIGenericString::from(" "),
            CategoryCode::Parameter => Yellow.paint(char.to_string()),
            CategoryCode::AlignmentTab => Blue.paint(char.to_string()),
            CategoryCode::MathShift => Purple.paint(char.to_string()),
            CategoryCode::Subscript => Cyan.paint(char.to_string()),
            CategoryCode::Superscript => Cyan.bold().paint(char.to_string()),
            CategoryCode::Letter => White.bold().paint(char.to_string()),
            _ => ANSIGenericString::from(char.to_string())
        };
        write!(f,"{}",colour)
    }
}
impl Token {
    pub fn deexpand(self) -> Token {
        Token {
            char:self.char,
            catcode:self.catcode,
            name_opt: self.name_opt,
            //cmdname:self.cmdname,
            reference: self.reference,
            expand:false
        }

    }
    pub fn new(char:u8,catcode:CategoryCode,name_opt: Option<TeXStr>,rf:Option<Arc<SourceReference>>,expand:bool) -> Token {
        let name = match name_opt {
            Some(uv) => uv,
            None => trivial_name(char)
        };
        Token {
            char,
            catcode,
            name_opt: name,
            reference: rf,
            expand
        }
    }
    pub fn name(&self) -> &TeXStr {
        &self.name_opt
    }
    pub fn cmdname(&self) -> TeXStr {
        match self.catcode {
            CategoryCode::Active => active_name(self.char),
            _ => self.name_opt.clone()
        }
    }

    pub fn dummy() -> Token {
        Token::new(0,CategoryCode::Escape,Some("relax".into()),None,false)
    }
    pub fn cloned(&self) -> Token {
        Token {
            char:self.char,
            catcode:self.catcode,
            //cmdname:self.cmdname.clone(),
            name_opt:self.name_opt.clone(),
            reference:self.reference.clone(),
            expand:true
        }
    }
    pub fn clean(&self) -> Token {
        Token {
            char:self.char,
            catcode:self.catcode,
            //cmdname:self.cmdname.clone(),
            name_opt:self.name_opt.clone(),
            reference:None,
            expand:true
        }
    }
    pub fn copied(&self,er:&mut ExpansionRef) -> Token {
        if COPY_TOKENS_FULL {
            Token {
                char:self.char,
                catcode:self.catcode,
                //cmdname:self.cmdname.clone(),
                name_opt:self.name_opt.clone(),
                reference:Some(er.as_src_ref()),
                expand:true
            }
            //Token::new(self.char,self.catcode,Some(self.name_opt.clone()),SourceReference::Exp(er),true)
        } else {
            Token {
                char:self.char,
                catcode:self.catcode,
                //cmdname:self.cmdname.clone(),
                name_opt:self.name_opt.clone(),
                reference:self.reference.clone(),
                expand:true
            }
        }
    }
}

pub fn active_name(u:u8) -> TeXStr {
    ACTIVE_NAMES.get(u as usize).unwrap().clone()
}
pub fn trivial_name(u:u8) -> TeXStr {
    SIMPLE_NAMES.get(u as usize).unwrap().clone()
}

lazy_static! {
    pub static ref EMPTY_NAME: TeXStr = TeXStr::new(&[]);
    pub static ref ACTIVE_NAMES: [TeXStr;256] = {
        <[TeXStr;256]>::try_from((0..256).map(|i| TeXStr::new(&[0,1,2,3,4,255,254,253,252,251,i as u8])).collect::<Vec<TeXStr>>()).ok().unwrap()
    };
    pub static ref SIMPLE_NAMES: [TeXStr;256] = {
        <[TeXStr;256]>::try_from((0..256).map(|i| TeXStr::new(&[i as u8])).collect::<Vec<TeXStr>>()).ok().unwrap()
    };
}

#[derive(Clone)]
pub struct LaTeXFile {
    pub id: TeXStr,
    pub path: Option<TeXStr>,
    ch : Vec<LaTeXObject>
}
impl LaTeXFile {
    pub(crate) fn new(fp : TeXStr, path:TeXStr) -> LaTeXFile {
        LaTeXFile {
            id:fp,
            path:Some(path),
            ch : Vec::new()
        }
    }
    pub(crate) fn add(&mut self,tk : LaTeXObject) {
        self.ch.push(tk)
    }
}

#[derive(Clone)]
pub struct Comment {
    pub text: String,
    pub reference : SourceReference
}

#[derive(Clone)]
pub enum LaTeXObject {
    Comment(Comment),
    Token(Token),
    File(LaTeXFile)
}