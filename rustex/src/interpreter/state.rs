use std::collections::HashMap;
use crate::catcodes::{CategoryCode, CategoryCodeScheme, STARTING_SCHEME};
use crate::commands::TeXCommand;
use crate::interpreter::{Interpreter, TeXMode};
use crate::utils::{PWD, TeXError, TeXString, TeXStr};
use crate::{TeXErr,log};

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

#[derive(Clone)]
struct StackFrame {
    //parent: Option<&'a StackFrame<'a>>,
    pub(crate) catcodes: CategoryCodeScheme,
    pub(crate) commands: HashMap<TeXStr,Option<TeXCommand>>,
    pub(crate) registers: HashMap<i32,i32>,
    pub(crate) dimensions: HashMap<i32,i32>,
    pub(crate) skips : HashMap<i32,Skip>,
    pub(crate) muskips : HashMap<i32,MuSkip>,
    pub(crate) toks : HashMap<i32,Vec<Token>>,
    pub(in crate::interpreter::state) tp : Option<GroupType>,
    pub(crate) sfcodes : HashMap<u8,i32>,
    pub(crate) lccodes : HashMap<u8,u8>,
    pub(crate) uccodes : HashMap<u8,u8>,
    pub(crate) mathcodes : HashMap<u8,i32>,
    pub(crate) delcodes : HashMap<u8,i32>,
    pub(crate) boxes: HashMap<i32,TeXBox>,
    pub(crate) currfont : Arc<Font>,
    pub(crate) aftergroups : Vec<Token>,
    pub(crate) fontstyle : FontStyle,
    pub(crate) textfonts: [Option<Arc<Font>>;16],
    pub(crate) scriptfonts: [Option<Arc<Font>>;16],
    pub(crate) scriptscriptfonts: [Option<Arc<Font>>;16],
    pub(crate) displaymode: bool
}

fn newfonts() -> [Option<Arc<Font>>;16] {
    [
        None,None,None,None,None,None,None,None,None,None,None,None,None,None,None,None
    ]
}

impl StackFrame {
    pub(crate) fn initial_pdf_etex() -> StackFrame {
        use crate::commands::conditionals::conditional_commands;
        use crate::commands::primitives::tex_commands;
        use crate::commands::pdftex::pdftex_commands;
        let mut cmds: HashMap<TeXStr,Option<TeXCommand>> = HashMap::new();
        for c in conditional_commands() {
            let c = c.as_command();
            cmds.insert(c.name().unwrap().clone(),Some(c));
        }
        for c in tex_commands() {
            let c = c.as_command();
            cmds.insert(c.name().unwrap().clone(),Some(c));
        }
        for c in pdftex_commands() {
            let c = c.as_command();
            cmds.insert(c.name().unwrap().clone(),Some(c));
        }
        let mut reg: HashMap<i32,i32> = HashMap::new();
        reg.insert(-(crate::commands::primitives::MAG.index as i32),1000);
        reg.insert(-(crate::commands::primitives::FAM.index as i32),-1);

        let mut dims: HashMap<i32,i32> = HashMap::new();
        dims.insert(-(crate::commands::pdftex::PDFPXDIMEN.index as i32),65536);

        let skips: HashMap<i32,Skip> = HashMap::new();
        let muskips: HashMap<i32,MuSkip> = HashMap::new();
        let toks: HashMap<i32,Vec<Token>> = HashMap::new();
        let sfcodes: HashMap<u8,i32> = HashMap::new();
        let mut lccodes: HashMap<u8,u8> = HashMap::new();
        let mut uccodes: HashMap<u8,u8> = HashMap::new();
        for i in 97..123 {
            uccodes.insert(i,i-32);
            lccodes.insert(i-32,i);
        }
        let boxes: HashMap<i32,TeXBox> = HashMap::new();
        let mathcodes : HashMap<u8,i32> = HashMap::new();
        let delcodes : HashMap<u8,i32> = HashMap::new();
        StackFrame {
            //parent: None,
            catcodes: STARTING_SCHEME.clone(),
            commands: cmds,
            registers:reg,
            dimensions:dims,
            skips,toks,sfcodes,lccodes,uccodes,muskips,boxes,mathcodes,delcodes,
            tp:None,aftergroups:vec!(),
            currfont:NULL_FONT.try_with(|x| x.clone()).unwrap(),
            fontstyle:FontStyle::Text,
            textfonts:newfonts(),
            scriptfonts:newfonts(),
            scriptscriptfonts:newfonts(),
            displaymode:false
        }
    }
    pub(crate) fn new(parent: &StackFrame,tp : GroupType) -> StackFrame {
        let reg: HashMap<i32,i32> = HashMap::new();
        let dims: HashMap<i32,i32> = HashMap::new();
        let skips: HashMap<i32,Skip> = HashMap::new();
        let muskips: HashMap<i32,MuSkip> = HashMap::new();
        let toks: HashMap<i32,Vec<Token>> = HashMap::new();
        let sfcodes: HashMap<u8,i32> = HashMap::new();
        let lccodes: HashMap<u8,u8> = HashMap::new();
        let uccodes: HashMap<u8,u8> = HashMap::new();
        /*for i in 97..123 {
            uccodes.insert(i,i-32);
            lccodes.insert(i-32,i);
        }*/
        let boxes: HashMap<i32,TeXBox> = HashMap::new();
        let mathcodes : HashMap<u8,i32> = HashMap::new();
        let delcodes : HashMap<u8,i32> = HashMap::new();
        StackFrame {
            //parent: Some(parent),
            catcodes: parent.catcodes.clone(),
            commands: HashMap::new(),
            registers:reg,
            dimensions:dims,aftergroups:vec!(),
            skips,toks,sfcodes,lccodes,uccodes,muskips,boxes,mathcodes,delcodes,
            tp:Some(tp),currfont:parent.currfont.clone(),
            fontstyle:parent.fontstyle,
            textfonts:newfonts(),
            scriptfonts:newfonts(),
            scriptscriptfonts:newfonts(),displaymode:parent.displaymode
        }
    }
}

