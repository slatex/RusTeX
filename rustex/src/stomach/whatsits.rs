use crate::interpreter::Interpreter;
use crate::utils::TeXError;

pub struct ExecutableWhatsit {
    pub _apply : Box<dyn FnOnce(&Interpreter) -> Result<(),TeXError>>
}
pub enum Whatsit {
    Exec(ExecutableWhatsit)
}