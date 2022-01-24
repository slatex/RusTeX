#[derive(Clone,PartialEq)]
enum MouthState { N,S,M }

use std::borrow::{Borrow, BorrowMut};
use std::str::from_utf8;
use std::sync::Arc;
use crate::ontology::{Comment, Expansion, LaTeXFile, Token, LaTeXObject};
use crate::catcodes::{CategoryCode, CategoryCodeScheme};
use crate::references::{SourceFileReference, SourceReference};
use crate::utils::{TeXStr, TeXString};

pub enum Mouth {
    Token(TokenMouth),
    Str(StringMouth),
    File(StringMouth),
    FileLike(StringMouth)
}

impl Mouth {
    pub(crate) fn preview(&self) -> TeXString {
        match self {
            Mouth::Token(tm) => tm.preview(),
            Mouth::Str(tm) => tm.preview(),
            Mouth::File(tm) => tm.preview(),
            Mouth::FileLike(tm) => tm.preview()
        }
    }
    pub(crate) fn has_next(&mut self,catcodes:&CategoryCodeScheme, nocomment : bool) -> bool {
        match self {
            Mouth::Token(tm) => tm.has_next(nocomment),
            Mouth::Str(sm) => sm.has_next(catcodes,nocomment,false),
            Mouth::File(sm) => sm.has_next(catcodes,nocomment,false),
            Mouth::FileLike(sm) => sm.has_next(catcodes,nocomment,false)
        }
    }
    pub(crate) fn get_next(&mut self,catcodes:&CategoryCodeScheme) -> Token {
        match self {
            Mouth::Token(tm) => tm.pop_next(true),
            Mouth::Str(sm) => sm.pop_next(catcodes,true),
            Mouth::File(sm) => sm.pop_next(catcodes,true),
            Mouth::FileLike(sm) => sm.pop_next(catcodes,true)
        }
    }
}

pub struct TokenMouth {
    tokens : Vec<Token>
}
impl TokenMouth {
    fn new(tokens:Vec<Token>) -> TokenMouth {
        let mut tm = TokenMouth { tokens };
        tm.tokens.reverse();
        tm
    }
    fn has_next(&mut self, _nocomment: bool) -> bool {
        !self.tokens.is_empty()
    }
    fn pop_next(&mut self, _nocomment: bool) -> Token {
        self.tokens.pop().unwrap()
    }
    fn preview(&self) -> TeXString {
        crate::interpreter::tokens_to_string_default(&self.tokens.iter().rev().map(|x| x.clone()).collect())
    }
}

use crate::interpreter::Interpreter;

#[derive(Clone)]
pub (in crate) enum StringMouthSource {
    File(LaTeXFile),
    Exp(Expansion)
}

