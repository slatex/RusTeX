use std::fmt::{Debug, Display, Formatter};
use std::ops::Deref;
use std::path::{Path, PathBuf};

lazy_static! {
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
    use std::process::Command;
    use std::{str,env};
    if s.starts_with("nul:") && cfg!(target_os = "windows") {
        Some(PathBuf::from(s))
    } else if s.is_empty() {
        None
    } else {
        env::set_current_dir(indir).expect("Could not switch to directory");
        let rs : Vec<u8> = Command::new("kpsewhich")
            .arg(s).output().expect("kpsewhich not found!")
            .stdout;
        match str::from_utf8(rs.as_slice()) {
            Ok(v) => Some(PathBuf::from(v.trim_end()).canonicalize().unwrap()),
            Err(_) => panic!("utils.rs 34")
        }
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
        let mut bt = Backtrace::new_unresolved();
        let mut frames = Vec::new();
        for b in bt.frames() {
            frames.push(b.clone())
        }
        frames.remove(0);
        frames.remove(0);
        Backtrace::from(frames)
    }
    pub fn new(msg:String) -> TeXError {
        TeXError {msg,source:Box::new(None),backtrace:TeXError::backtrace()}
    }
    pub fn derive(self,msg:String) -> TeXError {
        TeXError {msg,source:Box::new(Some(self)),backtrace:TeXError::backtrace()}
    }
    pub fn throw<A>(mut self) -> A {
        self.backtrace.resolve();
        panic!("{}",self)
    }
    pub fn print(&mut self) {
        self.backtrace.resolve();
        println!("{}",self)
    }
}

impl Debug for TeXError {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(f,"Debug: {}",self.msg);
        self.backtrace.fmt(f)
    }
}

impl Display for TeXError {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(f,"{}\n",self.msg);
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