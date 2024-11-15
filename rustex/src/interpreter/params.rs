pub trait CommandListener : Sync {
    fn apply(&self,name:&TeXStr,cmd: &Option<TeXCommand>,file:&TeXStr,line:&String,state:&mut State) -> Option<Option<TeXCommand>>;
    fn def_cmd(&self,ret:Vec<Token>,protected:bool,long:bool,pars:Vec<ParamToken>) -> TeXCommand {
        let mut arity = 0;
        for t in &pars { if let ParamToken::Param(i) = t { arity = max(arity,*i)}}
        let sig = Signature {
            elems:pars,endswithbrace:false,arity
        };
        PrimitiveTeXCommand::Def(DefMacro {
            protected,long,sig,ret
        }).as_command()
    }
}

pub trait InterpreterParams {
    fn singlethreaded(&self) -> bool;
    fn do_log(&self) -> bool;
    fn set_log(&mut self,b : bool);
    fn store_in_file(&self) -> bool;
    fn copy_tokens_full(&self) -> bool;
    fn copy_commands_full(&self) -> bool;
    fn log(&self,s:&str);
    fn write_16(&self,s:&str);
    fn write_17(&self,s:&str);
    fn write_18(&self,s:&str);
    fn write_neg_1(&self,s:&str);
    fn write_other(&self,s:&str);
    fn file_open(&self,s:&str);
    fn file_close(&self);
    fn error(&self,t:TeXError);
    fn message(&self,s:&str);
    fn command_listeners(&self) -> &Vec<Box<dyn CommandListener>>;
}

pub struct DefaultParams {
    pub log:bool,
    pub singlethreaded:bool,
    pub listeners: Vec<Box<dyn CommandListener>>
}

impl DefaultParams {
    pub fn new(log:bool,singlethreaded:bool,listeners:Option<Vec<Box<dyn CommandListener>>>) -> DefaultParams {
        DefaultParams {
            log,singlethreaded,
            listeners: match listeners {
                Some(v) => v,
                None => DefaultParams::default_listeners()
            }
        }
    }
    pub fn default_listeners() -> Vec<Box<dyn CommandListener>> {
        crate::commands::latex_bindings::all_listeners()
    }
}

use std::cmp::max;
use ansi_term::Colour::*;
use crate::commands::{DefMacro, ParamToken, PrimitiveTeXCommand, Signature, TeXCommand};
use crate::interpreter::state::State;
use crate::ontology::Token;
use crate::utils::{TeXError, TeXStr};

impl InterpreterParams for DefaultParams {
    fn singlethreaded(&self) -> bool { self.singlethreaded }
    fn do_log(&self) -> bool { self.log }
    fn set_log(&mut self,b: bool) { self.log = b }
    fn store_in_file(&self) -> bool { false }
    fn copy_tokens_full(&self) -> bool { true }
    fn copy_commands_full(&self) -> bool { true }
    fn log(&self,_s:&str) {}
    fn write_16(&self,s:&str) { print!("{}",White.bold().paint(s)) }
    fn write_17(&self,s:&str) { print!("{}",s) }
    fn write_18(&self,_s:&str) { }
    fn write_neg_1(&self,s:&str) { print!("{}",Black.on(Blue).paint(s)) }
    fn write_other(&self,s:&str) { print!("{}",Black.on(Green).paint(s)) }
    fn file_open(&self, s: &str) { print!("{}",s) }
    fn file_close(&self) { print!(")") }
    fn message(&self,s:&str) { print!("{}",Yellow.paint(s)) }
    fn error(&self,t:TeXError) {
        println!("{}",Red.paint(std::format!("{}",t)))
    }
    fn command_listeners(&self) -> &Vec<Box<dyn CommandListener>> { &self.listeners }
}
pub struct NoOutput {
    pub listeners: Vec<Box<dyn CommandListener>>
}
impl NoOutput {
    pub fn new(listeners:Option<Vec<Box<dyn CommandListener>>>) -> NoOutput {
        NoOutput {
            listeners: match listeners {
                Some(v) => v,
                None => DefaultParams::default_listeners()
            }
        }
    }
}
impl InterpreterParams for NoOutput {
    fn singlethreaded(&self) -> bool { false}
    fn do_log(&self) -> bool { false }
    fn set_log(&mut self,_b: bool) {}
    fn store_in_file(&self) -> bool { false }
    fn copy_tokens_full(&self) -> bool { true }
    fn copy_commands_full(&self) -> bool { true }
    fn log(&self,_s:&str) {}
    fn write_16(&self,_s:&str) {}
    fn write_17(&self,_s:&str) {}
    fn write_18(&self,_s:&str) {}
    fn write_neg_1(&self,_s:&str) {}
    fn write_other(&self,_s:&str) {}
    fn file_open(&self, _s: &str) {}
    fn file_close(&self) {}
    fn message(&self, _s: &str) {}
    fn error(&self,t:TeXError) {
        println!("{}",Red.paint(std::format!("{}",t)))
    }
    fn command_listeners(&self) -> &Vec<Box<dyn CommandListener>> { &self.listeners }
}