impl StringMouthSource {
    pub fn pop_file(self) -> Option<LaTeXFile> {
        match self {
            StringMouthSource::File(s) => Some(s),
            _ => None
        }
    }
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

#[derive(Clone)]
pub struct StringMouth {
    mouth_state:MouthState,
    peekbuffer : Option<Token>,
    string : Option<TeXString>,
    allstrings : Vec<TeXString>,
    pub line: usize,
    pub pos: usize,
    atendofline:Option<u8>,
    charbuffer:Option<(u8,usize,usize)>,
    pub(in crate) source : StringMouthSource,
    iseof : bool
}

impl StringMouth {
    pub(in crate::interpreter) fn is_eof(&self) -> bool {
        self.iseof
    }
    pub(in crate::interpreter) fn read_line(&mut self, catcodes:&CategoryCodeScheme) -> Vec<Token> {
        match &self.string {
            None => {
                if self.allstrings.is_empty() {
                    self.iseof = true;
                    vec!()
                } else {
                    let string = self.allstrings.pop().unwrap();
                    self.string = Some(string);
                    self.read_line(catcodes)
                }
            }
            Some(s) => {
                let mut ret : Vec<Token> = vec!();
                for c in s.0.iter() {
                    match catcodes.get_code(*c) {
                        CategoryCode::Space if ret.is_empty() => (),
                        CategoryCode::Space => ret.push(Token::new(*c,CategoryCode::Space,None,SourceReference::None,true)),
                        _ => ret.push(Token::new(*c,CategoryCode::Other,None,SourceReference::None,true)),
                    }
                }
                self.string = None;
                if ret.is_empty() && !self.is_eof() {
                    self.read_line(catcodes)
                } else { ret }
            }
        }
    }
    pub(in crate::interpreter) fn read(&mut self, catcodes:&CategoryCodeScheme, nocomment:bool) -> Vec<Token> {
        self.has_next(catcodes,nocomment,false);
        let currentline = self.line;
        let mut ret:Vec<Token> = vec!();
        let mut braces = 0;
        while self.has_next(catcodes,nocomment,false) && (self.line == currentline || braces > 0) {
            match self.pop_next(catcodes,nocomment) {
                tk if tk.catcode == CategoryCode::BeginGroup => {
                    ret.push(tk);
                    braces +=1;
                }
                tk if tk.catcode == CategoryCode::EndGroup => {
                    ret.push(tk);
                    braces -=1;
                }
                tk if tk.catcode == CategoryCode::Parameter => {
                    ret.push(tk.clone());
                    ret.push(tk);
                }
                tk => {ret.push(tk);}
            }
        }
        match ret.last() {
            Some(tk) if tk.catcode == CategoryCode::Space && tk.char == catcodes.endlinechar => {ret.pop();}
            Some(tk) if tk.char == 0 && match tk.name() {s if s == "EOF" => true,_ => false} && ret.len() == 1 => {ret.pop();self.iseof=true}
            None => self.iseof = true,
            _ => ()
        }
        ret
    }
    pub fn new_from_file(catcodes:&CategoryCodeScheme, file:&Arc<VFile>) -> StringMouth {
        use crate::interpreter::files::VFileBase;
        let ltxf = LaTeXFile::new(file.id.clone(),match &file.source {
            VFileBase::Real(p) => p.clone(),
            VFileBase::Virtual => file.id.clone()
        });
        let string = match &*file.string.read().unwrap() {
            Some(s) => Some(s.clone()),
            None => None
        };
        StringMouth::new_i(catcodes.newlinechar,StringMouthSource::File(ltxf),string)
    }
    pub fn new<'a>(newlinechar:u8, source:Expansion, string : TeXString) -> StringMouth {
        StringMouth::new_i(newlinechar,StringMouthSource::Exp(source),Some(string))
    }
    fn new_i(newlinechar:u8, source:StringMouthSource, string : Option<TeXString>) -> StringMouth {
        match string {
            Some(string) => {
                let it = if string.is_empty() {
                    vec![]
                } else if newlinechar==u8::MAX {
                    vec![string]
                } else {
                    let mut ret = string.split(newlinechar);
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
                    source,
                    iseof: false
                }
            }
            None =>
                StringMouth {
                    mouth_state:MouthState::N,
                    allstrings:vec!(),
                    string:None,
                    peekbuffer:None,
                    atendofline:None,
                    line:0,
                    pos:0,
                    charbuffer: None,
                    source,
                    iseof: true
                }
        }
    }
    fn do_line(&mut self,endlinechar:u8) -> bool {
        self.atendofline =  None;
        if self.allstrings.is_empty() {
            self.string = None;
            false
        } else {
            match endlinechar {
                u8::MAX => {},
                o => self.atendofline =  Some(o)
            };
            //self.allstrings.pop().unwrap().trim_end().as_bytes();
            let mut string = self.allstrings.pop().unwrap();
            loop {
                match string.0.last() {
                    Some(32) => string.0.pop(),
                    _ => break
                };
            }
            self.string = Some(string);
            self.line += 1;
            self.pos = 0;
            self.mouth_state = MouthState::N;
            true
        }
    }

