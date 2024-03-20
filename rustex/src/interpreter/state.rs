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


#[derive(Copy,Clone,PartialEq)]
pub enum FontStyle {
    Text,Script,Scriptscript
}
impl Default for FontStyle {
    fn default() -> Self { FontStyle::Text }
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
}
impl Default for GroupType {
    fn default() -> Self {
        GroupType::Begingroup
    }
}
impl Display for GroupType {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(f,"{}",match self {
            GroupType::Token => "{",
            GroupType::Begingroup => "\\begingroup",
            GroupType::Box(BoxMode::LeftRight) => "\\left\\right",
            GroupType::Box(BoxMode::M | BoxMode::DM) => "$",
            GroupType::Box(_) => "\\box"
        })
    }
}

#[derive(Clone)]
pub struct State {
    pub tp: store::LinkedValue<GroupType>,
    pub catcodes:store::LinkedCatScheme,
    pub commands: store::StateStore<TeXStr,Option<TeXCommand>,store::RusTeXMap<TeXStr,Option<TeXCommand>>>,

    pub registers_prim: store::StateStore<usize,i32,[i32;79]>,
    pub registers: store::StateStore<u16,i32,Vec<i32>>,
    pub dimensions_prim: store::StateStore<usize,i32,[i32;33]>,
    pub dimensions: store::StateStore<u16,i32,Vec<i32>>,
    pub skips_prim: store::StateStore<usize,Skip,[Skip;17]>,
    pub skips: store::StateStore<u16,Skip,Vec<Skip>>,
    pub muskips_prim: store::StateStore<usize,MuSkip,[MuSkip;3]>,
    pub muskips: store::StateStore<u16,MuSkip,Vec<MuSkip>>,
    pub toks_prim: store::StateStore<usize,Vec<Token>,[Vec<Token>;11]>,
    pub toks: store::StateStore<u16,Vec<Token>,Vec<Vec<Token>>>,

    pub boxes: store::StateStore<u16,TeXBox,Vec<TeXBox>>,
    pub sfcodes : store::StateStore<u8,i32,[i32;256]>,
    pub lccodes : store::StateStore<u8,u8,[u8;256]>,
    pub uccodes : store::StateStore<u8,u8,[u8;256]>,
    pub mathcodes : store::StateStore<u8,i32,[i32;256]>,
    pub delcodes : store::StateStore<u8,i32,[i32;256]>,
    pub parshape : store::LinkedValue<Vec<(i32,i32)>>,
    pub hangindent : store::LinkedValue<i32>,
    pub hangafter : store::LinkedValue<usize>,
    pub(crate) textfonts: store::StateStore<usize,ArcFont,[ArcFont;16]>,
    pub(crate) scriptfonts: store::StateStore<usize,ArcFont,[ArcFont;16]>,
    pub(crate) scriptscriptfonts: store::StateStore<usize,ArcFont,[ArcFont;16]>,
    pub(crate) currfont : store::LinkedValue<ArcFont>,
    pub(crate) aftergroups : store::LinkedValueOpt<Vec<Token>>,
    pub(crate) fontstyle : store::LinkedValue<FontStyle>,
    pub(crate) displaymode: store::LinkedValue<bool>,

    // DIRECT ------------------------------------------
    pub(in crate) conditions:Vec<Option<bool>>,
    pub(in crate) outfiles:store::RusTeXMap<u8,Arc<VFile>>,
    pub(in crate) infiles:store::RusTeXMap<u8,StringMouth>,
    pub(in crate) incs : u8,
    pub(in crate) mode:TeXMode,
    pub(in crate) afterassignment : Option<Token>,
    pub(in crate) pdfmatches : Vec<TeXStr>,
    pub(in crate) pdfcolorstacks: Vec<Vec<TeXStr>>,
    pub(in crate) pdfobjs: store::RusTeXMap<u16,TeXStr>,
    pub(in crate) pdfxforms: Vec<PDFXForm>,
    pub(in crate) indocument_line:Option<(TeXStr,usize)>,
    pub(in crate) indocument:bool,
    pub(in crate) insetbox:bool,
    pub(in crate) vadjust:Vec<Whatsit>,
    pub (in crate) inserts:store::RusTeXMap<u16,Vec<Whatsit>>,
    pub(in crate) pagegoal:i32,
    pub(in crate) pdfximages:Vec<PDFXImage>,
    pub(in crate) aligns: Vec<Option<Vec<Token>>>,
    pub(in crate) topmark : Vec<Token>,
    pub(in crate) firstmark : Vec<Token>,
    pub(in crate) botmark : Vec<Token>,
    pub(in crate) splitfirstmark : Vec<Token>,
    pub(in crate) splitbotmark : Vec<Token>,
    // TODO -----------------------------------------
    pub (in crate) filestore:store::RusTeXMap<TeXStr,Arc<VFile>>,
}

