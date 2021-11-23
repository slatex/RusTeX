use std::fmt::{Display, Formatter};
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
    cmdname : TeXStr,
    pub reference: Box<SourceReference>,
    pub(in crate) expand:bool
}
impl PartialEq for Token {
    fn eq(&self, other: &Self) -> bool {
        self.char == other.char && self.catcode == other.catcode && &self.name_opt == &other.name_opt /* match (&self.name_opt,&other.name_opt) {
            (None,None) => true,
            (Some(a),Some(b)) => a == b,
            _ => false
        } */
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
    pub fn new(char:u8,catcode:CategoryCode,name_opt: Option<TeXStr>,rf:SourceReference,expand:bool) -> Token {
        let name = match name_opt {
            Some(uv) => uv,
            None => TeXStr::new(&[char])
        };
        Token {
            char,
            catcode,
            name_opt: name.clone(),
            cmdname: match catcode {
                CategoryCode::Active => TeXStr::new(&[0,1,2,3,4,255,254,253,252,251,char]),
                CategoryCode::Escape => name,
                _ => TeXStr::new(&[])
            },
            reference: Box::new(rf),
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
    pub fn copied(&self,er:ExpansionRef) -> Token {
        if COPY_TOKENS_FULL {
            Token::new(self.char,self.catcode,Some(self.name_opt.clone()),SourceReference::Exp(er),true)
        } else { self.clone() }
    }
}

#[derive(Clone)]
pub struct LaTeXFile {
    pub path: String,
    ch : Vec<LaTeXObject>
}
impl LaTeXFile {
    pub(crate) fn new(fp : String) -> LaTeXFile {
        LaTeXFile {
            path:fp,
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