use std::borrow::BorrowMut;
use std::path::{PathBuf, Path};
use crate::utils::{TeXError, TEXMF1, TEXMF2, TeXStr};
use std::fs;
use crate::utils::TeXString;

#[derive(Clone)]
pub enum VFileBase {
    Real(PathBuf),
    Virtual
}

#[derive(Clone)]
pub struct VFile {
    pub source:VFileBase,
    pub(in crate) string: Option<TeXString>,
    pub(in crate::interpreter) id : String
}

extern crate pathdiff;

use std::collections::HashMap;

pub(in crate::interpreter) struct FileStore {
    pub files: HashMap<String,VFile>
}

use std::cell::RefMut;

impl VFile {
    pub(in crate::interpreter) fn new<'a>(fp : &Path, in_file: &Path, filestore:&mut RefMut<FileStore>) -> VFile {
        use crate::{LANGUAGE_DAT,UNICODEDATA_TXT};
        let simplename = if fp.starts_with(TEXMF1.as_path()) || fp.starts_with(TEXMF2.as_path()) {
            "<texmf>/".to_owned() + fp.file_name().expect("wut").to_ascii_uppercase().to_str().unwrap()
        } else {
            pathdiff::diff_paths(fp,in_file).unwrap().to_str().unwrap().to_ascii_uppercase()
        };
        let opt = filestore.files.remove(simplename.as_str());
        match opt {
            Some(vf) => vf,
            _ => {
                if simplename == "<texmf>/LANGUAGE.DAT" {
                    VFile {
                        source:VFileBase::Virtual,
                        string:Some(LANGUAGE_DAT.into()),
                        id:simplename,
                    }
                } else if simplename == "<texmf>/UNICODEDATA.TXT" {
                    VFile {
                        source:VFileBase::Virtual,
                        string:Some(UNICODEDATA_TXT.into()),
                        id:simplename
                    }
                } else {
                    VFile {
                        source:VFileBase::Real(fp.to_path_buf()),
                        string:if fp.exists() {fs::read(fp).ok().map(|x| x.into())} else {Some("".into())},
                        id:simplename
                    }
                }
            }
        }

        /*
        int.state.as_mut().expect("Interpreter currently has no state!").files.entry(simplename.clone()).or_insert_with(||{

        })

         */
    }
}
struct FInfoEntry {
    char: u8,
    width_index: u8,
    height_index: u8,
    depth_index: u8,
    char_ic_index: u8,
    tag_field: u8,
    remainder: u8
}
impl FInfoEntry {
    pub fn ligature(&self) -> Option<u8> {
        match self.tag_field {
            1 => Some(self.remainder),
            _ => None
        }
    }
    pub fn larger(&self) -> Option<u8> {
        match self.tag_field {
            2 => Some(self.remainder),
            _ => None
        }
    }
    pub fn ext(&self) -> Option<u8> {
        match self.tag_field {
            3 => Some(self.remainder),
            _ => None
        }
    }
}

pub struct FontFile {
    hyphenchar:u8,
    skewchar:u8,
    dimen:HashMap<u8,i32>,
    size : Option<f32>,
    types : Vec<TeXStr>,
    widths:HashMap<u8,i32>,
    heights: HashMap<u8,i32>,
    depths: HashMap<u8,i32>,
    ics:HashMap<u8,i32>,
    lps:HashMap<u8,u8>,
    rps: HashMap<u8,u8>,
    ligs:HashMap<(u8,u8),u8>

}
struct FontState {
    pub ret : Vec<u8>,
    pub i : u8
}
impl FontState {
    pub fn pop(&mut self) -> (u8,u8,u8,u8) {
        self.i += 1;
        (self.ret.pop().unwrap(),self.ret.pop().unwrap(),self.ret.pop().unwrap(),self.ret.pop().unwrap())
    }
}
impl FontFile {
    pub fn new(pb : PathBuf) -> FontFile {
        let mut state = FontState {
            ret:fs::read(pb).unwrap(),
            i:0
        };
        state.ret.reverse();

        let mut hyphenchar : u8 = 45;
        let mut skewchar : u8 = 255;
        let mut dimen: HashMap<u8,i32> = HashMap::new();
        let mut size: Option<f32> = None;
        let mut types: Vec<TeXStr> = Vec::new();
        let mut widths:HashMap<u8,i32> = HashMap::new();
        let mut heights: HashMap<u8,i32> = HashMap::new();
        let mut depths: HashMap<u8,i32> = HashMap::new();
        let mut ics:HashMap<u8,i32> = HashMap::new();
        let mut lps:HashMap<u8,u8> = HashMap::new();
        let mut rps: HashMap<u8,u8> = HashMap::new();
        let mut ligs:HashMap<(u8,u8),u8> = HashMap::new();

        fn read_int(s : &mut FontState) -> (u16,u16) {
            let (a,b,c,d) = s.pop();
            let i1 = ((a as u16) << 8) | (b as u16);
            let i2 = ((c as u16) << 8) | (d as u16);
            (i1,i2)
        };
        fn read_float(s : &mut FontState) -> f32 {
            let (a,b,c,d) = s.pop();
            let int = ((a as i32) << 24) | ((b as i32) << 16) |
                ((c as i32) << 8) | (d as i32);
            let f = ((int & 0x7fffffff) as f32) / ((1 << 20) as f32);
            if int < 0 {-f} else {f}
        };
        fn skip(s : &mut FontState, len:u8) {
            for _ in 0..len {
                s.pop();
            }
        };
        fn read_fifo(s : &mut FontState,char:u8) -> FInfoEntry {
            let (a,b,c,d) = s.pop();
            let width_index = 0x000000FF & a;
            let (height_index,depth_index) = {
                let byte = (0x000000FF & b);
                let second = byte % 16;
                let first = (byte - second) / 16;
                (first,second)
            };
            let (char_ic_index,tag_field) = {
                let full = 0x000000FF & c;
                let second = full % 4;
                let first = (full - second) / 4;
                (first,second)
            };
            let remainder = 0x000000FF & d;

            FInfoEntry {
                char,width_index,height_index,depth_index,char_ic_index,tag_field,remainder
            }
        };

        let (lf,lh) = read_int(state.borrow_mut());
        let (bc,ec) = read_int(state.borrow_mut());
        let (nw,nh) = read_int(state.borrow_mut());
        let (nd,ni) = read_int(state.borrow_mut());
        let (nl,nk) = read_int(state.borrow_mut());
        let (ne,np) = read_int(state.borrow_mut());
        assert_eq!(lf,6+lh+(ec-bc+1)+nw+nh+nd+ni+nk+nl+ne+np);

        todo!();
        FontFile {
            hyphenchar,skewchar,dimen,size,types,widths,heights,depths,ics,lps,rps,ligs
        }
    }
}