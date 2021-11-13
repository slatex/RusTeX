use robusta_jni::bridge;

#[bridge]
pub mod java {
    use rustex::commands::TeXCommand;
    use rustex::interpreter::state::State;
    use rustex::utils::{kpsewhich, PWD};
    use rustex::interpreter::Interpreter;
    use robusta_jni::jni::objects::AutoLocal;
    use robusta_jni::jni::errors::Result as JniResult;
    use robusta_jni::jni::JNIEnv;
    use rustex::commands::ExternalCommand;
    use robusta_jni::convert::{Signature, IntoJavaValue, FromJavaValue, TryIntoJavaValue, TryFromJavaValue, Field};
    use crate::javabridge::{JavaCommand, JCommand};

    #[derive(Signature, TryIntoJavaValue, IntoJavaValue, TryFromJavaValue,FromJavaValue)]
    #[package(com.jazzpirate.rustex.bridge)]
    pub struct JInterpreter<'env: 'borrow, 'borrow> {
        #[instance]
        raw: AutoLocal<'env, 'borrow>,
        #[field] pub pointer: Field<'env, 'borrow, i64>
    }
    impl<'env: 'borrow, 'borrow> JInterpreter<'env,'borrow> {
        #[constructor]
        pub extern "java" fn new(env: &'borrow JNIEnv<'env>) -> JniResult<Self> {}
        fn getInt(&self) -> &Interpreter {
            use rustex::utils::decode_pointer;
            decode_pointer(self.pointer.get().unwrap())
        }


        pub extern "jni" fn jobname(self) -> String {
            let int = self.getInt();
            int.jobinfo.path.file_stem().unwrap().to_str().unwrap().to_string()
        }

    }

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

    #[derive(Signature)]
    #[package(com.jazzpirate.rustex.bridge)]
    struct Bridge {}
    impl Bridge {
        pub extern "jni" fn test<'env,'borrow>(env: &'borrow JNIEnv<'env>,mut vec: Vec<JExecutable<'env,'borrow>>) -> bool {
            use std::rc::Rc;
            let mut nvec : Vec<TeXCommand> = Vec::new();
            while !vec.is_empty() {
                let je = JavaCommand {
                    je:JCommand::Exec(vec.pop().unwrap()),
                    env
                };
                nvec.push(TeXCommand::Ext(Rc::new(je)))
            }
            let mut st = State::with_commands(nvec);
            let pdftex_cfg = kpsewhich("pdftexconfig.tex",&PWD).expect("pdftexconfig.tex not found");
            let latex_ltx = kpsewhich("latex.ltx",&PWD).expect("No latex.ltx found");

            println!("{}",pdftex_cfg.to_str().expect("wut"));
            println!("{}",latex_ltx.to_str().expect("wut"));
            st = Interpreter::do_file_with_state(&pdftex_cfg,st);
            st = Interpreter::do_file_with_state(&latex_ltx,st);
            true
        }
    }
}

use robusta_jni::jni::JNIEnv;
use rustex::commands::ExternalCommand;
use rustex::interpreter::Interpreter;
use crate::javabridge::java::{JExecutable, JInterpreter};

enum JCommand<'env,'borrow> {
    Exec(JExecutable<'env,'borrow>)
}

struct JavaCommand<'env,'borrow> {
    pub je : JCommand<'env,'borrow>,
    pub env: &'borrow JNIEnv<'env>
}

use robusta_jni::jni::errors::Result as JniResult;
use rustex::ontology::Expansion;
use rustex::utils::TeXError;

impl JavaCommand<'_,'_> {
    fn with_int<'a,A>(&self,int:&Interpreter,f : Box<dyn Fn(&JInterpreter) -> JniResult<A> + 'a>) -> A {
        use rustex::utils::encode_pointer;
        let mut ji = JInterpreter::new(self.env).unwrap();
        ji.pointer.set(encode_pointer(int));
        f(&ji).unwrap()
    }
}

impl<'env,'borrow> ExternalCommand for JavaCommand<'env,'borrow> {
    fn expandable(&self) -> bool { match &self.je { _ => false } }
    fn assignable(&self) -> bool { match &self.je { _ => false } }
    fn has_num(&self) -> bool { match &self.je { _ => false } }
    fn name(&self) -> String {
        match &self.je {
            JCommand::Exec(je) => je.name.get().unwrap()
        }
    }
    fn execute(&self, int: &Interpreter) -> Result<(),TeXError> {
        match &self.je {
            JCommand::Exec(e) =>
                match self.with_int(int,Box::new(|i| e.execute(self.env,i))) {
                    true => Ok(()),
                    _ => Err(TeXError::new("Nope".to_string()))
                }
        }
        /*
        use rustex::utils::encode_pointer;
        let mut ji = JInterpreter::new(self.env).unwrap();
        ji.pointer.set(encode_pointer(int));
        self.je.execute(self.env,&ji).unwrap()
         */
    }

    fn expand(&self, int: &Interpreter) -> Result<Expansion, TeXError> {
        match &self.je { _ => Err(TeXError::new("Nope".to_string())) }
    }
    fn assign(&self, int: &Interpreter, global: bool) -> Result<(), TeXError> {
        match &self.je { _ => Err(TeXError::new("Nope".to_string())) }
    }
    fn get_num(&self, int: &Interpreter) -> Result<i32, TeXError> {
        match &self.je { _ => Err(TeXError::new("Nope".to_string())) }
    }
}