// ------------------------------------------------------------------------------------------------

#[derive(Clone)]
pub struct State {
    stacks: Vec<StackFrame>,
    pub(in crate) conditions:Vec<Option<bool>>,
    pub(in crate) outfiles:HashMap<u8,Arc<VFile>>,
    pub(in crate) infiles:HashMap<u8,StringMouth>,
    pub(in crate) incs : u8,
    fontfiles: HashMap<TeXStr,Arc<FontFile>>,
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
    pub (in crate) filestore:HashMap<TeXStr,Arc<VFile>>,
}

// sudo apt install libkpathsea-dev

impl State {
    pub fn new() -> State {
        let state = State {
            stacks: vec![StackFrame::initial_pdf_etex()],
            conditions: vec![],
            outfiles:HashMap::new(),
            infiles:HashMap::new(),
            incs:0,
            fontfiles: HashMap::new(),
            mode:TeXMode::Vertical,
            afterassignment:None,
            pdfmatches : vec!(),
            pdfobjs : HashMap::new(),
            pdfcolorstacks: vec!(vec!()),
            pdfxforms:vec!(),
            indocument_line:None,indocument:false,insetbox:false,
            vadjust:vec!(),inserts:HashMap::new(),
            pagegoal:0,pdfximages:vec!(),aligns:vec!(),
            topmark:vec!(),botmark:vec!(),firstmark:vec!(),splitbotmark:vec!(),splitfirstmark:vec!(),
            filestore:HashMap::new()
        };

        state
    }

    pub fn stack_depth(&self) -> usize {
        self.stacks.len() - 1
    }

    pub fn get_font(&mut self,int:&Interpreter,name:TeXStr) -> Result<Arc<FontFile>,TeXError> {
        match self.fontfiles.get(&name) {
            Some(ff) => Ok(Arc::clone(ff)),
            None => {
                let ret = unsafe{int.kpsewhich(from_utf8_unchecked(name.iter()))};
                match ret {
                    Some((pb,_)) if pb.exists() => {
                        let f = Arc::new(FontFile::new(pb));
                        self.fontfiles.insert(name,Arc::clone(&f));
                        Ok(f)
                    }
                    _ => {
                        println!("Here! {}", int.current_line());
                        TeXErr!((int,None),"Font file {} not found",name)
                    }
                }
            }
        }
    }

    pub fn with_commands(procs:Vec<TeXCommand>) -> State {
        let mut st = State::new();
        for p in procs {
            let name = p.name().unwrap();
            st.stacks.last_mut().unwrap().commands.insert(name.clone(),Some(p));
        }
        st
    }

