use crate::ontology::ExpansionRef;
use crate::utils::TeXStr;

#[derive(Clone)]
pub enum SourceReference {
    File(TeXStr,(usize,usize),(usize,usize)),
    Exp(ExpansionRef),
    None
}

#[derive(PartialEq,Clone)]
pub struct SourceFileReference {
    pub file:TeXStr,
    pub start:(usize,usize),
    pub end:(usize,usize)
}
impl SourceFileReference {
    pub fn as_string(&self) -> String {
        self.file.to_string() + "#(" + &self.start.0.to_string() + "," + &self.start.1.to_string() +
            "):(" + &self.end.0.to_string() + "," + &self.end.1.to_string() + ")"
    }
}