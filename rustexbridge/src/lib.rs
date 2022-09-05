mod util;
mod javaparams;

#[cfg(test)]
mod tests {
    #[test]
    fn it_works() {
        let result = 2 + 2;
        assert_eq!(result, 4);
    }
}

use rustex::interpreter::state::State;
use jni::JNIEnv;
use jni::objects::{JClass, JObject, JString, JValue};
use jni::sys::{jarray, jboolean, jlong, jstring};

pub static mut MAIN_STATE : Option<State> = None;

#[no_mangle]
pub extern "system" fn Java_info_kwarc_rustex_RusTeXBridge_initializeMain(
    env: JNIEnv,
    _class: JClass,
    path:JString
) -> jboolean {
    unsafe {
        match MAIN_STATE {
            Some(_) => (),
            None => {
                let state = State::pdf_latex();
                MAIN_STATE = Some(state);
                rustex::PDFIUM_PATH = Some(env.get_string(path).expect("Couldn't get java string!").into())
            }
        }
    }
    jboolean::from(true)
}

struct Sandbox(State);

impl Sandbox {
    pub(in crate) fn box_object(self) -> jlong {
        let this  = Box::new(self);
        let this: *mut Sandbox = Box::into_raw(this);
        this as jlong
    }
    pub(in crate) fn from_pointer(pt : jlong) -> Sandbox {
        let r = pt as *mut Sandbox;
        let r = unsafe{ r.read() };
        r
    }
    pub(in crate) fn set_pointer(self,env: JNIEnv,cls: JClass) {
        let ptr = self.box_object();
        env.set_field(cls,"ptr","J",JValue::Long(ptr))
            .unwrap();
    }
}

#[no_mangle]
pub extern "system" fn Java_info_kwarc_rustex_RusTeXBridge_newsb(
    env: JNIEnv,
    cls: JClass
) {
    let state = unsafe { MAIN_STATE.as_ref().unwrap().clone() };
    Sandbox(state).set_pointer(env,cls)
}

use crate::javaparams::JavaParams;

#[no_mangle]
pub extern "system" fn Java_info_kwarc_rustex_RusTeXBridge_parseI(
    env: JNIEnv,
    cls: JClass,ptr:jlong,p:JObject,file:JString, memory_j:JObject,use_main:jboolean) -> jstring {
    let Sandbox(mut state) = Sandbox::from_pointer(ptr);
    let st = if use_main == 1 {
        unsafe{MAIN_STATE.as_mut().unwrap()}.clone()
    } else {
        state.clone()
    };
    let memories = util::mems_from_java(&env,memory_j);
    let (s,ret) = util::do_file(env, file, st, &JavaParams::new(&env, p));
    if use_main == 1 {
        util::do_memories(unsafe{MAIN_STATE.as_mut().unwrap()}, s, &memories)
    } else {
        util::do_memories(&mut state, s, &memories)
    };
    Sandbox(state).set_pointer(env,cls);
    ret
}

#[no_mangle]
pub extern "system" fn Java_info_kwarc_rustex_RusTeXBridge_parseStringI(
    env: JNIEnv,
    cls: JClass,ptr:jlong,text:JString,p:JObject,file:JString, memory_j:JObject,use_main:jboolean) -> jstring {
    let Sandbox(mut state) = Sandbox::from_pointer(ptr);
    let st = if use_main == 1 {
        unsafe{MAIN_STATE.as_mut().unwrap()}.clone()
    } else {
        state.clone()
    };
    let memories = util::mems_from_java(&env,memory_j);
    let (s,ret) = util::do_string(env, file, text, st, &JavaParams::new(&env, p));
    if use_main == 1 {
        util::do_memories(unsafe{MAIN_STATE.as_mut().unwrap()}, s, &memories)
    } else {
        util::do_memories(&mut state, s, &memories)
    };
    Sandbox(state).set_pointer(env,cls);
    ret
}