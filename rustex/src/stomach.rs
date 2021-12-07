use std::borrow::BorrowMut;
use crate::stomach::whatsits::Whatsit;
use crate::{Interpreter, TeXErr};
use crate::utils::TeXError;

pub mod whatsits;

pub trait Stomach {
    fn new_group(&mut self);
    fn pop_group(&mut self,int:&Interpreter) -> Result<Vec<Whatsit>,TeXError>;
    fn add(&mut self,wi:Whatsit);
    fn close_group(&mut self,int:&Interpreter) -> Result<(),TeXError> {
        for w in self.pop_group(int)? {
            self.add(w)
        };
        Ok(())
    }
    fn last_whatsit(&self) -> Option<Whatsit>;
}

pub trait BufferedStomach : Stomach {
    fn buffer_mut(&mut self) -> &mut Vec<Vec<Whatsit>>;
    fn buffer(&self) -> &Vec<Vec<Whatsit>>;
}

impl<S> Stomach for S where S:BufferedStomach {
    fn new_group(&mut self) {
        self.buffer_mut().push(vec!())
    }
    fn pop_group(&mut self,int:&Interpreter) -> Result<Vec<Whatsit>, TeXError> {
        let buf = self.buffer();
        if buf.len() < 2 {
            TeXErr!((int,None),"Can't close group in stomach!")
        } else {
            Ok(self.buffer_mut().pop().unwrap())
        }
    }
    fn add(&mut self, wi: Whatsit) {
        self.buffer_mut().last_mut().unwrap().push(wi)
    }
    fn last_whatsit(&self) -> Option<Whatsit> {
        let buf = self.buffer();
        for ls in buf.iter().rev() {
            for v in ls.iter().rev() {
                match v {
                    w@Whatsit::Box(_) => return Some(w.clone()),
                    w@Whatsit::Simple(_) => return Some(w.clone()),
                    _ => ()
                }
            }
        }
        return None
    }
}

pub struct EmptyStomach {
    buff : Vec<Vec<Whatsit>>
}
impl EmptyStomach {
    pub fn new() -> EmptyStomach {
        EmptyStomach {
            buff:vec!(vec!())
        }
    }
}

impl BufferedStomach for EmptyStomach {
    fn buffer_mut(&mut self) -> &mut Vec<Vec<Whatsit>> {
        self.buff.borrow_mut()
    }
    fn buffer(&self) -> &Vec<Vec<Whatsit>> {
        &self.buff
    }

}