
#[derive(Copy,Clone,PartialEq)]
pub enum TeXMode {
    Vertical, InternalVertical, Horizontal, RestrictedHorizontal, Math, Displaymath
}

use std::borrow::BorrowMut;
use crate::ontology::{Expansion, Token};
use crate::catcodes::{CategoryCode, CategoryCodeScheme};
use std::path::{Path, PathBuf};
use crate::commands::{TeXCommand, PrimitiveTeXCommand, ProvidesWhatsit};
use crate::interpreter::files::{VFile};
use crate::interpreter::mouth::Mouths;
use crate::interpreter::state::{GroupType, State};
use crate::utils::{TeXError, TeXString, TeXStr, MaybeThread};
use std::sync::Arc;

pub mod mouth;
pub mod state;
pub(crate) mod files;
pub mod dimensions;
pub mod methods;
pub mod params;


pub fn tokenize(s : TeXString,cats: &CategoryCodeScheme) -> Vec<Token> {
    let mut retvec: Vec<Token> = Vec::new();
    for next in s.0 {
        retvec.push(Token::new(next,cats.get_code(next),None,None,true))
    }
    retvec
}

use chrono::{DateTime,Local};

pub struct Jobinfo {
    pub path : PathBuf,
    pub time: DateTime<Local>
}

impl Jobinfo {
    pub fn new(p : PathBuf) -> Jobinfo {
        //let p = pathdiff::diff_paths(p.as_path(),std::env::current_dir().unwrap().as_path()).unwrap();
        Jobinfo {
            path:p,
            time:Local::now()
        }
    }
    pub fn in_file(&self) -> &Path {
        self.path.parent().unwrap()
    }
}

pub struct Interpreter<'a> {
    pub state:State,
    pub jobinfo:Jobinfo,
    pub(crate) mouths:Mouths,
    pub stomach:&'a mut dyn Stomach,
    pub params:&'a dyn InterpreterParams
}
use crate::{TeXErr,FileEnd};
use crate::commands::primitives::{ENDTEMPLATE, LEFT, RIGHT};
use crate::interpreter::params::InterpreterParams;

pub fn string_to_tokens(s : TeXString) -> Vec<Token> {
    use crate::catcodes::OTHER_SCHEME;
    tokenize(s,&OTHER_SCHEME)
}
pub fn tokens_to_string_default(tks:&Vec<Token>) -> TeXString {
    tokens_to_string(tks,&crate::catcodes::OTHER_SCHEME)
}

pub fn tokens_to_string(tks:&Vec<Token>,catcodes:&CategoryCodeScheme) -> TeXString {
    let mut ret : Vec<u8> = vec!();
    let escapechar = catcodes.escapechar;
    for tk in tks {
        match tk.catcode {
            CategoryCode::Escape => {
                let name = tk.name();
                if escapechar != 255 { ret.push(catcodes.escapechar) }
                for s in name.iter() { ret.push(*s) }
                if name.len() > 1 {
                    ret.push(32)
                } else if name.len() == 1 {
                    match catcodes.get_code(*name.iter().first().unwrap()) {
                        CategoryCode::Letter => ret.push(32),
                        _ => ()
                    }
                } else {
                    ret.append(&mut vec!(99,115,110,97,109,101)); // csname
                    if catcodes.escapechar != 255 { ret.push(catcodes.escapechar) }
                    ret.append(&mut vec!(101,110,100,99,115,110,97,109,101)) // endcsname
                }
            }
            _ => ret.push(tk.char)
        }
    }
    ret.into()
}

use crate::stomach::{NoShipoutRoutine, Stomach, StomachMessage, Whatsit};
use crate::stomach::math::{GroupedMath, MathChar, MathGroup, MathKernel};
use crate::interpreter::state::FontStyle;
use crate::stomach::colon::Colon;
use crate::stomach::simple::Indent;
use crate::stomach::whatsits::{PrintChar, SpaceChar, WhatsitTrait};

