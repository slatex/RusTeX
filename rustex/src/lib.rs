pub mod ontology;
pub mod references;
pub mod interpreter;
pub mod utils;
pub mod commands;
pub mod catcodes;
pub mod stomach;
pub mod fonts;
pub mod kpathsea;
//pub mod new_state;
//mod new_mouth;

pub static mut LOG : bool = false;
static STORE_IN_FILE : bool = false;
static COPY_TOKENS_FULL : bool = true;
static COPY_COMMANDS_FULL : bool = true;
static INSERT_RUSTEX_ATTRS: bool = false;

#[macro_export]
macro_rules! log {
    () => ();
    ($arg:tt) => ( if unsafe{crate::LOG} {println!($arg);} );
    ($head:tt,$($tl:expr),*) => ( if unsafe {crate::LOG} {
        //println!($head,$($tl),*);
        let retstr = std::format!("{}",std::format_args!($head,$($tl),*));
        println!("{} {}",ansi_term::Colour::Red.bold().paint("Log:"),retstr);
        //println!($head,$(ansi_term::Colour::Yellow.bold().paint($tl)),*);
    })
}



#[macro_export]
macro_rules! TeXErr {
    ($tk:expr => $head:tt$(,$tl:expr)*) => (
        return Err(crate::utils::TeXError::new(std::format!($head$(,$tl)*),Some($tk)))
    );
    ($head:tt$(,$tl:expr)*) => (
        return Err(crate::utils::TeXError::new(std::format!($head$(,$tl)*),None))
    )
}

#[macro_export]
macro_rules! FileEnd {
    () => (TeXErr!("File ended unexpectedly"))
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

use pdfium_render::prelude::*;

pub static mut PDFIUM_PATH : Option<String> = None;
static mut PDFIUM : Option<Pdfium> = None;
pub fn pdfium() -> Option<&'static Pdfium> {
    match unsafe{&PDFIUM} {
        Some(pdf) => Some(pdf),
        _ => unsafe {
            let mut lib = Pdfium::bind_to_system_library();
            match lib {
                Err(_) => {
                    let path = match &PDFIUM_PATH {
                        Some(s) => Pdfium::pdfium_platform_library_name_at_path(s),
                        _ => match std::env::current_exe() {
                            Ok(p) => Pdfium::pdfium_platform_library_name_at_path(p.parent().unwrap().to_str().unwrap().to_string() + "/"),
                            _ => Pdfium::pdfium_platform_library_name_at_path("./lib/")
                        }
                    };
                    lib = Pdfium::bind_to_library(path);
                }
                _ => ()
            }
            let libbind = match lib {
                Ok(ok) => ok,
                _ => return None
            };
            //let libbind = Pdfium::bind_to_statically_linked_library().unwrap();
            PDFIUM = Some(Pdfium::new(libbind));
            PDFIUM.as_ref()
        }
    }
}
pub fn pdf_to_img(path:&str) -> Option<image::DynamicImage> {
    match pdfium() {
        Some(pdfium) => {
            match pdfium.load_pdf_from_file(&path,None) {
                Ok(doc) => {
                    match doc.pages().iter().next().unwrap().get_bitmap_with_config(&PdfBitmapConfig::new()) {
                        Ok(mut bmp) => Some(bmp.as_image()),
                        _ => None
                    }
                }
                _ => None
            }
        }
        None => None
    }
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