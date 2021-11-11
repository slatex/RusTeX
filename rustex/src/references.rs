use std::rc::Rc;

#[derive(Clone)]
pub struct FileReference {
    pub file:String,
    pub start:(usize,usize),
    pub end:(usize,usize)
}

use crate::ontology::{Expansion, Token};

#[derive(Clone)]
pub struct ExpansionReference {
    pub exp: Rc<Expansion>,
    pub tk : Token
}

#[derive(Clone)]
pub enum SourceReference {
    File(FileReference),
    Exp(ExpansionReference),
    None
}