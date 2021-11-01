use crate::references::{SourceReference, FileReference};
use std::rc::Rc;

// ------------------------------------------------

pub struct Comment {
    pub text: String,
    pub reference : FileReference
}

impl Comment {
    pub fn as_object(self) -> LaTeXObject {
        LaTeXObject::Comment(self)
    }
}

// ------------------------------------------------


pub trait PrimitiveToken {
    //fn origstring(&self) -> &'a str;
    fn reference(&self) -> &SourceReference;
}

pub trait TokenReference {}

// -----------------------------------------------

use crate::catcodes::CategoryCode;


// #[derive(Debug, Copy, Clone)]

pub struct PrimitiveCharacterToken {
    _char: u8,
    _reference: SourceReference,
    _catcode: CategoryCode
}

impl PrimitiveToken for PrimitiveCharacterToken {
    fn reference(&self) -> &SourceReference {
        &self._reference
    }
}

impl PrimitiveCharacterToken {
    pub fn new(c : u8, catcode : CategoryCode, r: SourceReference) -> PrimitiveCharacterToken {
        PrimitiveCharacterToken {
            _char:c,
            _catcode:catcode,
            _reference:r
        }
    }
    pub fn as_token(self) -> Token {
        Token::Char(CharacterToken::Prim(self))
    }
}


// ------------------------------------------------

pub struct PrimitiveControlSequence {
    _name:String,
    _reference: SourceReference,
}

impl PrimitiveControlSequence {
    pub fn new(name:String,rf:SourceReference) -> PrimitiveControlSequence {
        PrimitiveControlSequence {
            _name:name,
            _reference:rf
        }
    }
    pub fn as_token(self) -> Token {
        Token::Command(Command::Cs(ControlSequence::Prim(self)))
    }
}
impl PrimitiveToken for PrimitiveControlSequence {
    fn reference(&self) -> &SourceReference {
        &self._reference
    }
}

pub struct Expansion {
    pub cs : Rc<ControlSequence>,
    pub exp : Vec<Rc<Token>>
}

// ------------------------------------------------
use crate::utils::FilePath;

pub struct LaTeXFile {
    pub path: FilePath,
    ch : Vec<Rc<LaTeXObject>>
}
impl LaTeXFile {
    fn new(fp : FilePath) -> LaTeXFile {
        LaTeXFile {
            path:fp,
            ch : Vec::new()
        }
    }
    pub(crate) fn add(&mut self,tk : Rc<LaTeXObject>) {
        self.ch.push(tk)
    }
}

// --------------------------------------------------------------------------

pub enum ControlSequence {
    Prim(PrimitiveControlSequence),
    Ref
}

pub enum Command {
    Cs(ControlSequence),
    Active(CharacterToken)
}

pub enum CharacterToken {
    Prim(PrimitiveCharacterToken),
    Ref
}

impl CharacterToken {
    pub fn get_char(&self) -> u8 {todo!()}
    pub fn catcode(&self) -> &CategoryCode {todo!()}
}

pub enum Token {
    Command(Command),
    Char(CharacterToken)
}

impl Token {
    pub fn origstring(&self) -> &str {
        ""
    }
    pub fn as_object(self) -> LaTeXObject {
        LaTeXObject::Token(self)
    }
}

pub enum LaTeXObject {
    Comment(Comment),
    Token(Token)
}