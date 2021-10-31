use crate::ontology::LaTeXFile;

#[derive(Copy, Clone)]
pub struct FileReference<'a> {
    pub file:&'a LaTeXFile,
    pub start:(u32,u32),
    pub end:(u32,u32)
}

use crate::ontology::Expansion;

#[derive(Copy, Clone)]
pub struct ExpansionReference<'a> {
    pub exp: &'a Expansion
}

#[derive(Copy, Clone)]
pub enum SourceReference<'a> {
    File(FileReference<'a>),
    Exp(ExpansionReference<'a>),
    None
}