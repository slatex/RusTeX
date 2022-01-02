use std::fmt::{Debug, Display, Formatter};
use std::hash::{Hash, Hasher};
use std::io::Write;
use std::ops::{AddAssign, Deref};
use std::path::{Path, PathBuf};
use std::str::{from_utf8, from_utf8_unchecked};
use std::rc::Rc;
use std::sync::Arc;

pub fn u8toi16(i : u8) -> i16 {
    i16::from(i)
}

#[derive(Clone)]
pub struct TeXStr(Arc<Vec<u8>>);
impl PartialEq for TeXStr {
    fn eq(&self, other: &Self) -> bool {
        *self.0.deref() == *other.0.deref()
    }
}
impl Eq for TeXStr {}
impl Hash for TeXStr {
    fn hash<H: Hasher>(&self, state: &mut H) {
        (*self.0.deref()).hash(state)
    }
}
impl TeXStr {
    pub fn to_string(&self) -> String {
        display(self.iter())
    }
    pub fn to_utf8(&self) -> String {
        from_utf8(self.iter()).unwrap().to_string()
    }
    pub fn iter(&self) -> &[u8] {
        self.0.deref()
    }
    pub fn new(chars: &[u8]) -> TeXStr {
        TeXStr(Arc::new(chars.to_vec()))
    }
    pub fn len(&self) -> usize {
        self.0.deref().len()
    }
}
impl From<&str> for TeXStr {
    fn from(s: &str) -> Self {
        TeXStr(Arc::new(s.as_bytes().to_vec()))
    }
}
impl From<TeXString> for TeXStr {
    fn from(ts: TeXString) -> Self {
        TeXStr(Arc::new(ts.0))
    }
}
impl From<String> for TeXStr {
    fn from(s: String) -> Self {
        TeXStr(Arc::new(s.as_bytes().to_vec()))
    }
}
impl PartialEq<str> for TeXStr {
    fn eq(&self, other: &str) -> bool {
        self.0.deref() == other.as_bytes()
    }
}

impl Display for TeXStr {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(f,"{}",display(self.0.deref()))
    }
}

fn is_ascii(u:&u8) -> bool {
    32 <= *u && *u <= 126
}

fn display(us:&[u8]) -> String {
    let mut ret : Vec<u8> = vec!();
    for u in us { match u {
        0 => for x in "\\u0000".as_bytes() {
            ret.push(*x)
        }
        13 => ret.push(10),
        _ if is_ascii(u) => {
            ret.push(*u)
        }
        _ => {
            for x in ("\\u00".to_string() + &format!("{:X}", u)).as_bytes() {
                ret.push(*x)
            }
        }
    }}
    unsafe { from_utf8_unchecked(&ret).to_string() }
}

#[derive(Clone,PartialEq,Eq,Hash)]
pub struct TeXString(pub Vec<u8>);
impl TeXString {
    pub fn is_empty(&self) -> bool {
        self.0.is_empty()
    }
    pub fn to_string(&self) -> String {
        display(self.0.as_slice())
    }
    pub fn to_utf8(&self) -> String {
        from_utf8(&self.0).unwrap().to_string()
    }
    pub fn len(&self) -> usize {
        self.0.len()
    }
    pub fn split(&self,u:u8) -> Vec<TeXString> {
        self.0.split(|x| *x == u).map(|x| x.into()).collect()
    }
}

impl Display for TeXString {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(f,"{}",display(self.0.as_slice()))
    }
}
impl From<Vec<u8>> for TeXString {
    fn from(v: Vec<u8>) -> Self {
        TeXString(v)
    }
}
impl From<&str> for TeXString {
    fn from(s: &str) -> Self {
        TeXString(s.as_bytes().to_vec())
    }
}
impl From<String> for TeXString {
    fn from(s: String) -> Self {
        TeXString(s.as_bytes().to_vec())
    }
}
impl From<u8> for TeXString {
    fn from(u: u8) -> Self {
        TeXString(vec!(u))
    }
}
impl From<&[u8]> for TeXString {
    fn from(s: &[u8]) -> Self {
        TeXString(s.to_vec())
    }
}

impl From<TeXStr> for TeXString {
    fn from(s: TeXStr) -> Self {
        TeXString(s.0.deref().to_vec())
    }
}

