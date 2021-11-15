use crate::commands::{IntCommand, PrimitiveExecutable, TeXCommand};
use crate::interpreter::Interpreter;
use crate::ontology::{Expansion, Token};
use crate::VERSION_INFO;

pub static ETEXREVISION : PrimitiveExecutable = PrimitiveExecutable {
    expandable:true,
    _apply: |cs: Token, int: &Interpreter| {
        Ok(Some(Expansion {
            cs,
            exp: Interpreter::string_to_tokens(VERSION_INFO.etexrevision())
        }))
    },
    name: "etexrevision"
};

pub static ETEXVERSION : IntCommand = IntCommand {
    _getvalue: |int| {
        Ok(VERSION_INFO.etexversion().parse().unwrap())
    },
    name: "eTeXversion"
};

pub static UNEXPANDED: PrimitiveExecutable = PrimitiveExecutable {
    name:"unexpanded",
    expandable:true,
    _apply:|tk,int| {todo!()}
};

pub fn etex_commands() -> Vec<TeXCommand> {vec![
    TeXCommand::Primitive(&ETEXREVISION),
    TeXCommand::Primitive(&UNEXPANDED),
    TeXCommand::Int(&ETEXVERSION),
]}