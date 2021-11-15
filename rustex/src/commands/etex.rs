use crate::commands::{IntCommand, PrimitiveExecutable, TeXCommand};
use crate::interpreter::Interpreter;
use crate::ontology::{Expansion, Token};
use crate::VERSION_INFO;

pub static ETEXREVISION : PrimitiveExecutable = PrimitiveExecutable {
    expandable:true,
    _apply: |cs: Token, int: &Interpreter| {
        int.push_tokens(Interpreter::string_to_tokens(VERSION_INFO.etexrevision()));
        Ok(())
    },
    name: "etexrevision"
};


pub static ETEXVERSION : IntCommand = IntCommand {
    _getvalue: |int| {
        Ok(VERSION_INFO.etexversion().parse().unwrap())
    },
    name: "eTeXversion"
};

pub fn etex_commands() -> Vec<TeXCommand> {vec![
    TeXCommand::Primitive(&ETEXREVISION),
    TeXCommand::Int(&ETEXVERSION),
]}