use std::path::Path;
use rustex::interpreter::Interpreter;

fn do_latexltx() {
    use rustex::interpreter::state::default_pdf_latex_state;
    let state = default_pdf_latex_state();
    state.get_command(&"eTeXversion".into()).expect("");
    println!("\n\nSuccess! \\o/")
}

fn do_thesis() {
    use rustex::interpreter::state::default_pdf_latex_state;
    let state = default_pdf_latex_state();
    //unsafe{ rustex::LOG = true };
    Interpreter::do_file_with_state(Path::new("/home/jazzpirate/work/LaTeX/Papers/19 - Thesis/thesis.tex"),state);
}
/*
fn do_test() {
    let state = State::new();
    use rustex::utils::{kpsewhich,PWD};
    use std::fs;
    let latexltx = kpsewhich("pdftexconfig.tex",&PWD).expect("latex.ltx not found!");
    let content = fs::read_to_string(latexltx).unwrap();
    let dummyexp = Expansion(Token::dummy())//::dummy(vec!());
    let mut mouth = StringMouth::new(state.newlinechar(),dummyexp,content.into());
    let mut ret: Vec<Token> = Vec::new();
    while mouth.has_next(state.catcodes(),true) {
        ret.push(mouth.pop_next(state.catcodes(),true))
    }
    println!("Length: {}",ret.len());
    print!("\nResult: ");
    for r in ret {
        print!("{}",r)
    }
}

 */

//use ansi_term::Colour;
//use chrono::{DateTime,Local};

fn main() {
    //eprintln!("{}, {}, {}, {}, {}, {}, {}, {}","black".black(),"red".red(),"green".green(),"blue".blue(),"magenta".magenta(),"purple".purple(),"cyan".cyan(),"white".white());
    //eprintln!("{}, {}, {}, {}, {}, {}, {}, {}","black".bright_black(),"red".bright_red(),"green".bright_green(),"blue".bright_blue(),"magenta".bright_magenta(),
    //         "purple".bright_purple(),"cyan".bright_cyan(),"white".bright_white());
    //eprintln!("\033[31;1;4mThis is a test\033[0m");
    //println!("Another test: {}",Colour::Red.bold().paint("test"));
    //default_pdf_latex_state().dummy();
    //println!("{}, {}, {}, {}, {}",VERSION_INFO.texversion(),VERSION_INFO.etexversion(),VERSION_INFO.etexrevision(),VERSION_INFO.pdftexversion(),VERSION_INFO.pdftexrevision());
    //"bla bla\n bla bla".as_bytes().iter_mut().multipeek()
    // https://doc.rust-lang.org/book/ch15-04-rc.html

    do_latexltx()
    //do_thesis()

}