    fn next_char(&mut self,endlinechar:u8) -> Option<(u8, usize, usize)> {
        loop {
            if let Some(tk) = self.charbuffer.take() { return Some(tk) } else {
                match self.string {
                    None => match self.do_line(endlinechar) {
                        true => {},
                        false => return None
                    }
                    Some(ref str) => {
                        if str.len() <= self.pos {
                            match self.atendofline {
                                Some(cb) => {
                                    self.atendofline = None;
                                    return Some((cb, self.line, self.pos))
                                },
                                None => match self.do_line(endlinechar) {
                                    true => {},
                                    false => return None
                                }
                            }
                        } else {
                            let ret = str.0.get(self.pos).unwrap();
                            self.pos += 1;
                            return Some((*ret, self.line, self.pos))
                        }
                    }
                }
            }
        }
    }

    fn do_s(&mut self,catcodes:&CategoryCodeScheme) {
        while self.has_next(catcodes,true,false) {
            let next = self.next_char(catcodes.endlinechar).unwrap();
            match catcodes.get_code(next.0) {
                CategoryCode::Space => {}
                CategoryCode::EOL => {
                    self.do_line(catcodes.endlinechar);
                    break
                },
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
            Some(r) => self.make_file_reference(r,line,pos)
        }
    }
    fn make_file_reference(&self,f : &LaTeXFile,line:usize,pos:usize) -> SourceReference {
        SourceReference::File(f.path.as_ref().unwrap().clone(),(line,pos),(self.line,self.pos))
    }

    pub fn has_next(&mut self,catcodes:&CategoryCodeScheme, nocomment: bool,allowignore:bool) -> bool {
        match self.peekbuffer {
            Some(_) => true,
            None => {
                loop {
                    match self.mouth_state {
                        MouthState::S => {
                            self.mouth_state = MouthState::M;
                            self.do_s(catcodes);
                        }
                        _ => ()
                    }
                    match self.next_char(catcodes.endlinechar) {
                        None => return false, // ret = Some(false),
                        Some(next) => match catcodes.get_code(next.0) {
                            CategoryCode::Ignored if !allowignore && STORE_IN_FILE => {
                                let file = self.source.get_file();
                                match file {
                                    Some(ltxf) => {
                                        let nrf = SourceReference::File(
                                            ltxf.path.as_ref().unwrap().clone(),
                                            (next.1, next.2),
                                            (self.line, self.pos)
                                        );
                                        let tk = Token::new(next.0, CategoryCode::Ignored, None, nrf, true);
                                        ltxf.add(LaTeXObject::Token(tk))
                                        // TODO
                                    }
                                    _ => {}
                                }
                            }
                            CategoryCode::Ignored if !allowignore => {}
                            CategoryCode::Comment if !allowignore => if nocomment {
                                let mut rest: Vec<u8> = (*self.string.as_ref().unwrap()).0[self.pos..].to_vec();//..slice(self.pos as usize,self.string.unwrap().len()).to_vec();
                                rest.insert(0, next.0);
                                match (STORE_IN_FILE, self.source.get_file()) {
                                    (true, Some(ltxf)) => {
                                        let txt = std::str::from_utf8(rest.as_slice()).unwrap().to_string();
                                        let end = txt.len();
                                        self.pos += end;
                                        let nrf = SourceReference::File(ltxf.path.as_ref().unwrap().clone(),
                                                                        (next.1, next.2), (self.line, self.pos)
                                        );
                                        let tk = Comment {
                                            text: txt,
                                            reference: nrf
                                        };
                                        ltxf.add(LaTeXObject::Comment(tk))
                                    }
                                    _ => {}
                                }
                                self.do_line(catcodes.endlinechar);
                                loop {
                                    match self.next_char(catcodes.endlinechar) {
                                        None => break,
                                        Some(n) => {
                                            let cc = catcodes.get_code(n.0);
                                            match cc {
                                                CategoryCode::Space | CategoryCode::EOL => {}
                                                _ => {
                                                    self.charbuffer = Some(n);
                                                    break
                                                }
                                            }
                                        }
                                    }
                                }
                            }
                            CategoryCode::Space if self.mouth_state == MouthState::N => {}
                            CategoryCode::Superscript => {
                                let string = self.string.as_ref().unwrap();
                                let len = string.0[self.pos..].len();
                                let peek = string.0.get(self.pos);
                                if len > 1 && peek.is_some() && *peek.unwrap() == next.0 {
                                    let (startl, startpos) = (next.1, next.2);
                                    self.pos += 1;
                                    let next = *string.0.get(self.pos).unwrap();
                                    self.pos += 1;
                                    let maybenext = string.0.get(self.pos);
                                    fn cond(i: u8) -> bool { (48 <= i && i <= 57) || (97 <= i && i <= 102) }
                                    if (cond(next)) && maybenext.is_some() && cond(*maybenext.unwrap()) {
                                        self.pos += 1;
                                        self.charbuffer = Some((u8::from_str_radix(from_utf8(&[next, *maybenext.unwrap()]).unwrap(), 16).unwrap(), startl, startpos))
                                    } else if next < 128 {
                                        self.charbuffer = Some(((((next as i16) - 64) as u8), startl, startpos))
                                    } else { panic!("Invalid character after ^^") }
                                } else {
                                    self.charbuffer = Some(next);
                                    return true
                                }
                            },
                            _ => {
                                self.charbuffer = Some(next);
                                return true
                            }
                        }
                    }
                }
            }
        }
    }
    pub fn pop_next(&mut self,catcodes:&CategoryCodeScheme, nocomment: bool) -> Token {
        if !self.has_next(catcodes,true,false) {panic!("Mouth is empty")}
        if let Some(tk) = self.peekbuffer.take() { tk } else {
            let (char,l,p) = self.next_char(catcodes.endlinechar).unwrap();
            let ret = match catcodes.get_code(char) {
                CategoryCode::Escape => {
                    self.mouth_state = MouthState::M;
                    match &self.string {
                        Some(s) => {
                            if self.pos == s.len() {
                                let tk = Token::new(char,CategoryCode::Escape,Some("".into()),self.make_reference(l,p),true);
                                self.do_line(catcodes.endlinechar);
                                return tk
                            }
                        }
                        None => unreachable!()
                    }
                    let mut buf : Vec<u8> = Vec::new();
                    let maybecomment = self.next_char(catcodes.endlinechar);
                    match maybecomment {
                        Some((tk,_,_)) if catcodes.get_code(tk) == CategoryCode::Comment || catcodes.get_code(tk) == CategoryCode::Ignored => {
                            Token::new(char,CategoryCode::Escape,Some(TeXStr::new(&[tk])),self.make_reference(l,p),true)
                        }
                        None => {
                            Token::new(char,CategoryCode::Escape,Some(TeXStr::new(&[])),self.make_reference(l,p),true)
                        }
                        _ => {
                            self.charbuffer = maybecomment;
                            if !self.has_next(catcodes,true,true) {panic!("Mouth is empty")}
                            let mut nc = self.next_char(catcodes.endlinechar).unwrap();
                            match catcodes.get_code(nc.0) {
                                CategoryCode::Letter => {
                                    let line = self.line;
                                    self.mouth_state = MouthState::M;
                                    buf.push(nc.0);
                                    while line == self.line && {
                                        nc = self.next_char(catcodes.endlinechar).unwrap();
                                        matches!(catcodes.get_code(nc.0),CategoryCode::Letter)
                                    } {
                                        buf.push(nc.0);
                                    }
                                    self.charbuffer = Some(nc);
                                    self.mouth_state = MouthState::S;
                                }
                                //CategoryCode::EOL => self.mouth_state = MouthState::S,
                                CategoryCode::Space => {
                                    buf.push(nc.0);
                                    self.mouth_state = MouthState::S
                                }
                                _ => {
                                    buf.push(nc.0);
                                    self.mouth_state = MouthState::M
                                }
                            }
                            Token::new(char,CategoryCode::Escape,Some(TeXStr::new(buf.as_slice())),self.make_reference(l,p),true)
                        }
                    }
                }
                CategoryCode::EOL if matches!(self.mouth_state,MouthState::M) => {
                    self.mouth_state = MouthState::S;
                    Token::new(32,CategoryCode::Space,None,self.make_reference(l,p),true)
                }
                CategoryCode::EOL if matches!(self.mouth_state,MouthState::N) => {
                    while self.has_next(catcodes,nocomment,false) {
                        let (n,l2,p2) = self.next_char(catcodes.endlinechar).unwrap();
                        if !matches!(catcodes.get_code(n),CategoryCode::EOL) {
                            self.charbuffer = Some((n,l2,p2));
                            break
                        }
                    }
                    Token::new(char,CategoryCode::Escape,Some("par".into()),self.make_reference(l,p),true)
                }
                CategoryCode::Space if matches!(self.mouth_state,MouthState::M) => {
                    self.mouth_state = MouthState::S;
                    Token::new(32,CategoryCode::Space,None,self.make_reference(l,p),true)
                }
                o => {
                    self.mouth_state = MouthState::M;
                    Token::new(char,o,None,self.make_reference(l,p),true)
                }
            };
            match (STORE_IN_FILE,self.source.get_file()) {
                (true,Some(ltxf)) => {
                    ltxf.add(LaTeXObject::Token(ret.clone()))
                }
                _ => {}
            }
            ret
        }
    }
    fn preview(&self) -> TeXString {
        let mut rest : Vec<u8> = match self.string.as_ref() {
            Some(r) => r.0[self.pos..].to_vec(),
            _ => vec!()
        }; //(*self.string.as_ref().unwrap().0)[self.pos..].to_vec();
        for s in self.allstrings.iter().rev() {
            for c in &s.0 {
                rest.push(*c)
            }
        }
        match self.charbuffer {
            None => (),
            Some((c,_,_)) => rest.insert(0,c)
        }
        match self.atendofline {
            None => (),
            Some(c) => rest.push(c)
        }
        rest.into()
    }
}

use crate::interpreter::files::VFile;
use crate::interpreter::params::InterpreterParams;
use crate::STORE_IN_FILE;

pub (in crate) struct Mouths {
    pub mouths: Vec<Mouth>,
    buffer: Option<Token>,
}

impl Mouths {
    pub fn new() -> Mouths {
        Mouths {
            mouths:Vec::new(),
            buffer:None
        }
    }
    pub(in crate::interpreter::mouth) fn has_next(&mut self,catcodes:&CategoryCodeScheme,io:&dyn InterpreterParams) -> Result<bool,EOF> {
        match self.buffer {
            Some(_) => Ok(true),
            _ => loop {
                match self.mouths.last_mut() {
                    None => return Ok(false),
                    Some(m) => {
                        if m.has_next(catcodes,true) {return Ok(true)} else {
                            match self.mouths.pop().unwrap() {
                                Mouth::File(f) if self.mouths.is_empty() => {
                                    self.mouths.push(Mouth::File(f));
                                    return Ok(false)
                                }
                                Mouth::File(fm) if STORE_IN_FILE => {
                                    io.file_close();
                                    let lastfile = self.mouths.iter_mut().rev().find(|x| match x {
                                        Mouth::File(_) => true,
                                        _ => false
                                    });
                                    match lastfile {
                                        Some(Mouth::File(nfm)) => {
                                            match nfm.source.borrow_mut() {
                                                StringMouthSource::File(f) => f.add(LaTeXObject::File(fm.source.pop_file().unwrap())),
                                                _ => panic!("This can't happen!")
                                            }
                                        }
                                        _ => panic!("This shouldn't happen!")
                                    }
                                    return Err(EOF {})
                                }
                                Mouth::File(_) => {
                                    io.file_close();
                                    return Err(EOF {})
                                }
                                Mouth::FileLike(_) => {
                                    return Err(EOF {})
                                }
                                _ => {}
                            }
                        }
                    }
                }
            }
        }
    }

