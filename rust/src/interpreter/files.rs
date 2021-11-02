use crate::interpreter::Interpreter;
use crate::utils::FilePath;

enum VFileBase {
    Real(FilePath),
    Virtual(String)
}

pub struct VFile {
    source:VFileBase,
    string: Option<String>
}

impl VFile {
    pub fn new(fp : FilePath, int : &Interpreter) {

    }
}