use crate::references::{SourceReference, FileReference, ExpansionReference};
use std::rc::Rc;
use std::str::from_utf8;
use crate::catcodes::CategoryCode;
use crate::COPY_TOKENS_FULL;

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
    pub nameOpt : Option<String>,
    pub reference: Box<SourceReference>
}
impl Token {
    pub fn name(&self) -> String {
        match &self.nameOpt {
            Some(name) => self.nameOpt.as_ref().unwrap().to_owned(),
            None => from_utf8(&[self.char]).expect("This should not happen").to_owned()
        }
    }
    pub fn cmdname(&self) -> String {
        match self.catcode {
            CategoryCode::Active => "\\\\RusTeX\\Active\\Character\\".to_string() + &self.name(),
            CategoryCode::Escape => self.name(),
            _ => panic!("This should not happen!")
        }
    }
    pub fn as_string(&self) -> String {
        match self.catcode {
            CategoryCode::Escape => from_utf8(&[self.char]).expect("This should not happen").to_owned() + &self.name(),
            _ => "\'".to_owned() + from_utf8(&[self.char]).expect("This should not happen") + "\'" + CategoryCode::toint(&self.catcode).to_string().as_str()
        }
    }
    pub fn dummy() -> Token {
        Token {
            char: 0,
            catcode: CategoryCode::Escape,
            nameOpt: None,
            reference: Box::new(SourceReference::None)
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
                nameOpt: self.nameOpt.clone(),
                reference: Box::new(nref)
            }
        } else { todo!() }
    }
}

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

pub struct Comment {
    pub text: String,
    pub reference : FileReference
}

pub enum LaTeXObject {
    Comment(Comment),
    Token(Token),
    File(LaTeXFile)
}