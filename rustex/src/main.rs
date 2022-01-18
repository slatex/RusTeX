use std::borrow::BorrowMut;
use std::collections::VecDeque;
use std::io::Write;
use std::path::Path;
use rustex::interpreter::Interpreter;
use rustex::interpreter::params::{DefaultParams, NoOutput};
use rustex::stomach::html::HTMLColon;
use rustex::stomach::NoShipoutRoutine;
use rustex::utils::TeXError;

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
    let mut int = Interpreter::with_state(state,stomach.borrow_mut(),&DefaultParams {log : false});
    let s = int.do_file(Path::new(filename),HTMLColon::new(true));
    let mut file = std::fs::File::create(rustex::LOG_FILE).unwrap();
    file.write_all(s.as_bytes());
    println!("\n\nSuccess! \\o/\nResult written to {}",rustex::LOG_FILE)
}

fn do_smglom() {
    use rustex::interpreter::state::default_pdf_latex_state;
    let mut state = default_pdf_latex_state();
    let arr = vec!("/home/jazzpirate/work/MathHub/smglom/IWGS/source/jupyterNB.en.tex",
                   "/home/jazzpirate/work/MathHub/smglom/analysis/source/BBPformula.en.tex",
                   "/home/jazzpirate/work/MathHub/smglom/analysis/source/alternatingharmonicseries.en.tex",
                   "/home/jazzpirate/work/MathHub/smglom/analysis/source/asymptoticdensity.en.tex",
                   "/home/jazzpirate/work/MathHub/smglom/analysis/source/baxterhickersonfunction.en.tex",
                   "/home/jazzpirate/work/MathHub/smglom/analysis/source/chebyshevfunction.en.tex",
                   "/home/jazzpirate/work/MathHub/smglom/analysis/source/cosineintegralbig.en.tex",
                   "/home/jazzpirate/work/MathHub/smglom/analysis/source/cosineintegralint.en.tex",
                   "/home/jazzpirate/work/MathHub/smglom/analysis/source/cosineintegralsmall.en.tex",
                   "/home/jazzpirate/work/MathHub/smglom/analysis/source/generalharmonicseries.en.tex",
                   "/home/jazzpirate/work/MathHub/smglom/analysis/source/gregorynumber.en.tex",
                   "/home/jazzpirate/work/MathHub/smglom/analysis/source/harmonicseries.en.tex",
                   "/home/jazzpirate/work/MathHub/smglom/analysis/source/hurwitzzetafunction.en.tex",
                   "/home/jazzpirate/work/MathHub/smglom/analysis/source/hyperboliccosineintegral.en.tex");
    for filename in arr {
        println!("{}",filename);
        let mut stomach = NoShipoutRoutine::new();
        let mut int = Interpreter::with_state(state.clone(), stomach.borrow_mut(), &DefaultParams { log:false });
        let s = int.do_file(Path::new(filename), HTMLColon::new(true));
        /*let frame = int.state.into_inner().stacks.remove(0);
        for (n,cmd) in frame.commands {
            if n.to_string().starts_with("c_stex_module_") {
                state.stacks.first_mut().unwrap().commands.insert(n,cmd);
            }
        }*/
    }
}

fn generate_test() -> Vec<Vec<String>> {
    let mut ret : Vec<Vec<String>> = vec!();
    for i in 0..10000 {
        ret.push(vec!());
        for j in 0..10000 {
            ret.last_mut().unwrap().push(i.to_string() + "_" + &j.to_string())
        }
    }
    ret
}

trait TestIter {
    fn next(&mut self) -> Option<String>;
}

struct TestA(Vec<Vec<String>>,Vec<String>);
impl TestA {
    pub fn new(mut v:Vec<Vec<String>>) -> TestA {
        v.reverse();
        TestA(v,vec!())
    }
}
impl TestIter for TestA {
    fn next(&mut self) -> Option<String> {
        if self.1.is_empty() {
            if self.0.is_empty() {
                return None
            }
            self.1 = self.0.pop().unwrap();
            self.1.reverse();
            return self.next()
        }
        self.1.pop()
    }
}
struct TestB(Vec<Vec<String>>,usize,usize);
impl TestB {
    pub fn new(v:Vec<Vec<String>>) -> TestB {
        TestB(v,0,0)
    }
}
impl TestIter for TestB {
    fn next(&mut self) -> Option<String> {
        match self.0.get(self.1) {
            None => return None,
            Some(v) => match v.get(self.2) {
                None => {
                    self.1 += 1;
                    self.2 = 0;
                    return self.next()
                }
                Some(s) => {
                    self.2 += 1;
                    Some(s.clone())
                }
            }
        }
    }
}

