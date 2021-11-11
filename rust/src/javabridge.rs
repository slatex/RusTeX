use robusta_jni::bridge;
use crate::javabridge::java::{JExecutable, JInterpreter};

#[bridge]
pub mod java {
    use crate::commands::TeXCommand;
    use crate::interpreter::state::State;
    use crate::utils::{kpsewhich,PWD};
    use crate::interpreter::Interpreter;
    use robusta_jni::jni::objects::AutoLocal;
    use robusta_jni::jni::errors::Result as JniResult;
    use robusta_jni::jni::JNIEnv;
    use crate::commands::ExternalCommand;
    use robusta_jni::convert::{Signature, IntoJavaValue, FromJavaValue, TryIntoJavaValue, TryFromJavaValue, Field};
    use crate::javabridge::JavaCommand;

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
        fn getInt(&self) -> &mut Interpreter {
            use crate::utils::decode_pointer_mut;
            decode_pointer_mut(self.pointer.get().unwrap())
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
                    je:vec.pop().unwrap(),
                    env
                };
                nvec.push(TeXCommand::Ext(Rc::new(je)))
            }
            let mut st = State::with_commands(nvec);
            let pdftex_cfg = kpsewhich("pdftexconfig.tex",&PWD).expect("pdftexconfig.tex not found");
            let latex_ltx = kpsewhich("latex.ltx",&PWD).expect("No latex.ltx found");

            println!("{}",pdftex_cfg.to_str().expect("wut"));
            println!("{}",latex_ltx.to_str().expect("wut"));
            st = Interpreter::do_file_with_state(&pdftex_cfg,st,Some(env));
            st = Interpreter::do_file_with_state(&latex_ltx,st,Some(env));
            true
        }
    }
}

use robusta_jni::jni::JNIEnv;
use crate::commands::ExternalCommand;
use crate::interpreter::Interpreter;

struct JavaCommand<'env,'borrow> {
    pub(in crate::javabridge) je : JExecutable<'env,'borrow>,
    pub(in crate::javabridge) env: &'borrow JNIEnv<'env>
}
impl<'env,'borrow> ExternalCommand for JavaCommand<'env,'borrow> {
    fn name(&self) -> String {
        self.je.name.get().unwrap()
    }

    fn execute(&self, int: &mut Interpreter) -> bool {
        use crate::utils::encode_pointer_mut;
        let mut ji = JInterpreter::new(self.env).unwrap();
        ji.pointer.set(encode_pointer_mut(int));
        self.je.execute(self.env,&ji).unwrap()
    }
}