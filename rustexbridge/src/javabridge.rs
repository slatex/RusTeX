use std::path::Path;
use rustex::interpreter::state::State;

pub static mut MAIN_STATE : Option<State> = None;

use jni::JNIEnv;
use jni::objects::{JClass, JList, JObject, JString, JValue};
use jni::sys::{jarray, jboolean, jobjectArray, jsize, jstring};
use rustex::interpreter::Interpreter;
use rustex::interpreter::params::{CommandListener, DefaultParams, InterpreterParams};
use rustex::stomach::html::HTMLColon;
use rustex::utils::TeXError;


#[no_mangle]
pub extern "system" fn Java_info_kwarc_rustex_Bridge_initialize(
    _env: JNIEnv,
    _class: JClass
) -> jboolean {
    unsafe {
        match MAIN_STATE {
            Some(_) => (),
            None => {
                let state = State::pdf_latex();
                MAIN_STATE = Some(state);
            }
        }
    }
    jboolean::from(true)
}

#[macro_export]
macro_rules! javastring {
    ($env:expr,$s:expr) => ($env.new_string($s).expect("Couldn't create java string!"))
}
#[macro_export]
macro_rules! jobj {
    ($s:expr) => (JValue::Object(JObject::from(($s))))
}
#[macro_export]
macro_rules! jarray {
    ($env:expr,$v:expr,$clsstr:expr,$init:expr,$sym:ident => $tdo:expr) => ({
        let mut _i:u16 = 0;
        let _ret = $env.new_object_array(jsize::from($v.len() as u16),$clsstr,$init).unwrap();
        for $sym in $v {
            $env.set_object_array_element(_ret,jsize::from(_i),$tdo);
            _i += 1;
        }
        _ret
    })
}

#[no_mangle]
pub extern "system" fn Java_info_kwarc_rustex_Bridge_parse(
    env: JNIEnv,
    _class: JClass,
    file:JString,params:JObject,memory_j:JObject
) -> jstring {
    let state = unsafe { MAIN_STATE.as_ref().unwrap().clone() };
    let filename : String = env
        .get_string(file)
        .expect("Couldn't get java string!")
        .into();
    let p = JavaParams::new(&env,params);
    let mut memories : Vec<String> = vec!();
    for m in JList::from_env(&env,memory_j).unwrap().iter().unwrap() {
        memories.push(env.get_string(JString::from(m)).unwrap().into())
    }
    let (s,ret) = Interpreter::do_file_with_state(Path::new(&filename),state,HTMLColon::new(true),&p);
    let mut topcommands = Box::new(s.commands);
    loop {
        match topcommands.parent {
            Some(p) => topcommands = p,
            _ => break
        }
    }
    for (n,cmd) in topcommands.values.unwrap() {
        if memories.iter().any(|x| n.to_string().starts_with(x) ) {
            unsafe { MAIN_STATE.as_mut().unwrap().commands.set(n,cmd,true);}
        }
    }
    javastring!(env,ret).into_inner()
}

struct JavaParams<'borrow,'env> {
    env:&'borrow JNIEnv<'env>,
    params:JObject<'env>,
    singlethreaded:bool,
    do_log:bool,
    store_in_file:bool,
    copy_tokens_full:bool,
    copy_commands_full:bool,
    pub listeners: Vec<Box<dyn CommandListener>>
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
            listeners: DefaultParams::default_listeners()
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
        let output = jobj!(javastring!(self.env,s));
        self.env.call_method(self.params,"log","(Ljava/lang/String;)V",&[output]).unwrap();
    }
    fn write_16(&self, s: &str) {
        let output = jobj!(javastring!(self.env,s));
        self.env.call_method(self.params,"write_16","(Ljava/lang/String;)V",&[output]).unwrap();
    }
    fn write_17(&self, s: &str) {
        let output = jobj!(javastring!(self.env,s));
        self.env.call_method(self.params,"write_17","(Ljava/lang/String;)V",&[output]).unwrap();
    }
    fn write_18(&self, s: &str) {
        let output = jobj!(javastring!(self.env,s));
        self.env.call_method(self.params,"write_18","(Ljava/lang/String;)V",&[output]).unwrap();
    }
    fn write_neg_1(&self, s: &str) {
        let output = jobj!(javastring!(self.env,s));
        self.env.call_method(self.params,"write_neg_1","(Ljava/lang/String;)V",&[output]).unwrap();
    }
    fn write_other(&self, s: &str) {
        let output = jobj!(javastring!(self.env,s));
        self.env.call_method(self.params,"write_other","(Ljava/lang/String;)V",&[output]).unwrap();
    }
    fn file_open(&self, s: &str) {
        let output = jobj!(javastring!(self.env,s));
        self.env.call_method(self.params,"file_open","(Ljava/lang/String;)V",&[output]).unwrap();
    }
    fn message(&self, s: &str) {
        let output = jobj!(javastring!(self.env,s));
        self.env.call_method(self.params,"message","(Ljava/lang/String;)V",&[output]).unwrap();
    }
    fn file_close(&self) {
        self.env.call_method(self.params,"file_close","()V",&[]).unwrap();
    }
    fn error(&self, t: TeXError) {
        let a1 = jobj!(javastring!(self.env,t.msg));

        let emptystr = javastring!(self.env,"");
        let emptyvec = jarray!(self.env,vec!("".to_string(),"".to_string()),"Ljava/lang/String;",emptystr,
            e => javastring!(self.env,e)
        );
        let retstr = jarray!(self.env,t.textrace,"[Ljava/lang/String;",emptyvec,p => {
            let v = vec!(p.0,p.1);
            jarray!(self.env,v,"Ljava/lang/String;",emptystr,s => javastring!(self.env,s))
        });
        let filestr = jarray!(self.env,t.toplinepos,"[Ljava/lang/String;",emptyvec,p => {
            let v = vec!(p.0,p.1.to_string(),p.2.to_string());
            jarray!(self.env,v,"Ljava/lang/String;",emptystr,s => javastring!(self.env,s))
        });

        self.env.call_method(self.params,"error_i","(Ljava/lang/String;[[Ljava/lang/String;[[Ljava/lang/String;)V",
                             &[a1,JValue::Object(JObject::from(retstr)),JValue::Object(JObject::from(filestr))]).unwrap();
    }
    fn command_listeners(&self) -> &Vec<Box<dyn CommandListener>> {
        &self.listeners
    }
}