    pub(in crate::interpreter::mouth) fn next_token(&mut self,catcodes:&CategoryCodeScheme,io:&dyn InterpreterParams) -> Result<Token,EOF> {
        match self.buffer {
            Some(_) => Ok(self.buffer.take().unwrap()),
            _ => if self.has_next(catcodes,io)? {
                Ok(self.mouths.last_mut().unwrap().get_next(catcodes))
            } else {
                panic!("Mouths empty!")
            }
        }
    }
    pub(in crate::interpreter) fn push_expansion(&mut self, exp : Expansion) {
        if self.buffer.is_some() {
            let buf = self.buffer.take().unwrap();
            self.push_tokens(vec!(buf))
        }
        if !exp.2.is_empty() {
            let nm = Mouth::Token(TokenMouth::new(exp.2));
            self.mouths.push(nm)
        }
    }
    pub(in crate::interpreter) fn push_tokens(&mut self, tks : Vec<Token>) {
        let mut ntks = tks;
        match self.buffer.take() {
            Some(t) => ntks.push(t),
            _ => ()
        }
        if !ntks.is_empty() {
            let nm = Mouth::Token(TokenMouth::new(ntks));
            self.mouths.push(nm)
        }
    }
    pub(in crate::interpreter::mouth) fn push_file(&mut self,catcodes:&CategoryCodeScheme,file:&Arc<VFile>) {
        if self.buffer.is_some() {
            let buf = self.buffer.take().unwrap();
            self.push_tokens(vec!(buf))
        }
        self.mouths.push(Mouth::File(StringMouth::new_from_file(catcodes,file)))
    }
    pub(in crate::interpreter::mouth) fn push_string(&mut self,catcodes:&CategoryCodeScheme,exp:Expansion,string : TeXString,filelike:bool) {
        if self.buffer.is_some() {
            let buf = self.buffer.take().unwrap();
            self.push_tokens(vec!(buf))
        }
        if filelike {
            self.mouths.push(Mouth::FileLike(StringMouth::new(catcodes.newlinechar,exp,string)))
        } else {
            self.mouths.push(Mouth::Str(StringMouth::new(catcodes.newlinechar,exp,string)))
        }
    }

