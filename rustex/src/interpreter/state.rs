pub mod store;

use ahash::RandomState;
use std::collections::{HashMap, VecDeque};
use std::fmt::{Display, Formatter};
use std::hash::Hash;
use std::marker::PhantomData;
use std::ops::Deref;
use std::path::Path;
use std::sync::Arc;
use crate::catcodes::{CategoryCode, CategoryCodeScheme, STARTING_SCHEME};
use crate::commands::TeXCommand;
use crate::fonts::{ArcFont, Font, FontFile, NULL_FONT};
use crate::interpreter::dimensions::{MuSkip, Skip};
use crate::interpreter::mouth::StringMouth;
use crate::interpreter::TeXMode;
use crate::stomach::boxes::{BoxMode, TeXBox};
use crate::stomach::simple::{PDFXForm, PDFXImage};
use crate::stomach::{Stomach, Whatsit};
use crate::{Interpreter, log, TeXErr, TeXString, Token};
use crate::commands::conditionals::conditional_commands;
use crate::commands::pdftex::pdftex_commands;
use crate::commands::pgfsvg::pgf_commands;
use crate::commands::primitives::tex_commands;
use crate::commands::rustex_specials::rustex_special_commands;
use crate::utils::{PWD, TeXError, TeXStr};
use crate::interpreter::files::VFile;
use crate::interpreter::params::{InterpreterParams, NoOutput};
use crate::interpreter::state::store::PrimStore;
use crate::stomach::colon::NoColon;

pub type RusTeXMap<A,B> = HashMap<A,B,RandomState>;

#[derive(Copy,Clone,PartialEq)]
pub enum FontStyle {
    Text,Script,Scriptscript
}
impl FontStyle {
    pub fn inc(&self) -> FontStyle {
        use FontStyle::*;
        match self {
            Text => Script,
            _ => Scriptscript
        }
    }
}

#[derive(Copy,Clone,PartialEq)]
pub enum GroupType {
    Token,
    Begingroup,
    Box(BoxMode),
    Math,
    LeftRight
}
impl Display for GroupType {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(f,"{}",match self {
            GroupType::Token => "{",
            GroupType::Begingroup => "\\begingroup",
            GroupType::Box(_) => "\\box",
            GroupType::Math => "$",
            GroupType::LeftRight => "\\left\\right"
        })
    }
}

pub trait HasDefault {
    fn default() -> Self;
}
impl<A> HasDefault for Option<A> {
    fn default() -> Self { None }
}
impl HasDefault for i32 {
    fn default() -> Self { 0 }
}
impl HasDefault for usize {
    fn default() -> Self { 0 }
}
impl HasDefault for u8 {
    fn default() -> Self { 0 }
}
impl HasDefault for CategoryCode {
    fn default() -> Self { CategoryCode::Other }
}
impl HasDefault for bool {
    fn default() -> Self { false }
}
impl HasDefault for GroupType {
    fn default() -> Self {
        GroupType::Box(BoxMode::V)
    }
}
impl HasDefault for Skip {
    fn default() -> Self {
        Skip {base:0,stretch:None,shrink:None}
    }
}
impl HasDefault for MuSkip {
    fn default() -> Self {
        MuSkip {base:0,stretch:None,shrink:None}
    }
}
impl<A> HasDefault for Vec<A> {
    fn default() -> Self {
        vec!()
    }
}
impl HasDefault for TeXBox {
    fn default() -> Self {
        TeXBox::Void
    }
}
impl HasDefault for ArcFont {
    fn default() -> Self {
        unsafe{NULL_FONT.try_with(|x| x.clone()).unwrap_unchecked()}
    }
}
impl HasDefault for FontStyle {
    fn default() -> Self {
        FontStyle::Text
    }
}

