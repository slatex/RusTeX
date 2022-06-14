use std::cmp::Ordering;
use std::sync::Arc;
use crate::commands::PrimitiveTeXCommand;
use crate::stomach::Whatsit;
use crate::stomach::whatsits::WhatsitTrait;
use crate::Token;
use crate::utils::TeXStr;

#[derive(Clone)]
pub enum SourceReference {
    File(TeXStr,(usize,usize),(usize,usize)),
    Exp(Token,Arc<PrimitiveTeXCommand>)
}

#[derive(PartialEq,Clone)]
pub struct SourceFileReference {
    pub file:TeXStr,
    pub start:(usize,usize),
    pub end:(usize,usize)
}
impl PartialOrd for SourceFileReference {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        if self.file == other.file {
            self.start.partial_cmp(&other.start)
        } else {None}
    }
}
impl SourceFileReference {
    pub fn as_string(&self) -> String {
        self.file.to_string() + "#(" + &self.start.0.to_string() + ";" + &self.start.1.to_string() +
            ")(" + &self.end.0.to_string() + ";" + &self.end.1.to_string() + ")"
    }
    pub fn merge(&self,other:&SourceFileReference) -> Option<SourceFileReference> {
        if self.file == other.file {
            Some(SourceFileReference {
                file:self.file.clone(),
                start:self.start,
                end:other.end
            })
        } else { None }
    }
    pub fn from_wi_lists(ls : &Vec<Vec<Whatsit>>) -> Option<SourceFileReference> {
        match ls.iter().find_map(|w| w.iter().find_map(|w| w.get_ref())) {
            Some(start) => {
                match ls.iter().rev().find_map(|w| w.iter().rev().find_map(|w| w.get_ref())) {
                    Some(end) => start.merge(&end),
                    _ => None
                }
            }
            _ => None
        }
    }
    pub fn from_wi_list(ls : &Vec<Whatsit>) -> Option<SourceFileReference> {
        match ls.iter().find_map(|w| w.get_ref()) {
            Some(start) => {
                match ls.iter().rev().find_map(|w| w.get_ref()) {
                    Some(end) => start.merge(&end),
                    _ => None
                }
            }
            _ => None
        }
    }
}