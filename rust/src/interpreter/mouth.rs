enum MouthState { N,S,M }

use std::borrow::{Borrow, BorrowMut};
use std::iter::{Peekable, Map};
use std::ops::Deref;
use std::rc::Rc;
use std::slice::IterMut;
use std::str::{Chars, from_utf8, from_utf8_unchecked, Split};
use crate::ontology::{Comment, Expansion, LaTeXFile, Token, LaTeXObject, PrimitiveControlSequence, TokenI, LaTeXObjectI, PrimitiveActiveCharacterToken};
use crate::catcodes::{CategoryCode, CategoryCodeScheme};
use crate::ontology::{PrimitiveCharacterToken,PrimitiveToken};
use crate::references::{SourceReference,FileReference};
use crate::interpreter::state::State;

use crate::debug;

pub enum Mouth {
    Token(TokenMouth),
    Str(StringMouth),
    File(StringMouth)
}

impl Mouth {
    pub(crate) fn has_next(&mut self,state:&State, nocomment : bool) -> bool {
        match self {
            Mouth::Token(tm) => tm.has_next(nocomment),
            Mouth::Str(sm) => sm.has_next(state,nocomment),
            Mouth::File(sm) => sm.has_next(state,nocomment)
        }
    }
    pub(crate) fn get_next(&mut self,state:&State) -> Rc<Token> {
        match self {
            Mouth::Token(tm) => tm.pop_next(true),
            Mouth::Str(sm) => sm.pop_next(state,true),
            Mouth::File(sm) => sm.pop_next(state,true)
        }
    }
}

pub struct TokenMouth {
    exp: Expansion,
    tokens : Vec<Rc<Token>>
}
impl TokenMouth {
    fn new(exp:Expansion) -> TokenMouth {
        let mut vec : Vec<Rc<Token>> = Vec::new();
        for tk in &exp.exp {
            vec.push(tk.copied(&exp))
        }
        TokenMouth {
            exp:exp,
            tokens:vec
        }
    }
    fn has_next(&mut self, nocomment: bool) -> bool {
        !self.tokens.is_empty()
    }
    fn pop_next(&mut self, nocomment: bool) -> Rc<Token> {
        self.tokens.remove(0)
    }
    fn preview(&mut self) -> String {
        self.tokens.iter().map(|x| {x.origstring()}).collect::<Vec<&str>>().join("")
    }
    fn pushback(&mut self) {}
    fn peek(&mut self) -> Rc<Token> {
        Rc::clone(self.tokens.first().expect(""))
    }
}

use crate::interpreter::Interpreter;
extern crate itertools;
use itertools::Itertools;
use self::itertools::MultiPeek;

enum StringMouthSource {
    File(LaTeXFile),
    Exp(Expansion)
}

impl StringMouthSource {
    pub fn get_file(&mut self) -> Option<&mut LaTeXFile> {
        match self {
            StringMouthSource::File(s) => Some(s),
            _ => None
        }
    }
    pub fn get_file_ref(&self) -> Option<&LaTeXFile> {
        match self {
            StringMouthSource::File(r) => Some(r),
            _ => None
        }
    }
}

pub struct StringMouth {
    mouth_state:MouthState,
    peekbuffer : Option<Rc<Token>>,
    string : Option<String>,
    allstrings : Vec<String>,
    line: usize,
    pos: usize,
    atendofline:Option<u8>,
    charbuffer:Option<(u8,usize,usize)>,
    source : StringMouthSource,
}