impl Interpreter<'_> {
    pub fn tokens_to_string(&self,tks:&Vec<Token>) -> TeXString {
        tokens_to_string(tks,self.state.catcodes.get_scheme())
    }

    pub fn kpsewhich(&self,filename: &str) -> Option<(PathBuf,bool)> {
        crate::kpathsea::kpsewhich(filename,self.jobinfo.in_file())
    }

    pub fn get_file(&mut self,filename : &str) -> Result<Arc<VFile>,TeXError> {
        /*if filename.contains("tetrapod") {
            unsafe {crate::LOG = true}
            println!("Here!")
        }*/
        match self.kpsewhich(filename) {
            None =>TeXErr!("File {} not found",filename),
            Some((p,b)) => {
                Ok(VFile::new(&p,b,self.jobinfo.in_file(),&mut self.state.filestore))
            }
        }
    }
    pub fn with_state<'a>(s : State,stomach: &'a mut dyn Stomach,params:&'a dyn InterpreterParams) -> Interpreter<'a> {
        Interpreter {
            state:s,
            jobinfo:Jobinfo::new(PathBuf::new()),
            mouths:Mouths::new(),
            stomach:stomach,
            params
        }
    }

    fn predoc_toploop(&mut self) -> Result<bool,TeXError> {
        while self.has_next() {
            let next = self.next_token();
            let indoc = self.state.indocument;
            if !indoc {
                let isline = match self.state.indocument_line.as_ref() {
                    Some((f,i)) if self.current_file() == *f && self.line_no() > *i => true,
                    _ => false
                };
                if isline {
                    self.state.borrow_mut().indocument_line = None;
                    self.requeue(next);
                    return Ok(true)
                } else {
                    match next.catcode {
                        CategoryCode::Escape if &next.cmdname() == "document" => {
                            self.state.borrow_mut().indocument_line = Some((self.current_file(), self.line_no()))
                        }
                        _ => ()
                    }
                }
            }
            self.do_top(next,false)?
        }
        Ok(false)
    }

    pub fn do_file<A:'static,B:'static>(&mut self,p:&Path,mut colon:A) -> B where A:Colon<B>,B: Send {
        self.jobinfo = Jobinfo::new(p.to_path_buf());
        let vf:Arc<VFile>  = VFile::new(p,false,self.jobinfo.in_file(),&mut self.state.borrow_mut().filestore);
        self.push_file(vf);
        self.insert_every(&crate::commands::primitives::EVERYJOB);
        let cont = match self.predoc_toploop() {
            Ok(b) => b,
            Err(mut e) => {
                e.throw(self);
                self.params.error(e);
                return colon.close()
            }
        };
        if cont {
            let (receiver,fnt,color) = self.stomach.borrow_mut().on_begin_document(&mut self.state);
            colon.initialize(fnt,color,self);
            let mut colonthread = if self.params.singlethreaded() {
                MaybeThread::Single(receiver,Box::new(move |rec,end| {
                    if end {
                        loop {
                            match rec.try_iter().next() {
                                Some(StomachMessage::WI(w)) => colon.ship_whatsit(w),
                                _ => break
                            }
                        }
                        Some(colon.close())
                    } else {
                        match rec.try_iter().next() {
                            Some(StomachMessage::WI(w)) => {
                                colon.ship_whatsit(w);
                                None
                            }
                            Some(StomachMessage::End) => {
                                Some(colon.close())
                            }
                            None => None
                        }
                    }
                }),None)
            } else {
                MaybeThread::Multi(std::thread::spawn(move || {
                    for msg in receiver {
                        match msg {
                            StomachMessage::End => return colon.close(),
                            StomachMessage::WI(w) => colon.ship_whatsit(w)
                        }
                    }
                    return colon.close() // sender dropped => TeXError somewhere
                }))
            };
            /*std::thread::spawn(move || {
                for msg in receiver {
                    match msg {
                        StomachMessage::End => return colon.close(),
                        StomachMessage::WI(w) => colon.ship_whatsit(w)
                    }
                }
                return colon.close() // sender dropped => TeXError somewhere
            });*/

            while self.has_next() {
                colonthread.next();
                let next = self.next_token();
                match self.do_top(next,false) {
                    Ok(b) => b,
                    Err(mut e) =>  {
                        e.throw(self);
                        self.params.error(e);
                        self.stomach.finish(&mut self.state);
                        return match colonthread.join() {
                            Ok(r) => r,
                            _ => panic!("Error in colon thread")
                        }
                    }
                };
            }

            self.stomach.borrow_mut().finish(&mut self.state);
            match colonthread.join() {
                Ok(r) => return r,
                Err(_) => panic!("Error in colon thread")
            }
        } else {
            colon.close()
        }
    }
    pub fn do_file_with_state<A:'static,B:'static>(p : &Path, s : State, colon:A,params:&dyn InterpreterParams) -> (State,B) where A:Colon<B>,B:Send {
        let mut stomach = NoShipoutRoutine::new();
        let mut int = Interpreter::with_state(s,stomach.borrow_mut(),params);
        let ret = int.do_file(p,colon);
        (int.state,ret)
    }

    pub fn get_command(&self,s : &TeXStr) -> Result<TeXCommand,TeXError> {
        match self.state.commands.get(s) {
            Some(p) => Ok(p),
            None if s.len() == 0 => {
                let catcode = CategoryCode::Other;//self.catcodes.borrow().get_code(char);
                let tk = Token::new(self.state.catcodes.get_scheme().endlinechar,catcode,None,None,true);
                Ok(PrimitiveTeXCommand::Char(tk).as_command())
            }
            None if s.len() == 1 => {
                let char = *s.iter().first().unwrap();
                let tk = Token::new(char,CategoryCode::Other,None,None,true);
                Ok(PrimitiveTeXCommand::Char(tk).as_command())
            }
            None => TeXErr!("Unknown control sequence: \\{}",s)
        }
    }

    pub fn do_top(&mut self,next:Token,inner:bool) -> Result<(),TeXError> {
        use crate::commands::primitives;
        use crate::catcodes::CategoryCode::*;
        use TeXMode::*;
        use PrimitiveTeXCommand::*;

        let mode = self.state.mode;
        /*if self.current_line().starts_with("/home/jazzpirate/work/LaTeX/Papers/19 - Thesis/img/int-partial-biview.tex (14, 65)") {
            unsafe { crate::LOG = true }
            println!("Here!: {}",self.preview())

        }*/
        match (next.catcode,mode) {
            (EOL,_) if next.name() == "EOF" => Ok(()),
            (Active | Escape,_) => {
                let p = self.get_command(&next.cmdname())?;
                if p.assignable() {
                    return p.assign(next,self,false)
                } else if p.expandable(true) {
                    return p.expand(next,self)
                }
                match (&*p.orig,mode) {
                    (Primitive(p),Vertical | InternalVertical) if **p == primitives::PAR => {
                        self.stomach.reset_par(&mut self.state);
                        Ok(())
                    },
                    (Primitive(p),Vertical | InternalVertical) if **p == primitives::INDENT || **p == primitives::NOINDENT => {
                        self.switch_to_h(next)
                    }
                    (Primitive(p),Horizontal) if **p == primitives::PAR => self.end_paragraph(inner),
                    (Primitive(np),_) => {
                        let mut exp = Expansion::new(next,p.orig.clone());
                        np.apply(&mut exp,self)?;
                        if !exp.2.is_empty() {
                            self.push_expansion(exp)
                        }
                        Ok(())
                    },
                    (Ext(exec),_) =>
                        exec.execute(self).map_err(|x| x.derive("External Command ".to_owned() + &exec.name() + " errored!")),
                    (Char(tk),_) => {
                        self.requeue(tk.clone());
                        Ok(())
                    },
                    (Whatsit(w),_) if w.allowed_in(mode) => {
                        let next = w.get(&next,self)?;
                        self.stomach_add(next)
                    },
                    (Whatsit(w), Vertical | InternalVertical) if w.allowed_in(TeXMode::Horizontal) => {
                        self.switch_to_h(next)
                    }
                    (Whatsit(w), Horizontal) if w.allowed_in(TeXMode::Vertical) => {
                        self.requeue(next);
                        self.end_paragraph(inner)
                    }
                   /* (Whatsit(crate::commands::ProvidesWhatsit::Simple(p)), RestrictedHorizontal) if **p == VFIL || **p == VFILL => {
                        Ok(())
                    }*/
                    _ => TeXErr!(next.clone() => "TODO: {} in {}",next,self.current_line())

                }
            },
            (BeginGroup,_) => Ok(self.state.push(self.stomach,GroupType::Token)),
            (EndGroup,_) => self.pop_group(GroupType::Token),
            (Space | EOL, Vertical | InternalVertical | Math | Displaymath ) => Ok(()),
            (Space | EOL, Horizontal | RestrictedHorizontal) => {
                let font = self.state.currfont.get(&());
                let sourceref = self.update_reference(&next);
                self.stomach_add(crate::stomach::Whatsit::Space(SpaceChar {
                    font,sourceref,nonbreaking:false
                }))
            }
            (Letter | Other , Horizontal | RestrictedHorizontal) => {
                let font = self.state.currfont.get(&());
                let sourceref = self.update_reference(&next);
                self.stomach_add(crate::stomach::Whatsit::Char(PrintChar {
                    char:next.char,
                    font,sourceref
                }))
            }
            (MathShift, Horizontal) => {
                self.do_math(false)
            },
            (MathShift, RestrictedHorizontal) => {
                self.do_math(true)
            },
            (Letter | Other, Math | Displaymath) => {
                let mc = self.state.mathcodes.get(&next.char);
                match mc {
                    32768 => {
                        self.requeue(Token::new(next.char,CategoryCode::Active,None,None,true));
                        Ok(())
                    }
                    _ => {
                        let wi = self.do_math_char(Some(next),mc as u32);
                        self.stomach_add(crate::stomach::Whatsit::Math(MathGroup::new(
                              crate::stomach::math::MathKernel::MathChar(wi),
                              self.state.displaymode.get(&()))))?;
                        Ok(())
                    }
                }
            }
            (Letter | Other | MathShift,Vertical | InternalVertical) => self.switch_to_h(next),
            (AlignmentTab,_) => {
                let align = self.state.borrow_mut().aligns.pop();
                self.state.borrow_mut().aligns.push(None);
                match align {
                    Some(Some(v)) => {
                        self.requeue(ENDTEMPLATE.try_with(|x| x.clone()).unwrap());
                        self.push_tokens(v);
                        Ok(())
                    }
                    _ => TeXErr!(next => "Misplaced alignment tab")
                }
            }
            _ => TeXErr!(next.clone() => "Urgh: {}",next),
        }
    }

    pub fn do_math_char(&self,tk:Option<Token>,mc:u32) -> MathChar {
        let num = mc;
        let (mut cls,mut fam,pos) = {
            if num == 0 && tk.is_some() {
                (0,1,tk.as_ref().unwrap().char as u32)
            } else {
                let char = num % (16 * 16);
                let rest = (num - char) / (16 * 16);
                let fam = rest % 16;
                (((rest - fam) / 16) % 16, fam, char)
            }
        };
        if cls == 7 {
            match self.state.registers.get(&-(crate::commands::primitives::FAM.index as i32)) {
                i if i < 0 || i > 15 => {
                    cls = 0;
                    //num = 256 * fam + pos
                }
                i => {
                    cls = 0;
                    fam = i as u32;
                    //num = 256 * fam + pos
                }
            }
        }
        let mode = self.state.fontstyle.get(&());
        let font = match mode {
            FontStyle::Text => self.state.textfonts.get(&(fam as usize)),
            FontStyle::Script => self.state.scriptfonts.get(&(fam as usize)),
            FontStyle::Scriptscript => self.state.scriptscriptfonts.get(&(fam as usize)),
        };
        crate::stomach::math::MathChar {
            class:cls,family:fam,position:pos,font,
            sourceref:match &tk {
                Some(tk) => self.update_reference(tk),
                _ => None
            }
        }
    }

    fn switch_to_h(&mut self, next:Token) -> Result<(),TeXError> {
        let indent = match next.catcode {
            CategoryCode::Escape | CategoryCode::Active => {
                let pr = self.get_command(&next.cmdname())?;
                match &*pr.orig {
                    PrimitiveTeXCommand::Primitive(c) if **c == crate::commands::primitives::NOINDENT => 0,
                    PrimitiveTeXCommand::Primitive(c) if **c == crate::commands::primitives::INDENT =>
                        self.state.dimensions.get(&-(crate::commands::primitives::PARINDENT.index as i32)),
                    _ => {
                        self.requeue(next);
                        self.state.dimensions.get(&-(crate::commands::primitives::PARINDENT.index as i32))
                    }
                }
            }
            _ => {
                self.requeue(next);
                self.state.dimensions.get(&-(crate::commands::primitives::PARINDENT.index as i32))
            }
        };
        self.state.borrow_mut().mode = TeXMode::Horizontal;
        self.insert_every(&crate::commands::primitives::EVERYPAR);
        let parskip = self.state.skips.get(&-(crate::commands::primitives::PARSKIP.index as i32));
        self.stomach.borrow_mut().start_paragraph(parskip.base);
        if indent != 0 {
            self.stomach_add(Indent {
                dim: indent,
                sourceref: None
            }.as_whatsit())?
        }
        Ok(())
    }

    fn end_paragraph(&mut self,inner : bool) -> Result<(),TeXError> {
        if inner { self.state.mode = TeXMode::InternalVertical }
        else { self.state.mode = TeXMode::Vertical }
        self.stomach.end_paragraph(&mut self.state)?;
        let vadjusts = std::mem::take(&mut self.state.vadjust);
        for w in vadjusts {
            self.stomach_add(w)?
        }
        Ok(())
    }

    fn do_math(&mut self, inner : bool) -> Result<(),TeXError> {
        use crate::catcodes::CategoryCode::*;
        use crate::stomach::Whatsit as WI;
        self.state.push(self.stomach,GroupType::Math);
        let mode = if inner {
            self.insert_every(&crate::commands::primitives::EVERYMATH);
            TeXMode::Math
        } else {
            let next = self.next_token();
            match next.catcode {
                MathShift => {
                    self.insert_every(&crate::commands::primitives::EVERYDISPLAY);
                    self.state.displaymode.set((),true,false);
                    TeXMode::Displaymath
                }
                _ => {
                    self.requeue(next);
                    self.insert_every(&crate::commands::primitives::EVERYMATH);
                    TeXMode::Math
                }
            }
        };
        let _oldmode = self.state.mode;
        self.state.mode = mode;

        let mut mathgroup: Option<MathGroup> = None;
        while self.has_next() {
            let next = self.next_token();
            match next.catcode {
                MathShift if mode == TeXMode::Displaymath => {
                    let nnext = self.next_token();
                    match nnext.catcode {
                        MathShift => {
                            self.state.mode = _oldmode;
                            for g in mathgroup.take() {
                                self.stomach_add(WI::Math(g))?
                            }
                            let mut ret = self.get_whatsit_group(GroupType::Math)?;
                            {
                                let mut first : Vec<WI> = vec!();
                                let mut second : Vec<WI> = vec!();
                                for x in ret.drain(0..) {
                                    if !second.is_empty() {
                                        second.push(x)
                                    } else {
                                        match x {
                                            WI::Above(_) => second.push(x),
                                            _ => first.push(x)
                                        }
                                    }
                                }
                                if second.is_empty() {
                                    ret = first
                                } else {
                                    let head = second.remove(0);
                                    match head {
                                        WI::Above(mut mi) => {
                                            mi.set(first,second);
                                            ret = vec!(WI::Above(mi))
                                        },
                                        _ => TeXErr!("Should be unreachable!")
                                    }
                                }
                            }
                            self.stomach_add(WI::Math(MathGroup::new(MathKernel::Group(GroupedMath(ret)),true)))?;
                            return Ok(())
                        }
                        _ => TeXErr!(nnext => "displaymode must be closed with $$")
                    }
                },
                MathShift => {
                    self.state.mode = _oldmode;
                    for g in mathgroup.take() {
                        self.stomach_add(WI::Math(g))?
                    }
                    let ret = self.get_whatsit_group(GroupType::Math)?;
                    self.stomach_add(WI::Math(MathGroup::new(MathKernel::Group(GroupedMath(ret)),false)))?;
                    return Ok(())
                }
                EndGroup => TeXErr!(next => "{}","Unexpected } in math environment"),
                _ => {
                    let p = self.get_command(&next.cmdname());
                    match p {
                        Ok(tc) => match &*tc.orig {
                            PrimitiveTeXCommand::Whatsit(ProvidesWhatsit::Math(mw)) if **mw == RIGHT => {
                                TeXErr!(next => "{}","Unexpected \\right in math environment")
                            }
                            _ => ()
                        }
                        _ => ()
                    }
                    self.requeue(next);
                    let ret = self.read_math_whatsit(match mathgroup.as_mut() {
                        Some(mg) => Some(mg),
                        _ => None
                    })?;
                    match ret {
                        Some(WI::Ls(v)) if v.is_empty() => (),
                        Some(WI::Ls(mut v)) => {
                            for g in mathgroup.take() {
                                self.stomach_add(WI::Math(g))?
                            }
                            let last = v.pop();
                            for w in v { self.stomach_add(w)? }
                            match last {
                                Some(WI::Math(mg)) => {
                                    match mathgroup.replace(mg) {
                                        Some(m) => self.stomach_add(WI::Math(m))?,
                                        _ => ()
                                    }
                                },
                                Some(w) => self.stomach_add(w)?,
                                None => ()
                            }
                        }
                        Some(WI::Math(mg)) => {
                            match mathgroup.replace(mg) {
                                Some(m) => self.stomach_add(WI::Math(m))?,
                                _ => ()
                            }
                        },
                        Some(w) => {
                            for g in mathgroup.take() {
                                self.stomach_add(WI::Math(g))?
                            }
                            self.stomach_add(w)?
                        },
                        None => ()
                    }
                }
            }
        }
        FileEnd!()
    }

    fn read_math_group(&mut self,finish: fn(&Token,&mut Interpreter) -> Result<bool,TeXError>) -> Result<Option<()>,TeXError> {
        use crate::catcodes::CategoryCode::*;
        use crate::commands::PrimitiveTeXCommand::*;
        use crate::stomach::Whatsit as WI;
        use crate::commands::ProvidesWhatsit;

        let mut mathgroup: Option<MathGroup> = None;

        while self.has_next() {
            let next = self.next_token();
            if finish(&next,self)? {
                self.requeue(next);
                for g in mathgroup.take() {
                    self.stomach_add(WI::Math(g))?
                }
                return Ok(Some(()))
            }
            match next.catcode {
                EndGroup => TeXErr!(next => "{}","Unexpected } in math environment"),
                Space | EOL=> (),
                Active | Escape => {
                    let p = self.get_command(&next.cmdname())?;
                    if p.assignable() {
                        p.assign(next,self,false)?
                    } else if p.expandable(true) {
                        p.expand(next,self)?
                    } else {
                        self.requeue(next);
                        let ret = self.read_math_whatsit(match mathgroup.as_mut() {
                            Some(mg) => Some(mg),
                            _ => None
                        })?;
                        match ret {
                            Some(WI::Ls(v)) if v.is_empty() => (),
                            Some(WI::Ls(mut v)) => {
                                for g in mathgroup.take() {
                                    self.stomach_add(WI::Math(g))?
                                }
                                let last = v.pop();
                                for w in v { self.stomach_add(w)? }
                                match last {
                                    Some(WI::Math(mg)) => {
                                        match mathgroup.replace(mg) {
                                            Some(m) => self.stomach_add(WI::Math(m))?,
                                            _ => ()
                                        }
                                    },
                                    Some(w) => self.stomach_add(w)?,
                                    None => ()
                                }
                            }
                            Some(WI::Math(mg)) => {
                                match mathgroup.replace(mg) {
                                    Some(m) => self.stomach_add(WI::Math(m))?,
                                    _ => ()
                                }
                            },
                            Some(w) => {
                                for g in mathgroup.take() {
                                    self.stomach_add(WI::Math(g))?
                                }
                                self.stomach_add(w)?
                            },
                            None => {
                                let next = self.next_token();
                                match next.catcode {
                                    CategoryCode::MathShift => {
                                        return Ok(None)
                                    }
                                    _ => {
                                        self.requeue(next)
                                    }
                                }
                            }
                        }
                    }
                }
                _ => {
                    self.requeue(next);
                    let ret = self.read_math_whatsit(match mathgroup.as_mut() {
                        Some(mg) => Some(mg),
                        _ => None
                    })?;
                    match ret {
                        Some(WI::Ls(v)) if v.is_empty() => (),
                        Some(WI::Ls(mut v)) => {
                            for g in mathgroup.take() {
                                self.stomach_add(WI::Math(g))?
                            }
                            let last = v.pop();
                            for w in v { self.stomach_add(w)? }
                            match last {
                                Some(WI::Math(mg)) => {
                                    match mathgroup.replace(mg) {
                                        Some(m) => self.stomach_add(WI::Math(m))?,
                                        _ => ()
                                    }
                                },
                                Some(w) => self.stomach_add(w)?,
                                None => ()
                            }
                        }
                        Some(WI::Math(mg)) => {
                            match mathgroup.replace(mg) {
                                Some(m) => self.stomach_add(WI::Math(m))?,
                                _ => ()
                            }
                        },
                        Some(w) => {
                            for g in mathgroup.take() {
                                self.stomach_add(WI::Math(g))?
                            }
                            self.stomach_add(w)?
                        },
                        None => {
                            let next = self.next_token();
                            match next.catcode {
                                CategoryCode::MathShift => {
                                    return Ok(None)
                                }
                                _ => {
                                    self.requeue(next)
                                }
                            }
                        }
                    }
                }
            }

        }
        FileEnd!()
    }

    pub fn read_math_whatsit(&mut self,previous: Option<&mut MathGroup>) -> Result<Option<Whatsit>,TeXError> {
        use crate::catcodes::CategoryCode::*;
        use crate::commands::PrimitiveTeXCommand::*;
        use crate::stomach::Whatsit as WI;
        use crate::commands::ProvidesWhatsit;
        while self.has_next() {
            let next = self.next_token();
            match next.catcode {
                MathShift | EndGroup => {
                    self.requeue(next);
                    return Ok(None)
                }
                BeginGroup => {
                    self.state.push(self.stomach,GroupType::Math);
                    match self.read_math_group(|t,_| Ok(t.catcode == EndGroup))? {
                        None => return Ok(None),
                        _ => ()
                    }
                    self.next_token();

                    let mut ret = self.get_whatsit_group(GroupType::Math)?;
                    {
                        let mut first : Vec<WI> = vec!();
                        let mut second : Vec<WI> = vec!();
                        for x in ret.drain(0..) {
                            if !second.is_empty() {
                                second.push(x)
                            } else {
                                match x {
                                    WI::Above(_) => second.push(x),
                                    _ => first.push(x)
                                }
                            }
                        }
                        if second.is_empty() {
                            ret = first
                        } else {
                            let head = second.remove(0);
                            match head {
                                WI::Above(mut mi) => {
                                    mi.set(first,second);
                                    ret = vec!(WI::Above(mi))
                                },
                                _ => TeXErr!("Should be unreachable!")
                            }
                        }
                    }
                    return Ok(Some(WI::Math(MathGroup::new(MathKernel::Group(GroupedMath(ret)),self.state.displaymode.get(&())))))
                },
                Superscript => {
                    let oldmode = self.state.fontstyle.get(&());
                    //println!("Here! {}",self.preview());
                    //unsafe { crate::LOG = true }
                    self.state.fontstyle.set((),oldmode.inc(),false);
                    let read = self.read_math_whatsit(None)?;
                    let ret = match read {
                        Some(WI::Math(m)) if m.subscript.is_none() && m.superscript.is_none() => m.kernel,
                        _ => {
                            TeXErr!(next => "Expected Whatsit after ^")
                        }
                    };
                    self.state.fontstyle.set((),oldmode,false);
                    match previous {
                        Some(mg) => {
                            mg.superscript = Some(ret);
                            return Ok(None)
                        },
                        _ => {
                            let mut mg = MathGroup::new(MathKernel::Group(GroupedMath(vec!())),self.state.displaymode.get(&()));
                            mg.superscript = Some(ret);
                            return Ok(Some(WI::Math(mg)))
                        },
                    }
                }
                Subscript => {
                    let oldmode = self.state.fontstyle.get(&());
                    self.state.fontstyle.set((),oldmode.inc(),false);
                    let read = self.read_math_whatsit(None)?;
                    let ret = match read {
                        Some(WI::Math(m)) if m.subscript.is_none() && m.superscript.is_none() => m.kernel,
                        _ => TeXErr!(next => "Expected Whatsit after _")
                    };
                    self.state.fontstyle.set((),oldmode,false);
                    match previous {
                        Some(mg) => {
                            mg.subscript = Some(ret);
                            return Ok(None)
                        },
                        _ => {
                            let mut mg = MathGroup::new(MathKernel::Group(GroupedMath(vec!())),self.state.displaymode.get(&()));
                            mg.subscript = Some(ret);
                            return Ok(Some(WI::Math(mg)))
                        },
                    }
                }
                Active | Escape => {
                    let p = self.get_command(&next.cmdname())?;
                    if p.assignable() {
                        p.assign(next,self,false)?
                    } else if p.expandable(true) {
                        p.expand(next,self)?
                    } else {
                        match &*p.orig {
                            Whatsit(ProvidesWhatsit::Math(mw)) if **mw == RIGHT => {
                                self.requeue(next);
                                return Ok(None)
                            }
                            Whatsit(ProvidesWhatsit::Math(mw)) if **mw == LEFT => {
                                self.state.push(self.stomach,GroupType::LeftRight);
                                (LEFT._get)(&next,self,None)?;
                                match self.read_math_group(|t,i| Ok((t.catcode == Active || t.catcode == Escape) && {
                                    let p = i.get_command(&t.cmdname())?;
                                    match &*p.orig {
                                        Whatsit(ProvidesWhatsit::Math(mw)) if **mw == RIGHT => true,
                                        _ => false
                                    }
                                }))? {
                                    None => return Ok(None),
                                    _ => ()
                                }
                                let next = self.next_token();
                                (RIGHT._get)(&next,self,None)?;

                                let mut ret = self.get_whatsit_group(GroupType::LeftRight)?;
                                {
                                    let mut first : Vec<WI> = vec!();
                                    let mut second : Vec<WI> = vec!();
                                    for x in ret.drain(0..) {
                                        if !second.is_empty() {
                                            second.push(x)
                                        } else {
                                            match x {
                                                WI::Above(_) => second.push(x),
                                                _ => first.push(x)
                                            }
                                        }
                                    }
                                    if second.is_empty() {
                                        ret = first
                                    } else {
                                        let head = second.remove(0);
                                        match head {
                                            WI::Above(mut mi) => {
                                                mi.set(first,second);
                                                ret = vec!(WI::Above(mi))
                                            },
                                            _ => TeXErr!("Should be unreachable!")
                                        }
                                    }
                                }
                                return Ok(Some(WI::Math(MathGroup::new(MathKernel::Group(GroupedMath(ret)),self.state.displaymode.get(&())))))
                            }
                            Primitive(np) => {
                                let mut exp = Expansion::new(next, p.orig.clone());
                                np.apply(&mut exp, self)?;
                                if !exp.2.is_empty() {
                                    self.push_expansion(exp)
                                }
                            },
                            Ext(exec) =>
                                exec.execute(self).map_err(|x| x.derive("External Command ".to_owned() + &exec.name() + " errored!"))?,
                            Char(tk) => {
                                self.requeue(tk.clone())
                            },
                            Whatsit(ProvidesWhatsit::Math(mw)) => {
                                return match (mw._get)(&next,self,previous)? {
                                    Some(k) => Ok(Some(WI::Math(MathGroup::new(k,self.state.displaymode.get(&()))))),
                                    _ => Ok(None)
                                }
                            },
                            Whatsit(w) if w.allowed_in(self.state.mode) => {
                                let next = w.get(&next, self)?;
                                return Ok(Some(next))
                            },
                            MathChar(mc) => match mc {
                                32768 => {
                                    self.requeue(Token::new(next.char, CategoryCode::Active, None, None, true))
                                }
                                _ => {
                                    let wi = self.do_math_char(Some(next),*mc);
                                    let ret = crate::stomach::Whatsit::Math(MathGroup::new(
                                        crate::stomach::math::MathKernel::MathChar(wi),
                                        self.state.displaymode.get(&())));
                                    return Ok(Some(ret))
                                }
                            },
                            _ => TeXErr!(next.clone() => "TODO: {} in {}",next,self.current_line())
                        }
                    }
                },
                Space | EOL=> (),
                Letter | Other => {
                    let mc = self.state.mathcodes.get(&(next.char as u8));
                    match mc {
                        32768 => {
                            self.requeue(Token::new(next.char,CategoryCode::Active,None,None,true))
                        }
                        _ => {
                            let wi = self.do_math_char(Some(next),mc as u32);
                            let ret = crate::stomach::Whatsit::Math(MathGroup::new(
                                crate::stomach::math::MathKernel::MathChar(wi),
                                self.state.displaymode.get(&())));
                            return Ok(Some(ret))
                        }
                    }
                }
                AlignmentTab => {
                    let align = self.state.borrow_mut().aligns.pop();
                    self.state.borrow_mut().aligns.push(None);
                    match align {
                        Some(Some(v)) => {
                            self.requeue(ENDTEMPLATE.try_with(|x| x.clone()).unwrap());
                            self.push_tokens(v);
                            ()
                        }
                        _ => TeXErr!(next => "Misplaced alignment tab")
                    }
                }
                _ => TeXErr!(next.clone() => "Urgh: {}",next),
            }
        }
        FileEnd!()
    }

    /*pub fn assert_has_next(&self) -> Result<(),TeXError> {
        if self.has_next() {Ok(())} else  {
            FileEnd!(self)
        }
    } */
}