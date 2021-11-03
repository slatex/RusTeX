use std::path::{Path,PathBuf};


use std::env;

lazy_static! {
    pub static ref PWD : PathBuf = env::current_dir().expect("No current directory!")
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
            Ok(v) => Some(PathBuf::from(v.trim_end())),
            Err(_) => panic!("utils.rs 34")
        }
    }
}