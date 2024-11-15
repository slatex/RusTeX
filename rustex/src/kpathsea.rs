use std::collections::hash_map::Entry;
use std::collections::HashMap;
use std::path::{Path, PathBuf};

static mut KPATHSEA : Option<Kpathsea> = None;

pub static mut LOG : bool = false;

fn check(pb : &PathBuf) -> Option<PathBuf> {
    match pb.parent() {
        Some(p) if p.is_dir() => {
            for f in p.read_dir().unwrap() {
                let f = f.as_ref().unwrap();
                if f.path().is_file() && f.file_name().to_ascii_uppercase() == pb.file_name().unwrap().to_ascii_uppercase() { return Some( pb.clone() )}
                if f.path().is_file() && f.file_name().to_ascii_uppercase().to_str().unwrap() == pb.file_name().unwrap().to_ascii_uppercase().to_str().unwrap().to_string() + ".TEX" { return Some( p.join(pb.file_name().unwrap().to_str().unwrap().to_string() + ".tex") )}
            }
            None
        }
        _ => None
    }
}


pub fn kpsewhich(s : &str, indir : &Path) -> Option<(PathBuf,bool)> {
    if s.starts_with("nul:") && cfg!(target_os = "windows") {
        return Some((PathBuf::from(s),true))
    } else if s.starts_with("nul:") {
        return Some((indir.to_path_buf().join(s),true))
    } else if s.is_empty() {
        return None
    }
    /*if s.contains("macros") {
        println!("Here")
    }*/
    let default = indir.to_path_buf().join(s);
    match check(&default) {
        Some(p) => return Some((p,false)),
        _ => ()
    }
    let kpathsea = match unsafe { &KPATHSEA } {
        None => unsafe {
            KPATHSEA = Some(Kpathsea::init());
            KPATHSEA.as_ref().unwrap()
        }
        Some(k) => k
    };
    let split : Vec<&str> = s.split(".").collect();
    let (file,ext) = match split.len() {
        1 => (s.to_uppercase(),"".to_string()),
        2 => (split.first().unwrap().to_uppercase(),split.last().unwrap().to_uppercase()),
        _ => (
            split[..split.len()-2].iter().map(|x| x.to_string() + ".").collect::<String>().to_uppercase() + &split[split.len()-2].to_uppercase(),
              split.last().unwrap().to_uppercase())
    };
    if kpathsea.recdot {
        match recurse_dot(&file,&ext,indir) {
            Some((p,b)) => return Some((p,b)),
            _ => ()
        }
    }
    match kpathsea.map.get(&ext) {
        Some(m) => match m.get(&file) {
            Some(f) => Some((f.clone(),true)),
            _ => match kpathsea.map.get("TEX").unwrap().get( &(if ext.is_empty() {file} else {file + "." + &ext})) {
                Some(f) => Some((f.clone(),true)),
                _ => Some((default,false))
            }
        }
        _ => Some((default,false))
    }
}
fn recurse_dot(file : &str, ext:&str, indir : &Path) -> Option<(PathBuf,bool)> {
    //let mut dirs : Vec<PathBuf> = vec!();
    for entry in std::fs::read_dir(indir).unwrap() {
        let p = entry.unwrap().path();
        if p.is_dir() {
            match recurse_dot(file,ext,&p) {
                Some((p,b)) => return Some((p,b)),
                _ => ()
            }
        } else if p.is_file() {
            let f = p.file_name().unwrap().to_str().unwrap().to_uppercase();
            if !f.starts_with(file) {return None}
            if (ext != "" && f == file.to_string() + "." + ext) || (ext == "" && f == file) {
                return Some((p,true))
            }
            if ext == "" && f == file.to_string() + ".TEX" {
                return Some((p,true))
            }
        }
    }
    None
}

struct Kpathsea {
    recdot: bool,
    pub map : HashMap<String,HashMap<String,PathBuf>>
}
impl Kpathsea {
    pub fn init() -> Kpathsea {
        let loc = std::str::from_utf8(std::process::Command::new("kpsewhich")
            .args(vec!("-var-value","SELFAUTOLOC")).output().expect("kpsewhich not found!")
            .stdout.as_slice()).unwrap().trim().to_string();
        let rs : Vec<String> = std::str::from_utf8(std::process::Command::new("kpsewhich")
            .args(vec!("-a","texmf.cnf")).output().expect("kpsewhich not found!")
            .stdout.as_slice()).unwrap().split(|c| c=='\r' || c == '\n').map(|x| x.trim().to_string()).filter(|s| !s.is_empty()).collect();
        if rs.is_empty() && loc.contains("miktex") { Self::miktex(loc) } else {Self::texlive(rs,loc)}
    }