impl StringMouth {
    pub fn new_from_file<'a,'b>(state:&'b State<'b>, file:VFile) -> StringMouth {
        let ltxf = LaTeXFile::new(file.id);
        let string = file.string.expect("This shouldn't happen");
        StringMouth::new_i(state,StringMouthSource::File(ltxf),string)
    }
    pub fn new<'a,'b>(state:&'b State<'b>, source:Expansion, string : &'a str) -> StringMouth {
        StringMouth::new_i(state,StringMouthSource::Exp(source),string.to_string())
    }
    fn new_i<'a,'b>(state:&'b State<'b>, source:StringMouthSource, string : String) -> StringMouth {
        use std::str::{from_utf8};
        let newlinechar = state.newlinechar();
        let it = if string.is_empty() {
            vec![]
        } else if newlinechar==u8::MAX {
            vec![string]
        } else {
            let mut ret:Vec<String> = Vec::new();
            for s in string.split(from_utf8(&[newlinechar]).unwrap()) {
                ret.push(s.to_string())
            }
            ret.reverse();
            ret
        };
        StringMouth {
            mouth_state:MouthState::N,
            allstrings:it,
            string:None,
            peekbuffer:None,
            atendofline:None,
            line:0,
            pos:0,
            charbuffer: None,
            source
        }
    }
    fn do_line(&mut self,state:&State) -> bool {
        if self.allstrings.is_empty() { false } else {
            match state.endlinechar() {
                u8::MAX => {},
                o => self.atendofline =  Some(o)
            };
            //self.allstrings.pop().unwrap().trim_end().as_bytes();
            let string = self.allstrings.pop().unwrap();
            self.string = Some(string);
            self.line += 1;
            self.pos = 0;
            self.mouth_state = MouthState::N;
            true
        }
    }

    fn next_char(&mut self,state:&State) -> Option<(u8, usize, usize)> {
        loop {
            if let Some(tk) = self.charbuffer.take() { return Some(tk) } else {
                match self.string {
                    None => match self.do_line(state) {
                        true => {},
                        false => return None
                    }
                    Some(ref str) => {
                        if str.len() <= self.pos {
                            match self.atendofline {
                                Some(cb) => {
                                    self.atendofline = None;
                                    return Some((cb, self.line + 1, self.pos))
                                },
                                None => match self.do_line(state) {
                                    true => {},
                                    false => return None
                                }
                            }
                        } else {
                            let ret = str.as_bytes().get(self.pos).unwrap();
                            self.pos += 1;
                            return Some((*ret, self.line + 1, self.pos))
                        }
                    }
                }
            }
        }
    }

    fn do_s(&mut self,state:&State) {
        use crate::catcodes::CategoryCode;
        while (self.has_next(state,true)) {
            let next = self.next_char(state).unwrap();
            match state.catcodes().get_code(next.0) {
                CategoryCode::Space | CategoryCode::EOL => {}
                _ => {
                    self.charbuffer = Some(next);
                    break
                }
            }
        }
    }
    fn make_reference(&self,line:usize,pos:usize) -> SourceReference {
        match self.source.get_file_ref() {
            None => SourceReference::None,
            Some(r) => SourceReference::File(self.make_file_reference(r,line,pos))
        }
    }
    fn make_file_reference(&self,f : &LaTeXFile,line:usize,pos:usize) -> FileReference {
        FileReference {
            file:f.path.clone(),
            start: (line,pos),
            end: (self.line,self.pos)
        }
    }