    pub(in crate::interpreter) fn requeue(&mut self, tk : Token) {
        self.buffer = Some(tk)
    }

    pub fn line_no(&self) -> (usize,usize) {
        match self.mouths.iter().rev().find(|m| match m {
            Mouth::File(_sm) => true,
            _ => false
        }) {
            Some(Mouth::File(m)) => {
                match &m.source {
                    StringMouthSource::File(_) => {
                        (m.line,m.pos)
                    }
                    _ => (0,0)
                }
            }
            _ => (0,0)
        }
    }

    pub fn current_file(&self) -> TeXStr {
        match self.mouths.iter().rev().find(|m| match m {
            Mouth::File(_sm) => true,
            _ => false
        }) {
            Some(Mouth::File(m)) => {
                match &m.source {
                    StringMouthSource::File(lf) => {
                        lf.path.as_ref().unwrap().clone()
                    }
                    _ => "".into()
                }
            }
            _ => "".into()
        }
    }

    pub fn current_line(&self) -> String {
        match self.mouths.iter().rev().find(|m| match m {
            Mouth::File(_sm) => true,
            _ => false
        }) {
            Some(Mouth::File(m)) => {
                match &m.source {
                    StringMouthSource::File(lf) => {
                        lf.path.as_ref().unwrap().to_string() + " (" + m.line.to_string().as_str() + ", " + m.pos.to_string().as_str() + ")"
                    }
                    _ => "".to_string()
                }
            }
            _ => "".to_string()
        }
    }
    pub fn end_input(&mut self,io:&dyn InterpreterParams) {
        loop {
            match self.mouths.last() {
                Some(Mouth::File(_)) => {
                    self.mouths.pop();
                    io.file_close();
                    return ()
                }
                Some(_) => {self.mouths.pop();}
                _ => panic!("Mouth empty!")
            }
        }
    }
    pub fn preview(&self) -> TeXString {
        let mut ret : TeXString = "".into();
        for s in self.mouths.iter().rev() {
            ret += s.preview()
        }
        match self.buffer.borrow() {
            None => ret,
            Some(tk) => crate::interpreter::tokens_to_string_default(&vec!(tk.clone())) + ret
        }
    }
    pub fn close(&mut self) {
        self.mouths.clear()
    }
}

impl Interpreter<'_> {
    pub fn end(&mut self) { self.mouths.close() }
    pub fn preview(&self) -> TeXString {
        self.mouths.borrow().preview()
    }
    pub fn push_file(&mut self,file:Arc<VFile>) {
        use crate::interpreter::files::VFileBase;
        if !self.mouths.borrow().mouths.is_empty() {
            self.params.file_open(&match file.source {
                VFileBase::Real(ref pb) => "\n(".to_string() + &pb.to_string(),
                _ => "\n(".to_string() + &file.id.to_string()
            });
        }
        self.mouths.push_file(self.state.catcodes.get_scheme(),&file);
        //self.filestore.borrow_mut().files.insert(file.id.clone(),file);
    }
    pub fn push_string(&mut self,exp:Expansion,str:TeXString,filelike:bool) {
        self.mouths.push_string(self.state.catcodes.get_scheme(),exp,str,filelike)
    }
    pub fn push_expansion(&mut self,exp:Expansion) {
        self.mouths.push_expansion(exp)
    }
    pub fn push_tokens(&mut self,tks:Vec<Token>) {
        self.mouths.push_tokens(tks)
    }
    pub fn next_token(&mut self) -> Token {
        let ret = self.mouths.next_token(self.state.catcodes.get_scheme(),self.params);
        match ret {
            Ok(t) => t,
            Err(_) => {
                self.doeof();
                self.next_token()
            }
        }
    }
    pub fn requeue(&mut self,token:Token) {
        self.mouths.requeue(token)
    }
    pub fn has_next(&mut self) -> bool {
        let ret = self.mouths.has_next(self.state.catcodes.get_scheme(),self.params);
        match ret {
            Ok(t) => t,
            Err(_) => {
                self.doeof();
                self.has_next()
            }
        }
    }
    pub(in crate::interpreter::mouth) fn doeof(&mut self) {
        self.push_tokens(vec!(self.eof_token()));
        self.insert_every(&crate::commands::primitives::EVERYEOF)
    }

    pub fn eof_token(&self) -> Token {
        Token::new(0,CategoryCode::EOL,Some("EOF".into()),SourceReference::None,true)
    }
    pub fn end_input(&mut self) {
        self.mouths.end_input(self.params)
    }
    pub fn update_reference(&self,tk : &Token) -> Option<SourceFileReference> {
        let mut rf = &*tk.reference;
        loop {
            match rf {
                SourceReference::File(f, s, _) => {
                    let end = self.mouths.borrow().line_no();
                    return Some(SourceFileReference {
                        file: f.clone(),
                        start: s.clone(),
                        end: end
                    })
                }
                SourceReference::Exp(er) => {
                    rf = &*er.0.reference;
                }
                _ => return None
            }
        }
    }
    pub fn current_line(&self) -> String {
        self.mouths.borrow().current_line()
    }
    pub fn line_no(&self) -> usize {
        self.mouths.borrow().line_no().0
    }
    pub fn current_file(&self) -> TeXStr {
        self.mouths.borrow().current_file()
    }
}

struct EOF {}