use std::borrow::Borrow;
use std::collections::HashMap;
use std::fmt::{Display, Formatter};
use std::hash::Hash;
use std::marker::PhantomData;
use std::path::Path;
use std::sync::Arc;
use crate::catcodes::{CategoryCode, CategoryCodeScheme, STARTING_SCHEME};
use crate::commands::TeXCommand;
use crate::fonts::{Font, FontFile, NULL_FONT};
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
use crate::stomach::colon::NoColon;

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
impl HasDefault for Arc<Font> {
    fn default() -> Self {
        NULL_FONT.try_with(|x| x.clone()).unwrap()
    }
}
impl HasDefault for FontStyle {
    fn default() -> Self {
        FontStyle::Text
    }
}

pub trait StateStore<K,V>:Sized where V:HasDefault {
    fn get(&self,k:&K) -> Option<&V>;
    fn set(&mut self,k:K,v:V);
    fn remove(&mut self,k:&K);
    fn new() -> Self;
}
impl<K:Hash+Eq,V:HasDefault> StateStore<K,V> for HashMap<K,V> {
    fn get(&self, k: &K) -> Option<&V> {
        HashMap::get(self,k)
    }
    fn set(&mut self, k: K, v: V) {
        self.insert(k,v);
    }
    fn remove(&mut self, k: &K) {
        HashMap::remove(self,k);
    }
    fn new() -> Self {
        HashMap::new()
    }
}

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
    fn new() -> Self { Var(None) }
}
impl StateStore<usize,Arc<Font>> for [Option<Arc<Font>>;16] {
    fn get(&self, k: &usize) -> Option<&Arc<Font>> { self[*k].as_ref() }
    fn set(&mut self, k: usize, v: Arc<Font>) { self[k] = Some(v) }
    fn remove(&mut self, k: &usize) { self[*k] = None }
    fn new() -> Self { newfonts() }
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
    pub values:Option<A>,
    pub parent:Option<Box<LinkedStateValue<K,V,A>>>
}
impl<K,V:HasDefault+Clone,A:StateStore<K,V>> std::default::Default for LinkedStateValue<K,V,A> {
    fn default() -> Self {
        LinkedStateValue {
            k:PhantomData,
            v:PhantomData,
            values:None,parent:None
        }
    }
}

impl<K,V:HasDefault+Clone,A:StateStore<K,V>> LinkedStateValue<K,V,A> {
    pub fn get_maybe(&self,k:&K) -> Option<&V> {
        match &self.values {
            Some(a) => match a.get(k) {
                Some(v) => return Some(v),
                _ => ()
            }
            _ => ()
        }
        match &self.parent {
            None => None,
            Some(p) => p.get_maybe(k)
        }
    }
    pub fn get(&self,k:&K) -> V {
        match &self.values {
            Some(a) => match a.get(k) {
                Some(v) => return v.clone(),
                _ => ()
            }
            _ => ()
        }
        match &self.parent {
            None => HasDefault::default(),
            Some(p) => p.get(k)
        }
    }
    fn set_locally(&mut self, k : K, v : V) {
        match self.values {
            Some(ref mut s) => {
                s.set(k,v);
            }
            ref mut o@None => {
                *o = Some(StateStore::new());
                o.as_mut().unwrap().set(k,v);
            }
        }
    }
    fn set_globally(&mut self,k:K,v:V) {
        match self.parent {
            Some(ref mut p) => {
                for s in &mut self.values { s.remove(&k); }
                p.set_globally(k,v)
            }
            None => {
                self.set_locally(k, v)
            }
        }
    }
    pub fn set(&mut self,k:K,v:V,globally:bool) {
        if globally {self.set_globally(k,v)} else {self.set_locally(k,v)}
    }
    fn push(&mut self) {
        *self = LinkedStateValue {
            k:PhantomData,v:PhantomData,
            parent : Some(Box::new(std::mem::take(self))),
            values:None
        }
    }
    fn pop(&mut self) {
        assert!(self.parent.is_some());
        *self = std::mem::take(self.parent.as_mut().unwrap())
    }
}
impl LinkedStateValue<i32,TeXBox,HashMap<i32,TeXBox>> {
    pub fn take(&mut self,k:i32) -> TeXBox {
        match self.values {
            Some(ref mut v) => match v.remove(&k) {
                Some(b) => return b,
                _ => ()
            }
            _ => ()
        }
        match self.parent {
            None => TeXBox::Void,
            Some(ref mut p) => p.take(k)
        }
    }
}
impl <B> LinkedStateValue<(),Vec<B>,Var<Vec<B>>> {
    pub fn add(&mut self,b:B) {
        match self.values {
            Some(Var(Some(ref mut v))) => v.push(b),
            _ => self.values = Some(Var(Some(vec!(b))))
        }
    }
}