    fn texlive(rs: Vec<String>,loc:String) -> Kpathsea {
        let mut vars = HashMap::<String,String>::new();
        {
            let selfautoloc = PathBuf::from(loc);
            match selfautoloc.parent() {
                Some(p) => {
                    vars.insert("SELFAUTODIR".to_string(),p.to_str().unwrap().to_string());
                    match p.parent() {
                        Some(pp) => {
                            vars.insert("SELFAUTOPARENT".to_string(),pp.to_str().unwrap().to_string());
                            match pp.parent() {
                                Some(ppp) => {vars.insert("SELFAUTOGRANDPARENT".to_string(),ppp.to_str().unwrap().to_string());},
                                _ => {vars.insert("SELFAUTOGRANDPARENT".to_string(),pp.to_str().unwrap().to_string());}
                            }
                        }
                        None => {
                            vars.insert("SELFAUTOPARENT".to_string(),p.to_str().unwrap().to_string());
                            vars.insert("SELFAUTOGRANDPARENT".to_string(),p.to_str().unwrap().to_string());
                        }
                    }
                },
                _ => {
                    vars.insert("SELFAUTODIR".to_string(),selfautoloc.to_str().unwrap().to_string());
                    vars.insert("SELFAUTOPARENT".to_string(),selfautoloc.to_str().unwrap().to_string());
                    vars.insert("SELFAUTOGRANDPARENT".to_string(),selfautoloc.to_str().unwrap().to_string());
                }
            }
            vars.insert("SELFAUTOLOC".to_string(),selfautoloc.to_str().unwrap().to_string());
        }
        for r in rs {
            let p = PathBuf::from(r);
            if p.exists() {
                let lines : Vec<String> = std::str::from_utf8(std::fs::read(p).unwrap().as_slice()).unwrap().split(|c| c=='\r' || c == '\n').map(|x| x.trim().to_string()).collect();
                for l in lines {
                    if !l.starts_with("%") && !l.is_empty() {
                        let mut kb : Vec<String> = l.split("=").map(|x| x.trim().to_string()).collect();
                        if kb.len() == 2 {
                            let v = kb.pop().unwrap();
                            let k = kb.pop().unwrap();
                            match vars.entry(k) {
                                Entry::Occupied(_) => (),
                                Entry::Vacant(e) => { e.insert(v); }
                            }
                        }
                    }
                }
            }
        }
        let filestrs : Vec<String> = vec!(
            vars.get("VARTEXFONTS").map(|x| x.replace("\\","/")),
            vars.get("VFFONTS").map(|x| x.replace("\\","/")),
            vars.get("TFMFONTS").map(|x| x.replace("\\","/")),
            std::env::vars().find(|a| a.0 == "TEXINPUTS").map(|x| x.1.replace("\\","/")),
            vars.get("TEXINPUTS").map(|x| x.clone())
        ).into_iter().flatten().collect();
        vars.insert("progname".to_string(),"pdflatex".to_string());
        Self::finalize(filestrs,vars)
    }

    fn miktex(loc:String) -> Kpathsea {
        let pdftex_map = std::str::from_utf8(std::process::Command::new("kpsewhich")
            .args(vec!("pdftex.map")).output().expect("kpsewhich not found!")
            .stdout.as_slice()).unwrap().trim().to_string();
        let miktex = Path::new(&loc).parent().unwrap().parent().unwrap().parent().unwrap();
        let appdata = Path::new(&pdftex_map.split("MiKTeX").next().unwrap()).join("MiKTeX");
        let mut filestrs: Vec<String> = vec!(
            miktex.join("tex").join("generic").display().to_string().replace("\\","/") + "//",
            miktex.join("tex").join("latex").display().to_string().replace("\\","/") + "//",
            miktex.join("fonts").display().to_string().replace("\\","/") + "//",
            appdata.join("fonts").display().to_string().replace("\\","/") + "//"
        );
        if let Some((_,tip)) = std::env::vars().find(|a| a.0 == "TEXINPUTS") {
            filestrs.insert(0,tip.replace("\\","/"))
        }
        Self::finalize(filestrs,HashMap::default())
    }

