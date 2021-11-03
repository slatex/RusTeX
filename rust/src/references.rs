use crate::ontology::LaTeXFile;
use std::rc::Rc;

pub struct FileReference {
    pub file:String,
    pub start:(u32,u32),
    pub end:(u32,u32)
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