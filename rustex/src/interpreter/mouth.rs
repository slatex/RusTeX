enum MouthState { N,S,M }

use std::borrow::BorrowMut;
use std::rc::Rc;
use std::str::from_utf8;
use crate::ontology::{Comment, Expansion, LaTeXFile, Token, LaTeXObject};
use crate::catcodes::{CategoryCode, CategoryCodeScheme};
use crate::references::{SourceReference,FileReference};

pub enum Mouth {
    Token(TokenMouth),
    Str(StringMouth),
    File(StringMouth)
}

impl Mouth {
    pub(crate) fn has_next(&mut self,catcodes:&CategoryCodeScheme, nocomment : bool) -> bool {
        match self {
            Mouth::Token(tm) => tm.has_next(nocomment),
            Mouth::Str(sm) => sm.has_next(catcodes,nocomment),
            Mouth::File(sm) => sm.has_next(catcodes,nocomment)
        }
    }
    pub(crate) fn get_next(&mut self,catcodes:&CategoryCodeScheme) -> Token {
        match self {
            Mouth::Token(tm) => tm.pop_next(true),
            Mouth::Str(sm) => sm.pop_next(catcodes,true),
            Mouth::File(sm) => sm.pop_next(catcodes,true)
        }
    }
}

pub struct TokenMouth {
    exp: Rc<Expansion>,
    tokens : Vec<Token>
}
impl TokenMouth {
    fn new(exp:Expansion,copy:bool) -> TokenMouth {
        let mut vec : Vec<Token> = Vec::new();
        let rc = Rc::new(exp);
        if copy {
            for tk in &rc.exp {
                vec.push(tk.copied(Rc::clone(&rc)))
            }
        } else {
            for tk in &rc.exp {
                vec.push(tk.clone())
            }
        }
        TokenMouth {
            exp:rc,
            tokens:vec
        }
    }
    fn has_next(&mut self, _nocomment: bool) -> bool {
        !self.tokens.is_empty()
    }
    fn pop_next(&mut self, _nocomment: bool) -> Token {
        self.tokens.remove(0)
    }
    fn preview(&mut self) -> String {
        self.tokens.iter().map(|x| {x.name()}).collect::<Vec<_>>().join("")
    }
    fn pushback(&mut self) {}
    fn peek(&mut self) -> Token {
        self.tokens.first().expect("").clone()
    }
}

use crate::interpreter::Interpreter;

