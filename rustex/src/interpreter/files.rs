use std::path::{PathBuf,Path};
use crate::utils::{TEXMF1,TEXMF2};
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