impl From<&TeXStr> for TeXString {
    fn from(s: &TeXStr) -> Self {
        TeXString(s.0.deref().to_vec())
    }
}
impl std::ops::Add for TeXString {
    type Output = TeXString;
    fn add(self, rhs: Self) -> Self::Output {
        let mut new : Vec<u8> = self.0.clone();
        for u in rhs.0 {
            new.push(u)
        }
        TeXString(new)
    }
}

impl PartialEq<str> for TeXString {
    fn eq(&self, other: &str) -> bool {
        self.0 == other.as_bytes()
    }
}
impl AddAssign for TeXString {
    fn add_assign(&mut self, rhs: Self) {
        for u in rhs.0 {
            self.0.push(u)
        }
    }
}

impl AddAssign<&TeXStr> for TeXString {
    fn add_assign(&mut self, rhs: &TeXStr) {
        for u in rhs.0.deref() {
            self.0.push(*u)
        }
    }
}

impl AddAssign<&str> for TeXString {
    fn add_assign(&mut self, rhs: &str) {
        for u in rhs.as_bytes() {
            self.0.push(*u)
        }
    }
}

impl AddAssign<u8> for TeXString {
    fn add_assign(&mut self, rhs: u8) {
        self.0.push(rhs)
    }
}

impl AddAssign<String> for TeXString {
    fn add_assign(&mut self, rhs: String) {
        for u in rhs.as_bytes() {
            self.0.push(*u)
        }
    }
}

use kpathsea::Kpaths;
thread_local! {
    pub static kpaths : Kpaths = kpathsea::Kpaths::new().unwrap();
}

lazy_static! {
    //pub static ref kpaths : Kpaths = kpathsea::Kpaths::new().unwrap();
    pub static ref PWD : PathBuf = std::env::current_dir().expect("No current directory!")
        .as_path().to_path_buf();
    pub static ref TEXMF1 : PathBuf = kpsewhich("article.sty",&PWD).expect("article.sty not found")
        .as_path().parent().unwrap().parent().unwrap().parent().unwrap().parent().unwrap().to_path_buf();//.up().up().up().up();
    pub static ref TEXMF2 : PathBuf = kpsewhich("pdftexconfig.tex",&PWD).expect("pdftexconfig.tex not found")
        .as_path().parent().unwrap().parent().unwrap().parent().unwrap().parent().unwrap().to_path_buf();
    /*
    kpsewhich("article.sty").getOrElse(
    error("article.sty not found - do you have LaTeX installed?", None)
  ).up.up.up.up :: kpsewhich("pdftexconfig.tex").getOrElse{???}.up.up.up.up :: Nil
     */
}

pub fn kpsewhich(s : &str, indir : &Path) -> Option<PathBuf> {
    //use std::process::Command;
    //use std::{str,env};
    use std::env;
    if s.starts_with("nul:") && cfg!(target_os = "windows") {
        Some(PathBuf::from(s))
    } else if s.starts_with("nul:") {
        Some(indir.to_path_buf().join(s))
    } else if s.is_empty() {
        None
    } else {
        env::set_current_dir(indir).expect("Could not switch to directory");
        let ret = match kpaths.try_with(|x| x.find_file(s)).unwrap() {
            Some(v) =>
                PathBuf::from(v.trim_end()).canonicalize().unwrap_or_else(|_| indir.to_path_buf().join(s)),
            None => indir.to_path_buf().join(s)
        };
        Some(ret)
        /*
        let rs : Vec<u8> = Command::new("kpsewhich")
            .arg(s).output().expect("kpsewhich not found!")
            .stdout;
        match str::from_utf8(rs.as_slice()) {
            Ok(v) => Some(PathBuf::from(v.trim_end()).canonicalize().unwrap_or_else(|_| indir.to_path_buf().join(s))),
            Err(_) => panic!("utils.rs 34")
        }
         */
    }
}

pub fn with_encoded_pointer<'a,S,T>(obj:&'a T,f: fn(i:i64) -> S) -> S {
    let i = encode_pointer(obj);
    f(i)
}

pub fn with_encoded_pointer_mut<'a,S,T>(obj:&'a mut T,f: fn(i:i64) -> S) -> S {
    let i = encode_pointer_mut(obj);
    f(i)
}

pub fn encode_pointer<'a,T>(obj:&'a T) -> i64 {
    let bx = Box::new(obj);
    unsafe { std::mem::transmute::<Box<&T>,*mut u8>(bx) as i64 }
}

pub fn decode_pointer<'a,T>(i:i64) -> &'a T {
    unsafe {
        let bx: Box<&T> = std::mem::transmute(i as *mut u8);
        *bx
    }
}

