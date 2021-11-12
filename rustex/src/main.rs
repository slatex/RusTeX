use rustex::interpreter::mouth::StringMouth;
use rustex::ontology::{Expansion, Token};
use rustex::references::SourceReference;
use rustex::interpreter::state::State;

fn do_latexltx() {
    use rustex::interpreter::state::default_pdf_latex_state;
    let state = default_pdf_latex_state();
    println!("{}",state.get_command("etexversion").expect(""))
}

fn do_test() {
    use std::rc::Rc;
    let state = State::new();
    use rustex::utils::{kpsewhich,PWD};
    use std::fs;
    let latexltx = kpsewhich("pdftexconfig.tex",&PWD).expect("latex.ltx not found!");
    let content = fs::read_to_string(latexltx).unwrap();
    let dummyexp = Expansion::dummy(vec!());
    let mut mouth = StringMouth::new(&state,dummyexp,content.as_str());
    let mut ret: Vec<Token> = Vec::new();
    while mouth.has_next(&state,true) {
        ret.push(mouth.pop_next(&state,true))
    }
    println!("Length: {}",ret.len());
    let string : String = ret.iter().map(|x| x.as_string()).collect::<Vec<_>>().join("");
    //println!("chars: {}", string.as_bytes().iter().map(|x| x.to_string()).collect::<Vec<_>>().join(", "));
    println!("Result: {}", string);

}

use colored::*;
// https://crates.io/crates/colored
use ansi_term::Colour;

fn main() {
    eprintln!("{}, {}, {}, {}, {}, {}, {}, {}","black".black(),"red".red(),"green".green(),"blue".blue(),"magenta".magenta(),"purple".purple(),"cyan".cyan(),"white".white());
    eprintln!("{}, {}, {}, {}, {}, {}, {}, {}","black".bright_black(),"red".bright_red(),"green".bright_green(),"blue".bright_blue(),"magenta".bright_magenta(),
             "purple".bright_purple(),"cyan".bright_cyan(),"white".bright_white());
    eprintln!("\033[31;1;4mThis is a test\033[0m");
    println!("Another test: {}",Colour::Red.bold().paint("test"));
    //default_pdf_latex_state().dummy();
    //println!("{}, {}, {}, {}, {}",VERSION_INFO.texversion(),VERSION_INFO.etexversion(),VERSION_INFO.etexrevision(),VERSION_INFO.pdftexversion(),VERSION_INFO.pdftexrevision());
    //"bla bla\n bla bla".as_bytes().iter_mut().multipeek()
    // https://doc.rust-lang.org/book/ch15-04-rc.html
    do_latexltx()


}