    pub fn get_command(&self, name: &TeXStr) -> Option<TeXCommand> {
        for sf in self.stacks.iter().rev() {
            match sf.commands.get(name) {
                Some(r) => return r.clone(),
                _ => {}
            }
        }
        None
    }
    pub fn get_register(&self, index:i32) -> i32 {
        for sf in self.stacks.iter().rev() {
            match sf.registers.get(&index) {
                Some(r) => return *r,
                _ => {}
            }
        }
        0
    }
    pub fn get_sfcode(&self, index:u8) -> i32 {
        for sf in self.stacks.iter().rev() {
            match sf.sfcodes.get(&index) {
                Some(r) => return *r,
                _ => {}
            }
        }
        0
    }
    pub fn get_dimension(&self, index:i32) -> i32 {
        for sf in self.stacks.iter().rev() {
            match sf.dimensions.get(&index) {
                Some(r) => return *r,
                _ => {}
            }
        }
        0
    }

    pub (in crate::interpreter::state) fn lccode(&self,i:u8) -> u8 {
        for sf in self.stacks.iter().rev() {
            match sf.lccodes.get(&i) {
                Some(r) => return *r,
                _ => {}
            }
        }
        i
    }

    pub (in crate::interpreter::state) fn uccode(&self,i:u8) -> u8 {
        for sf in self.stacks.iter().rev() {
            match sf.uccodes.get(&i) {
                Some(r) => return *r,
                _ => {}
            }
        }
        i
    }

    pub (in crate::interpreter::state) fn mathcode(&self,i:u8) -> i32 {
        for sf in self.stacks.iter().rev() {
            match sf.mathcodes.get(&i) {
                Some(r) => return *r,
                _ => {}
            }
        }
        0
    }

    pub (in crate::interpreter::state) fn delcode(&self,i:u8) -> i32 {
        for sf in self.stacks.iter().rev() {
            match sf.delcodes.get(&i) {
                Some(r) => return *r,
                _ => {}
            }
        }
        0
    }

    pub fn get_skip(&self, index:i32) -> Skip {
        for sf in self.stacks.iter().rev() {
            match sf.skips.get(&index) {
                Some(r) => return *r,
                _ => {}
            }
        }
        Skip{
            base: 0,
            stretch: None,
            shrink: None
        }
    }