enum StringMouthSource {
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

pub struct StringMouth {
    mouth_state:MouthState,
    peekbuffer : Option<Token>,
    string : Option<String>,
    allstrings : Vec<String>,
    line: usize,
    pos: usize,
    atendofline:Option<u8>,
    charbuffer:Option<(u8,usize,usize)>,
    pub(in crate::interpreter::mouth) source : StringMouthSource,
}

impl StringMouth {
    pub fn new_from_file(catcodes:&CategoryCodeScheme, file:VFile) -> StringMouth {
        let ltxf = LaTeXFile::new(file.id);
        let string = file.string.expect("This shouldn't happen");
        StringMouth::new_i(catcodes.newlinechar,StringMouthSource::File(ltxf),string)
    }
    pub fn new<'a>(newlinechar:u8, source:Expansion, string : &'a str) -> StringMouth {
        StringMouth::new_i(newlinechar,StringMouthSource::Exp(source),string.to_string())
    }
    fn new_i(newlinechar:u8, source:StringMouthSource, string : String) -> StringMouth {
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
    fn do_line(&mut self,endlinechar:u8) -> bool {
        if self.allstrings.is_empty() { false } else {
            match endlinechar {
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
                                    return Some((cb, self.line + 1, self.pos))
                                },
                                None => match self.do_line(endlinechar) {
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

    fn do_s(&mut self,catcodes:&CategoryCodeScheme) {
        while self.has_next(catcodes,true) {
            let next = self.next_char(catcodes.endlinechar).unwrap();
            match catcodes.get_code(next.0) {
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

    pub fn has_next(&mut self,catcodes:&CategoryCodeScheme, nocomment: bool) -> bool {
        match self.peekbuffer {
            Some(_) => true,
            None => {
                loop {
                    match self.next_char(catcodes.endlinechar) {
                        None => return false, // ret = Some(false),
                        Some(next) => match self.mouth_state {
                            MouthState::S => {
                                self.charbuffer = Some(next);
                                self.mouth_state = MouthState::M;
                                self.do_s(catcodes);
                            }
                            _ => match catcodes.get_code(next.0) {
                                CategoryCode::Ignored if STORE_IN_FILE => {
                                    let file = self.source.get_file();
                                    match file {
                                        Some(ltxf) => {
                                            let nrf = FileReference {
                                                file:ltxf.path.clone(),
                                                start: (next.1,next.2),
                                                end: (self.line,self.pos)
                                            };
                                            let tk = Token {
                                                char: next.0,
                                                catcode: CategoryCode::Ignored,
                                                name_opt: None,
                                                reference: Box::new(SourceReference::File(nrf))
                                            };
                                            ltxf.add(LaTeXObject::Token(tk))
                                            // TODO
                                        }
                                        _ => {}
                                    }
                                }
                                CategoryCode::Ignored => {}
                                CategoryCode::Comment => if nocomment {
                                    let mut rest : Vec<u8> = (*self.string.as_ref().unwrap().as_bytes())[self.pos..].to_vec();//..slice(self.pos as usize,self.string.unwrap().len()).to_vec();
                                    rest.insert(0,next.0);
                                    match (STORE_IN_FILE, self.source.get_file()) {
                                        (true,Some(ltxf)) => {
                                            let txt = std::str::from_utf8(rest.as_slice()).unwrap().to_string();
                                            let end = txt.len();
                                            self.pos += end;
                                            let nrf = FileReference {
                                                file:ltxf.path.clone(),
                                                start: (next.1,next.2),
                                                end: (self.line,self.pos)
                                            };//self.make_file_reference(ltxf,next.1,next.2);
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
                                    if len > 1 && peek.is_some() && *peek.unwrap() == next.0 {
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
    pub fn pop_next(&mut self,catcodes:&CategoryCodeScheme, nocomment: bool) -> Token {
        if !self.has_next(catcodes,true) {panic!("Mouth is empty")}
        if let Some(tk) = self.peekbuffer.take() { tk } else {
            let (char,l,p) = self.next_char(catcodes.endlinechar).unwrap();
            let ret = match catcodes.get_code(char) {
                CategoryCode::Escape => {
                    let mut buf : Vec<u8> = Vec::new();
                    let maybecomment = self.next_char(catcodes.endlinechar);
                    match maybecomment {
                        Some((tk,_,_)) if matches!(catcodes.get_code(tk),CategoryCode::Comment) => {
                            Token {
                                char,
                                catcode: CategoryCode::Escape,
                                name_opt: Some(from_utf8(&[tk]).unwrap().to_owned()),
                                reference: Box::new(self.make_reference(l,p))
                            }
                        }
                        None => {
                            Token {
                                char,
                                catcode: CategoryCode::Escape,
                                name_opt: Some("".to_owned()),
                                reference: Box::new(self.make_reference(l,p))
                            }
                        }
                        _ => {
                            self.charbuffer = maybecomment;
                            if !self.has_next(catcodes,true) {panic!("Mouth is empty")}
                            let mut nc = self.next_char(catcodes.endlinechar).unwrap();
                            match catcodes.get_code(nc.0) {
                                CategoryCode::Letter => {
                                    self.mouth_state = MouthState::M;
                                    buf.push(nc.0);
                                    while self.has_next(catcodes,true) && {
                                        nc = self.next_char(catcodes.endlinechar).unwrap();
                                        matches!(catcodes.get_code(nc.0),CategoryCode::Letter)
                                    } {
                                        buf.push(nc.0);
                                    }
                                    self.charbuffer = Some(nc);
                                    self.mouth_state = MouthState::S;
                                }
                                CategoryCode::EOL => self.mouth_state = MouthState::M,
                                CategoryCode::Space => {
                                    buf.push(nc.0);
                                    self.mouth_state = MouthState::S
                                }
                                _ => {
                                    buf.push(nc.0);
                                    self.mouth_state = MouthState::M
                                }
                            }
                            let name = from_utf8(buf.as_slice()).unwrap();
                            Token {
                                char,
                                catcode: CategoryCode::Escape,
                                name_opt: Some(name.to_owned()),
                                reference: Box::new(self.make_reference(l,p))
                            }
                        }
                    }
                }
                CategoryCode::EOL if matches!(self.mouth_state,MouthState::M) => {
                    self.mouth_state = MouthState::S;
                    Token {
                        char,
                        catcode:CategoryCode::Space,
                        name_opt:None,
                        reference: Box::new(self.make_reference(l,p))
                    }
                }
                CategoryCode::EOL if matches!(self.mouth_state,MouthState::N) => {
                    while self.has_next(catcodes,nocomment) {
                        let (n,l2,p2) = self.next_char(catcodes.endlinechar).unwrap();
                        if !matches!(catcodes.get_code(n),CategoryCode::EOL) {
                            self.charbuffer = Some((n,l2,p2));
                            break
                        }
                    }
                    Token {
                        char,
                        catcode:CategoryCode::Escape,
                        name_opt:Some("par".to_owned()),
                        reference:Box::new(self.make_reference(l,p))
                    }
                }
                CategoryCode::Space if matches!(self.mouth_state,MouthState::M) => {
                    self.mouth_state = MouthState::S;
                    Token {
                        char,
                        catcode:CategoryCode::Space,
                        name_opt:None,
                        reference:Box::new(self.make_reference(l,p))
                    }
                }
                o => {
                    self.mouth_state = MouthState::M;
                    Token {
                        char,
                        catcode:o,
                        name_opt:None,
                        reference:Box::new(self.make_reference(l,p))
                    }
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
    fn peek(&mut self,catcodes:&CategoryCodeScheme) -> Token {
        let next = self.pop_next(catcodes,true);
        self.peekbuffer = Some(next.clone());
        next
    }
    fn preview(&mut self) -> String {
        todo!()
    }
    fn pushback(&mut self) {
        todo!()
    }
}

use crate::interpreter::files::VFile;
use crate::STORE_IN_FILE;

pub (in crate::interpreter) struct Mouths {
    mouths: Vec<Mouth>,
    buffer: Option<Token>
}

impl Mouths {
    pub fn new() -> Mouths {
        Mouths {
            mouths:Vec::new(),
            buffer:None
        }
    }
    pub(in crate::interpreter::mouth) fn has_next(&mut self,catcodes:&CategoryCodeScheme) -> bool {
        match self.buffer {
            Some(_) => true,
            _ => loop {
                match self.mouths.last_mut() {
                    None => return false,
                    Some(m) => {
                        if m.has_next(catcodes,true) {return true} else {
                            match self.mouths.pop().unwrap() {
                                Mouth::File(f) if self.mouths.is_empty() => {
                                    self.mouths.push(Mouth::File(f));
                                    return false
                                }
                                Mouth::File(fm) if STORE_IN_FILE => {
                                    let lastfile = self.mouths.iter_mut().rev().find(|x| match x {
                                        Mouth::File(_) => true,
                                        _ => false
                                    });
                                    match lastfile {
                                        Some(Mouth::File(nfm)) =>
                                            match nfm.source.borrow_mut() {
                                                StringMouthSource::File(f) => f.add(LaTeXObject::File(fm.source.pop_file().unwrap())),
                                                _ => panic!("This can't happen!")
                                            }
                                        _ => panic!("This shouldn't happen!")
                                    }
                                }
                                _ => {}
                            }
                        }
                    }
                }
            }
        }
    }

    pub(in crate::interpreter) fn next_token(&mut self,catcodes:&CategoryCodeScheme) -> Token {
        match self.buffer {
            Some(_) => self.buffer.take().unwrap(),
            _ => if self.has_next(catcodes) {
                self.mouths.last_mut().unwrap().get_next(catcodes)
            } else {
                panic!("Mouths empty!")
            }
        }
    }
    pub(in crate::interpreter) fn push_expansion(&mut self, exp : Expansion) {
        if !exp.exp.is_empty() {
            let nm = Mouth::Token(TokenMouth::new(exp,true));
            self.mouths.push(nm)
        }
    }
    pub(in crate::interpreter) fn push_tokens(&mut self, tks : Vec<Token>) {
        if !tks.is_empty() {
            let nm = Mouth::Token(TokenMouth::new(Expansion::dummy(tks),false));
            self.mouths.push(nm)
        }
    }
    /*
    pub(in crate::interpreter) fn push_file<'a>(&'a mut self,state : &'a State<'a>, file : VFile) {
        let fm = StringMouth::new_from_file(state,file);
        self.mouths.push(Mouth::File(fm))
    }

     */
    pub(in crate::interpreter::mouth) fn push_file(&mut self,catcodes:&CategoryCodeScheme,file:VFile) {
        self.mouths.push(Mouth::File(StringMouth::new_from_file(catcodes,file)))
    }

    pub(in crate::interpreter) fn requeue(&mut self, tk : Token) {
        self.buffer = Some(tk)
    }

    pub fn current_line(&self) -> String {
        match self.mouths.iter().rev().find(|m| match m {
            Mouth::File(_sm) => true,
            _ => false
        }) {
            Some(Mouth::File(m)) => {
                match &m.source {
                    StringMouthSource::File(lf) => {
                        lf.path.clone() + " (" + m.line.to_string().as_str() + ", " + m.pos.to_string().as_str() + ")"
                    }
                    _ => "".to_string()
                }
            }
            _ => "".to_string()
        }
    }
}

impl Interpreter<'_> {
    pub fn push_file(&self,file:VFile) {
        self.mouths.borrow_mut().push_file(&self.state_catcodes(),file)
    }
    pub fn push_expansion(&self,exp:Expansion) {
        self.mouths.borrow_mut().push_expansion(exp)
    }
    pub fn push_tokens(&self,tks:Vec<Token>) {
        self.mouths.borrow_mut().push_tokens(tks)
    }
    pub fn next_token(&self) -> Token {
        self.mouths.borrow_mut().next_token(&self.state_catcodes())
    }
    pub fn requeue(&self,token:Token) {
        self.mouths.borrow_mut().requeue(token)
    }
    pub fn has_next(&self) -> bool {
        self.mouths.borrow_mut().has_next(&self.state_catcodes())
    }
    /*
    pub fn push_file(&mut self,file : VFile) {
        let mut fm = StringMouth::new_from_file(&self.state,file);
        self.mouths.mouths.push(Mouth::File(fm))
    }

     */
}
