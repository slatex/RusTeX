enum MouthState { N,S,M }

use std::borrow::BorrowMut;
use std::iter::{Peekable, Map};
use std::rc::Rc;
use std::slice::IterMut;
use std::str::{Chars, from_utf8, Split};
use crate::ontology::{Comment, Expansion, LaTeXFile, Token,LaTeXObject};
use crate::catcodes::{CategoryCode, CategoryCodeScheme};
use crate::ontology::{PrimitiveCharacterToken,PrimitiveToken};
use crate::references::{SourceReference,FileReference};
use crate::state::State;

pub trait Mouth {
    fn has_next(&mut self,nocomment : bool) -> bool;
    fn pop_next(&mut self,nocomment : bool) -> Rc<dyn Token>;
    fn preview(&mut self) -> String;
    fn pushback(&mut self);
    fn peek(&mut self) -> Rc<dyn Token>;
}

pub struct TokenMouth {
    tokens : Vec<Rc<dyn Token>>
}
impl Mouth for TokenMouth {
    fn has_next(&mut self, nocomment: bool) -> bool {
        !self.tokens.is_empty()
    }
    fn pop_next(&mut self, nocomment: bool) -> Rc<dyn Token> {
        self.tokens.remove(0)
    }
    fn preview(&mut self) -> String {
        self.tokens.iter().map(|x| {x.origstring()}).collect::<Vec<&str>>().join("")
    }
    fn pushback(&mut self) {}
    fn peek(&mut self) -> Rc<dyn Token> {
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
    pub fn getFile(&mut self) -> Option<&mut LaTeXFile> {
        match self {
            StringMouthSource::File(s) => Some(s),
            _ => None
        }
    }
}

struct StringMouth<'a> {
    _str: &'a str,
    mouth_state:MouthState,
    interpreter_state:&'a State<'a>,
    peekbuffer : Option<Box<dyn Token>>,
    string : Option<MultiPeek<std::slice::Iter<'a,u8>>>,//Option<std::slice::Iter<'a,u8>>,
    allstrings : Vec<&'a str>,
    line: u32,
    pos: u32,
    atendofline:Option<u8>,
    charbuffer:Option<(u8,u32,u32)>,
    source : StringMouthSource,
}

