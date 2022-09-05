use rustex::interpreter::params::{CommandListener, DefaultParams, InterpreterParams};
use jni::JNIEnv;
use jni::objects::{JObject,JValue};
use rustex::utils::TeXError;

pub (in crate) struct JavaParams<'borrow,'env> {
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

use crate::{javastring,jobj,jarray};

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