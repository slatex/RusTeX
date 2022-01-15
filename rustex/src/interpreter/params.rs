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
    fn file_clopen(&self,s:&str);
    fn message(&self,s:&str);
}

pub struct DefaultParams {
    pub log:bool
}
use ansi_term::Colour::*;
impl InterpreterParams for DefaultParams {
    fn singlethreaded(&self) -> bool { false}
    fn do_log(&self) -> bool { self.log }
    fn set_log(&mut self,b: bool) { self.log = b }
    fn store_in_file(&self) -> bool { false }
    fn copy_tokens_full(&self) -> bool { true }
    fn copy_commands_full(&self) -> bool { true }
    fn log(&self,s:&str) {}
    fn write_16(&self,s:&str) { print!("{}",White.bold().paint(s)) }
    fn write_17(&self,s:&str) { print!("{}",s) }
    fn write_18(&self,s:&str) { }
    fn write_neg_1(&self,s:&str) { print!("{}",Black.on(Blue).paint(s)) }
    fn write_other(&self,s:&str) { print!("{}",Black.on(Green).paint(s)) }
    fn file_clopen(&self, s: &str) { print!("{}",s) }
    fn message(&self,s:&str) { print!("{}",Yellow.paint(s)) }
}
pub struct NoOutput {}
impl InterpreterParams for NoOutput {
    fn singlethreaded(&self) -> bool { false}
    fn do_log(&self) -> bool { false }
    fn set_log(&mut self,b: bool) {}
    fn store_in_file(&self) -> bool { false }
    fn copy_tokens_full(&self) -> bool { true }
    fn copy_commands_full(&self) -> bool { true }
    fn log(&self,s:&str) {}
    fn write_16(&self,s:&str) {}
    fn write_17(&self,s:&str) {}
    fn write_18(&self,s:&str) {}
    fn write_neg_1(&self,s:&str) {}
    fn write_other(&self,s:&str) {}
    fn file_clopen(&self, s: &str) {}
    fn message(&self, s: &str) {}
}