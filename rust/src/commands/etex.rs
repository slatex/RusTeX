use crate::commands::{Expandable,TeXCommand,Executable};
use crate::interpreter::Interpreter;
use crate::ontology::{ControlSequence,Expansion,Token};
use crate::VERSION_INFO;

pub static ETEREVISION : Expandable = Expandable {
    apply: |cs: ControlSequence, int: Interpreter| {
        Expansion {
            cs: cs,
            exp:Interpreter::string_to_tokens(VERSION_INFO.etexrevision())
        }
    }
};

pub fn etex_commands() -> Vec<(&'static str,&'static dyn TeXCommand)> {vec![
    ("etexrevision",&ETEREVISION)
]}