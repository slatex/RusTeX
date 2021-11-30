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
}

pub trait BufferedStomach : Stomach {
    fn buffer(&mut self) -> &mut Vec<Vec<Whatsit>>;
}

impl<S> Stomach for S where S:BufferedStomach {
    fn new_group(&mut self) {
        self.buffer().push(vec!())
    }
    fn pop_group(&mut self,int:&Interpreter) -> Result<Vec<Whatsit>, TeXError> {
        let buf = self.buffer();
        if buf.len() < 2 {
            TeXErr!((int,None),"Can't close group in stomach!")
        } else {
            Ok(self.buffer().pop().unwrap())
        }
    }
    fn add(&mut self, wi: Whatsit) {
        self.buffer().last_mut().unwrap().push(wi)
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
    fn buffer(&mut self) -> &mut Vec<Vec<Whatsit>> {
        self.buff.borrow_mut()
    }
}