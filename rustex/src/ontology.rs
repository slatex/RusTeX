use std::fmt::{Display, Formatter};
use std::path::PathBuf;
use crate::references::SourceReference;
use std::rc::Rc;
use ansi_term::ANSIGenericString;
use crate::catcodes::CategoryCode;
use crate::commands::TeXCommand;
use crate::COPY_TOKENS_FULL;
use crate::utils::{TeXString,TeXStr};

#[derive(Clone)]
pub struct Expansion(pub Token,pub Rc<TeXCommand>,pub Vec<Token>);

impl Expansion {
    pub fn get_ref(&self) -> ExpansionRef { ExpansionRef(self.0.clone(),Rc::clone(&self.1)) }
}

#[derive(Clone)]
pub struct ExpansionRef(pub(crate) Token,pub(crate) Rc<TeXCommand>);

#[derive(Clone)]
pub struct Token {
    pub char : u8,
    pub catcode : CategoryCode,
    name_opt: TeXStr,
    pub(in crate) cmdname : TeXStr,
    pub reference: Rc<SourceReference>,
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
            cmdname:self.cmdname,
            reference: self.reference,
            expand:false
        }

    }
    pub fn new(char:u8,catcode:CategoryCode,name_opt: Option<TeXStr>,rf:SourceReference,expand:bool) -> Token {
        let name = match name_opt {
            Some(uv) => uv,
            None => TeXStr::new(&[char])
        };
        Token {
            char,
            catcode,
            cmdname: match catcode {
                CategoryCode::Active => TeXStr::new(&[0,1,2,3,4,255,254,253,252,251,char]),
                CategoryCode::Escape => name.clone(),
                _ => TeXStr::new(&[])
            },
            name_opt: name,
            reference: Rc::new(rf),
            expand
        }
    }
    pub fn name(&self) -> &TeXStr {
        &self.name_opt
    }
    pub fn cmdname(&self) -> &TeXStr {
        &self.cmdname
    }

    pub fn dummy() -> Token {
        Token::new(0,CategoryCode::Escape,Some("relax".into()),SourceReference::None,false)
    }
    pub fn cloned(&self) -> Token {
        Token {
            char:self.char,
            catcode:self.catcode,
            cmdname:self.cmdname.clone(),
            name_opt:self.name_opt.clone(),
            reference:self.reference.clone(),
            expand:true
        }
    }
    pub fn copied(&self,er:ExpansionRef) -> Token {
        if COPY_TOKENS_FULL {
            Token {
                char:self.char,
                catcode:self.catcode,
                cmdname:self.cmdname.clone(),
                name_opt:self.name_opt.clone(),
                reference:Rc::new(SourceReference::Exp(er)),
                expand:true
            }
            //Token::new(self.char,self.catcode,Some(self.name_opt.clone()),SourceReference::Exp(er),true)
        } else {
            Token {
                char:self.char,
                catcode:self.catcode,
                cmdname:self.cmdname.clone(),
                name_opt:self.name_opt.clone(),
                reference:self.reference.clone(),
                expand:true
            }
        }
    }
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