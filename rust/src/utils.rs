use std::path::PathBuf;

#[derive(Clone)]
pub struct FilePath {
    _path: String,
    _pb : PathBuf
}
impl FilePath {
    pub fn new(s : &str) -> FilePath {
        FilePath {
            _path: s.to_owned(),
            _pb : PathBuf::from(s)
        }
    }
    pub fn from_path(pb : PathBuf) -> FilePath {
        FilePath {
            _path: pb.to_str().expect("Can't happen").to_owned(),
            _pb: pb
        }
    }
    pub fn path(&self) -> &str {
        self._path.as_str()
    }
}

pub fn kpsewhich(s : &str, indir : &FilePath) -> Option<FilePath> {
    use std::process::Command;
    use std::{str,env};
    if s.starts_with("nul:") && cfg!(target_os = "windows") {
        Some(FilePath::new(s))
    } else if s.is_empty() {
        None
    } else {
        env::set_current_dir(indir.path()).expect("Could not switch to directory");
        let rs : Vec<u8> = Command::new("kpsewhich")
            .arg(s).output().expect("kpsewhich not found!")
            .stdout;
        match str::from_utf8(rs.as_slice()) {
            Ok(v) => Some(FilePath::new(v)),
            Err(_) => panic!("")
        }
    }
}