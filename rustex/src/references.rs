use std::rc::Rc;

use crate::ontology::{Expansion,ExpansionRef};

#[derive(Clone)]
pub enum SourceReference {
    File(String,(usize,usize),(usize,usize)),
    Exp(ExpansionRef),
    None
}