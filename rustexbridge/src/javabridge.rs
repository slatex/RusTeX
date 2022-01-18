use std::path::Path;
use rustex::interpreter::state::State;

pub static mut MAIN_STATE : Option<State> = None;

use jni::JNIEnv;
use jni::objects::{GlobalRef, JClass, JList, JObject, JString, JValue};
use jni::sys::{jboolean, jbyteArray, jint, jlong, jstring};
use rustex::interpreter::Interpreter;
use rustex::interpreter::params::{DefaultParams, InterpreterParams};
use rustex::stomach::html::HTMLColon;


#[no_mangle]
pub extern "system" fn Java_info_kwarc_rustex_Bridge_initialize(
    _env: JNIEnv,
    _class: JClass
) -> jboolean {
    unsafe {
        match MAIN_STATE {
            Some(_) => (),
            None => {
                use rustex::interpreter::state::default_pdf_latex_state;
                let state = default_pdf_latex_state();
                MAIN_STATE = Some(state);
            }
        }
    }
    jboolean::from(true)
}

#[no_mangle]
pub extern "system" fn Java_info_kwarc_rustex_Bridge_parse(
    env: JNIEnv,
    _class: JClass,
    file:JString,params:JObject//,memory_j:JObject
) -> jstring {
    let state = unsafe { MAIN_STATE.as_ref().unwrap().clone() };
    let filename : String = env
        .get_string(file)
        .expect("Couldn't get java string!")
        .into();
    let mut p = JavaParams::new(&env,params);
    /*let mut memories : Vec<String> = vec!();
    for m in JList::from_env(&env,memory_j).unwrap().iter().unwrap() {
        memories.push(env.get_string(JString::from(m)).unwrap().into())
    }*/
    let (mut s,ret) = Interpreter::do_file_with_state(Path::new(&filename),state,HTMLColon::new(true),&p);
    /*
    let frame = s.stacks.remove(0);
    for (n,cmd) in frame.commands {
        if memories.iter().any(|x| n.to_string().starts_with(x) ) {
            unsafe { MAIN_STATE.as_mut().unwrap().stacks.first_mut().unwrap().commands.insert(n,cmd);}
        }
    }*/
    let output = env
        .new_string(ret)
        .expect("Couldn't create java string!");
    // Finally, extract the raw pointer to return.
    output.into_inner()
}

struct JavaParams<'borrow,'env> {
    env:&'borrow JNIEnv<'env>,
    params:JObject<'env>,
    singlethreaded:bool,
    do_log:bool,
    store_in_file:bool,
    copy_tokens_full:bool,
    copy_commands_full:bool
}
impl<'borrow,'env> JavaParams<'borrow,'env> {
    pub fn new(env:&'borrow JNIEnv<'env>,params:JObject<'env>) -> JavaParams<'borrow,'env> {
        JavaParams {
            env,params,
            singlethreaded:env.get_field(params,"singlethreaded","Z").unwrap().z().unwrap(),
            do_log:env.get_field(params,"do_log","Z").unwrap().z().unwrap(),
            store_in_file:env.get_field(params,"store_in_file","Z").unwrap().z().unwrap(),
            copy_tokens_full:env.get_field(params,"copy_tokens_full","Z").unwrap().z().unwrap(),
            copy_commands_full:env.get_field(params,"copy_commands_full","Z").unwrap().z().unwrap(),
        }
    }
}
impl<'borrow,'env> InterpreterParams for JavaParams<'borrow,'env> {
    fn singlethreaded(&self) -> bool { self.singlethreaded }
    fn do_log(&self) -> bool { self.do_log }
    fn set_log(&mut self, b: bool) {
        self.env.set_field(self.params,"do_log","Z",JValue::Bool(b.into())).unwrap();
        self.do_log = b
    }
    fn store_in_file(&self) -> bool { self.store_in_file }
    fn copy_tokens_full(&self) -> bool { self.copy_tokens_full }
    fn copy_commands_full(&self) -> bool { self.copy_commands_full }
    fn log(&self, s: &str) {
        let output = JValue::Object(JObject::from(self.env
            .new_string(s)
            .expect("Couldn't create java string!").into_inner()));
        self.env.call_method(self.params,"log","(Ljava/lang/String;)V",&[output]);
    }
    fn write_16(&self, s: &str) {
        let output = JValue::Object(JObject::from(self.env
            .new_string(s)
            .expect("Couldn't create java string!").into_inner()));
        self.env.call_method(self.params,"write_16","(Ljava/lang/String;)V",&[output]);
    }
    fn write_17(&self, s: &str) {
        let output = JValue::Object(JObject::from(self.env
            .new_string(s)
            .expect("Couldn't create java string!").into_inner()));
        self.env.call_method(self.params,"write_17","(Ljava/lang/String;)V",&[output]);
    }
    fn write_18(&self, s: &str) {
        let output = JValue::Object(JObject::from(self.env
            .new_string(s)
            .expect("Couldn't create java string!").into_inner()));
        self.env.call_method(self.params,"write_18","(Ljava/lang/String;)V",&[output]);
    }
    fn write_neg_1(&self, s: &str) {
        let output = JValue::Object(JObject::from(self.env
            .new_string(s)
            .expect("Couldn't create java string!").into_inner()));
        self.env.call_method(self.params,"write_neg_1","(Ljava/lang/String;)V",&[output]);
    }
    fn write_other(&self, s: &str) {
        let output = JValue::Object(JObject::from(self.env
            .new_string(s)
            .expect("Couldn't create java string!").into_inner()));
        self.env.call_method(self.params,"write_other","(Ljava/lang/String;)V",&[output]);
    }
    fn file_clopen(&self, s: &str) {
        let output = JValue::Object(JObject::from(self.env
            .new_string(s)
            .expect("Couldn't create java string!").into_inner()));
        self.env.call_method(self.params,"file_clopen","(Ljava/lang/String;)V",&[output]);
    }
    fn message(&self, s: &str) {
        let output = JValue::Object(JObject::from(self.env
            .new_string(s)
            .expect("Couldn't create java string!").into_inner()));
        self.env.call_method(self.params,"message","(Ljava/lang/String;)V",&[output]);
    }
}