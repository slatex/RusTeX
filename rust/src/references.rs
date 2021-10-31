use crate::utils::FilePath;

#[derive(Clone)]
pub struct FileReference {
    pub file:FilePath,
    pub start:(u32,u32),
    pub end:(u32,u32)
}

use crate::ontology::Expansion;

#[derive(Copy, Clone)]
pub struct ExpansionReference<'a> {
    pub exp: &'a Expansion
}

#[derive(Clone)]
pub enum SourceReference<'a> {
    File(FileReference),
    Exp(ExpansionReference<'a>),
    None
}