    pub fn get_muskip(&self, index:i32) -> MuSkip {
        for sf in self.stacks.iter().rev() {
            match sf.muskips.get(&index) {
                Some(r) => return *r,
                _ => {}
            }
        }
        MuSkip{
            base: 0,
            stretch: None,
            shrink: None
        }
    }
    pub fn catcodes(&self) -> &CategoryCodeScheme {
        &self.stacks.last().expect("Stack frames empty").catcodes
    }
    pub fn tokens(&self,index:i32) -> Vec<Token> {
        for sf in self.stacks.iter().rev() {
            match sf.toks.get(&index) {
                Some(r) => return r.iter().map(|x| x.cloned()).collect(),
                _ => {}
            }
        }
        vec!()
    }
    pub fn close(mut self,int:Interpreter) -> State {
        self.stacks.last_mut().unwrap().catcodes = int.catcodes.borrow().clone();
        self
    }
    pub fn change(&mut self,int:&Interpreter,change:StateChange) {
        match change {
            StateChange::Displaymode(b) => self.stacks.last_mut().unwrap().displaymode = b,
            StateChange::Textfont(i,f,global) => {
                if global {
                    for s in self.stacks.iter_mut() {
                        s.textfonts[i as usize] = Some(f.clone())
                    }
                } else { self.stacks.last_mut().unwrap().textfonts[i as usize] = Some(f)}
            }
            StateChange::Scriptfont(i,f,global) => {
                if global {
                    for s in self.stacks.iter_mut() {
                        s.scriptfonts[i as usize] = Some(f.clone())
                    }
                } else { self.stacks.last_mut().unwrap().scriptfonts[i as usize] = Some(f)}
            }
            StateChange::Scriptscriptfont(i,f,global) => {
                if global {
                    for s in self.stacks.iter_mut() {
                        s.scriptscriptfonts[i as usize] = Some(f.clone())
                    }
                } else { self.stacks.last_mut().unwrap().scriptscriptfonts[i as usize] = Some(f)}
            }
            StateChange::Fontstyle(fs) => self.stacks.last_mut().unwrap().fontstyle = fs,
            StateChange::Aftergroup(tk) => self.stacks.last_mut().unwrap().aftergroups.push(tk),
            StateChange::Font(f,global) => {
                if global {
                    for s in self.stacks.iter_mut() {
                        s.currfont = f.clone()
                    }
                } else {
                    self.stacks.last_mut().unwrap().currfont = f
                }
            }
            StateChange::Register(index,value,global) => {
                if global {
                    for s in self.stacks.iter_mut() {
                        s.registers.insert(index,value);
                    }
                } else {
                    self.stacks.last_mut().unwrap().registers.insert(index,value);
                }
            }
            StateChange::Dimen(index,value,global) => {
                if global {
                    for s in self.stacks.iter_mut() {
                        s.dimensions.insert(index,value);
                    }
                } else {
                    self.stacks.last_mut().unwrap().dimensions.insert(index,value);
                }
            }
            StateChange::Skip(index,value,global) => {
                if global {
                    for s in self.stacks.iter_mut() {
                        s.skips.insert(index,value);
                    }
                } else {
                    self.stacks.last_mut().unwrap().skips.insert(index,value);
                }
            }
            StateChange::MuSkip(index,value,global) => {
                if global {
                    for s in self.stacks.iter_mut() {
                        s.muskips.insert(index,value);
                    }
                } else {
                    self.stacks.last_mut().unwrap().muskips.insert(index,value);
                }
            }
            StateChange::Cs(name,cmd,global) => {
                if global {
                    for s in self.stacks.iter_mut() {
                        s.commands.remove(&name);
                    }
                    match cmd {
                        Some(c) => self.stacks.first_mut().unwrap().commands.insert(name,Some(c)),
                        None => self.stacks.first_mut().unwrap().commands.remove(&name)
                    };
                } else if self.stacks.len() == 1 {
                    match cmd {
                        Some(c) => self.stacks.first_mut().unwrap().commands.insert(name,Some(c)),
                        None => self.stacks.first_mut().unwrap().commands.remove(&name)
                    };
                } else {
                    self.stacks.last_mut().unwrap().commands.insert(name,cmd);
                }
            }
            StateChange::Cat(char,catcode,global) => {
                int.catcodes.borrow_mut().catcodes[char as usize] = catcode;
                if global {
                    for s in self.stacks.iter_mut() {
                        s.catcodes.catcodes[char as usize] = catcode;
                    }
                }
            }
            StateChange::Newline(char,global) => {
                int.catcodes.borrow_mut().newlinechar = char;
                if global {
                    for s in self.stacks.iter_mut() {
                        s.catcodes.newlinechar = char;
                    }
                }
            }
            StateChange::Endline(char,global) => {
                int.catcodes.borrow_mut().endlinechar = char;
                if global {
                    for s in self.stacks.iter_mut() {
                        s.catcodes.endlinechar = char;
                    }
                }
            }
            StateChange::Escapechar(char,global) => {
                int.catcodes.borrow_mut().escapechar = char;
                if global {
                    for s in self.stacks.iter_mut() {
                        s.catcodes.escapechar = char;
                    }
                }
            }
            StateChange::Sfcode(char,value,global) => {
                if global {
                    for s in self.stacks.iter_mut() {
                        s.sfcodes.insert(char,value);
                    }
                } else {
                    self.stacks.last_mut().unwrap().sfcodes.insert(char,value);
                }
            }
            StateChange::Mathcode(char,value,global) => {
                if global {
                    for s in self.stacks.iter_mut() {
                        s.mathcodes.insert(char,value);
                    }
                } else {
                    self.stacks.last_mut().unwrap().mathcodes.insert(char,value);
                }
            }
            StateChange::Delcode(char,value,global) => {
                if global {
                    for s in self.stacks.iter_mut() {
                        s.delcodes.insert(char,value);
                    }
                } else {
                    self.stacks.last_mut().unwrap().delcodes.insert(char,value);
                }
            }
            StateChange::Tokens(i,tks,global) => {
                if global {
                    for s in self.stacks.iter_mut() {
                        s.toks.remove(&i);
                    }
                    self.stacks.first_mut().unwrap().toks.insert(i,tks);
                } else {
                    self.stacks.last_mut().unwrap().toks.insert(i,tks);
                }
            }
            StateChange::Lccode(i,u,global) => {
                if global {
                    for s in self.stacks.iter_mut() {
                        if u == 0 {
                            s.lccodes.remove(&i);
                        } else {
                            s.lccodes.insert(i, u);
                        }
                    }
                } else {
                    if u == 0 {
                        self.stacks.last_mut().unwrap().lccodes.remove(&i);
                    } else {
                        self.stacks.last_mut().unwrap().lccodes.insert(i, u);
                    }
                }
            }
            StateChange::Uccode(i,u,global) => {
                if global {
                    for s in self.stacks.iter_mut() {
                        if u == 0 {
                            s.uccodes.remove(&i);
                        } else {
                            s.uccodes.insert(i, u);
                        }
                    }
                } else {
                    if u == 0 {
                        self.stacks.last_mut().unwrap().uccodes.remove(&i);
                    } else {
                        self.stacks.last_mut().unwrap().uccodes.insert(i, u);
                    }
                }
            }
            StateChange::Box(index,value,global) => {
                if global {
                    for s in self.stacks.iter_mut() {
                        s.boxes.remove(&index);
                    }
                    self.stacks.first_mut().unwrap().boxes.insert(index,value);
                } else {
                    self.stacks.last_mut().unwrap().boxes.insert(index,value);
                }
            }
            StateChange::Pdfmatches(vec) => self.pdfmatches = vec
            //_ => todo!()
        }
    }