#[derive(Clone,PartialEq)]
pub struct LinkedCatScheme {
    scheme:Option<CategoryCodeScheme>,
    parent:Option<Box<LinkedCatScheme>>
}
impl std::default::Default for LinkedCatScheme {
    fn default() -> Self {
        LinkedCatScheme {scheme:None,parent:None}
    }
}
impl LinkedCatScheme {
    pub fn get_scheme(&self) -> &CategoryCodeScheme {
        match self.scheme {
            Some(ref scheme) => scheme,
            None => match self.parent {
                Some(ref bx) => bx.get_scheme(),
                None => unreachable!()
            }
        }
    }
    fn push(&mut self) {
        *self = LinkedCatScheme {
            parent : Some(Box::new(std::mem::take(self))),
            scheme : None
        }
    }
    pub fn set_newline(&mut self,v:u8,globally:bool) {
        match self.scheme {
            Some(ref mut s) => {
                s.newlinechar = v;
            }
            None => {
                self.scheme = Some(self.get_scheme().clone());
                self.scheme.as_mut().unwrap().newlinechar = v;
            }
        }
        if globally {
            for p in self.parent.as_mut() {
                p.set_newline(v,globally)
            }
        }
    }
    pub fn set_endline(&mut self,v:u8,globally:bool) {
        match self.scheme {
            Some(ref mut s) => {
                s.endlinechar = v;
            }
            None => {
                self.scheme = Some(self.get_scheme().clone());
                self.scheme.as_mut().unwrap().endlinechar = v;
            }
        }
        if globally {
            for p in self.parent.as_mut() {
                p.set_endline(v,globally)
            }
        }
    }
    pub fn set_escape(&mut self,v:u8,globally:bool) {
        match self.scheme {
            Some(ref mut s) => {
                s.escapechar = v;
            }
            None => {
                self.scheme = Some(self.get_scheme().clone());
                self.scheme.as_mut().unwrap().escapechar = v;
            }
        }
        if globally {
            for p in self.parent.as_mut() {
                p.set_escape(v,globally)
            }
        }
    }
    pub fn set(&mut self,k:u8,v: CategoryCode,globally:bool) {
        if globally {self.set_globally(k,v)} else {self.set_locally(k,v)}
    }
    fn set_locally(&mut self,k : u8,v : CategoryCode) {
        match self.scheme {
            Some(ref mut s) => {
                s.catcodes[k as usize] = v;
            }
            None => {
                self.scheme = Some(self.get_scheme().clone());
                self.scheme.as_mut().unwrap().catcodes[k as usize] = v;
            }
        }
    }
    fn set_globally(&mut self,k : u8,v : CategoryCode) {
        match self.scheme {
            Some(ref mut s) => {
                s.catcodes[k as usize] = v;
            }
            _ => ()
        }
        match self.parent {
            None => (),
            Some(ref mut p) => p.set_globally(k,v)
        }
    }
    fn pop(&mut self) {
        assert!(self.parent.is_some());
        *self = std::mem::take(self.parent.as_mut().unwrap())
    }
}

