use std::path::{PathBuf, Path};
use crate::utils::{TEXMF1, TEXMF2, TeXStr};
use std::fs;
use crate::utils::TeXString;

#[derive(Clone)]
pub enum VFileBase {
    Real(TeXStr),
    Virtual
}

#[derive(Clone)]
pub struct VFile {
    pub source:VFileBase,
    pub(in crate) string: Option<TeXString>,
    pub(in crate) id : TeXStr
}

extern crate pathdiff;

use std::collections::HashMap;

pub(in crate::interpreter) struct FileStore {
    pub files: HashMap<TeXStr,VFile>
}

use std::cell::RefMut;
use crate::HYPHEN_CFG;

impl VFile {
    pub(in crate::interpreter) fn new<'a>(fp : &Path, in_file: &Path, filestore:&mut RefMut<FileStore>) -> VFile {
        use crate::{LANGUAGE_DAT,UNICODEDATA_TXT};
        let simplename : TeXStr = (if fp.starts_with(TEXMF1.as_path()) || fp.starts_with(TEXMF2.as_path()) {
            "<texmf>/".to_owned() + fp.file_name().expect("wut").to_ascii_uppercase().to_str().unwrap()
        } else {
            pathdiff::diff_paths(fp,in_file).unwrap().to_str().unwrap().to_ascii_uppercase()
        }).as_str().into();
        let opt = filestore.files.remove(&simplename);
        match opt {
            Some(vf) => vf,
            _ => {
                if simplename.to_string() == "<texmf>/LANGUAGE.DAT" {
                    VFile {
                        source:VFileBase::Virtual,
                        string:Some(LANGUAGE_DAT.into()),
                        id:simplename,
                    }
                } else if simplename.to_string() == "<texmf>/HYPHEN.CFG" {
                    VFile {
                        source:VFileBase::Virtual,
                        string:Some(HYPHEN_CFG.into()),
                        id:simplename
                    }
                } /* else if simplename.to_string() == "<texmf>/UNICODEDATA.TXT" {
                    VFile {
                        source:VFileBase::Virtual,
                        string:Some(UNICODEDATA_TXT.into()),
                        id:simplename
                    }
                } */ else {
                    VFile {
                        source:VFileBase::Real(fp.to_str().unwrap().into()),
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