impl StringMouth<'_> {
    pub fn new<'a>(state:&'a State<'a>, source:Expansion, string : &'a str) -> StringMouth<'a> {
        use std::str::{from_utf8};
        let newlinechar = state.newlinechar();
        let it = if string.is_empty() {
            vec![]
        } else if newlinechar==u8::MAX {
            vec![string]
        } else {
            let mut ret:Vec<&str> = Vec::new();
            for s in string.split(from_utf8(&[newlinechar]).unwrap()) {
                ret.push(s)
            }
            ret
        };
        StringMouth {
            _str:string,
            mouth_state:MouthState::N,
            interpreter_state:state,
            allstrings:it,
            string:None,
            peekbuffer:None,
            atendofline:None,
            line:0,
            pos:0,
            charbuffer: None,
            source: StringMouthSource::Exp(source)
        }
    }
    fn do_line<'a>(&mut self) -> bool {
        if self.allstrings.is_empty() { false } else {
            match self.interpreter_state.endlinechar() {
                u8::MAX => {},
                o => self.atendofline =  Some(o)
            };
            //self.allstrings.pop().unwrap().trim_end().as_bytes();
            let mut string = self.allstrings.pop().unwrap().trim_end().as_bytes();
            let mut astr = (*string).iter().multipeek();
            self.string = Some(astr);
            self.line += 1;
            self.pos = 0;
            self.mouth_state = MouthState::N;
            true
        }
    }
    /*
    fn ignored(source: &StringMouthSource, tk : Box<dyn LaTeXObject>) {
        match source {
            StringMouthSource::File(mut ltxf) => ltxf.add(tk as Box<dyn LaTeXObject>),
            _ => {}
        }
    }

     */
    fn next_char(&mut self) -> Option<(u8, u32, u32)> {
        match self.charbuffer {
            Some(cb) => {
                self.charbuffer = None;
                Some(cb)
            },
            None => match self.string {
                None => match self.do_line() {
                    true => self.next_char(),
                    false => None
                }
                Some(ref mut it) => {
                    if it.len() == 0 {
                        match self.atendofline {
                            Some(cb) => {
                                self.atendofline = None;
                                Some((cb, self.line + 1, self.pos + 1))
                            },
                            None => match self.do_line() {
                                true => self.next_char(),
                                false => None
                            }
                        }
                    } else { Some((*it.next().unwrap(), self.line + 1, self.pos + 1)) }
                }
            }
        }
    }
    fn do_s(&mut self) {
        use crate::catcodes::CategoryCode;
        while (self.has_next(true)) {
            let next = self.next_char().unwrap();
            match self.interpreter_state.catcodes().get_code(next.0) {
                CategoryCode::Space | CategoryCode::EOL => { self.pos += 1 }
                _ => {
                    self.charbuffer = Some(next);
                    break
                }
            }
        }
    }
}
impl Mouth for StringMouth<'_> {
    fn has_next(&mut self, nocomment: bool) -> bool {
        match self.peekbuffer {
            Some(_) => true,
            None => {
                loop {
                    match self.next_char() {
                        None => return false, // ret = Some(false),
                        Some(next) => match self.mouth_state {
                            MouthState::S => {
                                self.charbuffer = Some(next);
                                self.mouth_state = MouthState::M;
                                self.do_s();
                            }
                            _ => match self.interpreter_state.catcodes().get_code(next.0) {
                                CategoryCode::Ignored => {
                                    match self.source.getFile() {
                                        Some(ltxf) => {
                                            let tk = PrimitiveCharacterToken::new(
                                                next.0,CategoryCode::Ignored,SourceReference::File(FileReference {
                                                file:ltxf.path.clone(),
                                                start: (next.1,next.2),
                                                end: (next.1,next.2+1)
                                            }));
                                            ltxf.add(Rc::new(tk))
                                            // TODO
                                        }
                                        _ => {}
                                    }
                                    self.pos += 1;
                                }
                                CategoryCode::Comment => if nocomment {
                                    let mut rest : Vec<u8> = self.string.as_mut().unwrap().map(|x| *x).collect();
                                    rest.insert(0,next.0);
                                    match self.source.getFile() {
                                        Some(ltxf) => {
                                            let txt = unsafe{std::str::from_utf8_unchecked(rest.as_slice())}.to_string();
                                            let end = txt.len() as u32;
                                            let tk = Comment {
                                                text: txt,
                                                reference: FileReference {
                                                    file: ltxf.path.clone(),
                                                    start: (next.1, next.2),
                                                    end: (next.1, next.2 + end)
                                                }
                                            };
                                            ltxf.add(Rc::new(tk))
                                        }
                                        _ => {}
                                    }
                                    self.do_line();
                                    loop {
                                        match self.next_char() {
                                            None => break,
                                            Some(n) => {
                                                let cc = self.interpreter_state.catcodes().get_code(n.0);
                                                match cc {
                                                    CategoryCode::Space => { self.pos += 1 }
                                                    CategoryCode::EOL => { self.pos += 1 }
                                                    _ => {
                                                        self.charbuffer = Some(n);
                                                        break
                                                    }
                                                }
                                            }
                                        }
                                    }
                                }
                                CategoryCode::Superscript => {
                                    let mut string = self.string.as_mut().unwrap();
                                    if string.len() > 1 && string.peek().is_some() && **string.peek().unwrap() == next.0 {
                                        let (startl,startpos) = (next.1,next.2);
                                        self.pos += 2;
                                        string.next();
                                        let next = *string.next().unwrap();
                                        let maybenext = string.peek().map(|x| **x);
                                        if ((48 <= next && next <= 57) || (97 <= next && next <= 102)) &&
                                            match maybenext {
                                                None => false,
                                                Some(c) => (c <= next && next <= c) || (c <= next && next <= c)
                                            } {
                                            self.pos += 1;
                                            self.charbuffer = Some((u8::from_str_radix(from_utf8(&[next,maybenext.unwrap()]).unwrap(),16).unwrap(),startl,startpos))
                                        }   else if next < 128 { self.charbuffer = Some((next-64,startl,startpos)) }
                                            else { panic!("Invalid character after ^^") }
                                    } else { self.charbuffer = Some(next); return true }
                                },
                                _ => { self.charbuffer = Some(next); return true }
                            }
                        }
                    }
                }
            }
        }
    }
    fn pop_next(&mut self, nocomment: bool) -> Rc<dyn Token> {
        todo!()
    }
    fn preview(&mut self) -> String {
        todo!()
    }
    fn pushback(&mut self) {
        todo!()
    }
    fn peek(&mut self) -> Rc<dyn Token> {
        todo!()
    }
}
/*
pub struct StringMouth<'a> {
    _str : &'a str,
    _mst : MouthState,
    _ist : &'a State<'a>,
    _strs : Vec<&'a str>,
    _string : Option<MultiPeek<std::slice::Iter<'a,u8>>>,
    _pkbf : Option<Box<dyn Token>>,
    _cbf:Option<(u8,u32,u32)>,
    _eol:Option<u8>,
    _line: u32,
    _pos: u32,
}
impl StringMouth<'_> {
}
pub struct FileMouth {

}
impl FileMouth {
    pub fn new<'a>(state:&State<'a>, ltf : LaTeXFile) -> StringMouth<'a> {
        panic!("TODO")
    }
}


 */
