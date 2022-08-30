use std::borrow::BorrowMut;
use std::io::Write;
use std::path::Path;
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

fn main() {
    let params : Parameters = Parameters::parse();
    use rustex::interpreter::state::State;
    //use magick_rust::{MagickWand, magick_wand_genesis};

    match params.input {
        None => {
            println!("No file given. Testing latex.ltx...");
            let state = State::pdf_latex();
            state.commands.get(&"eTeXversion".into()).expect("");
            println!("\n\nSuccess! \\o/")
        }
        Some(i) => {
            let path = Path::new(&i);
            let mut stomach = NoShipoutRoutine::new();
            let p = DefaultParams::new(false,params.singlethreaded,None);
            let state = State::pdf_latex();
            let mut int = Interpreter::with_state(state,stomach.borrow_mut(),&p);
            let s = match params.text {
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
                None => println!("\n\nSuccess!\n{}",s),
                Some(f) => {
                    let mut file = std::fs::File::create(&f).unwrap();
                    file.write_all(s.as_bytes()).expect("");
                    println!("\n\nSuccess! \\o/\nResult written to {}",f)
                }
            }

        }
    }
}