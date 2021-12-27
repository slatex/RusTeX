use std::borrow::BorrowMut;
use std::cell::RefCell;
use std::ops::Mul;
use std::path::{Path, PathBuf};
use rustex::interpreter::dimensions::{dimtostr, round_f};
use rustex::interpreter::Interpreter;
use rustex::stomach::NoShipoutRoutine;
use rustex::utils::TeXString;

// 1m14,902 / 1m11,366
// 1m30,370 / 1m26,295

fn do_latexltx() {
    use rustex::interpreter::state::default_pdf_latex_state;
    let state = default_pdf_latex_state();
    state.get_command(&"eTeXversion".into()).expect("");
    println!("\n\nSuccess! \\o/")
}

fn do_thesis() {
    do_other("/home/jazzpirate/work/LaTeX/Papers/19 - Thesis/thesis.tex")
}

fn do_other(filename : &str) {
    use rustex::interpreter::state::default_pdf_latex_state;
    let state = default_pdf_latex_state();
    let mut stomach = NoShipoutRoutine::new();
    let mut int = Interpreter::with_state(state,stomach.borrow_mut());
    int.do_file(Path::new(filename));
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
    //let pwd = std::env::current_dir().expect("No current directory!");

    //println!("PWD: {}",pwd.as_path().to_str().unwrap());
    //let test = include_str!("resources/hyphen.cfg");
    //let test2 : TeXString = test.as_bytes().into();
    //println!("{}",rustex::HYPHEN_CFG);
    //println!("{}\n\n{}",test,test2);
    //do_latexltx();
    let mut args: Vec<String> = std::env::args().collect();
    args.remove(0);
    if args.is_empty() {
        do_latexltx()
    } else {
        let mut str = "".to_string();
        for s in args {
            str += &s;
            if s.ends_with("\\") {
                str.pop();
                str.push(' ')
            } else { break }
        }
        //use fixed::types::I0F32;
        do_other(&str)
        /*for i in 1..100 {
            let div = i / 5;
            let numexp = ((i as f32) / 5.0).round() as i32;
            let dot = (((0.2 as f32 * 65536.0).floor() * (i as f32)) / 65536.0).floor() as i32;
            println!("{} {} {} {} {} {} {}",i,div,numexp,dot,dimtostr(dot),dimtostr(div),dimtostr(numexp))
        }*/
    }
    //do_thesis()
    //do_other()

}