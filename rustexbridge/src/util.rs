#[macro_export]
macro_rules! javastring {
    ($env:expr,$s:expr) => ($env.new_string($s).expect("Couldn't create java string!"))
}
#[macro_export]
macro_rules! jobj {
    ($s:expr) => (jni::objects::JValue::Object(JObject::from(($s))))
}
#[macro_export]
macro_rules! jarray {
    ($env:expr,$v:expr,$clsstr:expr,$init:expr,$sym:ident => $tdo:expr) => ({
        let mut _i:u16 = 0;
        let _ret = $env.new_object_array(jni::sys::jsize::from($v.len() as u16),$clsstr,$init).unwrap();
        for $sym in $v {
            $env.set_object_array_element(_ret,jni::sys::jsize::from(_i),$tdo).unwrap();
            _i += 1;
        }
        _ret
    })
}

use jni::JNIEnv;
use jni::objects::{JList, JMap, JObject, JString};
use jni::sys::jstring;

use rustex::interpreter::state::State;
use rustex::interpreter::Interpreter;
use rustex::stomach::html::HTMLColon;
use std::path::Path;

use crate::javaparams::JavaParams;

pub(in crate) fn do_file<'borrow,'env>(env:JNIEnv, file:JString, s:State, params:&JavaParams<'borrow,'env>)
                                       -> (bool,State,jstring) {
    let filename : String = env
        .get_string(file)
        .expect("Couldn't get java string!")
        .into();
    let (b,s,ret) = Interpreter::do_file_with_state(
        Path::new(&filename),
        s,HTMLColon::new(true),params);
    (b,s,javastring!(env,ret).into_inner())
}

pub(in crate) fn do_string<'borrow,'env>(env:JNIEnv, file:JString, text:JString, s:State, params:&JavaParams<'borrow,'env>)
                                         -> (bool,State,jstring) {
    let filename : String = env
        .get_string(file)
        .expect("Couldn't get java string!")
        .into();
    let parsetext : String =  env
        .get_string(text)
        .expect("Couldn't get java string!")
        .into();
    let (b,s,ret) = Interpreter::do_string_with_state(
        Path::new(&filename), s,parsetext.as_str(),
        HTMLColon::new(true),params);
    (b,s,javastring!(env,ret).into_inner())
}

pub(in crate) fn do_memories(old:&mut State, new:State, memories:&Vec<String>) {
    let mut topcommands = new.commands.destroy();
    for (n,cmd) in topcommands.drain() {
        if memories.iter().any(|x| n.to_string().starts_with(x) ) {
            old.commands.set(n,match cmd {
                None => None,
                Some(c) => Some(c.clean())
            },true);
        }
    }
}

pub(in crate) fn mems_from_java(env: &JNIEnv, memory_j:JObject) -> Vec<String> {
    let mut memories : Vec<String> = vec!();
    for m in JList::from_env(env,memory_j).unwrap().iter().unwrap() {
        memories.push(env.get_string(JString::from(m)).unwrap().into())
    }
    memories
}

pub(in crate) fn envs_from_java(env: &JNIEnv,envstrs_j:JObject) {
    for (k,v) in JMap::from_env(env,envstrs_j).unwrap().iter().unwrap() {
        let key : String = env.get_string(JString::from(k)).unwrap().into();
        let value: String = env.get_string(JString::from(v)).unwrap().into();
        std::env::set_var(key,value);
    }
}