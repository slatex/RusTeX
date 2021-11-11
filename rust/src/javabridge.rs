
use robusta_jni::bridge;

#[bridge]
pub mod java {
    use robusta_jni::convert::Signature;
    use crate::commands::bridge::JExecutable;
    use crate::commands::TeXCommand;
    use crate::interpreter::state::State;
    use crate::utils::{kpsewhich,PWD};
    use crate::interpreter::Interpreter;
    use robusta_jni::jni::JNIEnv;

    #[derive(Signature)]
    #[package(com.jazzpirate.rustex.bridge)]
    struct Bridge {}
    impl Bridge {
        pub extern "jni" fn test<'env,'borrow>(env: &'borrow JNIEnv<'env>,mut vec: Vec<JExecutable<'env,'borrow>>) -> bool {
            use std::rc::Rc;
            let mut nvec : Vec<TeXCommand> = Vec::new();
            while !vec.is_empty() {
                nvec.push(TeXCommand::Java(Rc::new(vec.pop().unwrap())))
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