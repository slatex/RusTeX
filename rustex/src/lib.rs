pub mod ontology;
pub mod references;
pub mod interpreter;
pub mod utils;
pub mod commands;
pub mod catcodes;

#[macro_use]
extern crate lazy_static;

static STORE_IN_FILE : bool = true;
static COPY_TOKENS_FULL : bool = true;

static LANGUAGE_DAT : &'static str = include_str!("resources/language.dat");
static UNICODEDATA_TXT : &'static str = include_str!("resources/UnicodeData.txt");

#[cfg(test)]
mod tests {
    #[test]
    fn it_works() {
        let result = 2 + 2;
        assert_eq!(result, 4);
    }
}

pub struct VersionInfo {
    _texversion: String,
    _etexversion: String,
    _etexrevision: String,
    _pdftexversion: String,
    _pdftexrevision: String
}

impl VersionInfo {
    pub fn new(texversion : &str, etexversion : &str, etexrevision : &str, pdftexversion: &str, pdftexrevision: &str) -> VersionInfo {
        VersionInfo {
            _texversion:texversion.to_owned(),
            _etexversion:etexversion.to_owned(),
            _etexrevision:etexrevision.to_owned(),
            _pdftexversion:pdftexversion.to_owned(),
            _pdftexrevision:pdftexrevision.to_owned()
        }
    }
    pub fn texversion(&self) -> &str {
        &self._texversion
    }
    pub fn etexversion(&self) -> &str {
        &self._etexversion
    }
    pub fn etexrevision(&self) -> &str {
        &self._etexrevision
    }
    pub fn pdftexversion(&self) -> &str {
        &self._pdftexversion
    }
    pub fn pdftexrevision(&self) -> &str {
        &self._pdftexrevision
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
            VersionInfo::new("0","2",".6","140","22")
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
            let pdftexversion = &(pdftexversion1.to_owned() + pdftexversion2);
            pos = retstr.find(|x:char| !x.is_ascii_digit()).expect("TeX version string malformed");
            let pdftexrevision = &retstr[0..pos];

            VersionInfo::new(texversion,etexversion,etexrevision,pdftexversion,pdftexrevision)
        }
    };
}

static DEBUG : bool = true;
pub fn debug(s : String) {
    if DEBUG {println!("{}",s)}
}