    pub (in crate::interpreter::state) fn push(&mut self,cc:CategoryCodeScheme,tp : GroupType) {
        let mut laststack = self.stacks.last_mut().unwrap();
        laststack.catcodes = cc;
        let sf = StackFrame::new(self.stacks.last().unwrap(),tp);
        self.stacks.push(sf)
    }
    pub (in crate::interpreter::state) fn pop(&mut self,int:&Interpreter,_tp : GroupType) -> Result<(&CategoryCodeScheme,Vec<Token>),TeXError> {
        if self.stacks.len() < 2 { TeXErr!((int,None),"No group here to end!")}
        match self.stacks.pop() {
            Some(sf) => match sf.tp {
                None => TeXErr!((int,None),"No group here to end!"),
                Some(ltp) if !matches!(ltp,_tp) => TeXErr!((int,None),"Group opened by {} ended by {}",ltp,_tp),
                _ => Ok((&self.stacks.last().unwrap().catcodes,sf.aftergroups))
            }
            None => TeXErr!((int,None),"No group here to end!")
        }
    }

    pub fn get_text_font(&self, i : u8) -> Arc<Font> {
        for s in self.stacks.iter().rev() {
            match s.textfonts.get(i as usize).unwrap() {
                Some(f) => return f.clone(),
                _ => ()
            }
        }
        NULL_FONT.try_with(|x| x.clone()).unwrap()
    }
    pub fn get_script_font(&self, i : u8) -> Arc<Font> {
        for s in self.stacks.iter().rev() {
            match s.scriptfonts.get(i as usize).unwrap() {
                Some(f) => return f.clone(),
                _ => ()
            }
        }
        NULL_FONT.try_with(|x| x.clone()).unwrap()
    }
    pub fn get_scriptscript_font(&self, i : u8) -> Arc<Font> {
        for s in self.stacks.iter().rev() {
            match s.scriptscriptfonts.get(i as usize).unwrap() {
                Some(f) => return f.clone(),
                _ => ()
            }
        }
        NULL_FONT.try_with(|x| x.clone()).unwrap()
    }
    pub fn font_style(&self) -> FontStyle {
        self.stacks.last().unwrap().fontstyle
    }
    pub fn display_mode(&self) -> bool {
        self.stacks.last().unwrap().displaymode
    }
}


pub fn default_pdf_latex_state() -> State {
    use crate::commands::pgfsvg::pgf_commands;
    use crate::commands::rustex_specials::rustex_special_commands;
    let mut st = State::new();
    let pdftex_cfg = crate::kpathsea::kpsewhich("pdftexconfig.tex",&PWD).expect("pdftexconfig.tex not found").0;
    let latex_ltx = crate::kpathsea::kpsewhich("latex.ltx",&PWD).expect("No latex.ltx found").0;

    //println!("{}",pdftex_cfg.to_str().expect("wut"));
    //println!("{}",latex_ltx.to_str().expect("wut"));
    st = Interpreter::do_file_with_state(&pdftex_cfg,st,NoColon::new(),&NoOutput {}).0;
    st = Interpreter::do_file_with_state(&latex_ltx,st,NoColon::new(),&NoOutput {}).0;
    if crate::PGF_AS_SVG {
        for c in pgf_commands() {
            let c = c.as_command();
            st.stacks.first_mut().unwrap().commands.insert(c.name().unwrap().clone(),Some(c));
        }
    }
    if crate::RUSTEX_SPECIALS {
        for c in rustex_special_commands() {
            let c = c.as_command();
            st.stacks.first_mut().unwrap().commands.insert(c.name().unwrap().clone(),Some(c));
        }
    }
    st
    /*

    let mut interpreter = Interpreter::new_from_state(st);
    {interpreter.do_file(pdftex_cfg.as_path());}
    {interpreter.do_file(latex_ltx.as_path());}
    interpreter.kill_state()

 */
}

