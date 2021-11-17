use crate::commands::{IntCommand, PrimitiveExecutable, TeXCommand};
use crate::interpreter::Interpreter;
use crate::ontology::{Expansion, Token};
use crate::VERSION_INFO;

pub static ETEXREVISION : PrimitiveExecutable = PrimitiveExecutable {
    expandable:true,
    _apply: |cs: Token, _int: &Interpreter| {
        Ok(Some(Expansion {
            cs,
            exp: Interpreter::string_to_tokens(VERSION_INFO.etexrevision.clone())
        }))
    },
    name: "etexrevision"
};

pub static ETEXVERSION : IntCommand = IntCommand {
    _getvalue: |_int| {
        Ok(VERSION_INFO.etexversion.to_string().parse().unwrap())
    },
    name: "eTeXversion"
};

pub static UNEXPANDED: PrimitiveExecutable = PrimitiveExecutable {
    name:"unexpanded",
    expandable:true,
    _apply:|_tk,_int| {todo!()}
};

pub fn etex_commands() -> Vec<TeXCommand> {vec![
    TeXCommand::Primitive(&ETEXREVISION),
    TeXCommand::Primitive(&UNEXPANDED),
    TeXCommand::Int(&ETEXVERSION),
]}