pub trait CommandListener : Sync {
    fn apply(&self,name:&TeXStr,cmd: &Option<TeXCommand>,file:&TeXStr,line:&String) -> Option<Option<TeXCommand>>;
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
        use crate::commands::latex_bindings::*;
        DefaultParams {
            log,singlethreaded,
            listeners: match listeners {
                Some(v) => v,
                None => vec!(
                    Box::new(UrlListener())
                )
            }
        }
    }
}

use ansi_term::Colour::*;
use crate::commands::TeXCommand;
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
        use crate::commands::latex_bindings::*;
        NoOutput {
            listeners: match listeners {
                Some(v) => v,
                None => vec!(
                    Box::new(UrlListener())
                )
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