    pub fn has_next(&mut self,state:&State, nocomment: bool) -> bool {
        match self.peekbuffer {
            Some(_) => true,
            None => {
                loop {
                    match self.next_char(state) {
                        None => return false, // ret = Some(false),
                        Some(next) => match self.mouth_state {
                            MouthState::S => {
                                self.charbuffer = Some(next);
                                self.mouth_state = MouthState::M;
                                self.do_s(state);
                            }
                            _ => match state.catcodes().get_code(next.0) {
                                CategoryCode::Ignored => {
                                    let file = self.source.get_file();
                                    match file {
                                        Some(ltxf) => {
                                            let nrf = FileReference {
                                                file:ltxf.path.clone(),
                                                start: (next.1,next.2),
                                                end: (self.line,self.pos)
                                            };
                                            let tk = PrimitiveCharacterToken::new(
                                                next.0,CategoryCode::Ignored,SourceReference::File((nrf)));
                                            ltxf.add(tk.as_object())
                                            // TODO
                                        }
                                        _ => {}
                                    }
                                }
                                CategoryCode::Comment => if nocomment {
                                    let mut rest : Vec<u8> = (*self.string.as_ref().unwrap().as_bytes())[self.pos..].to_vec();//..slice(self.pos as usize,self.string.unwrap().len()).to_vec();
                                    rest.insert(0,next.0);
                                    match self.source.get_file() {
                                        Some(ltxf) => {
                                            let txt = std::str::from_utf8(rest.as_slice()).unwrap().to_string();
                                            let end = txt.len();
                                            self.pos += end;
                                            let nrf = FileReference {
                                                file:ltxf.path.clone(),
                                                start: (next.1,next.2),
                                                end: (self.line,self.pos)
                                            };///self.make_file_reference(ltxf,next.1,next.2);
                                            let tk = Comment {
                                                text: txt,
                                                reference: nrf
                                            };
                                            ltxf.add(tk.as_object())
                                        }
                                        _ => {}
                                    }
                                    self.do_line(state);
                                    loop {
                                        match self.next_char(state) {
                                            None => break,
                                            Some(n) => {
                                                let cc = state.catcodes().get_code(n.0);
                                                match cc {
                                                    CategoryCode::Space | CategoryCode::EOL => { }
                                                    _ => {
                                                        self.charbuffer = Some(n);
                                                        break
                                                    }
                                                }
                                            }
                                        }
                                    }
                                }
                                CategoryCode::Space if matches!(self.mouth_state,MouthState::N) => { }
                                CategoryCode::Superscript => {
                                    let string = self.string.as_ref().unwrap();
                                    let len = string.as_bytes()[self.pos..].len();
                                    let peek = string.as_bytes().get(self.pos);
                                    if (len > 1 && peek.is_some() && (*peek.unwrap()) == next.0) {
                                        let (startl,startpos) = (next.1,next.2);
                                        self.pos += 1;
                                        let next = *string.as_bytes().get(self.pos).unwrap();
                                        self.pos += 1;
                                        let maybenext = string.as_bytes().get(self.pos as usize);
                                        fn cond(i:u8) -> bool { (48 <= i && i <= 57) || (97 <= i && i <= 102) }
                                        if (cond(next)) && maybenext.is_some() && cond(*maybenext.unwrap()) {
                                            self.pos += 1;
                                            self.charbuffer = Some((u8::from_str_radix(from_utf8(&[next,*maybenext.unwrap()]).unwrap(),16).unwrap(),startl,startpos))
                                        } else if next < 64 {
                                            panic!("next<64 in line {}, column {}",self.line,self.pos)
                                        }  else if next < 128 {
                                            self.charbuffer = Some((next-64,startl,startpos))
                                        }
                                            else { panic!("Invalid character after ^^") }
                                    } else {
                                        self.charbuffer = Some(next); return true
                                    }
                                },
                                _ => { self.charbuffer = Some(next); return true }
                            }
                        }
                    }
                }
            }
        }
    }
    pub fn pop_next(&mut self,state:&State, nocomment: bool) -> Rc<Token> {
        if !self.has_next(state,true) {panic!("Mouth is empty")}
        if let Some(tk) = self.peekbuffer.take() { tk } else {
            let (char,l,p) = self.next_char(state).unwrap();
            let catcode = |x:u8| state.catcodes().get_code(x);
            let ret = match catcode(char) {
                CategoryCode::Escape => {
                    let mut buf : Vec<u8> = Vec::new();
                    let string = self.string.as_ref().unwrap().as_bytes();
                    match string.get(self.pos) {
                        None => {self.mouth_state = MouthState::M}
                        Some(nc) => {
                            match catcode(*nc) {
                                CategoryCode::Letter => {
                                    while match string.get(self.pos) {
                                        None => false,
                                        Some(s) => matches!(catcode(*s),CategoryCode::Letter)
                                    } {
                                        let nc = string.get(self.pos).unwrap();
                                        self.pos += 1;
                                        buf.push(*nc);
                                    }
                                    self.mouth_state = MouthState::S;
                                }
                                CategoryCode::Space => {
                                    let nc = string.get(self.pos).unwrap();
                                    self.pos += 1;
                                    buf.push(*nc);
                                    self.mouth_state = MouthState::S
                                }
                                _ => {
                                    let nc = string.get(self.pos).unwrap();
                                    self.pos += 1;
                                    buf.push(*nc);
                                    self.mouth_state = MouthState::M
                                }
                            }
                        }
                    }
                    let name = from_utf8(buf.as_slice()).unwrap();
                    PrimitiveControlSequence::new(name.to_owned(),self.make_reference(l,p)).as_token()
                }
                CategoryCode::EOL if matches!(self.mouth_state,MouthState::M) => {
                    self.mouth_state = MouthState::S;
                    PrimitiveCharacterToken::new(char,CategoryCode::Space,self.make_reference(l,p)).as_token()
                }
                CategoryCode::EOL if matches!(self.mouth_state,MouthState::N) => {
                    while(self.has_next(state,nocomment)) {
                        let (n,l2,p2) = self.next_char(state).unwrap();
                        if !matches!(catcode(n),CategoryCode::EOL) {
                            self.charbuffer = Some((n,l2,p2));
                            break
                        }
                    }
                    PrimitiveControlSequence::new("par".to_owned(),self.make_reference(l,p)).as_token()
                }
                CategoryCode::Space if matches!(self.mouth_state,MouthState::M) => {
                    self.mouth_state = MouthState::S;
                    PrimitiveCharacterToken::new(char,CategoryCode::Space,self.make_reference(l,p)).as_token()
                }
                CategoryCode::Active => {
                    PrimitiveActiveCharacterToken::new(char,self.make_reference(l,p)).as_token()
                }
                _ => {
                    self.mouth_state = MouthState::M;
                    PrimitiveCharacterToken::new(char,catcode(char),self.make_reference(l,p)).as_token()
                }
            };
            let obj = LaTeXObject::Token(Rc::clone(&ret));

            match self.source.get_file() {
                Some(ltxf) => {
                    ltxf.add(Rc::new(obj))
                }
                _ => {}
            }
            ret
        }
    }
    fn peek(&mut self,state:&State) -> Rc<Token> {
        let next = self.pop_next(state,true);
        self.peekbuffer = Some(Rc::clone(&next));
        next
    }
    fn preview(&mut self) -> String {
        todo!()
    }
    fn pushback(&mut self) {
        todo!()
    }
}

