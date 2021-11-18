use std::rc::Rc;
use crate::commands::TeXCommand;

use crate::ontology::{Expansion, Token};

#[derive(Clone)]
pub enum SourceReference {
    File(String,(usize,usize),(usize,usize)),
    Exp(Token,TeXCommand),
    None
}