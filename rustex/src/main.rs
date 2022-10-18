use std::borrow::BorrowMut;
use std::io::Write;
use std::path::Path;
use std::thread;
use rustex::interpreter::Interpreter;
use rustex::interpreter::params::DefaultParams;
use rustex::stomach::html::HTMLColon;
use rustex::stomach::NoShipoutRoutine;

use clap::Parser;
use pdfium_render::prelude::PdfBitmapConfig;
//use rustex::imagemagick;

#[derive(Parser,Debug)]
#[clap(author, version, about, long_about = None)]
struct Parameters {
    /// Input file (tex)
    #[clap(short, long)]
    input: Option<String>,

    /// Input dir (tex)
    #[clap(short, long)]
    dir: Option<String>,

    /// Input string (tex)
    #[clap(short, long)]
    text: Option<String>,

    /// Output file (xhtml)
    #[clap(short, long)]
    output: Option<String>,

    /// use only one thread
    #[clap(short, long)]
    singlethreaded:bool

}
static mut SKIP : bool = false;
static SKIP_UNTIL : &str = "slides/exploiting-ci.en.tex"; //integernumbers.en.tex

fn main() {
    rustex::utils::with_stack_size(run)
}
fn run() {
    let params : Parameters = Parameters::parse();
    use rustex::interpreter::state::State;
    //use magick_rust::{MagickWand, magick_wand_genesis};

    match params.input {
        None => {
            match params.dir {
                None => {
                    println!("No file given. Testing latex.ltx...");
                    let state = State::pdf_latex();
                    state.commands.get(&"eTeXversion".into()).expect("");
                    println!("\n\nSuccess! \\o/")
                }
                Some(d) => {
                    let state = State::pdf_latex();
                    fn do_dir<P: AsRef<Path>>(s : P,mut st : State,out:Option<String>) {
                        for f in std::fs::read_dir(s).unwrap() {
                            let f = f.unwrap();
                            let path = f.path();
                            if std::fs::metadata(&path).unwrap().is_dir() {
                                do_dir(path,st.clone(),out.clone())
                            } else {
                                if path.to_str().unwrap().ends_with(".en.tex") {
                                    if unsafe{!SKIP} && path.to_str().unwrap().ends_with(SKIP_UNTIL) {
                                        unsafe {SKIP = true}
                                    };
                                    if unsafe{SKIP} {
                                        println!("------------\n\nDoing {}\n\n---------------\n", path.to_str().unwrap());
                                        let mut stomach = NoShipoutRoutine::new();
                                        let p = DefaultParams::new(false, false, None);
                                        let mut int = Interpreter::with_state(st.clone(), stomach.borrow_mut(), &p);
                                        let (success, s) = int.do_file(&path, HTMLColon::new(true));
                                        if success {
                                            let mut topcommands = Box::new(int.state.commands);
                                            loop {
                                                match topcommands.parent {
                                                    Some(p) => topcommands = p,
                                                    _ => break
                                                }
                                            }
                                            for (n,cmd) in topcommands.values.unwrap() {
                                                if n.to_string().starts_with("c_stex_module") {
                                                    st.commands.set(n,cmd,true);
                                                }
                                            }
                                        }
                                        match out {
                                            None => if success { println!("\n\nSuccess!\n{}", s) } else { println!("\n\nFailed\n{}", s) },
                                            Some(ref f) => {
                                                let mut file = std::fs::File::create(&f).unwrap();
                                                file.write_all(s.as_bytes()).expect("");
                                                if success {
                                                    println!("\n\nSuccess! \\o/\nResult written to {}", f)
                                                } else {
                                                    println!("\n\nFailed\nPartial result written to {}", f)
                                                }
                                            }
                                        }
                                    }
                                }
                            }
                        }
                    }
                    do_dir(d,state,params.output);
                }
            }
        }
        Some(i) => {
            let path = Path::new(&i);
            let mut stomach = NoShipoutRoutine::new();
            let p = DefaultParams::new(false,params.singlethreaded,None);
            let state = State::pdf_latex();
            let mut int = Interpreter::with_state(state,stomach.borrow_mut(),&p);
            let (success,s) = match params.text {
                Some(s) =>
                    int.do_string(path,s.as_str(),HTMLColon::new(true)),
                None => {
                    if !path.exists() {
                        println!("File {} not found", i)
                    }
                    int.do_file(path, HTMLColon::new(true))
                }
            };
            match params.output {
                None => if success {println!("\n\nSuccess!\n{}",s)} else {println!("\n\nFailed\n{}",s)},
                Some(f) => {
                    let mut file = std::fs::File::create(&f).unwrap();
                    file.write_all(s.as_bytes()).expect("");
                    if success {
                        println!("\n\nSuccess! \\o/\nResult written to {}", f)
                    } else {
                        println!("\n\nFailed\nPartial result written to {}",f)
                    }
                }
            }

        }
    }
}