pub trait StateStore<K,V>:Sized {
    fn get(&self,k:&K) -> Option<&V>;
    fn set(&mut self,k:K,v:V);
    fn remove(&mut self,k:&K);
    fn new_store() -> Self;
}
impl<K:Hash+Eq,V> StateStore<K,V> for RusTeXMap<K,V> {
    fn get(&self, k: &K) -> Option<&V> {
        RusTeXMap::get(self,k)
    }
    fn set(&mut self, k: K, v: V) {
        self.insert(k,v);
    }
    fn remove(&mut self, k: &K) {
        RusTeXMap::remove(self,k);
    }
    fn new_store() -> Self {
        RusTeXMap::default()
    }
}
/*
impl StateStore<TeXStr,Option<TeXCommand>> for qp_trie::Trie<Vec<u8>,Option<TeXCommand>> {
    fn get(&self, k: &TeXStr) -> Option<&Option<TeXCommand>> {
        self.get(k.0.as_slice())
    }

    fn set(&mut self, k: TeXStr, v: Option<TeXCommand>) {
        self.insert(k.0.to_vec(),v);
    }

    fn remove(&mut self, k: &TeXStr) {
        self.remove(k.0.as_slice());
    }

    fn new_store() -> Self {
        qp_trie::Trie::new()
    }
}
 */

#[derive(Clone,PartialEq)]
pub struct Var<V>(pub Option<V>) where V:HasDefault;
impl<V:HasDefault> StateStore<(),V> for Var<V> {
    fn get(&self, _k: &()) -> Option<&V> {
        match &self.0 {
            None => None,
            Some(v) => Some(v)
        }
    }
    fn set(&mut self, _k: (), v: V) { self.0 = Some(v) }
    fn remove(&mut self, _k: &()) { self.0 = None }
    fn new_store() -> Self { Var(None) }
}
impl StateStore<usize,Arc<Font>> for [Option<Arc<Font>>;16] {
    fn get(&self, k: &usize) -> Option<&Arc<Font>> { self[*k].as_ref() }
    fn set(&mut self, k: usize, v: Arc<Font>) { self[k] = Some(v) }
    fn remove(&mut self, k: &usize) { self[*k] = None }
    fn new_store() -> Self { newfonts() }
}

fn newfonts() -> [Option<Arc<Font>>;16] {
    [
        None,None,None,None,None,None,None,None,None,None,None,None,None,None,None,None
    ]
}

#[derive(Clone,PartialEq)]
pub struct LinkedStateValue<K,V:HasDefault,A:StateStore<K,V>> {
    k:PhantomData<K>,
    v:PhantomData<V>,
    pub ls : VecDeque<A>
}
impl<K,V:HasDefault+Clone,A:StateStore<K,V>> std::default::Default for LinkedStateValue<K,V,A> {
    fn default() -> Self {
        let mut ret = LinkedStateValue {
            k:PhantomData::default(),
            v:PhantomData::default(),
            ls:VecDeque::new()
        };
        ret.push();
        ret
    }
}

