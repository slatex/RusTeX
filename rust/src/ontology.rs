use crate::references::{SourceReference, FileReference};
use std::rc::Rc;

pub(crate) trait LaTeXObjectI {
    fn as_object(self) -> Rc<LaTeXObject>;
}

pub(crate) trait TokenI:LaTeXObjectI {
    fn as_token(self) -> Rc<Token>;
}
/*
impl<T> LaTeXObjectI for T where T : TokenI {
    fn as_object(self) -> Rc<LaTeXObject> {
        Rc::new(LaTeXObject::Token(self.as_token()))
    }
}
 */

pub(crate) trait PrimitiveToken : TokenI {
    //fn origstring(&self) -> &'a str;
    fn reference(&self) -> &SourceReference;
}

pub(crate) trait TokenReference : TokenI {
    fn previous(&self) -> Rc<Token>;
    fn orig(&self) -> Rc<dyn PrimitiveToken>;
}

// -----------------------------------------------

pub(crate) trait CharacterTokenI:TokenI {
    fn as_character_token(self) -> Rc<CharacterToken>;
}
/*
impl<T> TokenI for T where T: CharacterTokenI {
    fn as_token(self) -> Rc<Token> {
        Rc::new(Token::Char(self.as_character_token()))
    }
}

 */

use crate::catcodes::CategoryCode;

// #[derive(Debug, Copy, Clone)]

pub struct PrimitiveCharacterToken {
    _char: u8,
    _reference: SourceReference,
    _catcode: CategoryCode
}

impl LaTeXObjectI for PrimitiveCharacterToken {
    fn as_object(self) -> Rc<LaTeXObject> {
        Rc::new(LaTeXObject::Token(self.as_token()))
    }
}

impl TokenI for PrimitiveCharacterToken {
    fn as_token(self) -> Rc<Token> {
        Rc::new(Token::Char(self.as_character_token()))
    }
}

impl CharacterTokenI for PrimitiveCharacterToken {
    fn as_character_token(self) -> Rc<CharacterToken> {
        Rc::new(CharacterToken::Prim(Rc::new(self)))
    }
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
    pub(crate) fn as_character_token(self) -> Rc<CharacterToken> {
        Rc::new(CharacterToken::Prim(Rc::new(self)))
    }
}


// ------------------------------------------------

pub(crate) trait CommandI : TokenI {
    fn as_command(self) -> Rc<Command>;
}
/*
impl<T> TokenI for T where T : CommandI {
    fn as_token(self) -> Rc<Token> {
        Rc::new(Token::Command(self.as_command()))
    }
}
 */

pub(crate) trait ControlSequenceI : CommandI {
    fn as_cs(self) -> Rc<ControlSequence>;
}

/*
impl<A> CommandI for A where A : ControlSequenceI {
    fn as_command(self) -> Rc<Command> {
        Rc::new(Command::Cs(self.as_cs()))
    }
}
 */

pub struct PrimitiveControlSequence {
    _name:String,
    _reference: SourceReference,
}

impl CommandI for PrimitiveControlSequence {
    fn as_command(self) -> Rc<Command> {
        Rc::new(Command::Cs(self.as_cs()))
    }
}

impl TokenI for PrimitiveControlSequence {
    fn as_token(self) -> Rc<Token> {
        Rc::new(Token::Command(self.as_command()))
    }
}

impl LaTeXObjectI for PrimitiveControlSequence {
    fn as_object(self) -> Rc<LaTeXObject> {
        Rc::new(LaTeXObject::Token(self.as_token()))
    }
}

impl ControlSequenceI for PrimitiveControlSequence {
    fn as_cs(self) -> Rc<ControlSequence> {
        Rc::new(ControlSequence::Prim(Rc::new(self)))
    }
}

impl PrimitiveControlSequence {
    pub fn new(name:String,rf:SourceReference) -> PrimitiveControlSequence {
        PrimitiveControlSequence {
            _name:name,
            _reference:rf
        }
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


// ------------------------------------------------
pub struct Comment {
    pub text: String,
    pub reference : FileReference
}
impl Comment {
    pub(crate) fn as_object(self) -> Rc<LaTeXObject> {
        Rc::new(LaTeXObject::Comment(Rc::new(self)))
    }
}

// --------------------------------------------------------------------------

pub enum ControlSequence {
    Prim(Rc<PrimitiveControlSequence>),
    Ref
}

pub enum Command {
    Cs(Rc<ControlSequence>),
    Active(Rc<CharacterToken>)
}

pub enum CharacterToken {
    Prim(Rc<PrimitiveCharacterToken>),
    Ref
}

impl CharacterToken {
    pub fn get_char(&self) -> u8 {todo!()}
    pub fn catcode(&self) -> &CategoryCode {todo!()}
}

pub enum Token {
    Command(Rc<Command>),
    Char(Rc<CharacterToken>)
}

impl Token {
    pub fn origstring(&self) -> &str {
        ""
    }
    pub(crate) fn as_object(self) -> Rc<LaTeXObject> {
        Rc::new(LaTeXObject::Token(Rc::new(self)))
    }
}

pub enum LaTeXObject {
    Comment(Rc<Comment>),
    Token(Rc<Token>)
}