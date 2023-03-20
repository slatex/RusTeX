use ahash::RandomState;
use std::collections::HashMap;
use std::hash::Hash;
use crate::catcodes::{CategoryCode, CategoryCodeScheme};
use crate::commands::TeXCommand;
use crate::utils::TeXStr;

pub type RusTeXMap<A,B> = HashMap<A,B,RandomState>;

pub trait RegisterLike<K,V:Default> {
    fn new() -> Self;
    fn get_value(&self,k : &K) -> Option<&V>;
    fn remove(&mut self,k:&K) -> Option<V>;
    fn insert(&mut self,k:K,v:V) -> Option<V>;
}

#[derive(Clone)]
pub struct StateStore<K:Eq+Hash+Clone,V:Default+PartialEq+Clone,R:RegisterLike<K,V>> {
    store:R,
    diffs:Vec<RusTeXMap<K,V>>
}
impl<K:Eq+Hash+Clone,V:Default+PartialEq+Clone,R:RegisterLike<K,V>> StateStore<K,V,R> {
    pub fn new() -> Self {
        StateStore {store: R::new(), diffs:vec!()}
    }
    pub fn push(&mut self) {self.diffs.push(RusTeXMap::default())}
    fn insert_opt(&mut self,k:K,v:V) -> Option<V> {
        if v == V::default() {self.store.remove(&k)}
        else {self.store.insert(k,v)}
    }
    pub fn pop(&mut self) {
        for (k,v) in unsafe{self.diffs.pop().unwrap_unchecked()} {
            self.insert_opt(k,v);
        }
    }
    pub fn destroy(self) -> R {self.store}
    pub fn get(&self,k:&K) -> V {
        match self.store.get_value(k) {
            Some(v) => v.clone(),
            None => V::default()
        }
    }
    pub fn set(&mut self,k:K,v:V,globally:bool) {
        if globally { self.set_globally(k,v) }
        else { self.set_locally(k,v) }
    }
    pub fn set_locally(&mut self,k:K,v:V) {
        match self.diffs.last() {
            Some(old) if !old.contains_key(&k) => {
                let nold = self.insert_opt(k.clone(),v);
                unsafe{self.diffs.last_mut().unwrap_unchecked()}.insert(k,match nold {
                    None => V::default(),
                    Some(v) => v
                });
            }
            _ => {self.insert_opt(k,v);}
        }
    }
    pub fn set_globally(&mut self,k:K,v:V) {
        for cc in self.diffs.iter_mut() { cc.remove(&k); }
        self.insert_opt(k,v);
    }
    pub fn take(&mut self,k:K) -> V {
        match self.store.remove(&k) {
            Some(v) => v,
            None => V::default()
        }
    }
}
impl<K:Eq+Hash+Clone,V:Default+PartialEq+Clone,R:RegisterLike<K,V>> Default for StateStore<K,V,R> {
    fn default() -> Self {
        Self::new()
    }
}
#[derive(Clone)]
pub struct PrimStore<V:Default+PartialEq+Clone,const N:usize>([V;N],Vec<V>);
impl<V:Default+PartialEq+Clone,const N:usize> RegisterLike<i32,V> for PrimStore<V,N> {
    fn new() -> Self {PrimStore(array_init::array_init(|_| V::default()),Vec::with_capacity(256))}
    fn get_value(&self, k: &i32) -> Option<&V> {
        if *k < 0 {
            self.0.get((-*k) as usize)
        } else {
            self.1.get(*k as usize)
        }
    }
    fn remove(&mut self, k: &i32) -> Option<V> {
        let ret = if *k < 0 {
            std::mem::take(&mut self.0[(-*k) as usize])
        } else {
            if self.1.len() <= (*k as usize) { return None } else {
                let ret = std::mem::take(&mut self.1[(*k as usize)]);
                /*while self.1.len() > 256 && match self.1.last() {
                    Some(v) if *v == V::default() => true,
                    _ => false
                } {
                    self.1.pop();
                }*/
                ret
            }
        };
        if ret == V::default() { None } else { Some(ret) }
    }
    fn insert(&mut self, k: i32, v: V) -> Option<V> {
        let ret = if k < 0 {
            std::mem::replace(&mut self.0[(-k) as usize],v)
        } else {
            while self.1.len() <= (k as usize) {
                self.1.push(V::default())
            }
            std::mem::replace(&mut self.1[k as usize], v)
        };
        if ret == V::default() { None } else { Some(ret.clone()) }
    }
}

impl<V:Default+PartialEq+Clone> RegisterLike<u16,V> for Vec<V> {
    fn new() -> Self {Vec::with_capacity(256)}
    fn get_value(&self, k: &u16) -> Option<&V> {
        self.get(*k as usize)
    }
    fn remove(&mut self, k: &u16) -> Option<V> {
        if self.len() <= (*k as usize) {None} else {
            let ret = std::mem::take(&mut self[*k as usize]);
            /*while self.len() > 256 && match self.last() {
                Some(v) if *v == V::default() => true,
                _ => false
            } {
                self.pop();
            }*/
            if ret == V::default() {None} else {Some(ret)}
        }
    }
    fn insert(&mut self, k: u16, v: V) -> Option<V> {
        while self.len() <= (k as usize) {
            self.push(V::default())
        }
        let ret = std::mem::replace(&mut self[k as usize],v);
        if ret == V::default() {None} else {Some(ret.clone())}
    }
}
impl<V:Default+PartialEq+Clone,const N:usize> RegisterLike<usize,V> for [V;N] {
    fn new() -> Self { array_init::array_init(|_| V::default()) }
    fn get_value(&self, k: &usize) -> Option<&V> {
        let ret = &self[*k];
        if *ret == V::default() {None} else {Some(ret)}
    }
    fn remove(&mut self, k: &usize) -> Option<V> {
        let ret = std::mem::take(&mut (self[*k]));
        if ret == V::default() {None} else {Some(ret)}
    }
    fn insert(&mut self, k: usize, v: V) -> Option<V> {
        let ret = std::mem::replace(&mut (self[k]),v);
        if ret == V::default() {None} else {Some(ret)}
    }
}

impl<V:Default+PartialEq+Clone+Copy,const N:usize> RegisterLike<u8,V> for [V;N] {
    fn new() -> Self { [V::default(); N] }
    fn get_value(&self, k: &u8) -> Option<&V> {
        let ret = &self[*k as usize];
        if *ret == V::default() {None} else {Some(ret)}
    }
    fn remove(&mut self, k: &u8) -> Option<V> {
        let ret = std::mem::take(&mut (self[*k as usize]));
        if ret == V::default() {None} else {Some(ret)}
    }
    fn insert(&mut self, k: u8, v: V) -> Option<V> {
        let ret = std::mem::replace(&mut (self[k as usize]),v);
        if ret == V::default() {None} else {Some(ret)}
    }
}

impl RegisterLike<TeXStr,Option<TeXCommand>> for RusTeXMap<TeXStr,Option<TeXCommand>> {
    fn new() -> Self {RusTeXMap::default()}
    fn get_value(&self,k: &TeXStr) -> Option<&Option<TeXCommand>> { self.get(k) }
    fn remove(&mut self,k:&TeXStr) -> Option<Option<TeXCommand>> {
        RusTeXMap::remove(self,k)
    }
    fn insert(&mut self,k:TeXStr,v:Option<TeXCommand>) -> Option<Option<TeXCommand>> {
        RusTeXMap::insert(self,k,v)
    }
}