impl<K,V:HasDefault+Clone,A:StateStore<K,V>> LinkedStateValue<K,V,A> {
    pub fn get_maybe(&self,k:&K) -> Option<&V> {
        for store in self.ls.iter() {
            match store.get(k) {
                s@Some(_) => return s,
                _ => {}
            }
        }
        return None
    }
    pub fn get(&self,k:&K) -> V {
        match self.get_maybe(k) {
            Some(s) => s.clone(),
            None => HasDefault::default()
        }
    }
    fn set_locally(&mut self, k : K, v : V) {
        self.ls.front_mut().unwrap().set(k,v);
    }
    fn set_globally(&mut self,k:K,v:V) {
        for m in self.ls.iter_mut() {
            m.remove(&k);
        }
        self.ls.back_mut().unwrap().set(k,v);
    }
    pub fn set(&mut self,k:K,v:V,globally:bool) {
        if globally {self.set_globally(k,v)} else {self.set_locally(k,v)}
    }
    fn push(&mut self) {
        self.ls.push_front(StateStore::new_store())
    }
    fn pop(&mut self) {
        self.ls.pop_front();
    }
}
impl LinkedStateValue<i32,TeXBox,RusTeXMap<i32,TeXBox>> {
    pub fn take(&mut self,k:i32) -> TeXBox {
        for store in self.ls.iter_mut() {
            match store.remove(&k) {
                Some(b) => return b,
                _ => {}
            }
        }
        TeXBox::Void
    }
}
impl <B> LinkedStateValue<(),Vec<B>,Var<Vec<B>>> {
    pub fn add(&mut self,b:B) {
        match self.ls.front_mut().unwrap() {
            Var(Some(ref mut v)) => v.push(b),
            v => *v = Var(Some(vec!(b)))
        }
    }
}

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
    fn push(&mut self) {
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
    fn pop(&mut self) {
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

//type CommandStore = LinkedStateValue<TeXStr,Option<TeXCommand>,RusTeXMap<TeXStr,Option<TeXCommand>>>;
//type CommandStore = LinkedStateValue<TeXStr,Option<TeXCommand>,qp_trie::Trie<Vec<u8>,Option<TeXCommand>>>;

#[derive(Clone,PartialEq)]
pub struct CommandStore {
    ls:Vec<RusTeXMap<TeXStr,Option<Option<TeXCommand>>>>,
    map:RusTeXMap<TeXStr,Option<TeXCommand>>
}
impl Default for CommandStore {
    fn default() -> Self {
        CommandStore {
            ls: vec!(),
            map:RusTeXMap::default()
        }
    }
}
impl CommandStore {
    pub fn destroy(self) -> RusTeXMap<TeXStr,Option<TeXCommand>> {
        self.map
    }
    fn push(&mut self) {
        self.ls.push(RusTeXMap::default())
    }
    pub fn get(&self,k:&TeXStr) -> Option<TeXCommand> {
        match self.map.get(k) {
            None => None,
            Some(s) => s.clone()
        }
    }
    pub fn set(&mut self,k:TeXStr,v: Option<TeXCommand>,globally:bool) {
        if globally {self.set_globally(k,v)} else {self.set_locally(k,v)}
    }
    fn set_locally(&mut self,k:TeXStr,v: Option<TeXCommand>) {
        match self.ls.last_mut() {
            None => {self.map.insert(k.clone(),v);}
            Some(old) => {
                if old.contains_key(&k) {
                    self.map.insert(k.clone(),v);
                } else {
                    let old = self.map.insert(k.clone(),v);
                    unsafe{self.ls.last_mut().unwrap_unchecked()}.insert(k,old);
                }
            }
        };
    }
    fn set_globally(&mut self,k:TeXStr,v: Option<TeXCommand>) {
        for cc in self.ls.iter_mut() {
            cc.remove(&k);
        }
        self.map.insert(k,v);
    }
    fn pop(&mut self) {
        for (k,v) in unsafe{self.ls.pop().unwrap_unchecked()} {
            match v {
                None => self.map.remove(&k),
                Some(v) => self.map.insert(k,v)
            };
        }
    }
}


#[derive(Clone)]
pub struct State {
    pub tp: LinkedStateValue<(),GroupType,Var<GroupType>>,
    pub catcodes:LinkedCatScheme,
    pub commands: store::StateStore<TeXStr,Option<TeXCommand>,store::RusTeXMap<TeXStr,Option<TeXCommand>>>,//CommandStore,
    pub registers: store::StateStore<i32,i32,PrimStore<i32,79>>,//LinkedStateValue<i32,i32,RusTeXMap<i32,i32>>,
    pub dimensions: store::StateStore<i32,i32,PrimStore<i32,34>>,
    pub skips: store::StateStore<i32,Skip,PrimStore<Skip,18>>,
    pub muskips: store::StateStore<i32,MuSkip,PrimStore<MuSkip,4>>,
    pub toks: store::StateStore<i32,Vec<Token>,PrimStore<Vec<Token>,12>>,//LinkedStateValue<i32,Vec<Token>,RusTeXMap<i32,Vec<Token>>>,
    pub boxes: store::StateStore<u16,TeXBox,Vec<TeXBox>>,//LinkedStateValue<i32,TeXBox,RusTeXMap<i32,TeXBox>>,
    pub sfcodes : store::StateStore<u8,i32,[i32;256]>,//LinkedStateValue<u8,i32,RusTeXMap<u8,i32>>,
    pub lccodes : store::StateStore<u8,u8,[u8;256]>,//LinkedStateValue<u8,u8,RusTeXMap<u8,u8>>,
    pub uccodes : store::StateStore<u8,u8,[u8;256]>,//LinkedStateValue<u8,u8,RusTeXMap<u8,u8>>,
    pub mathcodes : store::StateStore<u8,i32,[i32;256]>,//LinkedStateValue<u8,i32,RusTeXMap<u8,i32>>,
    pub delcodes : store::StateStore<u8,i32,[i32;256]>,//LinkedStateValue<u8,i32,RusTeXMap<u8,i32>>,
    pub parshape : LinkedStateValue<(),Vec<(i32,i32)>,Var<Vec<(i32,i32)>>>,
    pub hangindent : LinkedStateValue<(),i32,Var<i32>>,
    pub hangafter : LinkedStateValue<(),usize,Var<usize>>,
    pub(crate) textfonts: store::StateStore<usize,ArcFont,[ArcFont;16]>,//LinkedStateValue<usize,Arc<Font>,[Option<Arc<Font>>;16]>,
    pub(crate) scriptfonts: store::StateStore<usize,ArcFont,[ArcFont;16]>,
    pub(crate) scriptscriptfonts: store::StateStore<usize,ArcFont,[ArcFont;16]>,
    pub(crate) currfont : LinkedStateValue<(),ArcFont,Var<ArcFont>>,
    pub(crate) aftergroups : LinkedStateValue<(),Vec<Token>,Var<Vec<Token>>>,
    pub(crate) fontstyle : LinkedStateValue<(),FontStyle,Var<FontStyle>>,
    pub(crate) displaymode: LinkedStateValue<(),bool,Var<bool>>,

    // DIRECT ------------------------------------------
    pub(in crate) conditions:Vec<Option<bool>>,
    pub(in crate) outfiles:RusTeXMap<u8,Arc<VFile>>,
    pub(in crate) infiles:RusTeXMap<u8,StringMouth>,
    pub(in crate) incs : u8,
    pub(in crate) mode:TeXMode,
    pub(in crate) afterassignment : Option<Token>,
    pub(in crate) pdfmatches : Vec<TeXStr>,
    pub(in crate) pdfcolorstacks: Vec<Vec<TeXStr>>,
    pub(in crate) pdfobjs: RusTeXMap<u16,TeXStr>,
    pub(in crate) pdfxforms: Vec<PDFXForm>,
    pub(in crate) indocument_line:Option<(TeXStr,usize)>,
    pub(in crate) indocument:bool,
    pub(in crate) insetbox:bool,
    pub(in crate) vadjust:Vec<Whatsit>,
    pub (in crate) inserts:RusTeXMap<u16,Vec<Whatsit>>,
    pub(in crate) pagegoal:i32,
    pub(in crate) pdfximages:Vec<PDFXImage>,
    pub(in crate) aligns: Vec<Option<Vec<Token>>>,
    pub(in crate) topmark : Vec<Token>,
    pub(in crate) firstmark : Vec<Token>,
    pub(in crate) botmark : Vec<Token>,
    pub(in crate) splitfirstmark : Vec<Token>,
    pub(in crate) splitbotmark : Vec<Token>,
    // TODO -----------------------------------------
    pub (in crate) filestore:RusTeXMap<TeXStr,Arc<VFile>>,
}

macro_rules! pass_on {
    ($s:tt,$e:ident$(,$tl:expr)*) => {
        $s.tp.$e($(,$tl)*);
        $s.catcodes.$e($(,$tl)*);
        $s.commands.$e($(,$tl)*);
        $s.registers.$e($(,$tl)*);
        $s.dimensions.$e($(,$tl)*);
        $s.skips.$e($(,$tl)*);
        $s.muskips.$e($(,$tl)*);
        $s.toks.$e($(,$tl)*);
        $s.sfcodes.$e($(,$tl)*);
        $s.lccodes.$e($(,$tl)*);
        $s.uccodes.$e($(,$tl)*);
        $s.mathcodes.$e($(,$tl)*);
        $s.delcodes.$e($(,$tl)*);
        $s.boxes.$e($(,$tl)*);
        $s.currfont.$e($(,$tl)*);
        $s.aftergroups.$e($(,$tl)*);
        $s.fontstyle.$e($(,$tl)*);
        $s.textfonts.$e($(,$tl)*);
        $s.scriptfonts.$e($(,$tl)*);
        $s.scriptscriptfonts.$e($(,$tl)*);
        $s.displaymode.$e($(,$tl)*);
        $s.parshape.$e($(,$tl)*);
        $s.hangindent.$e($(,$tl)*);
        $s.hangafter.$e($(,$tl)*);

    }
}
static mut FONT_FILES: Option<RusTeXMap<TeXStr,Arc<FontFile>>> = None;

macro_rules! unwrap {
    ($e:expr) => {
        match $e {
            Some(x) => x,
            None => TeXErr!("No group here to end")
        }
    }
}

impl State {
    pub fn push(&mut self,stomach:&mut dyn Stomach,gt:GroupType) {
        /*if self.stack_depth() > 249 {
            unsafe {crate::LOG = true}
            log!("Here!")
        }*/
        log!("Push: {} -> {}",gt,self.stack_depth() + 1);
        pass_on!(self,push);
        stomach.new_group(gt);
        self.tp.set_locally((),gt)
    }
    pub fn pop(&mut self,tp:GroupType) -> Result<Option<Vec<Token>>,TeXError> {
        log!("Pop: {} -> {}",tp,self.stack_depth());
        match unwrap!(unwrap!(self.tp.ls.front()).0) {
            t if t == tp => (),
            t => TeXErr!("Group opened by {} ended by {}",t,tp)
        }
        let ag = match self.aftergroups.ls.front_mut() {
            Some(ref mut v) => std::mem::take(&mut v.0),
            _ => None
        };
        pass_on!(self,pop);
        Ok(ag)
    }
    pub fn stack_depth(&self) -> usize {
        let mut curr = &self.tp;
        curr.ls.len()
    }
    pub fn new() -> State {
        let mut state = State {
            conditions:vec!(),
            outfiles:RusTeXMap::new_store(),
            infiles:RusTeXMap::new_store(),
            incs:0,
            mode:TeXMode::Vertical,
            afterassignment:None,
            pdfmatches:vec!(),
            pdfcolorstacks:vec!(vec!()),
            pdfobjs:RusTeXMap::new_store(),
            pdfxforms:vec!(),
            indocument_line:None,
            indocument:false,
            insetbox:false,
            vadjust:vec!(),
            inserts:RusTeXMap::new_store(),
            pagegoal:0,
            pdfximages:vec!(),
            aligns:vec!(),
            topmark:vec!(),
            botmark:vec!(),
            firstmark:vec!(),
            splitfirstmark:vec!(),
            splitbotmark:vec!(),
            tp:LinkedStateValue {
                k: PhantomData,
                v: PhantomData,
                ls : VecDeque::new()
            },
            catcodes: LinkedCatScheme::default(),
            commands: Default::default(),
            registers: Default::default(),
            dimensions: Default::default(),
            skips: Default::default(),
            muskips: Default::default(),
            toks: Default::default(),
            sfcodes: Default::default(),
            lccodes: Default::default(),
            uccodes: Default::default(),
            mathcodes: Default::default(),
            delcodes: Default::default(),
            boxes: Default::default(),
            currfont: Default::default(),
            aftergroups: Default::default(),
            fontstyle: Default::default(),
            textfonts: Default::default(),
            scriptfonts: Default::default(),
            scriptscriptfonts: Default::default(),
            displaymode: Default::default(),
            parshape: Default::default(),
            hangindent: Default::default(),
            hangafter: Default::default(),
            // TODO...
            filestore: Default::default(),
            //fontfiles: Default::default()
        };
        for c in conditional_commands() {
            let c = c.as_command();
            state.commands.set_locally(unsafe {c.name().unwrap_unchecked()},Some(c))
        }
        for c in tex_commands() {
            let c = c.as_command();
            state.commands.set_locally(unsafe {c.name().unwrap_unchecked()},Some(c))
        }
        for c in pdftex_commands() {
            let c = c.as_command();
            state.commands.set_locally(unsafe {c.name().unwrap_unchecked()},Some(c))
        }
        state.registers.set_locally(-(crate::commands::registers::MAG.index as i32),1000);
        state.registers.set_locally(-(crate::commands::registers::FAM.index as i32),-1);
        state.dimensions.set_locally(-(crate::commands::registers::PDFPXDIMEN.index as i32),65536);
        for i in 0..=255 {
            state.uccodes.set_locally(i,i);
            state.lccodes.set_locally(i,i);
        }
        for i in 97..123 {
            state.uccodes.set_locally(i,i-32);
            state.lccodes.set_locally(i-32,i);
            state.mathcodes.set_locally(i-32,
                (i as i32-32) +
                    (1 * 16 * 16) +
                    (7 * 16 * 16 * 16)
            );
            state.mathcodes.set_locally(i,
                                        (i as i32) +
                                            (1 * 16 * 16) +
                                            (7 * 16 * 16 * 16)
            );
        }
        for i in 48..58 {
            state.mathcodes.set_locally(i,
                                        (i as i32) +
                                            (0 * 16 * 16) +
                                            (7 * 16 * 16 * 16)
            );
        }
        state
    }
    pub fn pdf_latex() -> State {
        crate::utils::with_stack_size(|| {
            use crate::interpreter::params::DefaultParams;
            let mut state = State::new();
            let pdftex_cfg = crate::kpathsea::kpsewhich("pdftexconfig.tex", &PWD).expect("pdftexconfig.tex not found").0;
            let latex_ltx = crate::kpathsea::kpsewhich("latex.ltx", &PWD).expect("No latex.ltx found").0;
            let p = /* DefaultParams::new(false,false,None); // */ NoOutput::new(None);

            for c in pdftex_commands() {
                let c = c.as_command();
                state.commands.set_locally(unsafe {c.name().unwrap_unchecked()}, Some(c))
            }
            for c in rustex_special_commands() {
                let c = c.as_command();
                state.commands.set_locally(unsafe {c.name().unwrap_unchecked()}, Some(c))
            }

            state = Interpreter::do_file_with_state(&pdftex_cfg, state, NoColon::new(), &p).1;
            state = Interpreter::do_file_with_state(&latex_ltx, state, NoColon::new(), &p).1;
            for c in pgf_commands() {
                let c = c.as_command();
                state.commands.set_locally(unsafe {c.name().unwrap_unchecked()}, Some(c))
            }
            state
        })
    }
    pub fn file_read_line(&mut self,index:u8) -> Result<Vec<Token>,TeXError> {
        match self.infiles.get_mut(&index) {
            None => TeXErr!("No file open at index {}",index),
            Some(fm) => Ok(fm.read_line(self.catcodes.get_scheme()))
        }
    }
    pub fn file_read(&mut self,index:u8,nocomment:bool) -> Result<Vec<Token>,TeXError> {
        //use std::io::BufRead;
        match index {
            255 => {
                TeXErr!("Trying to read from stdin (not supported)")
                /*
                let stdin = std::io::stdin();
                let string = stdin.lock().lines().next().unwrap().unwrap();
                Ok(crate::interpreter::tokenize(string.into(),self.catcodes.get_scheme()))

                 */
            }
            i => {
                match self.infiles.get_mut(&i) {
                    None => TeXErr!("No file open at index {}",i),
                    Some(fm) => Ok(fm.read(self.catcodes.get_scheme(), nocomment))
                }
            }
        }
    }
    pub fn file_eof(&mut self,index:u8) -> Result<bool,TeXError> {
        match self.infiles.get_mut(&index) {
            None => TeXErr!("No file open at index {}",index),
            Some(fm) => {
                Ok(fm.is_eof())
            }
        }
    }
    pub fn file_openin(&mut self,index:u8,file:Arc<VFile>) -> Result<(),TeXError> {
        let mouth = StringMouth::new_from_file(&file,true);
        self.infiles.insert(index,mouth);
        Ok(())
    }
    pub fn file_closein(&mut self,index:u8) -> Result<(),TeXError> {
        match self.infiles.remove(&index) {
            Some(f) => {
                f.source.pop_file().unwrap();
            }
            None => ()
        }
        Ok(())
    }
    pub fn file_openout(&mut self,index:u8,file:Arc<VFile>) -> Result<(),TeXError> {
        file.string.write().unwrap().take();
        self.outfiles.insert(index,file);
        Ok(())
    }
    pub fn file_write(&mut self,index:u8,s:TeXString,params:&dyn InterpreterParams) -> Result<(),TeXError> {
        match index {
            17 => {
                params.write_17(s.to_utf8().as_str());
                Ok(())
            }
            16 => {
                params.write_16(s.to_utf8().as_str());
                Ok(())
            }
            18 => {
                params.write_18(s.to_utf8().as_str());
                Ok(())
            }
            255 => {
                params.write_neg_1(s.to_utf8().as_str());
                Ok(())
            }
            i if !self.outfiles.contains_key(&i) => {
                params.write_other(s.to_utf8().as_str());
                Ok(())
            }
            _ => {
                match self.outfiles.get_mut(&index) {
                    Some(f) => {
                        let mut string = f.string.write().unwrap();
                        match &mut*string {
                            None => {*string = Some(s) },
                            Some(st) => *st += s
                        }
                    }
                    None => TeXErr!("No file open at index {}",index)
                }
                Ok(())
            }
        }
    }
    pub fn get_font(&mut self,indir:&Path,name:TeXStr) -> Result<Arc<FontFile>,TeXError> {
        unsafe {
            match FONT_FILES {
                None => FONT_FILES = Some(RusTeXMap::new_store()),
                _ => ()
            }
            match FONT_FILES.as_ref().unwrap().get(&name) {
                Some(ff) => Ok(Arc::clone(ff)),
                None => {
                    let ret = crate::kpathsea::kpsewhich(std::str::from_utf8_unchecked(name.iter()),indir);
                    match ret {
                        Some((pb,_)) if pb.exists() => {
                            let f = Arc::new(FontFile::new(pb));
                            FONT_FILES.as_mut().unwrap().insert(name, Arc::clone(&f));
                            Ok(f)
                        }
                        _ => {
                            //println!("Here! {}", self.current_line());
                            TeXErr!("Font file {} not found",name)
                        }
                    }
                }
            }
        }
    }
    pub fn file_closeout(&mut self,index:u8) {
        self.outfiles.remove(&index);
    }
}

impl Interpreter<'_> {
    pub fn push_condition(&mut self,cond : Option<bool>) {
        //println!("CONDITION: ({}",self.current_line());
        self.state.conditions.push(cond)
    }
    pub fn pop_condition(&mut self) -> Option<bool> {
        //println!("CONDITION: {})",self.current_line());
        self.state.conditions.pop().unwrap()
    }
    pub fn change_command(&mut self,cmdname:TeXStr,proc:Option<TeXCommand>,globally:bool) {
        /* if cmdname.to_string() == "l_stex_current_symbol_str" {//(cmdname == TeXStr::new(&[0,1,2,3,4,255,254,253,252,251,95])) {//"scr@dte@chapter@init" || cmdname.to_string() == "scr@dte@tocline@init") {
            println!("Here! {}\n{}\n{}",cmdname,proc.as_ref().unwrap().meaning(&crate::catcodes::DEFAULT_SCHEME),self.current_line());
            println!("")
        }*/
        let file = self.current_file();
        let line = self.mouths.current_line();
        for cl in self.params.command_listeners() {
            match cl.apply(&cmdname,&proc,&file,&line) {
                Some(r) => {
                    self.state.commands.set(cmdname,r,globally);
                    return ()
                },
                _ => ()
            }
        }
        self.state.commands.set(cmdname,proc,globally)
    }
    pub fn pop_group(&mut self,tp:GroupType) -> Result<(),TeXError> {
        let ag = self.state.pop(tp)?;
        match ag {
            Some(v) => self.push_tokens(v),
            _ => ()
        }
        self.stomach.close_group()
    }
    pub fn get_whatsit_group(&mut self,tp:GroupType) -> Result<Vec<Whatsit>,TeXError> {
        let ag = self.state.pop(tp)?;
        match ag {
            Some(v) => self.push_tokens(v),
            _ => ()
        }
        self.stomach.pop_group(&mut self.state)
    }
    pub fn insert_afterassignment(&mut self) {
        match self.state.afterassignment.take() {
            Some(tk) => self.push_tokens(vec!(tk)),
            _ => ()
        }
    }
}