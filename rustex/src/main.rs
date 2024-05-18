use std::borrow::BorrowMut;
use std::collections::HashMap;
use std::env;
use std::fmt::Display;
use std::io::Write;
use std::ops::Deref;
use std::path::Path;
use rustex::interpreter::Interpreter;
use rustex::interpreter::params::{DefaultParams, NoOutput};
use rustex::stomach::html::HTMLColon;
use rustex::stomach::NoShipoutRoutine;

use clap::Parser;
use rustex::commands::pdftex::pdftex_commands;
use rustex::commands::pgfsvg::pgf_commands;
use rustex::commands::rustex_specials::rustex_special_commands;
use rustex::stomach::colon::NoColon;
use rustex::utils::PWD;

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
    singlethreaded:bool,

    /// process latex.ltx verbosely
    #[clap(long)]
    test:bool

}
static mut SKIP : bool = false;
static SKIP_UNTIL : &str = "source/mmt.tex"; //integernumbers.en.tex
static DOMAX: usize = 50;
static mut DONE:usize = 0;

fn main() {
    rustex::utils::with_stack_size(run)
}
fn run() {
    let params : Parameters = Parameters::parse();
    use rustex::interpreter::state::State;
    //use magick_rust::{MagickWand, magick_wand_genesis};
    use rustex::fonts::convert::*;

    if params.test {
        let state = rustex::utils::with_stack_size(|| {
            use rustex::interpreter::params::DefaultParams;
            println!("Testing latex.ltx...");
            let mut state = State::new();
            let pdftex_cfg = rustex::kpathsea::kpsewhich("pdftexconfig.tex", &PWD).expect("pdftexconfig.tex not found").0;
            let latex_ltx = rustex::kpathsea::kpsewhich("latex.ltx", &PWD).expect("No latex.ltx found").0;
            let p = DefaultParams::new(false, false, None);

            for c in pdftex_commands() {
                let c = c.as_command();
                state.commands.set_locally(unsafe {c.name().unwrap_unchecked()}, Some(c))
            }
            for c in rustex_special_commands() {
                let c = c.as_command();
                state.commands.set_locally(unsafe {c.name().unwrap_unchecked()}, Some(c))
            }

            state = Interpreter::do_file_with_state(&pdftex_cfg, state, NoColon::new(), &p).1;
            state = Interpreter::do_file_with_state(&latex_ltx, state, NoColon::new(), &p).1;
            for c in pgf_commands() {
                let c = c.as_command();
                state.commands.set_locally(unsafe {c.name().unwrap_unchecked()}, Some(c))
            }
            state
        });
        state.commands.get(&"eTeXversion".into()).expect("");
        println!("\n\nSuccess! \\o/");
        return
    }

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
                    let max = 10;
                    let mut done = 0;
                    let mut state = State::pdf_latex();
                    fn do_dir<P: AsRef<Path>,D:Display>(s : P,d:D,mut st : State,out:Option<String>) -> State {
                        //println!("{}",d);
                        for f in std::fs::read_dir(s).unwrap() {
                            let f = f.unwrap();
                            let path = f.path();
                            if std::fs::metadata(&path).unwrap().is_dir() {
                                let init = path.to_str().unwrap();
                                if !init.ends_with(".git") &&
                                    !init.ends_with("content") &&
                                    !init.ends_with("errors") &&
                                    !init.ends_with("narration") &&
                                    !init.ends_with("relational") &&
                                    !init.ends_with("buildresults") &&
                                    !init.ends_with("xhtml") &&
                                    !init.ends_with("export") &&
                                    !init.ends_with("lib")
                                    {
                                    st = do_dir(path.clone(),path.display(),st.clone(),out.clone())
                                }
                            } else {
                                if path.to_str().unwrap().ends_with(".tex") {
                                    if unsafe{!SKIP} && path.to_str().unwrap().ends_with(SKIP_UNTIL) {
                                        unsafe {SKIP = true}
                                    };
                                    if unsafe{SKIP && (DONE < DOMAX)} {
                                        unsafe {DONE += 1};
                                        println!("------------\n\nDoing {}\n\n---------------\n", path.to_str().unwrap());
                                        let mut stomach = NoShipoutRoutine::new();
                                        let p = DefaultParams::new(false, false, None);
                                        let mut int = Interpreter::with_state(st.clone(), stomach.borrow_mut(), &p);
                                        let (success, s) = int.do_file(&path, HTMLColon::new(true));
                                        if success {
                                            let mut topcommands = int.state.commands.destroy();
                                            for (n,cmd) in topcommands.into_iter() {
                                                if n.to_string().starts_with("c_stex_module") {
                                                    st.commands.set(n,cmd.map(|x| x.clean()),true);
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
                        st
                    }
                    state = do_dir(d.clone(),d,state,params.output.clone());
                    let mut stomach = NoShipoutRoutine::new();
                    let p = DefaultParams::new(false, false, None);
                    let mut int = Interpreter::with_state(state, stomach.borrow_mut(), &p);
                    let path = Path::new("/home/jazzpirate/work/MathHub/sTeX/DemoExamples/source/quickstart.tex");
                    int.do_file(path, HTMLColon::new(true));
                }
            }
        }
        Some(i) => {
            let mut path = Path::new(&i).to_path_buf();
            if !path.is_absolute() {
                path = env::current_dir().unwrap().join(path);
            }
            let mut stomach = NoShipoutRoutine::new();
            let p = DefaultParams::new(false,params.singlethreaded,None);
            let state = State::pdf_latex();
            let mut int = Interpreter::with_state(state,stomach.borrow_mut(),&p);
            let (success,s) = match params.text {
                Some(s) =>
                    int.do_string(&*path,s.as_str(),HTMLColon::new(true)),
                None => {
                    if !path.exists() {
                        println!("File {} not found", i)
                    }
                    int.do_file(&*path, HTMLColon::new(true))
                }
            };
            match params.output {
                None => if success {println!("\n\nSuccess!\n{}",s)} else {println!("\n\nFailed\n{}",s)},
                Some(f) => {
                   /* let f = match f.strip_prefix("\"").and_then(|f| f.strip_suffix("\"")) {
                        Some(s) => s.to_string(),
                        _ => f
                    };*/
                    let mut path = Path::new(&f).to_path_buf();
                    if !path.is_absolute() {
                        path = env::current_dir().unwrap().join(path);
                    }
                    std::fs::write(path,s.as_bytes()).expect("");
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