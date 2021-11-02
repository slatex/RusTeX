use crate::references::{SourceReference, FileReference};
use std::rc::Rc;

pub trait LaTeXObjectI {
    fn as_object(self) -> Rc<LaTeXObject>;
}

pub trait TokenI:LaTeXObjectI {
    fn as_token(self) -> Rc<Token>;
    fn as_string(&self) -> String;
    fn copied(&self,exp:&Expansion) -> Rc<Token>;
}
/*
impl<T> LaTeXObjectI for T where T : TokenI {
    fn as_object(self) -> Rc<LaTeXObject> {
        Rc::new(LaTeXObject::Token(self.as_token()))
    }
}
 */

pub(crate) trait PrimitiveToken<T> : TokenI {
    //fn origstring(&self) -> &'a str;
    fn reference(&self) -> &SourceReference;
}

pub(crate) trait TokenReference<T> : TokenI {
    fn previous(&self) -> Rc<T>;
    fn orig(&self) -> Rc<dyn PrimitiveToken<T>>;
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

use std::str::from_utf8;

impl TokenI for PrimitiveCharacterToken {
    fn as_token(self) -> Rc<Token> {
        Rc::new(Token::Char(self.as_character_token()))
    }
    fn as_string(&self) -> String {
        let mut ret = "<".to_string();
        if self._char == 13 {
            ret.push_str("\\r")
        } else if self._char == 173 {
            ret.push_str("\\u00AD")
        } else {
            ret.push_str(from_utf8(&[self._char]).expect(&*("Nope: ".to_owned() + self._char.to_string().as_str())))
        }
        ret.push(',');
        ret.push_str(self._catcode.toint().to_string().as_str());
        ret.push('>');
        ret
    }
    fn copied(&self, exp: &Expansion) -> Rc<Token> {
        todo!()
    }
}

impl CharacterTokenI for PrimitiveCharacterToken {
    fn as_character_token(self) -> Rc<CharacterToken> {
        Rc::new(CharacterToken::Prim(Rc::new(self)))
    }
}

impl PrimitiveToken<CharacterToken> for PrimitiveCharacterToken {

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
}


// ------------------------------------------------

pub trait CommandI : TokenI {
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
    fn as_string(&self) -> String {
        let mut ret = "<<".to_string();
        ret.push_str(self._name.as_str());
        ret.push_str(">>");
        ret
    }
    fn copied(&self, exp: &Expansion) -> Rc<Token> {
        todo!()
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
impl PrimitiveToken<ControlSequence> for PrimitiveControlSequence {
    fn reference(&self) -> &SourceReference {
        &self._reference
    }
}

// ---------------------------------------------------------------------------

pub(crate) trait ActiveCharacterTokenI : CommandI {
    fn as_active(self) -> Rc<ActiveCharacterToken>;
}

pub struct PrimitiveActiveCharacterToken {
    _char:u8,
    _reference:SourceReference
}

impl PrimitiveActiveCharacterToken {
    pub fn new(c : u8, r: SourceReference) -> PrimitiveActiveCharacterToken {
        PrimitiveActiveCharacterToken {
            _char:c,
            _reference:r
        }
    }
}

impl CommandI for PrimitiveActiveCharacterToken {
    fn as_command(self) -> Rc<Command> {
        Rc::new(Command::Active(self.as_active()))
    }
}

impl TokenI for PrimitiveActiveCharacterToken {
    fn as_token(self) -> Rc<Token> {
        Rc::new(Token::Command(self.as_command()))
    }
    fn as_string(&self) -> String {
        let mut ret = "<!<".to_string();
        ret.push_str(from_utf8(&[self._char]).unwrap());
        ret.push('>');
        ret
    }
    fn copied(&self, exp: &Expansion) -> Rc<Token> {
        todo!()
    }
}

impl LaTeXObjectI for PrimitiveActiveCharacterToken {
    fn as_object(self) -> Rc<LaTeXObject> {
        Rc::new(LaTeXObject::Token(self.as_token()))
    }
}

impl ActiveCharacterTokenI for PrimitiveActiveCharacterToken {
    fn as_active(self) -> Rc<ActiveCharacterToken> {
        Rc::new(ActiveCharacterToken::Prim(Rc::new(self)))
    }
}

pub struct Expansion {
    pub cs : Rc<Command>,
    pub exp : Vec<Rc<Token>>
}

impl Expansion {
    pub fn dummy(tks : Vec<Rc<Token>>) -> Expansion {
        Expansion {
            cs: Rc::new((Command::dummy())),
            exp: tks
        }
    }
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
impl ControlSequence {
    pub fn as_string(&self) -> String {
        match self {
            ControlSequence::Prim(p) => p.as_string(),
            ControlSequence::Ref => todo!()
        }
    }
    pub fn dummy() -> ControlSequence {
        ControlSequence::Prim(
            Rc::new(PrimitiveControlSequence::new("DUMMY".to_owned(),SourceReference::None))
        )
    }
}

pub enum ActiveCharacterToken {
    Prim(Rc<PrimitiveActiveCharacterToken>),
    Ref
}

impl ActiveCharacterToken {
    pub fn as_string(&self) -> String {
        match self {
            ActiveCharacterToken::Prim(p) => p.as_string(),
            ActiveCharacterToken::Ref => todo!()
        }
    }
}

pub enum Command {
    Cs(Rc<ControlSequence>),
    Active(Rc<ActiveCharacterToken>)
}


impl Command {
    pub fn as_string(&self) -> String {
        match self {
            Command::Cs(p) => p.as_string(),
            Command::Active(ac) => ac.as_string()
        }
    }
    pub fn dummy() -> Command {
        Command::Cs(Rc::new(ControlSequence::dummy()))
    }
}

pub enum CharacterToken {
    Prim(Rc<PrimitiveCharacterToken>),
    Ref
}

impl CharacterToken {
    pub fn get_char(&self) -> u8 {todo!()}
    pub fn catcode(&self) -> &CategoryCode {todo!()}
    pub fn as_string(&self) -> String {
        match self {
            CharacterToken::Prim(p) => p.as_string(),
            CharacterToken::Ref => todo!()
        }
    }
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
    pub fn as_string(&self) -> String {
        match self {
            Token::Command(cmd) => cmd.as_string(),
            Token::Char(ct) => ct.as_string(),
        }
    }
    pub fn copied(&self,exp:&Expansion) -> Rc<Token> {
        todo!()
    }
}

pub enum LaTeXObject {
    Comment(Rc<Comment>),
    Token(Rc<Token>),
    File(Rc<LaTeXFile>)
}