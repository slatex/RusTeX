
#[derive(Copy,Clone,PartialEq)]
pub enum TeXMode {
    Vertical, InternalVertical, Horizontal, RestrictedHorizontal, Math, Displaymath
}

use std::borrow::BorrowMut;
use std::cell::RefCell;
use std::collections::HashMap;
use crate::ontology::{Expansion, Token};
use crate::catcodes::{CategoryCode, CategoryCodeScheme};
use crate::references::SourceReference;
use std::path::{Path, PathBuf};
use crate::commands::{TeXCommand,PrimitiveTeXCommand};
use crate::interpreter::files::{FileStore, VFile};
use crate::interpreter::mouth::Mouths;
use crate::interpreter::state::{GroupType, State, StateChange};
use crate::utils::{TeXError, TeXString, TeXStr, kpsewhich};
use std::rc::Rc;

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
    filestore:RefCell<FileStore>,
    catcodes:RefCell<CategoryCodeScheme>,
    pub stomach:RefCell<&'a mut dyn Stomach>
}
use crate::{TeXErr,FileEnd};

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

use crate::stomach::{NoShipoutRoutine, Stomach, Whatsit};
use crate::stomach::whatsits::{MathGroup, MathKernel, SimpleWI};
use crate::interpreter::state::FontStyle;