struct TestC(Vec<Vec<String>>,Vec<String>);
impl TestC {
    pub fn new(v:Vec<Vec<String>>) -> TestC {
        TestC(v,vec!())
    }
}
impl TestIter for TestC {
    fn next(&mut self) -> Option<String> {
        if self.1.is_empty() {
            if self.0.is_empty() { return None }
            self.1 = self.0.remove(0);
            return self.next()
        }
        Some(self.1.remove(0))
    }
}

struct TestD(VecDeque<VecDeque<String>>);
impl TestD {
    pub fn new(mut v:Vec<Vec<String>>) -> TestD {
        TestD(VecDeque::from(v.drain(..).map(|x| VecDeque::from(x)).collect::<Vec<VecDeque<String>>>()))
    }
}
impl TestIter for TestD {
    fn next(&mut self) -> Option<String> {
        if self.0.is_empty() { return None }
        match self.0.get_mut(0).unwrap().pop_front() {
            None => {
                self.0.pop_front();
                return self.next()
            }
            s => s
        }
    }
}
// A: 0m14s B:0m13,147s
fn my_test() {
    use std::time::{Duration, Instant};
    let v = generate_test();
    let mut dur = Duration::new(0,0);

    dur = Duration::new(0,0);
    for _ in 0..10 {
        let start = Instant::now();
        let mut tst = TestD::new(v.clone());
        let (mut i, mut j) = (0, 0);
        loop {
            match tst.next() {
                Some(_) if i >= 100000 => {
                    i = 0;
                    j += 1
                }
                Some(_) => i += 1,
                _ => break
            }
        }
        let duration = start.elapsed();
        dur += duration;
        println!("Done {}:{}", i, j)
    }
    println!("D: {}",dur.as_secs_f64());

    dur = Duration::new(0,0);
    for _ in 0..10 {
        let start = Instant::now();
        let mut tst = TestA::new(v.clone());
        let (mut i, mut j) = (0, 0);
        loop {
            match tst.next() {
                Some(_) if i >= 100000 => {
                    i = 0;
                    j += 1
                }
                Some(_) => i += 1,
                _ => break
            }
        }
        let duration = start.elapsed();
        dur += duration;
        println!("Done {}:{}", i, j)
    }
    println!("A: {}",dur.as_secs_f64());

    dur = Duration::new(0,0);
    for _ in 0..10 {
        let start = Instant::now();
        let mut tst = TestB::new(v.clone());
        let (mut i, mut j) = (0, 0);
        loop {
            match tst.next() {
                Some(_) if i >= 100000 => {
                    i = 0;
                    j += 1
                }
                Some(_) => i += 1,
                _ => break
            }
        }
        let duration = start.elapsed();
        dur += duration;
        println!("Done {}:{}", i, j)
    }
    println!("B: {}",dur.as_secs_f64());

    dur = Duration::new(0,0);
    for _ in 0..10 {
        let start = Instant::now();
        let mut tst = TestC::new(v.clone());
        let (mut i, mut j) = (0, 0);
        loop {
            match tst.next() {
                Some(_) if i >= 100000 => {
                    i = 0;
                    j += 1
                }
                Some(_) => i += 1,
                _ => break
            }
        }
        let duration = start.elapsed();
        dur += duration;
        println!("Done {}:{}", i, j)
    }
    println!("C: {}",dur.as_secs_f64());

}
// A: 64.38171691   47.421736973  58
// B: 48.910752616  50.643248688  43
// D: 63.352201487  60.68231759   68

fn my_test_2() {
    let mut vec : Vec<String> = vec!();
    for i in 1..100 { vec.push(i.to_string()) }
    for s in vec.drain(..) {
        if s == "89" { break }
    }
    for s in vec { println!("{}",s) }
}
/*
fn err_Test_dummy() -> Result<(),TeXError> {
    todo!()
}

fn err_Test() {
    let a = err_Test_dummy().map_err(|e| e.derive("Bla".to_string()))?
}
 */

fn main() {
    //my_test_2()
    //my_test()
    //rustex::kpathsea::kpsewhich("pdftexconfig.tex",&std::env::current_dir().expect("No current directory!"));
    //do_smglom()
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

        do_other(&str)
    }
}