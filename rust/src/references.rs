use std::rc::Rc;

pub struct FileReference {
    pub file:String,
    pub start:(usize,usize),
    pub end:(usize,usize)
}

use crate::ontology::Expansion;

pub struct ExpansionReference {
    pub exp: Rc<Expansion>
}

pub enum SourceReference {
    File(FileReference),
    Exp(ExpansionReference),
    None
}