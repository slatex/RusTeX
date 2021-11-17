use std::fmt::{Display, Formatter};
use crate::references::{SourceReference, FileReference, ExpansionReference};
use std::rc::Rc;
use std::str::from_utf8;
use ansi_term::ANSIGenericString;
use crate::catcodes::CategoryCode;
use crate::COPY_TOKENS_FULL;
use crate::utils::TeXString;

#[derive(Clone)]
pub struct Expansion {
    pub cs : Token,
    pub exp : Vec<Token>
}

impl Expansion {
    pub fn dummy(tks : Vec<Token>) -> Expansion {
        Expansion {
            cs: Token::dummy(),
            exp: tks
        }
    }
}

#[derive(Clone)]
pub struct Token {
    pub char : u8,
    pub catcode : CategoryCode,
    pub name_opt: Option<TeXString>,
    pub reference: Box<SourceReference>,
    pub(in crate) expand:bool
}
impl PartialEq for Token {
    fn eq(&self, other: &Self) -> bool {
        self.char == other.char && self.catcode == other.catcode && match (self.name_opt.as_ref(),other.name_opt.as_ref()) {
            (None,None) => true,
            (Some(a),Some(b)) => a == b,
            _ => false
        }
    }
}
impl Display for Token {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        use ansi_term::Colour::*;
        let char : TeXString = self.char.into();
        let colour = match self.catcode {
            CategoryCode::Escape => Red.paint((char + self.cmdname()).to_string()),
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
    pub fn name(&self) -> TeXString {
        match &self.name_opt {
            Some(name) => name.clone(),
            None => vec!(self.char).into()
        }
    }
    pub fn cmdname(&self) -> TeXString {
        match self.catcode {
            CategoryCode::Active => vec!(0,1,2,3,4,255,254,253,252,251,self.char).into(),
            CategoryCode::Escape => self.name(),
            _ => panic!("This should not happen!")
        }
    }

    pub fn dummy() -> Token {
        Token {
            char: 0,
            catcode: CategoryCode::Escape,
            name_opt: Some("relax".into()),
            reference: Box::new(SourceReference::None),
            expand:false
        }
    }
    pub fn copied(&self,exp:Rc<Expansion>) -> Token {
        if COPY_TOKENS_FULL {
            let nref = SourceReference::Exp(ExpansionReference {
                exp,
                tk: self.clone()
            });
            Token {
                char: self.char,
                catcode: self.catcode,
                name_opt: self.name_opt.clone(),
                reference: Box::new(nref),
                expand:true
            }
        } else { todo!() }
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
    pub reference : FileReference
}

#[derive(Clone)]
pub enum LaTeXObject {
    Comment(Comment),
    Token(Token),
    File(LaTeXFile)
}