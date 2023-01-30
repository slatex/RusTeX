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

use std::borrow::BorrowMut;
use std::sync::{Mutex, MutexGuard};
use rustex::interpreter::state::State;
use jni::JNIEnv;
use jni::objects::{JClass, JObject, JString, JValue};
use jni::sys::{jarray, jboolean, jlong, jstring};

static mut MAIN_STATE : Mutex<Option<State>> = Mutex::new(None);
static mut ALL_STATES : Vec<Option<Sandbox>> = Vec::new();

#[macro_export]
macro_rules! main_state {
    ($sym:ident => $tdo:expr) => {
        {
            let mut __guard = unsafe{MAIN_STATE.lock()};
            let $sym = &mut *__guard.unwrap();
            $tdo
        }
    };
    () => {
        {
            let mut __guard = unsafe{MAIN_STATE.lock()};
            let __st = __guard.unwrap();
            __st.as_ref().unwrap().clone()
        }
    };
}

fn set_state(st : Sandbox) -> usize {
    unsafe{ALL_STATES.push(Some(st));ALL_STATES.len() - 1}
}
fn get_state(i : usize) -> Sandbox {
    unsafe{std::mem::replace(&mut ALL_STATES[i],None)}.unwrap()
}

#[no_mangle]
pub extern "system" fn Java_info_kwarc_rustex_RusTeXBridge_initializeMain(
    env: JNIEnv,
    _class: JClass,
    path:JString
) -> jboolean {
    main_state!(st => {
        match st {
            Some(_) => (),
            None => {
                let state = State::pdf_latex();
                *st = Some(state);
                unsafe{ rustex::PDFIUM_PATH = Some(env.get_string(path).expect("Couldn't get java string!").into()) }
            }
        }
    });
    jboolean::from(true)
}

struct Sandbox(State);

impl Sandbox {
    pub(in crate) fn box_object(self) -> jlong {
        /*let this  = Box::new(self);
        let this: *mut Sandbox = Box::into_raw(this);
        this as jlong */
        set_state(self) as i64
    }
    pub(in crate) fn from_pointer(pt : jlong) -> Sandbox {
        /*let ret = unsafe{ Box::from_raw(pt as *mut Sandbox) };
        *ret*/
        get_state(pt as usize)
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
    let state = main_state!();
    Sandbox(state).set_pointer(env,cls)
}

use crate::javaparams::JavaParams;

#[no_mangle]
pub extern "system" fn Java_info_kwarc_rustex_RusTeXBridge_parseI(
    env: JNIEnv,
    cls: JClass,ptr:jlong,p:JObject,file:JString, memory_j:JObject,envstrs_j:JObject,use_main:jboolean) -> jstring {
    util::envs_from_java(&env,envstrs_j);
    let memories = util::mems_from_java(&env,memory_j);
    if use_main == 1 {
        let st = main_state!();
        let (b,s,ret) = util::do_file(env, file, st, &JavaParams::new(&env, p));
        if b {
            main_state!(st => {
                util::do_memories(st.as_mut().unwrap(), s, &memories)
            })
        }
        ret
    } else {
        let Sandbox(mut state) = Sandbox::from_pointer(ptr);
        let st = state.clone();
        let (b,s,ret) = util::do_file(env, file, st, &JavaParams::new(&env, p));
        if b {
            main_state!(st => {
                util::do_memories(st.as_mut().unwrap(), s, &memories)
            })
        }
        Sandbox(state).set_pointer(env,cls);
        ret
    }
}

#[no_mangle]
pub extern "system" fn Java_info_kwarc_rustex_RusTeXBridge_parseStringI(
    env: JNIEnv,
    cls: JClass,ptr:jlong,text:JString,p:JObject,file:JString, memory_j:JObject,envstrs_j:JObject,use_main:jboolean) -> jstring {
    util::envs_from_java(&env,envstrs_j);
    let memories = util::mems_from_java(&env,memory_j);
    if use_main == 1 {
        let st = main_state!();
        let (b,s,ret) = util::do_string(env, file, text, st, &JavaParams::new(&env, p));
        if b {
            main_state!(st => {
                util::do_memories(st.as_mut().unwrap(), s, &memories)
            });
        }
        ret
    } else {
        let Sandbox(mut state) = Sandbox::from_pointer(ptr);
        let st = state.clone();
        let (b,s,ret) = util::do_string(env, file, text, st, &JavaParams::new(&env, p));
        if b {
            util::do_memories(&mut state, s, &memories)
        }
        Sandbox(state).set_pointer(env,cls);
        ret
    }
}