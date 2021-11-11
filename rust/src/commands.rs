pub mod primitives;
pub mod etex;
pub mod pdftex;

use crate::ontology::{Command, Expansion, Token};
use crate::interpreter::Interpreter;
use std::rc::Rc;
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

// -------------------------------------------------------------------------------------------------

use robusta_jni::bridge;
#[bridge]
pub mod bridge {
    use robusta_jni::convert::{Signature, IntoJavaValue, TryIntoJavaValue, TryFromJavaValue, Field};
    use robusta_jni::jni::objects::AutoLocal;
    use robusta_jni::jni::errors::Result as JniResult;
    use robusta_jni::jni::JNIEnv;
    use crate::interpreter::bridge::JInterpreter;

    #[derive(Signature, TryIntoJavaValue, IntoJavaValue, TryFromJavaValue)]
    #[package(com.jazzpirate.rustex.bridge)]
    pub struct JExecutable<'env: 'borrow, 'borrow> {
        #[instance]
        raw: AutoLocal<'env, 'borrow>,
        #[field] pub name: Field<'env, 'borrow, String>
    }

    impl<'env,'borrow> JExecutable<'env,'borrow> {
        pub extern "java" fn execute(&self,_env: &'borrow JNIEnv<'env>,_int:&JInterpreter) -> JniResult<bool> {}
    }
    impl<'env,'borrow>PartialEq for JExecutable<'env,'borrow> {
        fn eq(&self, other: &Self) -> bool {
            other.name.get().unwrap() == self.name.get().unwrap()
        }
    }
}

use bridge::JExecutable;

// -------------------------------------------------------------------------------------------------

//pub trait ExternalCommand {}

#[derive(Clone,PartialEq)]
pub enum TeXCommand<'env,'borrow> {
    Primitive(&'static PrimitiveExecutable),
    Register(&'static RegisterReference),
    Dimen(&'static DimenReference),
    Java(Rc<JExecutable<'env,'borrow>>),
    Def
}

impl<'env,'borrow> fmt::Display for TeXCommand<'env,'borrow> {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        match self {
            TeXCommand::Primitive(p) =>
                write!(f,"{}",p.name),
            _ => todo!("commands.rs 27")
        }
    }
}
impl<'env,'borrow> TeXCommand<'env,'borrow> {
    pub fn defmacro<'a>(_tks : Vec<Token>,_source:Rc<Token>,_protected:bool) -> TeXCommand<'a,'a> {
        todo!("commands.rs 33")
    }
    pub fn name(&self) -> String {
        match self {
            TeXCommand::Primitive(pr) => pr.name.to_string(),
            TeXCommand::Register(reg) => reg.name.to_string(),
            TeXCommand::Dimen(dr) => dr.name.to_string(),
            TeXCommand::Java(jr) => jr.name.get().unwrap(),
            TeXCommand::Def => todo!()
        }
    }
}