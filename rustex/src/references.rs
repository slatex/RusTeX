use crate::ontology::ExpansionRef;
use crate::utils::TeXStr;

#[derive(Clone)]
pub enum SourceReference {
    File(TeXStr,(usize,usize),(usize,usize)),
    Exp(ExpansionRef),
    None
}

#[derive(Clone)]
pub struct SourceFileReference {
    file:TeXStr,

}