use std::cell::Ref;
use std::fmt::{Display, Formatter};
use std::str::from_utf8_unchecked;
use std::sync::Arc;
use crate::fonts::{Font, FontFile, NULL_FONT};
use crate::interpreter::dimensions::{MuSkip, Skip};
use crate::interpreter::files::VFile;
use crate::interpreter::mouth::StringMouth;
use crate::interpreter::params::NoOutput;
use crate::interpreter::Token;
use crate::stomach::whatsits::Whatsit;
use crate::stomach::boxes::{BoxMode,TeXBox};
use crate::stomach::colon::NoColon;
use crate::stomach::simple::{PDFXForm, PDFXImage};

impl Interpreter<'_> {
    pub fn file_read_line(&self,index:u8) -> Result<Vec<Token>,TeXError> {
        match self.state.borrow_mut().infiles.get_mut(&index) {
            None => TeXErr!((self,None),"No file open at index {}",index),
            Some(fm) => Ok(fm.read_line(&self.catcodes.borrow()))
        }
    }
    pub fn file_read(&self,index:u8,nocomment:bool) -> Result<Vec<Token>,TeXError> {
        use std::io::BufRead;
        match index {
            255 => {
                let stdin = std::io::stdin();
                let string = stdin.lock().lines().next().unwrap().unwrap();
                Ok(crate::interpreter::tokenize(string.into(),&self.catcodes.borrow()))
            }
            i => {
                match self.state.borrow_mut().infiles.get_mut(&i) {
                    None => TeXErr!((self,None),"No file open at index {}",i),
                    Some(fm) => Ok(fm.read(&self.catcodes.borrow(), nocomment))
                }
            }
        }
    }
    pub fn file_eof(&self,index:u8) -> Result<bool,TeXError> {
        match self.state.borrow_mut().infiles.get_mut(&index) {
            None => TeXErr!((self,None),"No file open at index {}",index),
            Some(fm) => {
                Ok(fm.is_eof())
            }
        }
    }
    pub fn file_openin(&self,index:u8,file:Arc<VFile>) -> Result<(),TeXError> {
        let mut state = self.state.borrow_mut();
        /*if state.infiles.contains_key(&index) {
            TeXErr!((self,None),"File already open at {}",index)
        }*/
        let mouth = StringMouth::new_from_file(&self.catcodes.borrow(),&file);
        state.infiles.insert(index,mouth);
        Ok(())
    }
    pub fn file_closein(&self,index:u8) -> Result<(),TeXError> {
        let mut state = self.state.borrow_mut();
        match state.infiles.remove(&index) {
            Some(f) => {
                f.source.pop_file().unwrap();
            }
            None => ()//TeXErr!((self,None),"No file open at index {}",index),
        }
        Ok(())
    }
    pub fn file_openout(&self,index:u8,file:Arc<VFile>) -> Result<(),TeXError> {
        let mut state = self.state.borrow_mut();
        file.string.write().unwrap().take();
        /*if state.outfiles.contains_key(&index) {
            TeXErr!((self,None),"File already open at {}",index)
        }*/
        state.outfiles.insert(index,file);
        Ok(())
    }
    pub fn file_write(&self,index:u8,s:TeXString) -> Result<(),TeXError> {
        use ansi_term::Colour::*;
        match index {
            17 => {
                self.params.write_17(s.to_utf8().as_str());
                Ok(())
            }
            16 => {
                self.params.write_16(s.to_utf8().as_str());
                Ok(())
            }
            18 => {
                self.params.write_18(s.to_utf8().as_str());
                Ok(())
            }
            255 => {
                self.params.write_neg_1(s.to_utf8().as_str());
                Ok(())
            }
            i if !self.state.borrow().outfiles.contains_key(&i) => {
                self.params.write_other(s.to_utf8().as_str());
                Ok(())
            }
             _ => {
                 let mut state = self.state.borrow_mut();
                 match state.outfiles.get_mut(&index) {
                     Some(f) => {
                         let mut string = f.string.write().unwrap();
                         match &mut*string {
                             None => {*string = Some(s) },
                             Some(st) => *st += s
                         }
                     }
                     None => TeXErr!((self,None),"No file open at index {}",index)
                 }
                 Ok(())
             }
        }
    }
    pub fn file_closeout(&self,index:u8) -> Result<(),TeXError> {
        let mut state = self.state.borrow_mut();
        let file = state.outfiles.remove(&index);
        match file {
            Some(_) => {
                //let mut fs = self.state.borrow_mut().filestore.borrow_mut();
                //fs.files.insert(vf.id.clone(),vf);
            }
            None => ()//TeXErr!(self,"No file open at index {}",index)
        }
        Ok(())
    }
    pub fn change_state(&self,change:StateChange) {
        let mut state = self.state.borrow_mut();
        state.change(self,change)
    }
    pub fn new_group(&self,tp:GroupType) {
        log!("Push: {}",tp);
        self.state.borrow_mut().push(self.catcodes.borrow().clone(),tp);
        self.stomach.borrow_mut().new_group(tp);
    }
    pub fn pop_group(&self,tp:GroupType) -> Result<(),TeXError> {
        log!("Pop: {}",tp);
        {
            let mut state = self.state.borrow_mut();
            let (cc, ag) = state.pop(self, tp)?;
            self.push_tokens(ag);
            let mut scc = self.catcodes.borrow_mut();
            scc.catcodes = cc.catcodes.clone();
            scc.endlinechar = cc.endlinechar;
            scc.newlinechar = cc.newlinechar;
            scc.escapechar = cc.escapechar;
        }
        self.stomach.borrow_mut().close_group(self)
    }
    pub fn get_whatsit_group(&self,tp:GroupType) -> Result<Vec<Whatsit>,TeXError> {
        log!("Pop: {}",tp);
        {
            let mut state = self.state.borrow_mut();
            let (cc, ag) = state.pop(self, tp)?;
            self.push_tokens(ag);
            let mut scc = self.catcodes.borrow_mut();
            scc.catcodes = cc.catcodes.clone();
            scc.endlinechar = cc.endlinechar;
            scc.newlinechar = cc.newlinechar;
            scc.escapechar = cc.escapechar;
        }
        self.stomach.borrow_mut().pop_group(self)
    }

    pub fn state_catcodes(&self) -> Ref<'_,CategoryCodeScheme> {
        self.catcodes.borrow()
    }
    pub fn state_register(&self,i:i32) -> i32 { self.state.borrow().get_register(i) }
    pub fn state_dimension(&self,i:i32) -> i32 {
        self.state.borrow().get_dimension(i)
    }
    pub fn state_skip(&self,i:i32) -> Skip {
        self.state.borrow().get_skip(i)
    }
    pub fn state_muskip(&self,i:i32) -> MuSkip {
        self.state.borrow().get_muskip(i)
    }
    pub fn state_sfcode(&self,i:u8) -> i32 { self.state.borrow().get_sfcode(i) }
    pub fn state_tokens(&self,i:i32) -> Vec<Token> { self.state.borrow().tokens(i)}
    pub fn state_lccode(&self,i:u8) -> u8 { self.state.borrow().lccode(i) }
    pub fn state_uccode(&self,i:u8) -> u8 { self.state.borrow().uccode(i) }

    pub fn pushcondition(&self) -> usize {
        let mut state = self.state.borrow_mut();
        state.conditions.push(None);
        state.conditions.len() - 1
    }
    pub fn setcondition(&self,c : usize,val : bool) -> Result<usize,TeXError> {
        let conds = &mut self.state.borrow_mut().conditions;
        if c >= conds.len() {
            TeXErr!((self,None),"This should not happen!")
        }
        conds[c] = Some(val);
        //conds.remove(c as usize);
        //conds.insert(c as usize,Some(val));
        Ok(conds.len() - (c + 1))
    }
    pub fn popcondition(&self) {
        self.state.borrow_mut().conditions.pop();
    }
    pub fn getcondition(&self) -> Option<(usize,Option<bool>)> {
        let conds = &self.state.borrow().conditions;
        match conds.last() {
            Some(p) => Some((conds.len() - 1,*p)),
            None => None
        }
    }
    pub fn newincs(&self) -> u8 {
        let mut state = self.state.borrow_mut();
        state.incs += 1;
        state.incs
    }
    pub fn currcs(&self) -> u8 {
        self.state.borrow().incs
    }
    pub fn popcs(&self) -> Result<(),TeXError> {
        let mut state = self.state.borrow_mut();
        if state.incs > 0 {
            state.incs -= 1;
            Ok(())
        } else {
            TeXErr!((self,None),"spurious \\endcsname")
        }
    }
    pub fn state_get_command(&self,s:&TeXStr) -> Option<TeXCommand> {
        self.state.borrow().get_command(s)
    }
    pub fn state_get_font(&self,name:&str) -> Result<Arc<FontFile>,TeXError> {
        self.state.borrow_mut().get_font(self,name.into())
    }
    pub fn state_get_mathcode(&self,i:u8) -> i32 {
        self.state.borrow().mathcode(i)
    }
    pub fn state_get_delcode(&self,i:u8) -> i32 {
        self.state.borrow().delcode(i)
    }
    pub fn get_mode(&self) -> TeXMode {
        self.state.borrow().mode
    }
    pub fn set_mode(&self,tm:TeXMode) {
        self.state.borrow_mut().mode = tm
    }
    pub fn state_set_afterassignment(&self,tk:Token) {
        self.state.borrow_mut().afterassignment = Some(tk)
    }
    pub fn insert_afterassignment(&self) {
        match self.state.borrow_mut().afterassignment.take() {
            Some(tk) => self.push_tokens(vec!(tk)),
            _ => ()
        }
    }
    pub fn get_font(&self) -> Arc<Font> {
        self.state.borrow().stacks.last().unwrap().currfont.clone()
    }
    pub fn state_color_pop(&self,i:usize) {
        let stack = &mut self.state.borrow_mut().pdfcolorstacks;
        let len = stack.len();
        stack.get_mut(len - 1 - i).unwrap().pop();
    }
    pub fn state_color_set(&self,i:usize,color:TeXStr) {
        let stack = &mut self.state.borrow_mut().pdfcolorstacks;
        let len = stack.len();
        let cs = stack.get_mut(len - 1 - i).unwrap();
        cs.pop();
        cs.push(color);
    }
    pub fn state_color_push(&self,i:usize,color:TeXStr) {
        let stack = &mut self.state.borrow_mut().pdfcolorstacks;
        //let len = stack.len();
        stack.get_mut(i).unwrap().push(color);
    }
    pub fn state_color_push_stack(&self) -> usize {
        let stack = &mut self.state.borrow_mut().pdfcolorstacks;
        stack.push(vec!());
        stack.len() - 1
    }
    pub fn state_set_pdfobj(&self,i:u16,obj:TeXStr) {
        let objs = &mut self.state.borrow_mut().pdfobjs;
        objs.insert(i,obj);
    }
    pub fn state_get_box(&self,i:i32) -> TeXBox {
        for sf in self.state.borrow_mut().stacks.iter_mut().rev() {
            match sf.boxes.remove(&i) {
                Some(b) => return b,
                None => ()
            }
        }
        TeXBox::Void
    }
    pub fn state_copy_box(&self,i:i32) -> TeXBox {
        for sf in self.state.borrow().stacks.iter().rev() {
            match sf.boxes.get(&i) {
                Some(b) => return b.clone(),
                None => ()
            }
        }
        TeXBox::Void
    }
    pub fn state_set_pdfxform(&self,p:PDFXForm) {
        self.state.borrow_mut().pdfxforms.push(p)
    }
    pub fn state_get_pdfxform(&self,index:usize) -> Result<PDFXForm,TeXError> {
        let state = self.state.borrow();
        match state.pdfxforms.get(state.pdfxforms.len() - index) {
            None => TeXErr!((self,None),"No \\pdfxform at index {}",index),
            Some(f) =>
                Ok(f.clone())
        }
    }
}

pub enum StateChange {
    Register(i32,i32,bool),
    Dimen(i32,i32,bool),
    Skip(i32,Skip,bool),
    MuSkip(i32,MuSkip,bool),
    Cs(TeXStr,Option<TeXCommand>,bool),
    Cat(u8,CategoryCode,bool),
    Newline(u8,bool),
    Endline(u8,bool),
    Escapechar(u8,bool),
    Sfcode(u8,i32,bool),
    Tokens(i32,Vec<Token>,bool),
    Lccode(u8,u8,bool),
    Uccode(u8,u8,bool),
    Box(i32,TeXBox,bool),
    Mathcode(u8,i32,bool),
    Delcode(u8,i32,bool),
    Font(Arc<Font>,bool),
    Pdfmatches(Vec<TeXStr>),
    Aftergroup(Token),
    Fontstyle(FontStyle),
    Textfont(usize,Arc<Font>,bool),
    Scriptfont(usize,Arc<Font>,bool),
    Scriptscriptfont(usize,Arc<Font>,bool),
    Displaymode(bool)
}