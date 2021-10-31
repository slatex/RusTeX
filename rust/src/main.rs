use itertools::Itertools;

fn main() {
    use rustex::state::default_pdf_latex_state;
    use rustex::VERSION_INFO;
    extern crate itertools;
    use itertools::Itertools;
    default_pdf_latex_state().dummy();
    println!("{}, {}, {}, {}, {}",VERSION_INFO.texversion(),VERSION_INFO.etexversion(),VERSION_INFO.etexrevision(),VERSION_INFO.pdftexversion(),VERSION_INFO.pdftexrevision())
    //"bla bla\n bla bla".as_bytes().iter_mut().multipeek()
    // https://doc.rust-lang.org/book/ch15-04-rc.html
}