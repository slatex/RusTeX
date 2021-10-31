use itertools::Itertools;

fn main() {
    use rustex::state::default_pdf_latex_state;
    use rustex::VERSION_INFO;
    extern crate itertools;
    use itertools::Itertools;
    default_pdf_latex_state().dummy();
    println!("{}, {}, {}, {}, {}",VERSION_INFO.texversion(),VERSION_INFO.etexversion(),VERSION_INFO.etexrevision(),VERSION_INFO.pdftexversion(),VERSION_INFO.pdftexrevision());
    //"bla bla\n bla bla".as_bytes().iter_mut().multipeek()
    // https://doc.rust-lang.org/book/ch15-04-rc.html
    let test = "blablabla";
    use std::rc::Rc;
    let rca = Rc::new(test);
    println!("First: {}", Rc::strong_count(&rca));
    {
        let rcb = Rc::new(test);
        println!("Second: {}", Rc::strong_count(&rcb));
    }
    let testa = {
        let teststr = "bla bla bla";
        Rc::new(teststr)
    };
    println!("First: {}", Rc::strong_count(&testa));
}