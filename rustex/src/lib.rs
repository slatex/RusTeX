pub mod ontology;
pub mod references;
pub mod interpreter;
pub mod utils;
pub mod commands;
pub mod catcodes;
pub mod stomach;
mod fonts;

static SINGLETHREADED : bool = true;
static PGF_AS_SVG : bool = true;
pub static mut LOG : bool = false;
static STORE_IN_FILE : bool = false;
static COPY_TOKENS_FULL : bool = true;
static COPY_COMMANDS_FULL : bool = true;
pub static LOG_FILE : &str = "/home/jazzpirate/rustex.log.xhtml";

#[macro_export]
macro_rules! log {
    () => ();
    ($arg:tt) => (unsafe{ if crate::LOG {println!($arg)} });
    ($head:tt,$($tl:expr),*) => (unsafe { if crate::LOG {
        //println!($head,$($tl),*);
        let retstr = std::format!("{}",std::format_args!($head,$($tl),*));
        println!("{} {}",ansi_term::Colour::Red.bold().paint("Log:"),retstr);
        //println!($head,$(ansi_term::Colour::Yellow.bold().paint($tl)),*);
    }})
}

fn tex_stacktrace(int:&Interpreter,tk:Option<Token>) -> String {
    match tk {
        None if int.has_next() => tex_stacktrace(int,Some(int.next_token())),
        None => "(No tracing information available)".to_string(),
        Some(tk) => {
            let catcodes = int.state_catcodes().clone();
            crate::utils::stacktrace(tk,int,&catcodes)
        }
    }
}


#[macro_export]
macro_rules! TeXErr {
    (($int:tt,$tk:expr),$head:tt) => (return Err(crate::utils::TeXError::new(std::format!("{} in: {}:\n>>{}\n\n{}",$head,crate::interpreter::Interpreter::current_line($int),
        crate::interpreter::Interpreter::preview($int),crate::tex_stacktrace($int,$tk)))));
    (($int:tt,$tk:expr),$head:tt,$($tl:expr),*) => ({
        //println!($head,$($tl),*);
        let retstr = std::format!("{} in: {}:\n>>{}\n\n{}",std::format_args!($head,$($tl),*),crate::interpreter::Interpreter::current_line($int),
            crate::interpreter::Interpreter::preview($int),crate::tex_stacktrace($int,$tk));
        return Err(crate::utils::TeXError::new(retstr))
        //println!($head,$(ansi_term::Colour::Yellow.bold().paint($tl)),*);
    })
}

#[macro_export]
macro_rules! FileEnd {
    ($int:tt) => (TeXErr!(($int,None),"File ended unexpectedly"))
}

#[macro_use]
extern crate lazy_static;

use crate::interpreter::Interpreter;
use crate::ontology::Token;
use crate::utils::TeXString;


pub static LANGUAGE_DAT : &str = include_str!("resources/language.dat");
//pub static UNICODEDATA_TXT : &str = include_str!("resources/UnicodeData.txt");
pub static HYPHEN_CFG : &str = include_str!("resources/hyphen.cfg");
pub static PGFSYS_RUST: &str = include_str!("resources/pgfsys-rust.def");

#[cfg(test)]
mod tests {
    #[test]
    fn it_works() {
        let result = 2 + 2;
        assert_eq!(result, 4);
    }
}

pub struct VersionInfo {
    pub texversion: TeXString,
    pub etexversion: TeXString,
    pub etexrevision: TeXString,
    pub pdftexversion: TeXString,
    pub pdftexrevision: TeXString
}


lazy_static! {
    pub static ref VERSION_INFO : VersionInfo = {
        use std::process::Command;
        use std::str;

        let rs : Vec<u8> = Command::new("pdftex")
            .arg("--version").output().expect("pdftex not found!")
            .stdout;
        let mut retstr = match str::from_utf8(rs.as_slice()) {
            Ok(v) => v,
            Err(_) => panic!("utils.rs 68")
        };
        if retstr.starts_with("MiKTeX") {
            // TODO better
            VersionInfo{texversion:"0".into(),etexversion:"2".into(),
                etexrevision:".6".into(),pdftexversion:"140".into(),
                pdftexrevision:"22".into()
            }
        } else {
            retstr = retstr.strip_prefix("pdfTeX ").expect("Unknown TeX engine");
            let mut pos = retstr.find("-").expect("TeX version string malformed");
            let texversion = &retstr[0..pos];
            retstr = &retstr[pos+1..];
            pos = retstr.find(".").expect("TeX version string malformed");
            let etexversion = &retstr[0..pos];
            retstr = &retstr[pos..];
            pos = retstr.find("-").expect("TeX version string malformed");
            let etexrevision = &retstr[0..pos];
            retstr = &retstr[pos+1..];
            pos = retstr.find(".").expect("TeX version string malformed");
            let pdftexversion1 = &retstr[0..pos];
            retstr = &retstr[pos+1..];
            pos = retstr.find(".").expect("TeX version string malformed");
            let pdftexversion2 = &retstr[0..pos];
            retstr = &retstr[pos+1..];
            let pdftexversion = pdftexversion1.to_owned() + pdftexversion2;
            pos = retstr.find(|x:char| !x.is_ascii_digit()).expect("TeX version string malformed");
            let pdftexrevision = &retstr[0..pos];

            VersionInfo{
                texversion:texversion.into(),
                etexversion:etexversion.into(),
                etexrevision:etexrevision.into(),
                pdftexversion:pdftexversion.into(),
                pdftexrevision:pdftexrevision.into()}
        }
    };
}

static DEBUG : bool = true;
pub fn debug(s : String) {
    if DEBUG {println!("{}",s)}
}