pub fn encode_pointer_mut<'a,T>(obj:&'a mut T) -> i64 {
    let bx = Box::new(obj);
    unsafe { std::mem::transmute::<Box<&mut T>,*mut u8>(bx) as i64 }
}
pub fn decode_pointer_mut<'a,T>(i:i64) -> &'a mut T {
    unsafe {
        let bx: Box<&mut T> = std::mem::transmute(i as *mut u8);
        *bx
    }
}

use backtrace::Backtrace;

pub struct TeXError {
    msg:String,
    source:Box<Option<TeXError>>,
    backtrace : Backtrace
}

impl TeXError {
    fn backtrace() -> Backtrace {
        let bt = Backtrace::new_unresolved();
        let mut frames = Vec::new();
        for b in bt.frames() {
            frames.push(b.clone())
        }
        frames = Vec::from(frames.get(2..).unwrap());
        Backtrace::from(frames)
    }
    pub (in crate) fn new(msg:String) -> TeXError {
        TeXError {msg,source:Box::new(None),backtrace:TeXError::backtrace()}
    }
    pub fn derive(self,msg:String) -> TeXError {
        TeXError {msg,source:Box::new(Some(self)),backtrace:TeXError::backtrace()}
    }
    pub fn throw<A>(mut self, int: Option<&Interpreter>) -> A {
        std::io::stdout().flush();
        use std::path::PathBuf;
        self.backtrace.resolve();
        match int {
            None => panic!("{}",self),
            Some(i) => match i.stomach.borrow_mut().final_xml(i) {
                Ok(s) => {
                    let mut file = std::fs::File::create(crate::LOG_FILE).unwrap();
                    file.write_all(s.as_bytes());
                    panic!("{}\n\nLOG FILE WRITTEN\n\n",self)
                },
                Err(e) => e.throw(None)
            }
        }
    }
    pub fn print(&mut self) {
        std::io::stdout().flush();
        self.backtrace.resolve();
        println!("{}",self)
    }
}

impl Debug for TeXError {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(f,"Debug: {}",self.msg)?;
        self.backtrace.fmt(f)
    }
}

impl Display for TeXError {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(f,"{}\n",self.msg)?;
        self.backtrace.fmt(f)
    }
}
impl std::error::Error for TeXError {
    fn source(&self) -> Option<&(dyn std::error::Error + 'static)> {
        match self.source.deref() {
            Some(e) => Some(e),
            None => None
        }
    }
}

use crate::references::SourceReference;
use crate::ontology::ExpansionRef;
use crate::catcodes::{CategoryCode, CategoryCodeScheme};
use crate::ontology::Token;
use crate::commands::TeXCommand;
use crate::interpreter::Interpreter;

fn get_top(tk : Token) -> Token {
    let mut t = tk;
    loop {
        match &*t.reference {
            SourceReference::File(_,_,_) => return t,
            SourceReference::None => return t,
            SourceReference::Exp(ExpansionRef(nt,_)) => t = nt.clone()
        }
    }
}

pub fn stacktrace<'a>(tk : Token,int:&Interpreter,catcodes:&CategoryCodeScheme) -> String {
    (match tk.catcode {
        CategoryCode::Escape => "\\".to_string() + &tk.name().to_string(),
        _ => TeXString(vec!(tk.char)).to_string()
    }) + " - " +
    &match &*tk.reference {
        SourceReference::File(str,(sl,sp),(el,ep)) =>
            str.to_string() + " (" + &sl.to_string() + "," + &sp.to_string() + ") - (" + &el.to_string() + "," + &ep.to_string() + ")\n",
        SourceReference::None => "".to_string(),
        SourceReference::Exp(ExpansionRef(tk,cmd)) =>
            {
                let next = stacktrace(tk.clone(), int,catcodes);
                "Expanded from ".to_string() + &match tk.catcode {
                    CategoryCode::Escape => {
                        let cmd = int.state_get_command(&tk.cmdname());
                        "\\".to_string() + &tk.name().to_string() + " " + &match cmd {
                            Some(o) => o.meaning(&catcodes).to_string(),
                            _ => "".to_string()
                        }
                    },
                    _ => TeXString(vec!(tk.char)).to_string()
                } + " defined by " + &match &cmd.rf {
                    None => cmd.name().unwrap().to_string() + "\n",
                    Some(rf) => {
                        let st = stacktrace(get_top(rf.0.clone()), int,catcodes);
                        " at ".to_string() + &st
                    }
                } + &next
            }
    }
}