#[derive(Clone)]
pub struct State {
    pub tp: LinkedStateValue<(),GroupType,Var<GroupType>>,
    pub catcodes:LinkedCatScheme,
    pub commands: LinkedStateValue<TeXStr,Option<TeXCommand>,HashMap<TeXStr,Option<TeXCommand>>>,
    pub registers: LinkedStateValue<i32,i32,HashMap<i32,i32>>,
    pub dimensions: LinkedStateValue<i32,i32,HashMap<i32,i32>>,
    pub skips: LinkedStateValue<i32,Skip,HashMap<i32,Skip>>,
    pub muskips: LinkedStateValue<i32,MuSkip,HashMap<i32,MuSkip>>,
    pub toks: LinkedStateValue<i32,Vec<Token>,HashMap<i32,Vec<Token>>>,
    pub sfcodes : LinkedStateValue<u8,i32,HashMap<u8,i32>>,
    pub lccodes : LinkedStateValue<u8,u8,HashMap<u8,u8>>,
    pub uccodes : LinkedStateValue<u8,u8,HashMap<u8,u8>>,
    pub mathcodes : LinkedStateValue<u8,i32,HashMap<u8,i32>>,
    pub delcodes : LinkedStateValue<u8,i32,HashMap<u8,i32>>,
    pub boxes: LinkedStateValue<i32,TeXBox,HashMap<i32,TeXBox>>,
    pub parshape : LinkedStateValue<(),Vec<(i32,i32)>,Var<Vec<(i32,i32)>>>,
    pub hangindent : LinkedStateValue<(),i32,Var<i32>>,
    pub hangafter : LinkedStateValue<(),usize,Var<usize>>,
    pub(crate) currfont : LinkedStateValue<(),Arc<Font>,Var<Arc<Font>>>,
    pub(crate) aftergroups : LinkedStateValue<(),Vec<Token>,Var<Vec<Token>>>,
    pub(crate) fontstyle : LinkedStateValue<(),FontStyle,Var<FontStyle>>,
    pub(crate) textfonts: LinkedStateValue<usize,Arc<Font>,[Option<Arc<Font>>;16]>,
    pub(crate) scriptfonts: LinkedStateValue<usize,Arc<Font>,[Option<Arc<Font>>;16]>,
    pub(crate) scriptscriptfonts: LinkedStateValue<usize,Arc<Font>,[Option<Arc<Font>>;16]>,
    pub(crate) displaymode: LinkedStateValue<(),bool,Var<bool>>,