use std::path::Path;
use crate::interpreter::files::VFile;
use crate::interpreter::mouth::Mouth::Str;

pub (in crate::interpreter) struct Mouths {
    mouths: Vec<Mouth>,
    buffer: Option<Rc<Token>>
}

impl Mouths {
    pub fn new() -> Mouths {
        Mouths {
            mouths:Vec::new(),
            buffer:None
        }
    }
    pub(in crate::interpreter::mouth) fn has_next(&mut self,state:&State) -> bool {
        match self.buffer {
            Some(_) => true,
            _ => loop {
                match self.mouths.last_mut() {
                    None => return false,
                    Some(m) => {
                        if m.has_next(state,true) {return true} else {
                            match m {
                                Mouth::File(fm) => todo!(),
                                _ => if !self.next_mouth() { return false }
                            }
                        }
                    }
                }
            }
        }
    }
    fn next_mouth(&mut self) -> bool {
        if (self.mouths.len() > 1) {
            self.mouths.pop();
            true
        } else { false }
    }
    pub(in crate::interpreter) fn next_token(&mut self,state:&State) -> Rc<Token> {
        match self.buffer {
            Some(_) => self.buffer.take().unwrap(),
            _ => if self.has_next(state) {
                self.mouths.last_mut().unwrap().get_next(state)
            } else {
                panic!("Mouths empty!")
            }
        }
    }
    pub(in crate::interpreter::mouth) fn push_expansion(&mut self, exp : Expansion) {
        if !exp.exp.is_empty() {
            let nm = Mouth::Token(TokenMouth::new(exp));
            self.mouths.push(nm)
        }
    }
    pub(in crate::interpreter::mouth) fn push_tokens(&mut self, tks : Vec<Rc<Token>>) {
        if !tks.is_empty() {
            let nm = Mouth::Token(TokenMouth::new(Expansion::dummy(tks)));
            self.mouths.push(nm)
        }
    }
    /*
    pub(in crate::interpreter) fn push_file<'a>(&'a mut self,state : &'a State<'a>, file : VFile) {
        let fm = StringMouth::new_from_file(state,file);
        self.mouths.push(Mouth::File(fm))
    }

     */
    pub(in crate::interpreter::mouth) fn push_file(&mut self,state:&State,file:VFile) {
        self.mouths.push(Mouth::File(StringMouth::new_from_file(state,file)))
    }

    pub(in crate::interpreter) fn requeue(&mut self, tk : Rc<Token>) {
        self.buffer = Some(tk)
    }
}

impl Interpreter<'_,'_> {
    pub fn push_file(&mut self,file:VFile) {
        self.mouths.push_file(&self.state,file)
    }
    pub fn has_next(&mut self) -> bool {
        self.mouths.has_next(&self.state)
    }
    /*
    pub fn push_file(&mut self,file : VFile) {
        let mut fm = StringMouth::new_from_file(&self.state,file);
        self.mouths.mouths.push(Mouth::File(fm))
    }

     */
}
