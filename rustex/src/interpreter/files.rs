use std::path::Path;
use crate::utils::TeXStr;
use std::fs;
use crate::utils::TeXString;

#[derive(Clone,PartialEq)]
pub enum VFileBase {
    Real(TeXStr),
    Virtual
}

#[derive(Clone)]
pub struct VFile {
    pub source:VFileBase,
    pub(in crate) string: Arc<RwLock<Option<TeXString>>>,
    pub(in crate) id : TeXStr
}

extern crate pathdiff;

use std::collections::HashMap;


use std::sync::{Arc, RwLock};
use crate::{HYPHEN_CFG, /*PGFSYS_COMMON,*/ PGFSYS_RUST};

impl VFile {
    pub(in crate::interpreter) fn new<'a>(fp : &Path,intexmf:bool, in_file: &Path, filestore:&mut HashMap<TeXStr,Arc<VFile>>) -> Arc<VFile> {
        use crate::LANGUAGE_DAT;
        let simplename : TeXStr = (if intexmf {
            "<texmf>/".to_owned() + fp.file_name().expect("wut").to_ascii_uppercase().to_str().unwrap()
        } else if fp.to_str().unwrap().starts_with("nul:") {
            fp.to_str().unwrap().into()
        } else {
            pathdiff::diff_paths(fp,in_file).unwrap().to_str().unwrap().to_string()//.to_ascii_uppercase()
        }).as_str().into();
        let opt = filestore.get(&simplename);
        let vfile = match opt {
            Some(vf) => return vf.clone(),
            _ => {
                if simplename.to_string() == "<texmf>/LANGUAGE.DAT" {
                    VFile {
                        source:VFileBase::Virtual,
                        string:Arc::new(RwLock::new(Some(LANGUAGE_DAT.into()))),
                        id:simplename,
                    }
                } else if simplename.to_string() == "<texmf>/HYPHEN.CFG" {
                    VFile {
                        source:VFileBase::Virtual,
                        string:Arc::new(RwLock::new(Some(HYPHEN_CFG.into()))),
                        id:simplename
                    }
                } /*else if simplename.to_string() == "<texmf>/UNICODEDATA.TXT" {
                    unsafe {crate::LOG = true}
                    VFile {
                        source:VFileBase::Virtual,
                        string:Arc::new(RwLock::new(Some(UNICODEDATA_TXT.into()))),
                        id:simplename
                    }
                } */ else if simplename.to_string().contains("pgfsys-rust.def") {
                    VFile {
                        source:VFileBase::Virtual,
                        string:Arc::new(RwLock::new(Some(PGFSYS_RUST.into()))),
                        id:"<texmf>/PGFSYS-RUST.DEF".into()
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
                        string:Arc::new(RwLock::new(if fp.exists() {
                            fs::read(fp).ok().map(|x| x.into())
                        } else if simplename.to_string() == "nul:" {
                            Some("".into())
                        } else {None})),
                        id:simplename
                    }
                }
            }
        };
        let ret = Arc::new(vfile);
        filestore.insert(ret.id.clone(),ret.clone());
        ret
        /*
        int.state.as_mut().expect("Interpreter currently has no state!").files.entry(simplename.clone()).or_insert_with(||{

        })

         */
    }
}