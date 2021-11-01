use itertools::Itertools;

use rustex::mouth::{Mouth, StringMouth};
use rustex::state::default_pdf_latex_state;
use rustex::ontology::{CommandI, Expansion, PrimitiveControlSequence,Token};
use rustex::references::SourceReference;

fn main() {
    use rustex::state::default_pdf_latex_state;
    use rustex::VERSION_INFO;
    extern crate itertools;
    use itertools::Itertools;
    use std::rc::Rc;
    //default_pdf_latex_state().dummy();
    println!("{}, {}, {}, {}, {}",VERSION_INFO.texversion(),VERSION_INFO.etexversion(),VERSION_INFO.etexrevision(),VERSION_INFO.pdftexversion(),VERSION_INFO.pdftexrevision());
    //"bla bla\n bla bla".as_bytes().iter_mut().multipeek()
    // https://doc.rust-lang.org/book/ch15-04-rc.html
    let mut state = default_pdf_latex_state();
    let dummyexp = Expansion {
        cs: PrimitiveControlSequence::new("narf".to_owned(),SourceReference::None).as_command(),
        exp: vec![]
    };

    use rustex::utils::{kpsewhich,FilePath};
    use std::{env,fs};
    let latexltx = kpsewhich("latex.ltx",&FilePath::from_path(env::current_dir().unwrap())).unwrap();

    let content = fs::read_to_string(latexltx.path()).unwrap();

    let mut mouth = StringMouth::new(&state,dummyexp,content.as_str());
    let mut ret: Vec<Rc<Token>> = Vec::new();
    while mouth.has_next(true) {
        ret.push(mouth.pop_next(true))
    }
    println!("Length: {}",ret.len());
    let string : String = ret.iter().map(|x| x.as_string()).collect::<Vec<_>>().join("");
    //println!("chars: {}", string.as_bytes().iter().map(|x| x.to_string()).collect::<Vec<_>>().join(", "));
    println!("Result: {}", string);

}