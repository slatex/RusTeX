use itertools::Itertools;

struct A {
    a : String
}

struct B {
    i : i32
}

enum AB {
    L(A),
    R(B)
}

fn main() {
    use rustex::state::default_pdf_latex_state;
    use rustex::VERSION_INFO;
    extern crate itertools;
    use itertools::Itertools;
    use std::rc::Rc;
    default_pdf_latex_state().dummy();
    println!("{}, {}, {}, {}, {}",VERSION_INFO.texversion(),VERSION_INFO.etexversion(),VERSION_INFO.etexrevision(),VERSION_INFO.pdftexversion(),VERSION_INFO.pdftexrevision());
    //"bla bla\n bla bla".as_bytes().iter_mut().multipeek()
    // https://doc.rust-lang.org/book/ch15-04-rc.html
    let a = A{ a:"a".to_string()};
    let ab : AB = AB::L(a);
    let c : Rc<A> = match ab {
        AB::L(a) => Rc::new(a),
        _ => panic!()
    };
}