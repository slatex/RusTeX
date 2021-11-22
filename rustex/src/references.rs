use crate::ontology::ExpansionRef;

#[derive(Clone)]
pub enum SourceReference {
    File(String,(usize,usize),(usize,usize)),
    Exp(ExpansionRef),
    None
}