    fn finalize(filestrs : Vec<String>,vars: HashMap::<String,String>) -> Kpathsea {
        unsafe {
            if LOG {
                for s in &filestrs {
                    println!("{s}");
                }
            }
        }
        let home = if cfg!(target_os = "windows") {
            std::env::vars().find(|x| x.0 == "HOMEDRIVE").unwrap().1 +
                &std::env::vars().find(|x| x.0 == "HOMEPATH").unwrap().1
        } else {
            std::env::vars().find(|x| x.0 == "HOME").unwrap().1
        };
        let mut recdot = false;
        let dirs : Vec<String> = filestrs.into_iter().map(|x| Kpathsea::parse_string(x,&vars)).flatten().collect();
        unsafe {
            if LOG {
                println!("--------------------------------------");
                for s in &dirs {
                    println!("{s}");
                }
            }
        }
        let mut paths : Vec<(PathBuf,bool)> = vec!();
        for mut d in dirs {
            if d.starts_with(".") {
                if d == ".//" {
                    recdot = true
                }
            }
            else {
                let mut recurse : bool = false;
                if d.starts_with("~") {
                    d = home.clone() + &d[1..]
                }
                if d.ends_with("//") {
                    d.pop();d.pop();
                    recurse = true
                }
                if !d.trim().is_empty() {
                    let pb = PathBuf::from(d.trim());
                    if pb.exists() && ! paths.contains(&(pb.clone(),recurse)) {
                        paths.push((pb,recurse))
                    }
                }
            }
        }
        let mut map : HashMap<String,HashMap<String,PathBuf>> = HashMap::new();
        for (path,recurse) in paths { Kpathsea::fill_map(&mut map,path,recurse) }
        Kpathsea { map,recdot }
    }


    fn fill_map(map: &mut HashMap<String,HashMap<String,PathBuf>>, path : PathBuf, recurse: bool) {
        for entry in std::fs::read_dir(path).unwrap() {
            let p = entry.unwrap().path();
            if p.is_dir() && recurse {
                Kpathsea::fill_map(map,p,recurse)
            } else {
                if p.is_file() {
                    let ext = match p.extension() {
                        Some(s) => s.to_ascii_uppercase().to_str().unwrap().to_string(),
                        _ => "".to_string()
                    };
                    let filename = match p.file_stem() {
                        Some(s) => s.to_ascii_uppercase().to_str().unwrap().to_string(),
                        _ => "".to_string()
                    };
                    match map.entry(ext) {
                        Entry::Occupied(mut v) => match v.get_mut().entry(filename) {
                            Entry::Vacant(v) => {v.insert(p);}
                            _ => ()
                        },
                        Entry::Vacant(v) => {
                            v.insert(HashMap::new()).insert(filename,p);
                        }
                    }
                }
            }
        }
    }
    fn parse_string(s : String,vars:&HashMap<String,String>) -> Vec<String> {
        let mut fin : Vec<String> = vec!();
        let mut ret : Vec<String> = vec!("".to_string());
        let mut i : usize = 0;
        let chars : Vec<char> = s.chars().collect();
        loop {
            match chars.get(i) {
                None => {
                    fin.append(&mut ret);
                    return fin
                },
                Some(';') => {
                    i += 1;
                    fin.append(&mut std::mem::take(&mut ret));
                    ret.push("".to_string())
                },
                Some(':') if !cfg!(target_os = "windows") || chars.get(i+1) != Some(&'/') => {
                    i += 1;
                    fin.append(&mut std::mem::take(&mut ret));
                    ret.push("".to_string())
                },
                Some('$') => {
                    i += 1;
                    let mut varname : String = "".to_string();
                    loop {
                        match chars.get(i) {
                            Some(c) if c.is_ascii_alphabetic() => {
                                i += 1;
                                varname.push(*c)
                            },
                            _ => break
                        }
                    }
                    match vars.get(&varname) {
                        None => panic!("unknown variable name"),
                        Some(s) => {
                            let rets = Kpathsea::parse_string(s.clone(),vars);
                            let nrets = std::mem::take(&mut ret);
                            for o in nrets { for r in &rets { ret.push(o.clone() + r) }}
                        }
                    }
                },
                Some('!') => i += 1,
                Some('{') => {
                    i += 1;
                    let mut rets : Vec<String> = vec!("".to_string());
                    let mut inbracks : u8 = 0;
                    loop {
                        match chars.get(i) {
                            Some(',') if inbracks == 0 => {
                                i += 1;
                                rets.push("".to_string())
                            },
                            Some('{') => {
                                i += 1;
                                inbracks += 1
                            },
                            Some('}') if inbracks == 0 => {
                                i += 1;
                                break
                            }
                            Some('}') => {
                                i += 1;
                                inbracks -= 1
                            }
                            Some(c) => {
                                i += 1;
                                rets.last_mut().unwrap().push(*c)
                            }
                            None => panic!("Syntax error in texmf.cnf")
                        }
                    }
                    let allrets : Vec<String> = rets.into_iter().map(|x| Kpathsea::parse_string(x,vars)).flatten().collect();
                    let nrets = std::mem::take(&mut ret);
                    for o in nrets { for r in &allrets { ret.push(o.clone() + r) }}
                },
                Some(c) => {
                    i += 1;
                    for s in &mut ret { s.push(*c) }
                }
            }
        }
    }
}