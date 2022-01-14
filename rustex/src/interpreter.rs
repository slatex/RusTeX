
#[derive(Copy,Clone,PartialEq)]
pub enum TeXMode {
    Vertical, InternalVertical, Horizontal, RestrictedHorizontal, Math, Displaymath
}

use std::borrow::BorrowMut;
use std::cell::RefCell;
use crate::ontology::{Expansion, Token};
use crate::catcodes::{CategoryCode, CategoryCodeScheme};
use crate::references::SourceReference;
use std::path::{Path, PathBuf};
use crate::commands::{TeXCommand,PrimitiveTeXCommand};
use crate::interpreter::files::{VFile};
use crate::interpreter::mouth::Mouths;
use crate::interpreter::state::{GroupType, State, StateChange};
use crate::utils::{TeXError, TeXString, TeXStr, MaybeThread};
use std::sync::Arc;

pub mod mouth;
pub mod state;
mod files;
pub mod dimensions;
pub mod methods;


pub fn tokenize(s : TeXString,cats: &CategoryCodeScheme) -> Vec<Token> {
    let mut retvec: Vec<Token> = Vec::new();
    for next in s.0 {
        retvec.push(Token::new(next,cats.get_code(next),None,SourceReference::None,true))
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
    pub (in crate) state:RefCell<State>,
    pub jobinfo:Jobinfo,
    mouths:RefCell<Mouths>,
    catcodes:RefCell<CategoryCodeScheme>,
    pub stomach:RefCell<&'a mut dyn Stomach>
}
use crate::{TeXErr,FileEnd};
use crate::commands::primitives::ENDTEMPLATE;

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
        let catcodes = self.catcodes.borrow();
        tokens_to_string(tks,&catcodes)
    }

    pub fn kpsewhich(&self,filename: &str) -> Option<(PathBuf,bool)> {
        crate::kpathsea::kpsewhich(filename,self.jobinfo.in_file())
    }

    pub fn get_file(&self,filename : &str) -> Result<Arc<VFile>,TeXError> {
        /*if filename.contains("tetrapod") {
            unsafe {crate::LOG = true}
            println!("Here!")
        }*/
        match self.kpsewhich(filename) {
            None =>TeXErr!((self,None),"File {} not found",filename),
            Some((p,b)) => {
                Ok(VFile::new(&p,b,self.jobinfo.in_file(),&mut self.state.borrow_mut().filestore))
            }
        }
    }
    pub fn with_state(s : State,stomach: &mut dyn Stomach) -> Interpreter {
        let catcodes = s.catcodes().clone();
        Interpreter {
            state:RefCell::new(s),
            jobinfo:Jobinfo::new(PathBuf::new()),
            mouths:RefCell::new(Mouths::new()),
            catcodes:RefCell::new(catcodes),
            stomach:RefCell::new(stomach)
        }
    }

    fn predoc_toploop(&self) -> Result<bool,TeXError> {
        while self.has_next() {
            let next = self.next_token();
            let indoc = self.state.borrow().indocument;
            if !indoc {
                let isline = match self.state.borrow().indocument_line.as_ref() {
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
            Err(e) => return e.throw(Some(self))
        };
        if cont {
            let (receiver,fnt,color) = self.stomach.borrow_mut().on_begin_document(self);
            colon.initialize(fnt,color,self);
            let mut colonthread = if crate::SINGLETHREADED {
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
                    Err(e) => return e.throw(Some(self))
                };
            }

            self.stomach.borrow_mut().finish(self);
            match colonthread.join() {
                Ok(r) => return r,
                Err(_) => panic!("Error in colon thread")
            }
        } else {
            colon.close()
        }
    }
    pub fn do_file_with_state<A:'static,B:'static>(p : &Path, s : State, colon:A) -> (State,B) where A:Colon<B>,B:Send {
        let mut stomach = NoShipoutRoutine::new();
        let mut int = Interpreter::with_state(s,stomach.borrow_mut());
        let ret = int.do_file(p,colon);
        let st = int.state.borrow().clone();
        (st.close(int),ret)
    }

    pub fn get_command(&self,s : &TeXStr) -> Result<TeXCommand,TeXError> {
        match self.state.borrow().get_command(s) {
            Some(p) => Ok(p),
            None if s.len() == 0 => {
                let catcode = CategoryCode::Other;//self.catcodes.borrow().get_code(char);
                let tk = Token::new(self.catcodes.borrow().endlinechar,catcode,None,SourceReference::None,true);
                Ok(PrimitiveTeXCommand::Char(tk).as_command())
            }
            None if s.len() == 1 => {
                let char = *s.iter().first().unwrap();
                let catcode = CategoryCode::Other;//self.catcodes.borrow().get_code(char);
                let tk = Token::new(char,catcode,None,SourceReference::None,true);
                Ok(PrimitiveTeXCommand::Char(tk).as_command())
            }
            None => TeXErr!((self,None),"Unknown control sequence: \\{}",s)
        }
    }

    pub fn do_top(&self,next:Token,inner:bool) -> Result<(),TeXError> {
        use crate::commands::primitives;
        use crate::catcodes::CategoryCode::*;
        use TeXMode::*;
        use PrimitiveTeXCommand::*;

        let mode = self.get_mode();
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
                        self.stomach.borrow_mut().reset_par();
                        Ok(())
                    },
                    (Primitive(p),Vertical | InternalVertical) if **p == primitives::INDENT || **p == primitives::NOINDENT => {
                        self.switch_to_H(next)
                    }
                    (Primitive(p),Horizontal) if **p == primitives::PAR => self.end_paragraph(inner),
                    (Primitive(np),_) => {
                        let mut exp = Expansion(next,Arc::new(p.clone()),vec!());
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
                        self.stomach.borrow_mut().add(self,next)
                    },
                    (Whatsit(w), Vertical | InternalVertical) if w.allowed_in(TeXMode::Horizontal) => {
                        self.switch_to_H(next)
                    }
                    (Whatsit(w), Horizontal) if w.allowed_in(TeXMode::Vertical) => {
                        self.requeue(next);
                        self.end_paragraph(inner)
                    }
                    _ => TeXErr!((self,Some(next.clone())),"TODO: {} in {}",next,self.current_line())

                }
            },
            (BeginGroup,_) => Ok(self.new_group(GroupType::Token)),
            (EndGroup,_) => self.pop_group(GroupType::Token),
            (Space | EOL, Vertical | InternalVertical | Math | Displaymath ) => Ok(()),
            (Space | EOL, Horizontal | RestrictedHorizontal) => {
                let font = self.get_font();
                let sourceref = self.update_reference(&next);
                self.stomach.borrow_mut().add(self,crate::stomach::Whatsit::Space(SpaceChar {
                    font,sourceref
                }))
            }
            (Letter | Other , Horizontal | RestrictedHorizontal) => {
                let font = self.get_font();
                let sourceref = self.update_reference(&next);
                self.stomach.borrow_mut().add(self,crate::stomach::Whatsit::Char(PrintChar {
                    char:next.char,
                    font,sourceref
                }))
            }
            (MathShift, Horizontal) => self.do_math(false),
            (MathShift, RestrictedHorizontal) => self.do_math(true),
            (Letter | Other, Math | Displaymath) => {
                let mc = self.state_get_mathcode(next.char as u8);
                match mc {
                    32768 => {
                        self.requeue(Token::new(next.char,CategoryCode::Active,None,SourceReference::None,true));
                        Ok(())
                    }
                    _ => {
                        let wi = self.do_math_char(Some(next),mc as u32);
                        self.stomach.borrow_mut().add(self,
                          crate::stomach::Whatsit::Math(MathGroup::new(
                              crate::stomach::math::MathKernel::MathChar(wi),
                              self.state.borrow().display_mode())))?;
                        Ok(())
                    }
                }
            }
            (Letter | Other | MathShift,Vertical | InternalVertical) => self.switch_to_H(next),
            (AlignmentTab,_) => {
                let align = self.state.borrow_mut().aligns.pop();
                self.state.borrow_mut().aligns.push(None);
                match align {
                    Some(Some(v)) => {
                        self.requeue(ENDTEMPLATE.try_with(|x| x.clone()).unwrap());
                        self.push_tokens(v);
                        Ok(())
                    }
                    _ => TeXErr!((self,Some(next)),"Misplaced alignment tab")
                }
            }
            _ => TeXErr!((self,Some(next.clone())),"Urgh: {}",next),
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
            match self.state_register(-(crate::commands::primitives::FAM.index as i32)) {
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
        let mode = self.state.borrow().font_style();
        let font = match mode {
            FontStyle::Text => self.state.borrow().get_text_font(fam as u8),
            FontStyle::Script => self.state.borrow().get_script_font(fam as u8),
            FontStyle::Scriptscript => self.state.borrow().get_scriptscript_font(fam as u8),
        };
        crate::stomach::math::MathChar {
            class:cls,family:fam,position:pos,font,
            sourceref:match &tk {
                Some(tk) => self.update_reference(tk),
                _ => None
            }
        }
    }

    fn switch_to_H(&self,next:Token) -> Result<(),TeXError> {
        let indent = match next.catcode {
            CategoryCode::Escape | CategoryCode::Active => {
                let pr = self.get_command(&next.cmdname())?;
                match &*pr.orig {
                    PrimitiveTeXCommand::Primitive(c) if **c == crate::commands::primitives::NOINDENT => 0,
                    PrimitiveTeXCommand::Primitive(c) if **c == crate::commands::primitives::INDENT =>
                        self.state_dimension(-(crate::commands::primitives::PARINDENT.index as i32)),
                    _ => {
                        self.requeue(next);
                        self.state_dimension(-(crate::commands::primitives::PARINDENT.index as i32))
                    }
                }
            }
            _ => {
                self.requeue(next);
                self.state_dimension(-(crate::commands::primitives::PARINDENT.index as i32))
            }
        };
        self.state.borrow_mut().mode = TeXMode::Horizontal;
        self.insert_every(&crate::commands::primitives::EVERYPAR);
        let parskip = self.state_skip(-(crate::commands::primitives::PARSKIP.index as i32));
        self.stomach.borrow_mut().start_paragraph(self,parskip.base);
        if indent != 0 {
            self.stomach.borrow_mut().add(self, Indent {
                dim: indent,
                sourceref: None
            }.as_whatsit())?
        }
        Ok(())
    }

    fn end_paragraph(&self,inner : bool) -> Result<(),TeXError> {
        if inner { self.set_mode(TeXMode::InternalVertical) } else { self.set_mode(TeXMode::Vertical) }
        self.stomach.borrow_mut().end_paragraph(self)?;
        let vadjusts = std::mem::take(&mut self.state.borrow_mut().vadjust);
        for w in vadjusts {
            self.stomach.borrow_mut().add(self,w)?
        }
        Ok(())
    }

    fn do_math(&self, inner : bool) -> Result<(),TeXError> {
        use crate::catcodes::CategoryCode::*;
        use crate::stomach::Whatsit as WI;
        self.new_group(GroupType::Math);
        let mode = if inner {
            self.insert_every(&crate::commands::primitives::EVERYMATH);
            TeXMode::Math
        } else {
            let next = self.next_token();
            match next.catcode {
                MathShift => {
                    self.insert_every(&crate::commands::primitives::EVERYDISPLAY);
                    self.change_state(StateChange::Displaymode(true));
                    TeXMode::Displaymath
                }
                _ => {
                    self.requeue(next);
                    self.insert_every(&crate::commands::primitives::EVERYMATH);
                    TeXMode::Math
                }
            }
        };
        let _oldmode = self.get_mode();
        self.set_mode(mode);

        let mut mathgroup: Option<MathGroup> = None;
        while self.has_next() {
            let next = self.next_token();
            match next.catcode {
                MathShift if mode == TeXMode::Displaymath => {
                    let nnext = self.next_token();
                    match nnext.catcode {
                        MathShift => {
                            self.set_mode(_oldmode);
                            for g in mathgroup.take() {
                                self.stomach.borrow_mut().add(self,WI::Math(g))?
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
                                        _ => unreachable!()
                                    }
                                }
                            }
                            self.stomach.borrow_mut().add(self,WI::Math(MathGroup::new(MathKernel::Group(GroupedMath(ret)),true)))?;
                            return Ok(())
                        }
                        _ => TeXErr!((self,Some(nnext)),"displaymode must be closed with $$")
                    }
                },
                MathShift => {
                    self.set_mode(_oldmode);
                    for g in mathgroup.take() {
                        self.stomach.borrow_mut().add(self,WI::Math(g))?
                    }
                    let ret = self.get_whatsit_group(GroupType::Math)?;
                    self.stomach.borrow_mut().add(self,WI::Math(MathGroup::new(MathKernel::Group(GroupedMath(ret)),false)))?;
                    return Ok(())
                }
                EndGroup => TeXErr!((self,Some(next)),"Unexpected } in math environment"),
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
                                self.stomach.borrow_mut().add(self,WI::Math(g))?
                            }
                            let last = v.pop();
                            for w in v { self.stomach.borrow_mut().add(self,w)? }
                            match last {
                                Some(WI::Math(mg)) => {
                                    match mathgroup.replace(mg) {
                                        Some(m) => self.stomach.borrow_mut().add(self,WI::Math(m))?,
                                        _ => ()
                                    }
                                },
                                Some(w) => self.stomach.borrow_mut().add(self,w)?,
                                None => ()
                            }
                        }
                        Some(WI::Math(mg)) => {
                            match mathgroup.replace(mg) {
                                Some(m) => self.stomach.borrow_mut().add(self,WI::Math(m))?,
                                _ => ()
                            }
                        },
                        Some(w) => {
                            for g in mathgroup.take() {
                                self.stomach.borrow_mut().add(self,WI::Math(g))?
                            }
                            self.stomach.borrow_mut().add(self,w)?
                        },
                        None => ()
                    }
                }
            }
        }
        FileEnd!(self)
    }

    pub fn read_math_whatsit(&self,previous: Option<&mut MathGroup>) -> Result<Option<Whatsit>,TeXError> {
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
                    self.new_group(GroupType::Math);
                    let mut mathgroup: Option<MathGroup> = None;
                    while self.has_next() {
                        let next = self.next_token();
                        match next.catcode {
                            EndGroup => {
                                //let mode = self.state.borrow().display_mode();
                                for g in mathgroup.take() {
                                    self.stomach.borrow_mut().add(self,WI::Math(g))?
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
                                            _ => unreachable!()
                                        }
                                    }
                                }
                                return Ok(Some(WI::Math(MathGroup::new(MathKernel::Group(GroupedMath(ret)),self.state.borrow().display_mode()))))
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
                                            self.stomach.borrow_mut().add(self,WI::Math(g))?
                                        }
                                        let last = v.pop();
                                        for w in v { self.stomach.borrow_mut().add(self,w)? }
                                        match last {
                                            Some(WI::Math(mg)) => {
                                                match mathgroup.replace(mg) {
                                                    Some(m) => self.stomach.borrow_mut().add(self,WI::Math(m))?,
                                                    _ => ()
                                                }
                                            },
                                            Some(w) => self.stomach.borrow_mut().add(self,w)?,
                                            None => ()
                                        }
                                    }
                                    Some(WI::Math(mg)) => {
                                        match mathgroup.replace(mg) {
                                            Some(m) => self.stomach.borrow_mut().add(self,WI::Math(m))?,
                                            _ => ()
                                        }
                                    },
                                    Some(w) => {
                                        for g in mathgroup.take() {
                                            self.stomach.borrow_mut().add(self,WI::Math(g))?
                                        }
                                        self.stomach.borrow_mut().add(self,w)?
                                    },
                                    None => ()
                                }
                            }
                        }
                    }
                },
                Superscript => {
                    let oldmode = self.state.borrow().font_style();
                    //println!("Here! {}",self.preview());
                    //unsafe { crate::LOG = true }
                    self.change_state(StateChange::Fontstyle(oldmode.inc()));
                    let ret = match self.read_math_whatsit(None)? {
                        Some(WI::Math(m)) if m.subscript.is_none() && m.superscript.is_none() => m.kernel,
                        _ => TeXErr!((self,Some(next)),"Expected Whatsit after ^")
                    };
                    self.change_state(StateChange::Fontstyle(oldmode));
                    match previous {
                        Some(mg) => {
                            mg.superscript = Some(ret);
                            return Ok(None)
                        },
                        _ => {
                            let mut mg = MathGroup::new(MathKernel::Group(GroupedMath(vec!())),self.state.borrow().display_mode());
                            mg.superscript = Some(ret);
                            return Ok(Some(WI::Math(mg)))
                        },
                    }
                }
                Subscript => {
                    let oldmode = self.state.borrow().font_style();
                    self.change_state(StateChange::Fontstyle(oldmode.inc()));
                    let ret = match self.read_math_whatsit(None)? {
                        Some(WI::Math(m)) if m.subscript.is_none() && m.superscript.is_none() => m.kernel,
                        _ => TeXErr!((self,Some(next)),"Expected Whatsit after ^")
                    };
                    self.change_state(StateChange::Fontstyle(oldmode));
                    match previous {
                        Some(mg) => {
                            mg.subscript = Some(ret);
                            return Ok(None)
                        },
                        _ => {
                            let mut mg = MathGroup::new(MathKernel::Group(GroupedMath(vec!())),self.state.borrow().display_mode());
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
                            Primitive(np) => {
                                let mut exp = Expansion(next, Arc::new(p.clone()), vec!());
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
                                    Some(k) => Ok(Some(WI::Math(MathGroup::new(k,self.state.borrow().display_mode())))),
                                    _ => Ok(None)
                                }
                            },
                            Whatsit(w) if w.allowed_in(self.get_mode()) => {
                                let next = w.get(&next, self)?;
                                return Ok(Some(next))
                            },
                            MathChar(mc) => match mc {
                                32768 => {
                                    self.requeue(Token::new(next.char, CategoryCode::Active, None, SourceReference::None, true))
                                }
                                _ => {
                                    let wi = self.do_math_char(Some(next),*mc);
                                    let ret = crate::stomach::Whatsit::Math(MathGroup::new(
                                        crate::stomach::math::MathKernel::MathChar(wi),
                                        self.state.borrow().display_mode()));
                                    return Ok(Some(ret))
                                }
                            },
                            _ => TeXErr!((self,Some(next.clone())),"TODO: {} in {}",next,self.current_line())
                        }
                    }
                },
                Space | EOL=> (),
                Letter | Other => {
                    let mc = self.state_get_mathcode(next.char as u8);
                    match mc {
                        32768 => {
                            self.requeue(Token::new(next.char,CategoryCode::Active,None,SourceReference::None,true))
                        }
                        _ => {
                            let wi = self.do_math_char(Some(next),mc as u32);
                            let ret = crate::stomach::Whatsit::Math(MathGroup::new(
                                crate::stomach::math::MathKernel::MathChar(wi),
                                self.state.borrow().display_mode()));
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
                        _ => TeXErr!((self,Some(next)),"Misplaced alignment tab")
                    }
                }
                _ => TeXErr!((self,Some(next.clone())),"Urgh: {}",next),
            }
        }
        FileEnd!(self)
    }

    /*pub fn assert_has_next(&self) -> Result<(),TeXError> {
        if self.has_next() {Ok(())} else  {
            FileEnd!(self)
        }
    } */
}