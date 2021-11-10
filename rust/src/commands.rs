pub mod primitives;
pub mod etex;
pub mod pdftex;

use crate::ontology::{Command, Expansion, Token};
use crate::interpreter::Interpreter;
use std::rc::Rc;
use crate::references::SourceReference;
use std::fmt;
use std::fmt::Formatter;

pub struct PrimitiveExecutable {
    pub apply:fn(cs:Rc<Command>,itp:&Interpreter) -> Expansion,
    pub expandable : bool,
    pub name: &'static str
}
impl PartialEq for PrimitiveExecutable {
    fn eq(&self, other: &Self) -> bool {
        self.name == other.name
    }
}
#[derive(PartialEq)]
pub struct RegisterReference {
    pub index: i8,
    pub name: &'static str
}

#[derive(PartialEq)]
pub struct DimenReference {
    pub index: i8,
    pub name: &'static str
}

use robusta_jni::bridge;
#[bridge]
pub mod bridge {
    use robusta_jni::convert::{Signature, IntoJavaValue, FromJavaValue, TryIntoJavaValue, TryFromJavaValue, Field};
    use robusta_jni::jni::objects::AutoLocal;
    use crate::interpreter::Interpreter;
    use robusta_jni::jni::errors::Result as JniResult;
    use robusta_jni::jni::JNIEnv;

    #[derive(Signature, TryIntoJavaValue, IntoJavaValue, TryFromJavaValue)]
    #[package(com.jazzpirate.rustex.bridge)]
    pub struct JExecutable<'env: 'borrow, 'borrow> {
        #[instance]
        raw: AutoLocal<'env, 'borrow>,

        pub name: String
    }

    impl<'env,'borrow> JExecutable<'env,'borrow> {
        pub extern "java" fn execute(&self,env: &'borrow JNIEnv<'env>) -> JniResult<bool> {}
    }
    impl<'env,'borrow>PartialEq for JExecutable<'env,'borrow> {
        fn eq(&self, other: &Self) -> bool {
            other.name == self.name
        }
    }
}

use bridge::JExecutable;

#[derive(Copy,Clone,PartialEq)]
pub enum TeXCommand {
    Primitive(&'static PrimitiveExecutable),
    Register(&'static RegisterReference),
    Dimen(&'static DimenReference),
    Java(&'static JExecutable<'static,'static>),
    Def
}
impl fmt::Display for TeXCommand {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        match self {
            TeXCommand::Primitive(p) =>
                write!(f,"{}",p.name),
            _ => todo!("commands.rs 27")
        }
    }
}
impl TeXCommand {
    pub fn defmacro(tks : Vec<Token>,source:Rc<Token>,protected:bool) -> TeXCommand {
        todo!("commands.rs 33")
    }
    pub fn name(&self) -> &'static str {
        match self {
            TeXCommand::Primitive(pr) => pr.name,
            TeXCommand::Register(reg) => reg.name,
            TeXCommand::Dimen(dr) => dr.name,
            TeXCommand::Java(jr) => jr.name.as_str(),
            TeXCommand::Def => todo!()
        }
    }
}