macro_rules! pass_on {
    ($s:tt,$e:ident$(,$tl:expr)*) => {
        $s.catcodes.$e($(,$tl)*);
        $s.commands.$e($(,$tl)*);
        $s.registers_prim.$e($(,$tl)*);
        $s.registers.$e($(,$tl)*);
        $s.dimensions_prim.$e($(,$tl)*);
        $s.dimensions.$e($(,$tl)*);
        $s.skips_prim.$e($(,$tl)*);
        $s.skips.$e($(,$tl)*);
        $s.muskips_prim.$e($(,$tl)*);
        $s.muskips.$e($(,$tl)*);
        $s.toks_prim.$e($(,$tl)*);
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
static mut FONT_FILES: Option<store::RusTeXMap<TeXStr,Arc<FontFile>>> = None;

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
        self.tp.push_v(gt);
        stomach.new_group(gt);
    }
    pub fn pop(&mut self,tp:GroupType) -> Result<Option<Vec<Token>>,TeXError> {
        log!("Pop: {} -> {}",tp,self.stack_depth());
        match unwrap!(self.tp.ls.front()) {
            t if *t == tp => (),
            t => TeXErr!("Group opened by {} ended by {}",t,tp)
        }
        let ag = match self.aftergroups.ls.front_mut() {
            Some( v) => std::mem::take(v),
            _ => None
        };
        pass_on!(self,pop);
        self.tp.pop();
        Ok(ag)
    }
    pub fn stack_depth(&self) -> usize {
        let mut curr = &self.tp;
        curr.ls.len() - 1
    }
    pub fn new() -> State {
        let mut state = State {
            conditions:vec!(),
            outfiles:store::RusTeXMap::default(),
            infiles:store::RusTeXMap::default(),
            incs:0,
            mode:TeXMode::Vertical,
            afterassignment:None,
            pdfmatches:vec!(),
            pdfcolorstacks:vec!(vec!()),
            pdfobjs:store::RusTeXMap::default(),
            pdfxforms:vec!(),
            indocument_line:None,
            indocument:false,
            insetbox:false,
            vadjust:vec!(),
            inserts:store::RusTeXMap::default(),
            pagegoal:0,
            pdfximages:vec!(),
            aligns:vec!(),
            topmark:vec!(),
            botmark:vec!(),
            firstmark:vec!(),
            splitfirstmark:vec!(),
            splitbotmark:vec!(),
            tp:Default::default(),
            catcodes: store::LinkedCatScheme::default(),
            commands: Default::default(),
            registers_prim: Default::default(),
            dimensions_prim: Default::default(),
            skips_prim: Default::default(),
            muskips_prim: Default::default(),
            toks_prim: Default::default(),
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
        state.registers_prim.set_locally((crate::commands::registers::MAG.index -1) as usize,1000);
        state.registers_prim.set_locally((crate::commands::registers::FAM.index -1) as usize,-1);
        state.dimensions_prim.set_locally((crate::commands::registers::PDFPXDIMEN.index - 1) as usize,65536);
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
    pub fn get_font(&mut self,indir:&Path,name:TeXStr,params:&dyn InterpreterParams) -> Result<Arc<FontFile>,TeXError> {
        unsafe {
            match FONT_FILES {
                None => FONT_FILES = Some(store::RusTeXMap::default()),
                _ => ()
            }
            match FONT_FILES.as_ref().unwrap().get(&name) {
                Some(ff) => Ok(Arc::clone(ff)),
                None => {
                    let ret = crate::kpathsea::kpsewhich(std::str::from_utf8_unchecked(name.iter()),indir);
                    match ret {
                        Some((pb,_)) if pb.exists() => {
                            let f = Arc::new(FontFile::new(pb,params));
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
        /*if cmdname.to_string() == "prentry@@norm" {//(cmdname == TeXStr::new(&[0,1,2,3,4,255,254,253,252,251,95])) {//"scr@dte@chapter@init" || cmdname.to_string() == "scr@dte@tocline@init") {
            println!("Here! {}\n{}\n{}",cmdname,proc.as_ref().unwrap().meaning(&crate::catcodes::DEFAULT_SCHEME),self.current_line());
            println!("")
        }*/
        let file = self.current_file();
        let line = self.mouths.current_line();
        for cl in self.params.command_listeners() {
            match cl.apply(&cmdname,&proc,&file,&line,&mut self.state) {
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
        let ret = self.stomach.pop_group(&mut self.state)?;
        let ag = self.state.pop(tp)?;
        match ag {
            Some(v) => self.push_tokens(v),
            _ => ()
        }
        Ok(ret)
    }
    pub fn insert_afterassignment(&mut self) {
        match self.state.afterassignment.take() {
            Some(tk) => self.push_tokens(vec!(tk)),
            _ => ()
        }
    }
}