pub trait LaTeXObject {}

use crate::references::{SourceReference, FileReference};

// ------------------------------------------------

pub struct Comment {
    pub text: String,
    pub reference : FileReference
}
impl LaTeXObject for Comment {}

// ------------------------------------------------

pub trait Token : LaTeXObject {
    fn origstring(&self) -> &str;
}


pub trait PrimitiveToken : Token {
    //fn origstring(&self) -> &'a str;
    fn reference<'a>(&'a self) -> &'a SourceReference<'a>;
}

pub trait TokenReference : Token {}

// -----------------------------------------------

use crate::catcodes::CategoryCode;

pub trait CharacterToken {
    fn get_char(&self) -> u8;
    fn catcode(&self) -> &CategoryCode;
}

// #[derive(Debug, Copy, Clone)]

pub struct PrimitiveCharacterToken<'a> {
    _char: u8,
    _reference: SourceReference<'a>,
    _catcode: CategoryCode
}
impl CharacterToken for PrimitiveCharacterToken<'_> {
    fn get_char(&self) -> u8 {
        self._char
    }
    fn catcode(&self) -> &CategoryCode {
        &self._catcode
    }
}

impl LaTeXObject for PrimitiveCharacterToken<'_> {}

impl Token for PrimitiveCharacterToken<'_> {
    fn origstring(&self) -> &str {
        ""
    }
}

impl PrimitiveToken for PrimitiveCharacterToken<'_> {
    fn reference<'a>(&'a self) -> &'a SourceReference<'a> {
        &self._reference
    }
}

impl PrimitiveCharacterToken<'_> {
    pub fn new(c : u8, catcode : CategoryCode, r: SourceReference) -> PrimitiveCharacterToken {
        PrimitiveCharacterToken {
            _char:c,
            _catcode:catcode,
            _reference:r
        }
    }
}


// ------------------------------------------------

pub struct ControlSequence {}
impl ControlSequence {

}

pub struct Expansion {
    pub cs : ControlSequence,
    pub exp : Vec<Box<dyn Token>>
}

// ------------------------------------------------
use crate::utils::FilePath;
use std::rc::Rc;

pub struct LaTeXFile {
    pub path: FilePath,
    ch : Vec<Rc<dyn LaTeXObject>>
}
impl LaTeXFile {
    fn new(fp : FilePath) -> LaTeXFile {
        LaTeXFile {
            path:fp,
            ch : Vec::new()
        }
    }
    pub(crate) fn add(&mut self,tk : Rc<dyn LaTeXObject>) {
        self.ch.push(tk)
    }
}