    // DIRECT ------------------------------------------
    pub(in crate) conditions:Vec<Option<bool>>,
    pub(in crate) outfiles:HashMap<u8,Arc<VFile>>,
    pub(in crate) infiles:HashMap<u8,StringMouth>,
    pub(in crate) incs : u8,
    pub(in crate) mode:TeXMode,
    pub(in crate) afterassignment : Option<Token>,
    pub(in crate) pdfmatches : Vec<TeXStr>,
    pub(in crate) pdfcolorstacks: Vec<Vec<TeXStr>>,
    pub(in crate) pdfobjs: HashMap<u16,TeXStr>,
    pub(in crate) pdfxforms: Vec<PDFXForm>,
    pub(in crate) indocument_line:Option<(TeXStr,usize)>,
    pub(in crate) indocument:bool,
    pub(in crate) insetbox:bool,
    pub(in crate) vadjust:Vec<Whatsit>,
    pub (in crate) inserts:HashMap<u16,Vec<Whatsit>>,
    pub(in crate) pagegoal:i32,
    pub(in crate) pdfximages:Vec<PDFXImage>,
    pub(in crate) aligns: Vec<Option<Vec<Token>>>,
    pub(in crate) topmark : Vec<Token>,
    pub(in crate) firstmark : Vec<Token>,
    pub(in crate) botmark : Vec<Token>,
    pub(in crate) splitfirstmark : Vec<Token>,
    pub(in crate) splitbotmark : Vec<Token>,
    // TODO -----------------------------------------
    pub (in crate) filestore:HashMap<TeXStr,Arc<VFile>>,
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
static mut FONT_FILES: Option<HashMap<TeXStr,Arc<FontFile>>> = None;

impl State {
    pub fn push(&mut self,stomach:&mut dyn Stomach,gt:GroupType) {
        log!("Push: {} -> {}",gt,self.stack_depth() + 1);
        pass_on!(self,push);
        stomach.new_group(gt);
        self.tp.set_locally((),gt)
    }
    pub fn pop(&mut self,tp:GroupType) -> Result<Option<Vec<Token>>,TeXError> {
        log!("Pop: {} -> {}",tp,self.stack_depth());
        match self.tp.values.as_ref().unwrap().0.unwrap() {
            t if t == tp => (),
            t => TeXErr!("Group opened by {} ended by {}",t,tp)
        }
        let ag = match self.aftergroups.values {
            Some(ref mut v) => std::mem::take(&mut v.0),
            _ => None
        };
        pass_on!(self,pop);
        Ok(ag)
    }
    pub fn stack_depth(&self) -> usize {
        let mut curr = &self.tp;
        let mut ret : usize = 0;
        loop {
            match &curr.parent {
                None => return ret,
                Some(p) => {
                    ret += 1;
                    curr = p.borrow()
                }
            }
        }
    }
    pub fn new() -> State {
        let mut state = State {
            conditions:vec!(),
            outfiles:HashMap::new(),
            infiles:HashMap::new(),
            incs:0,
            mode:TeXMode::Vertical,
            afterassignment:None,
            pdfmatches:vec!(),
            pdfcolorstacks:vec!(vec!()),
            pdfobjs:HashMap::new(),
            pdfxforms:vec!(),
            indocument_line:None,
            indocument:false,
            insetbox:false,
            vadjust:vec!(),
            inserts:HashMap::new(),
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
                values: Some(Var(Some(GroupType::Begingroup))),
                parent: None
            },
            catcodes: LinkedCatScheme {
                scheme: Some(STARTING_SCHEME.clone()),
                parent: None
            },
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
            state.commands.set_locally(c.name().unwrap(),Some(c))
        }
        for c in tex_commands() {
            let c = c.as_command();
            state.commands.set_locally(c.name().unwrap(),Some(c))
        }
        for c in pdftex_commands() {
            let c = c.as_command();
            state.commands.set_locally(c.name().unwrap(),Some(c))
        }
        state.registers.set_locally(-(crate::commands::primitives::MAG.index as i32),1000);
        state.registers.set_locally(-(crate::commands::primitives::FAM.index as i32),-1);
        state.dimensions.set_locally(-(crate::commands::pdftex::PDFPXDIMEN.index as i32),65536);
        for i in 0..=255 {
            state.uccodes.set_locally(i,i);
            state.lccodes.set_locally(i,i);
        }
        for i in 97..123 {
            state.uccodes.set_locally(i,i-32);
            state.lccodes.set_locally(i-32,i);
        }
        state
    }
    pub fn pdf_latex() -> State {
        let mut state = State::new();
        let pdftex_cfg = crate::kpathsea::kpsewhich("pdftexconfig.tex",&PWD).expect("pdftexconfig.tex not found").0;
        let latex_ltx = crate::kpathsea::kpsewhich("latex.ltx",&PWD).expect("No latex.ltx found").0;
        let p = /* DefaultParams {
            log:false,
            singlethreaded:true
        }; // */ NoOutput::new(None);

        state = Interpreter::do_file_with_state(&pdftex_cfg,state,NoColon::new(),&p).0;
        state = Interpreter::do_file_with_state(&latex_ltx,state,NoColon::new(),&p).0;
        for c in pdftex_commands() {
            let c = c.as_command();
            state.commands.set_locally(c.name().unwrap(),Some(c))
        }
        for c in rustex_special_commands() {
            let c = c.as_command();
            state.commands.set_locally(c.name().unwrap(),Some(c))
        }
        for c in pgf_commands() {
            let c = c.as_command();
            state.commands.set_locally(c.name().unwrap(),Some(c))
        }
        state
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
        let mouth = StringMouth::new_from_file(self.catcodes.get_scheme(),&file);
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
                None => FONT_FILES = Some(HashMap::new()),
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
        /*if (cmdname.to_string() == "notin " || cmdname.to_string() == "not" || cmdname.to_string() == "c@ncel") {
            println!("Here! {}, {}, {}",cmdname,proc.as_ref().unwrap().meaning(&crate::catcodes::DEFAULT_SCHEME),self.current_line());
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