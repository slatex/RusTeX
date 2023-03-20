use ahash::RandomState;
use std::collections::{HashMap, VecDeque};
use std::hash::Hash;
use std::marker::PhantomData;
use crate::catcodes::{CategoryCode, CategoryCodeScheme, STARTING_SCHEME};
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
    diffs:Vec<Option<RusTeXMap<K,V>>>
}
impl<K:Eq+Hash+Clone,V:Default+PartialEq+Clone,R:RegisterLike<K,V>> StateStore<K,V,R> {
    pub fn new() -> Self {
        StateStore {store: R::new(), diffs:vec!()}
    }
    pub fn push(&mut self) {self.diffs.push(None)}
    fn insert_opt(&mut self,k:K,v:V) -> Option<V> {
        if v == V::default() {self.store.remove(&k)}
        else {self.store.insert(k,v)}
    }
    pub fn pop(&mut self) { // RusTeXMap::default()
        match unsafe{self.diffs.pop().unwrap_unchecked()} {
            Some(m) =>
                for (k,v) in m {
                    self.insert_opt(k,v);
                },
            _ => ()
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
            Some(Some(old)) if !old.contains_key(&k) => {
                let nold = self.insert_opt(k.clone(),v);
                unsafe{self.diffs.last_mut().unwrap_unchecked().as_mut().unwrap_unchecked()}.insert(k,match nold {
                    None => V::default(),
                    Some(v) => v
                });
            }
            Some(None) => {
                let nold = self.insert_opt(k.clone(),v);
                let mut n : RusTeXMap<K,V> = RusTeXMap::default();
                n.insert(k,match nold {
                    None => V::default(),
                    Some(v) => v
                });
                *unsafe{self.diffs.last_mut().unwrap_unchecked()} = Some(n);
            }
            _ => {self.insert_opt(k,v);}
        }
    }
    pub fn set_globally(&mut self,k:K,v:V) {
        for cc in self.diffs.iter_mut() { cc.as_mut().map(|m| m.remove(&k)); }
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

#[derive(Clone,PartialEq)]
pub struct LinkedValueOpt<V:Default+Clone> {
    v:PhantomData<V>,
    pub ls: VecDeque<Option<V>>
}
impl<V:Default+Clone> LinkedValueOpt<V> {
    pub fn get(&self) -> V {
        for s in self.ls.iter() {
            match s {
                Some(v) => {return v.clone()}
                _ => ()
            }
        }
        V::default()
    }
    fn set_locally(&mut self,v:V) { *self.ls.front_mut().unwrap() = Some(v)}
    fn set_globally(&mut self,v:V) {
        for m in self.ls.iter_mut() {
            *m = None;
        }
        *self.ls.back_mut().unwrap() = Some(v);
    }
    pub fn set(&mut self,v:V,globally:bool) {
        if globally {self.set_globally(v)} else {self.set_locally(v)}
    }
    pub fn push(&mut self) {self.ls.push_front(None)}
    pub fn pop(&mut self) {
        self.ls.pop_front();
    }
}
impl<V:Clone> LinkedValueOpt<Vec<V>> {
    pub fn add(&mut self,b:V) {
        match self.ls.front_mut().unwrap() {
            Some(ref mut v) => v.push(b),
            v => *v = Some(vec!(b))
        }
    }
}
impl<V:Default+Clone> Default for LinkedValueOpt<V> {
    fn default() -> Self {
        let mut ret: Self = LinkedValueOpt{v:PhantomData::default(),ls:VecDeque::new()};
        ret.push();
        ret
    }
}

#[derive(Clone,PartialEq)]
pub struct LinkedValue<V:Default+Clone> {
    v:PhantomData<V>,
    pub ls: VecDeque<V>
}
impl<V:Default+Clone> LinkedValue<V> {
    pub fn get(&self) -> V {
        self.ls.front().unwrap().clone()
    }
    pub fn set_locally(&mut self,v:V) { *self.ls.front_mut().unwrap() = v}
    fn set_globally(&mut self,v:V) {
        for m in self.ls.iter_mut() {
            *m = v.clone();
        }
    }
    pub fn set(&mut self,v:V,globally:bool) {
        if globally {self.set_globally(v)} else {self.set_locally(v)}
    }
    pub fn push_v(&mut self,v:V) {self.ls.push_front(v)}
    pub fn push(&mut self) {self.ls.push_front(self.ls.front().unwrap().clone())}
    pub fn pop(&mut self) {
        self.ls.pop_front();
    }
}
impl<V:Default+Clone> Default for LinkedValue<V> {
    fn default() -> Self {
        let mut ret: Self = LinkedValue{v:PhantomData::default(),ls:VecDeque::new()};
        ret.ls.push_front(V::default());
        ret
    }
}

// Catgeory Codes ------------------------------------------------------------

#[derive(Clone,PartialEq)]
pub struct LinkedCatScheme {
    ls : Vec<(RusTeXMap<u8,CategoryCode>,Option<u8>,Option<u8>,Option<u8>)>,
    scheme:CategoryCodeScheme
}
impl std::default::Default for LinkedCatScheme {
    fn default() -> Self {
        LinkedCatScheme {ls:vec!(),scheme:STARTING_SCHEME.clone()}
    }
}
impl LinkedCatScheme {
    pub fn get_scheme(&self) -> &CategoryCodeScheme {
        &self.scheme
    }
    pub fn push(&mut self) {
        self.ls.push((RusTeXMap::default(),None,None,None))
    }
    pub fn set_newline(&mut self,v:u8,globally:bool) {
        if globally {
            for cc in self.ls.iter_mut() {
                cc.1 = None
            }
        } else {
            match self.ls.last_mut() {
                Some((_,r@None,_,_)) => {*r = Some(self.scheme.newlinechar);}
                _ => {},
            }
        }
        self.scheme.newlinechar = v;
    }
    pub fn set_endline(&mut self,v:u8,globally:bool) {
        if globally {
            for cc in self.ls.iter_mut() {
                cc.2 = None
            }
        } else {
            match self.ls.last_mut() {
                Some((_,_,r@None,_)) => {*r = Some(self.scheme.endlinechar);}
                _ => {},
            }
        }
        self.scheme.endlinechar = v;
    }
    pub fn set_escape(&mut self,v:u8,globally:bool) {
        if globally {
            for cc in self.ls.iter_mut() {
                cc.3 = None
            }
        } else {
            match self.ls.last_mut() {
                Some((_,_,_,r@None)) => {*r = Some(self.scheme.escapechar);}
                _ => {},
            }
        }
        self.scheme.escapechar = v;
    }
    pub fn set(&mut self,k:u8,v: CategoryCode,globally:bool) {
        if globally {self.set_globally(k,v)} else {self.set_locally(k,v)}
    }
    fn set_locally(&mut self,k : u8,v : CategoryCode) {
        match self.ls.last_mut() {
            Some((m,_,_,_)) if !m.contains_key(&k) => {m.insert(k,self.scheme.catcodes[k as usize]);}
            _ => ()
        }
        self.scheme.catcodes[k as usize] = v;
    }
    fn set_globally(&mut self,k : u8,v : CategoryCode) {
        for cc in self.ls.iter_mut() {
            cc.0.remove(&k);
        }
        self.scheme.catcodes[k as usize] = v;
    }
    pub fn pop(&mut self) {
        match self.ls.pop() {
            Some((hm,nl,el,sc)) => {
                for (k,v) in hm {
                    self.scheme.catcodes[k as usize] = v;
                };
                match nl {
                    None => {},
                    Some(v) => self.scheme.newlinechar = v
                }
                match el {
                    None => {},
                    Some(v) => self.scheme.endlinechar = v
                }
                match sc {
                    None => {},
                    Some(v) => self.scheme.escapechar = v
                }
            }
            _ => ()
        }
    }
}