impl Interpreter<'_> {
    pub fn tokens_to_string(&self,tks:&Vec<Token>) -> TeXString {
        let catcodes = self.catcodes.borrow();
        tokens_to_string(tks,&catcodes)
    }

    pub fn kpsewhich(&self,filename: &str) -> Option<PathBuf> {
        kpsewhich(filename,self.jobinfo.in_file())
    }

    pub fn get_file(&self,filename : &str) -> Result<VFile,TeXError> {
        match self.kpsewhich(filename) {
            None =>TeXErr!((self,None),"File {} not found",filename),
            Some(p) => Ok(VFile::new(&p,self.jobinfo.in_file(),&mut self.filestore.borrow_mut()))
        }
    }
    pub fn with_state(s : State,stomach: &mut dyn Stomach) -> Interpreter {
        let catcodes = s.catcodes().clone();
        Interpreter {
            state:RefCell::new(s),
            jobinfo:Jobinfo::new(PathBuf::new()),
            mouths:RefCell::new(Mouths::new()),
            filestore:RefCell::new(FileStore {
                files:HashMap::new()
            }),
            catcodes:RefCell::new(catcodes),
            stomach:RefCell::new(stomach)
        }
    }
    pub fn do_file(&mut self,p:&Path) {
        self.jobinfo = Jobinfo::new(p.to_path_buf());
        let vf:VFile  = VFile::new(p,self.jobinfo.in_file(),&mut self.filestore.borrow_mut());
        self.push_file(vf);
        self.insert_every(&crate::commands::primitives::EVERYJOB);
        while self.has_next() {
            let next = self.next_token();
            if !self.state.borrow().indocument {
                let line = self.state.borrow().indocument_line;
                match line {
                    Some(i) if self.line_no() > i => {
                        self.state.borrow_mut().indocument_line = None;
                        self.stomach.borrow_mut().on_begin_document(self)
                    }
                    _ => match next.catcode {
                        CategoryCode::Escape if next.cmdname() == "document" => {
                            self.state.borrow_mut().indocument_line = Some(self.line_no())
                        }
                        _ => ()
                    }
                }
            }
            match self.do_top(next,false) {
                Ok(_) => {},
                Err(s) => s.throw(Some(&self))
            }
        }
    }
    pub fn do_file_with_state(p : &Path, s : State) -> State {
        let mut stomach = NoShipoutRoutine::new();
        let mut int = Interpreter::with_state(s,stomach.borrow_mut());
        int.jobinfo = Jobinfo::new(p.to_path_buf());
        let vf:VFile  = VFile::new(p,int.jobinfo.in_file(),&mut int.filestore.borrow_mut());
        int.push_file(vf);
        int.insert_every(&crate::commands::primitives::EVERYJOB);
        while int.has_next() {
            let next = int.next_token();
            if !int.state.borrow().indocument {
                let line = int.state.borrow().indocument_line;
                match line {
                    Some(i) if int.line_no() > i => {
                        int.state.borrow_mut().indocument_line = None;
                        int.stomach.borrow_mut().on_begin_document(&int)
                    }
                    _ => match next.catcode {
                        CategoryCode::Escape if next.cmdname() == "document" => {
                            int.state.borrow_mut().indocument_line = Some(int.line_no())
                        }
                        _ => ()
                    }
                }
            }
            match int.do_top(next,false) {
                Ok(_) => {},
                Err(s) => s.throw(Some(&int))
            }
        }
        let ret = int.state.borrow().clone();
        ret.close(int)
    }

    pub fn get_command(&self,s : &TeXStr) -> Result<TeXCommand,TeXError> {
        match self.state.borrow().get_command(s) {
            Some(p) => Ok(p),
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
        use crate::stomach::Whatsit as WI;
        let mode = self.get_mode();
        use crate::catcodes::CategoryCode::*;
        use TeXMode::*;
        use PrimitiveTeXCommand::*;
        match (next.catcode,mode) {
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
                        let mut exp = Expansion(next,Rc::new(p.clone()),vec!());
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
            (Letter | Other | Space, Horizontal | RestrictedHorizontal) => {
                let font = self.get_font();
                let rf = self.update_reference(&next);
                self.stomach.borrow_mut().add(self,crate::stomach::Whatsit::Char(next.char,font,rf))
            }
            (MathShift, Horizontal | RestrictedHorizontal) => {
                self.do_math(inner)
            }
            (Letter | Other, Math | Displaymath) => {
                let mc = self.state_get_mathcode(next.char as u8);
                match mc {
                    32768 => {
                        self.requeue(Token::new(next.char,CategoryCode::Active,None,SourceReference::None,true));
                        Ok(())
                    }
                    _ => {
                        let wi = self.do_math_char(next,mc as u32);
                        self.stomach.borrow_mut().add(self,wi)?;
                        Ok(())
                    }
                }
            }
            (Letter | Other | MathShift,Vertical | InternalVertical) => self.switch_to_H(next),
            _ => TeXErr!((self,Some(next.clone())),"Urgh: {}",next),
        }
    }

    pub fn do_math_char(&self,tk:Token,mc:u32) -> crate::stomach::Whatsit {
        let mut num = mc;
        let (mut cls,mut fam,mut pos) = {
            if num == 0 {
                (0,1,tk.char as u32)
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
                    num = 256 * fam + pos
                }
                i => {
                    cls = 0;
                    fam = i as u32;
                    num = 256 * fam + pos
                }
            }
        }
        let mode = self.state.borrow().font_style();
        let font = match mode {
            FontStyle::Text => self.state.borrow().getTextFont(fam as u8),
            FontStyle::Script => self.state.borrow().getScriptFont(fam as u8),
            FontStyle::Scriptscript => self.state.borrow().getScriptScriptFont(fam as u8),
        };
        crate::stomach::Whatsit::Math(MathGroup::new(
            MathKernel::MathChar(cls,fam,pos,font,self.update_reference(&tk)),
            self.state.borrow().display_mode()))
    }

    fn switch_to_H(&self,next:Token) -> Result<(),TeXError> {
        let indent = match next.catcode {
            CategoryCode::Escape | CategoryCode::Active => {
                let pr = self.get_command(next.cmdname())?;
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
        self.stomach.borrow_mut().start_paragraph(parskip.base);
        if indent != 0 {
            self.stomach.borrow_mut().add(self,crate::stomach::Whatsit::Simple(SimpleWI::Indent(indent,None)))?
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
        use crate::commands::PrimitiveTeXCommand::*;
        use crate::stomach::Whatsit as WI;
        use crate::commands::ProvidesWhatsit;
        self.new_group(GroupType::Math);
        let mode = if inner { TeXMode::Math } else {
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
                MathShift if self.get_mode() == TeXMode::Displaymath => todo!(),
                MathShift => {
                    let mode = self.state.borrow().display_mode();
                    self.set_mode(_oldmode);
                    for g in mathgroup.take() {
                        self.stomach.borrow_mut().add(self,WI::Math(g))?
                    }
                    let ret = self.get_whatsit_group(GroupType::Math)?;
                    self.stomach.borrow_mut().add(self,WI::Math(MathGroup::new(MathKernel::Group(ret),mode)));
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
                                let mode = self.state.borrow().display_mode();
                                for g in mathgroup.take() {
                                    self.stomach.borrow_mut().add(self,WI::Math(g))?
                                }
                                let ret = self.get_whatsit_group(GroupType::Math)?;
                                return Ok(Some(WI::Math(MathGroup::new(MathKernel::Group(ret),self.state.borrow().display_mode()))))
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
                    self.change_state(StateChange::Fontstyle(oldmode.inc()));
                    let ret = match self.read_math_whatsit(None)? {
                        Some(WI::Math(m)) if m.subscript.is_none() && m.superscript.is_none() => m.kernel,
                        _ => TeXErr!((self,Some(next)),"Expected Whatsit after ^")
                    };
                    self.change_state(StateChange::Fontstyle(oldmode));
                    match previous {
                        Some(mg) => {
                            mg.superscript.insert(ret);
                            return Ok(None)
                        },
                        _ => {
                            let mut mg = MathGroup::new(MathKernel::Group(vec!()),self.state.borrow().display_mode());
                            mg.superscript.insert(ret);
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
                            mg.subscript.insert(ret);
                            return Ok(None)
                        },
                        _ => {
                            let mut mg = MathGroup::new(MathKernel::Group(vec!()),self.state.borrow().display_mode());
                            mg.subscript.insert(ret);
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
                                let mut exp = Expansion(next, Rc::new(p.clone()), vec!());
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
                                    let wi = self.do_math_char(next, *mc);
                                    return Ok(Some(wi))
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
                            let wi = self.do_math_char(next,mc as u32);
                            return Ok(Some(wi))
                        }
                    }
                }
                Superscript | Subscript => {
                    self.requeue(next);
                    return Ok(None)
                }
                _ => TeXErr!((self,Some(next.clone())),"Urgh: {}",next),
            }
        }
        FileEnd!(self)
    }

    pub fn current_line(&self) -> String {
        self.mouths.borrow().current_line()
    }
    pub fn line_no(&self) -> usize {
        self.mouths.borrow().line_no().0
    }

    pub fn assert_has_next(&self) -> Result<(),TeXError> {
        if self.has_next() {Ok(())} else  {
            FileEnd!(self)
        }
    }
}