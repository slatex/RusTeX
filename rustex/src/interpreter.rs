
#[derive(Copy,Clone,PartialEq)]
pub enum TeXMode {
    Vertical, InternalVertical, Horizontal, RestrictedHorizontal, Math, Displaymath
}
impl Display for TeXMode {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        use TeXMode::*;
        write!(f,"{}",
            match self {
                Vertical => "vertical",
                InternalVertical => "internal vertical",
                Horizontal => "horizontal",
                RestrictedHorizontal => "restricted horizontal",
                Math => "math",
                Displaymath => "display math"
            }
        )
    }
}

use std::borrow::BorrowMut;
use std::fmt::{Display, Formatter};
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
use crate::commands::registers::PREVGRAF;
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
use crate::stomach::math::{Above, GroupedMath, MathChar, MathGroup, MathKernel};
use crate::interpreter::state::FontStyle;
use crate::stomach::boxes::BoxMode;
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

    pub fn do_string<A:'static,B:'static>(&mut self,p:&Path,text:&str,colon:A) -> (bool,B) where A:Colon<B>,B: Send {
        extern crate pathdiff;
        use files::VFileBase;
        use std::sync::RwLock;
        self.jobinfo = Jobinfo::new(p.to_path_buf());
        let simple:TeXStr = pathdiff::diff_paths(p,self.jobinfo.in_file()).unwrap().to_str().unwrap().into();
        let vf = Arc::new(VFile {
            source:VFileBase::Real(p.to_str().unwrap().into()),
            string:Arc::new(RwLock::new(Some(text.into()))),
            id:simple
        });
        self.state.borrow_mut().filestore.insert(vf.id.clone(),vf.clone());
        self.do_vfile(vf,colon)
    }
    pub fn do_file<A:'static,B:'static>(&mut self,p:&Path,colon:A) -> (bool,B) where A:Colon<B>,B: Send {
        self.jobinfo = Jobinfo::new(p.to_path_buf());
        let vf:Arc<VFile>  = VFile::new(p,false,self.jobinfo.in_file(),&mut self.state.borrow_mut().filestore);
        self.do_vfile(vf,colon)
    }

    pub fn do_string_with_state<A:'static,B:'static>(p : &Path, s : State,text:&str, colon:A,params:&dyn InterpreterParams) -> (bool,State,B) where A:Colon<B>,B:Send {
        let mut stomach = NoShipoutRoutine::new();
        let mut int = Interpreter::with_state(s,stomach.borrow_mut(),params);
        let ret = int.do_string(p,text,colon);
        (ret.0,int.state,ret.1)
    }

    pub fn do_file_with_state<A:'static,B:'static>(p : &Path, s : State, colon:A,params:&dyn InterpreterParams) -> (bool,State,B) where A:Colon<B>,B:Send {
        let mut stomach = NoShipoutRoutine::new();
        let mut int = Interpreter::with_state(s,stomach.borrow_mut(),params);
        let ret = int.do_file(p,colon);
        (ret.0,int.state,ret.1)
    }

    fn do_vfile<A:'static,B:'static>(&mut self,vf:Arc<VFile>,mut colon:A) -> (bool,B) where A:Colon<B>,B: Send {
        self.push_file(vf);
        self.insert_every(&crate::commands::registers::EVERYJOB);
        let cont = match self.predoc_toploop() {
            Ok(b) => b,
            Err(mut e) => {
                e.throw(self);
                self.params.error(e);
                return (false,colon.close())
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
                MaybeThread::Multi(std::thread::Builder::new().stack_size(crate::STACK_SIZE).spawn(move || {
                    for msg in receiver {
                        match msg {
                            StomachMessage::End => return colon.close(),
                            StomachMessage::WI(w) => colon.ship_whatsit(w)
                        }
                    }
                    return colon.close() // sender dropped => TeXError somewhere
                }).unwrap())
            };
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
                            Ok(r) => (false,r),
                            _ => panic!("Error in colon thread")
                        }
                    }
                };
            }

            self.stomach.borrow_mut().finish(&mut self.state);
            match colonthread.join() {
                Ok(r) => return (true,r),
                Err(_) => panic!("Error in colon thread")
            }
        } else {
            (true,colon.close())
        }
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
            None => TeXErr!("Unknown control sequence: \\{}: {}",s,self.preview())
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
                    return if next.expand {p.expand(next,self)} else {Ok(())}
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
            //(Active | Escape,_) => Ok(()),
            (BeginGroup,_) => Ok(self.state.push(self.stomach,GroupType::Token)),
            (EndGroup,_) => self.pop_group(GroupType::Token),
            (Space | EOL, Vertical | InternalVertical | Math | Displaymath ) => Ok(()),
            (Space | EOL, Horizontal | RestrictedHorizontal) => {
                let font = self.state.currfont.get();
                let sourceref = self.update_reference(&next);
                self.stomach_add(crate::stomach::Whatsit::Space(SpaceChar {
                    font,sourceref,nonbreaking:false
                }))
            }
            (Letter | Other , Horizontal | RestrictedHorizontal) => {
                let font = self.state.currfont.get();
                let sourceref = self.update_reference(&next);
                self.stomach_add(crate::stomach::Whatsit::Char(PrintChar {
                    char:next.char,charstr:font.file.chartable.as_ref().map(|ct| ct.get_char(next.char,self.params)).unwrap_or("???"),
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
                              MathKernel::MathChar(wi),
                              self.state.displaymode.get())))?;
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
            _ => TeXErr!(next.clone() => "Not allowed in {} mode: {} of category code {}",mode,next,next.catcode),
        }
    }

    pub fn do_math_char(&self,tk:Option<Token>,mc:u32) -> MathChar {
        let num = mc;
        let (mut cls,mut fam,pos) = {
            if num == 0 && tk.is_some() {
                (0,0,tk.as_ref().unwrap().char as u32)
            } else {
                let char = num % (16 * 16);
                let rest = (num - char) / (16 * 16);
                let fam = rest % 16;
                (((rest - fam) / 16) % 16, fam, char)
            }
        };
        if cls == 7 {
            match self.state.registers_prim.get(&(crate::commands::registers::FAM.index - 1)) {
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
        let mode = self.state.fontstyle.get();
        let font = match mode {
            FontStyle::Text => self.state.textfonts.get(&(fam as usize)),
            FontStyle::Script => self.state.scriptfonts.get(&(fam as usize)),
            FontStyle::Scriptscript => self.state.scriptscriptfonts.get(&(fam as usize)),
        };
        crate::stomach::math::MathChar {
            class:cls,family:fam,position:pos,charstr:font.file.chartable.as_ref().map(|ct| ct.get_char(pos as u8,self.params)).unwrap_or("???").into(),
            font,
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
                        self.state.dimensions_prim.get(&(crate::commands::registers::PARINDENT.index - 1)),
                    _ => {
                        self.requeue(next);
                        self.state.dimensions_prim.get(&(crate::commands::registers::PARINDENT.index - 1))
                    }
                }
            }
            _ => {
                self.requeue(next);
                self.state.dimensions_prim.get(&(crate::commands::registers::PARINDENT.index - 1))
            }
        };
        self.state.borrow_mut().mode = TeXMode::Horizontal;
        self.insert_every(&crate::commands::registers::EVERYPAR);
        let parskip = self.state.skips_prim.get(&(crate::commands::registers::PARSKIP.index - 1));
        self.stomach.borrow_mut().start_paragraph(parskip.base);
        self.state.registers_prim.set((PREVGRAF.index - 1),0,true);
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
        let display = if inner {false} else {
            let next = self.next_token();
            match next.catcode {
                MathShift => true,
                _ => {
                    self.requeue(next);
                    false
                }
            }
        };
        let _oldmode = self.state.mode;
        let bm = if display {
            let m = BoxMode::DM;
            self.state.push(self.stomach,GroupType::Box(m));
            self.state.mode = TeXMode::Displaymath;
            self.insert_every(&crate::commands::registers::EVERYDISPLAY);
            m
        } else {
            let m = BoxMode::M;
            self.state.push(self.stomach,GroupType::Box(m));
            self.state.mode = TeXMode::Math;
            self.insert_every(&crate::commands::registers::EVERYMATH);
            m
        };
        self.state.registers_prim.set((PREVGRAF.index - 1),10,true);
        self.state.displaymode.set(display,false);
        let ret = Interpreter::build_mathgroup(self.read_math_group(bm,true)?,display);
        self.state.mode = _oldmode;
        self.stomach_add(ret)?;
        return Ok(())
    }

    fn read_math_group(&mut self,mode:BoxMode,mathshift:bool) -> Result<Vec<Whatsit>,TeXError> {
        use crate::catcodes::CategoryCode::*;
        while self.has_next() {
            let next = self.next_token();
            match (next.catcode,mode) {
                (MathShift,BoxMode::DM) if mathshift => {
                    let nnext = self.next_token();
                    match nnext.catcode {
                        MathShift => {
                            let ret = self.get_whatsit_group(GroupType::Box(BoxMode::DM))?;
                            return Ok(ret)
                        }
                        _ => TeXErr!(nnext => "displaymode must be closed with $$")
                    }
                }
                (MathShift,BoxMode::M) if mathshift => {
                    let ret = self.get_whatsit_group(GroupType::Box(BoxMode::M))?;
                    return Ok(ret)
                }
                (MathShift,_) => TeXErr!("Unexpected $"),
                (EndGroup,_) if mathshift => TeXErr!(next => "{}","Unexpected } in math environment"),
                (EndGroup,m) => {
                    let ret = self.get_whatsit_group(GroupType::Box(m))?;
                    return Ok(ret)
                },
                _ => {
                    self.requeue(next);
                    match self.read_math_whatsit()? {
                        Some(wi) => self.stomach_add(wi)?,
                        _ => ()
                    }
                }
            }
        }
        FileEnd!()
    }

    fn get_last_math(&mut self) -> Result<MathGroup,TeXError> {
        use crate::stomach::Whatsit as WI;
        match self.stomach.get_last() {
            Some(WI::Math(mg)) => {
                return Ok(mg)
            }
            Some(o) => {
                let mg = MathGroup::new(MathKernel::Group(GroupedMath(vec!(o),false)),self.state.displaymode.get());
                return Ok(mg)
            }
            _ => return Ok(MathGroup::new(MathKernel::Group(GroupedMath(vec!(),false)),self.state.displaymode.get()))
        }
    }

    pub fn read_math_whatsit(&mut self) -> Result<Option<Whatsit>,TeXError> {
        use crate::catcodes::CategoryCode::*;
        use crate::commands::PrimitiveTeXCommand::*;
        use crate::stomach::Whatsit as WI;
        while self.has_next() {
            let next = self.next_token();
            match next.catcode {
                MathShift | EndGroup => {
                    self.requeue(next);
                    return Ok(None)
                }
                BeginGroup => {
                    let mode = match self.state.mode {
                        TeXMode::Math => BoxMode::M,
                        TeXMode::Displaymath => BoxMode::DM,
                        _ => unreachable!()
                    };
                    self.state.push(self.stomach,GroupType::Box(mode));
                    return Ok(Some(Interpreter::build_mathgroup(self.read_math_group(mode,false)?,self.state.displaymode.get())))
                }
                Superscript => {
                    let mut last = match self.get_last_math()? {
                        mg if mg.superscript.is_none() => mg,
                        _ => TeXErr!("Double superscript")
                    };
                    let oldmode = self.state.fontstyle.get();
                    self.state.fontstyle.set(oldmode.inc(),false);
                    let ret = match self.read_math_whatsit()? {
                        None => TeXErr!(next => "Expected Whatsit after ^"),
                        Some(WI::Math(m)) if m.subscript.is_none() && m.superscript.is_none() => m.kernel,
                        Some(wi) => MathKernel::Group(GroupedMath(vec!(wi),false))
                    };
                    self.state.fontstyle.set(oldmode,false);
                    last.superscript = Some(ret);
                    return Ok(Some(last.as_whatsit()))
                }
                Subscript => {
                    let mut last = match self.get_last_math()? {
                        mg if mg.subscript.is_none() => mg,
                        _ => TeXErr!("Double subscript")
                    };
                    let oldmode = self.state.fontstyle.get();
                    self.state.fontstyle.set(oldmode.inc(),false);
                    let ret = match self.read_math_whatsit()? {
                        None => TeXErr!(next => "Expected Whatsit after _"),
                        Some(WI::Math(m)) if m.subscript.is_none() && m.superscript.is_none() => m.kernel,
                        Some(wi) => MathKernel::Group(GroupedMath(vec!(wi),false))
                    };
                    self.state.fontstyle.set(oldmode,false);
                    last.subscript = Some(ret);
                    return Ok(Some(last.as_whatsit()))
                }
                Active | Escape => {
                    let p = self.get_command(&next.cmdname())?;
                    if p.assignable() {
                        p.assign(next, self, false)?
                    } else if p.expandable(true) {
                        p.expand(next, self)?
                    } else {match &*p.orig {
                        Whatsit(ProvidesWhatsit::Math(mw)) if **mw == RIGHT => {
                            self.requeue(next);
                            return Ok(None)
                        }
                        Whatsit(ProvidesWhatsit::Math(mw)) if **mw == LEFT => {
                            self.state.push(self.stomach,GroupType::Box(BoxMode::LeftRight));
                            let left = (LEFT._get)(&next,self)?;
                            while self.has_next() {
                                let next = self.next_token();
                                match &next.catcode {
                                    MathShift | EndGroup => {
                                        TeXErr!("\\left ended with {}",next)
                                    }
                                    Active | Escape => {
                                        let p = self.get_command(&next.cmdname())?;
                                        match &*p.orig {
                                            Whatsit(ProvidesWhatsit::Math(mw)) if **mw == RIGHT => {
                                                let right = (RIGHT._get)(&next,self)?;
                                                let mut v = self.get_whatsit_group(GroupType::Box(BoxMode::LeftRight))?;
                                                for le in left {
                                                    v.insert(0,le.as_whatsit_limits(self.state.displaymode.get()));
                                                }
                                                for ri in right {
                                                    v.push(ri.as_whatsit_limits(self.state.displaymode.get()));
                                                }
                                                return Ok(Some(Interpreter::build_mathgroup(v,self.state.displaymode.get())))
                                            }
                                            _ => {
                                                self.requeue(next);
                                                match self.read_math_whatsit()? {
                                                    Some(wi) => self.stomach_add(wi)?,
                                                    _ => ()
                                                }
                                            }
                                        }
                                    }
                                    _ => {
                                        self.requeue(next);
                                        match self.read_math_whatsit()? {
                                            Some(wi) => self.stomach_add(wi)?,
                                            _ => ()
                                        }
                                    }
                                }
                            }
                        }
                        Whatsit(ProvidesWhatsit::Math(mw)) => {
                            match (mw._get)(&next,self)? {
                                Some(krnl) => return Ok(Some(krnl.as_whatsit_limits(self.state.displaymode.get()))),
                                _ => return Ok(None)
                            }
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
                        Char(tk) => self.requeue(tk.clone()),
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
                                    self.state.displaymode.get()));
                                return Ok(Some(ret))
                            }
                        },
                        _ => TeXErr!(next.clone() => "TODO: {} in {}\n => {}",next,self.current_line(),self.preview())
                    }}
                }
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
                                MathKernel::MathChar(wi),
                                self.state.displaymode.get()));
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
                Parameter => TeXErr!(next.clone() => "Parameter Token {} not allowed in math mode",next),
                _ => TeXErr!(next.clone() => "Not allowed in {} mode: {} of category code {}",self.state.mode,next,next.catcode),
            }
        }
        FileEnd!()
    }

    fn build_mathgroup(mut vec:Vec<Whatsit>,limits:bool) -> Whatsit {
        let mut ret: Vec<Whatsit> = Vec::with_capacity(vec.len());
        let mut above:Option<Above> = None;
        for x in vec.into_iter() {
            match &mut above {
                Some(a) => a.bottom.push(x),
                None => match x {
                    Whatsit::Above(mut a) if !a.filled => {
                        a.filled = true;
                        a.top = std::mem::take(&mut ret);
                        above = Some(a)
                    },
                    _ => ret.push(x)
                }
            }
        }
        match above {
            Some(a) => a.as_whatsit(),
            _ => GroupedMath(ret,true).as_whatsit_limits(limits)
        }
    }
/*

    fn read_math_group(&mut self,finish: fn(&Token,&mut Interpreter) -> Result<bool,TeXError>) -> Result<Option<()>,TeXError> {
        use crate::catcodes::CategoryCode::*;
        use crate::stomach::Whatsit as WI;

        let mut mathgroup: Option<MathGroup> = None;

        while self.has_next() {
            let next = self.next_token();
            if finish(&next,self)? {
                self.requeue(next);
                if let Some(g) = mathgroup.take() {
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
                                if let Some(g) = mathgroup.take() {
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
                                if let Some(g) = mathgroup.take() {
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
                            if let Some(g) = mathgroup.take() {
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
                            if let Some(g) = mathgroup.take() {
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


 */

    /*pub fn assert_has_next(&self) -> Result<(),TeXError> {
        if self.has_next() {Ok(())} else  {
            FileEnd!(self)
        }
    } */
}