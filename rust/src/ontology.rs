pub trait LaTeXObject {}

use crate::references::{SourceReference, FileReference};

// ------------------------------------------------

pub struct Comment<'a> {
    pub text: String,
    pub reference : FileReference<'a>
}
impl LaTeXObject for Comment<'_> {}

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
#[derive(Copy, Clone)]
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

pub struct LaTeXFile {
    pub fp : FilePath,
    ch : Vec<Box<dyn LaTeXObject>>
}
impl LaTeXFile {
    fn new(fp : FilePath) -> LaTeXFile {
        LaTeXFile {
            fp:fp,
            ch : Vec::new()
        }
    }
    pub(crate) fn add(&mut self,tk : Box<dyn LaTeXObject>) {
        self.ch.push(tk)
    }
}