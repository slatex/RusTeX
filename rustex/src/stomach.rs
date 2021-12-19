use std::borrow::{Borrow, BorrowMut};
use std::ops::Deref;
use std::rc::Rc;
pub use crate::stomach::whatsits::Whatsit;
use crate::{Interpreter, TeXErr, Token};
use crate::commands::DefMacro;
use crate::fonts::{Font, Nullfont};
use crate::interpreter::state::{GroupType, StateChange};
use crate::utils::{TeXError, TeXStr};
use crate::stomach::whatsits::WIGroup;
use crate::stomach::whatsits::Paragraph;

pub mod whatsits;

pub enum StomachGroup {
    Top(Vec<Whatsit>),
    TeXGroup(GroupType,Vec<Whatsit>),
    Par(Paragraph),
    Other(WIGroup)
}
impl StomachGroup {
    pub fn new_from(&self) -> StomachGroup {
        use StomachGroup::*;
        match self {
            Top(_) => unreachable!(),
            TeXGroup(gt,_) => TeXGroup(gt.clone(),vec!()),
            Par(p) => Par(Paragraph::new(p.parskip)),
            Other(wig) => Other(wig.new_from())
        }
    }
    pub fn priority(&self) -> i16 {
        use StomachGroup::*;
        match self {
            Top(_) => 255,
            TeXGroup(GroupType::Box(_) | GroupType::Math,_) => 250,
            Par(_) => 240,
            TeXGroup(_,_) => 5,
            Other(w) => w.priority(),
        }
    }
    pub fn push(&mut self,wi:Whatsit) {
        use StomachGroup::*;
        match self {
            Top(t) => t.push(wi),
            TeXGroup(_,t) => t.push(wi),
            Par(t) => t.children.push(wi),
            Other(wg) => wg.push(wi),
        }
    }
    pub fn get(&self) -> &Vec<Whatsit> {
        use StomachGroup::*;
        match self {
            Top(t) => t,
            TeXGroup(_,t) => t,
            Par(t) => &t.children,
            Other(w) => w.children(),
            _ => todo!()
        }
    }
    pub fn get_mut(&mut self) -> &mut Vec<Whatsit> {
        use StomachGroup::*;
        match self {
            Top(t) => t.borrow_mut(),
            TeXGroup(_,t) => t.borrow_mut(),
            Par(t) => t.children.borrow_mut(),
            Other(WIGroup::FontChange(_,_,_,t)) => t.borrow_mut(),
            Other(WIGroup::ColorChange(_,_,t)) => t.borrow_mut(),
            Other(WIGroup::PDFLink(_,_,_,_,t)) => t.borrow_mut(),
            _ => todo!()
        }
    }
    pub fn get_d(self) -> Vec<Whatsit> {
        use StomachGroup::*;
        match self {
            Top(t) => t,
            TeXGroup(_,t) => t,
            Par(t) => t.children,
            Other(w) => w.children_d(),
            _ => todo!()
        }
    }
}

pub trait Stomach {
    fn base_mut(&mut self) -> &mut StomachBase;
    fn base(&self) -> &StomachBase;
    fn ship_whatsit(&mut self, wi:Whatsit);
    fn add(&mut self,int:&Interpreter, wi: Whatsit) -> Result<(),TeXError>;
    fn on_begin_document(&mut self, _: &Interpreter);

    // ---------------------------------------------------------------------------------------------

    fn start_paragraph(&mut self,parskip:i32) {
        self.base_mut().buffer.push(StomachGroup::Par(Paragraph::new(parskip)))
    }

    fn end_paragraph(&mut self,int:&Interpreter) -> Result<(),TeXError> {
        let mut p = self.end_paragraph_loop(int)?;
        let hangindent = self.base().hangindent;
        let hangafter = self.base().hangafter;
        let parshape = std::mem::take(&mut self.base_mut().parshape);
        p.close(int,hangindent,hangafter,parshape);
        self.add(int,Whatsit::Par(p))?;
        self.reset_par();
        Ok(())
    }

    fn end_paragraph_loop(&mut self,int:&Interpreter) -> Result<Paragraph,TeXError> {
        if self.base().buffer.len() < 2 {
            TeXErr!((int,None),"Can't close paragraph in stomach!")
        } else {
            let ret = self.base_mut().buffer.pop().unwrap();
            match ret {
                StomachGroup::Par(p) => Ok(p),
                StomachGroup::TeXGroup(gt,v) => {
                    for c in v { self.add(int,c)? }
                    let ret = self.end_paragraph_loop(int)?;
                    self.base_mut().buffer.push(StomachGroup::TeXGroup(gt,vec!()));
                    Ok(ret)
                }
                StomachGroup::Other(g) if g.closesWithGroup() => {
                    let ng = StomachGroup::Other(g.new_from());
                    self.add(int,Whatsit::Grouped(g))?;
                    let ret = self.end_paragraph_loop(int)?;
                    self.base_mut().buffer.push(ng);
                    Ok(ret)
                }
                _ => todo!()
            }
        }
    }

    fn new_group(&mut self,tp:GroupType) {
        self.base_mut().buffer.push(StomachGroup::TeXGroup(tp,vec!()))
    }
    fn pop_group(&mut self,int:&Interpreter) -> Result<Vec<Whatsit>, TeXError> {
        if self.base().buffer.len() < 2 {
            TeXErr!((int,None),"Can't close group in stomach!")
        } else {
            let ret = self.base_mut().buffer.pop().unwrap();
            match ret {
                StomachGroup::TeXGroup(_,v) => Ok(v),
                StomachGroup::Other(g) => {
                    let repushes = if g.closesWithGroup() {None} else {Some(g.new_from())};
                    if g.has_ink() {
                        self.base_mut().buffer.last_mut().unwrap().push(Whatsit::Grouped(g));
                    } else {
                        let buf = self.base_mut().buffer.last_mut().unwrap();
                        for c in g.children_d() {buf.push(c)}
                    }
                    let ret = self.pop_group(int)?;
                    for c in repushes {
                        self.base_mut().buffer.push(StomachGroup::Other(c))
                    }
                    Ok(ret)
                }
                StomachGroup::Par(_) => {
                    self.base_mut().buffer.push(ret);
                    self.end_paragraph(int)?;
                    self.pop_group(int)
                }
                _ => todo!()
            }
        }
    }

    fn close_group(&mut self,int:&Interpreter) -> Result<(),TeXError> {
        let mut cwgs: Vec<WIGroup> = vec!();
        for bg in self.base().buffer.iter().rev() {
            match bg {
                StomachGroup::Other(w) if w.closesWithGroup() => cwgs.push(w.new_from()),
                StomachGroup::TeXGroup(_,_) => break,
                _ => ()
            }
        }
        for c in cwgs { self._close_stomach_group(int,c)?; }

        let top = self.base_mut().buffer.pop();
        match top {
            None => {
                TeXErr!((int,None),"Stomach empty")
            },
            Some(StomachGroup::TeXGroup(_,v)) => {
                for c in v { self.add(int,c)? }
                Ok(())
            }
            Some(StomachGroup::Other(g)) if g.closesWithGroup() => {
                self.close_group(int)?;
                let mut ng = g.new_from();
                let mut nv = g.children_d();
                let mut latter : Vec<Whatsit> = vec!();
                loop {
                    match nv.last() {
                        None => break,
                        Some(s) if s.has_ink() => break,
                        Some(_) => latter.push(nv.pop().unwrap())
                    }
                }
                if !nv.is_empty() {
                    for v in nv { ng.push(v) }
                    self.add(int, Whatsit::Grouped(ng))?;
                }
                for r in latter { self.add(int,r)? }
                Ok(())
            }
            Some(p) => {
                self.close_group(int)?;
                self.base_mut().buffer.push(p);
                Ok(())
            }
        }
    }

    fn _close_stomach_group(&mut self, int:&Interpreter, wi:WIGroup) -> Result<(),TeXError> {
        let top = match self.base_mut().buffer.pop() {
            None => TeXErr!((int,None),"Error in Stomach: Stomach empty"),
            Some(p) => p
        };
        if top.priority() == wi.priority() {
            let mut ng = match top {
                StomachGroup::Other(ref g) => g.new_from(),
                _ => unreachable!()
            };
            let mut nv = top.get_d();
            let mut latter : Vec<Whatsit> = vec!();
            loop {
                match nv.last() {
                    None => break,
                    Some(s) if s.has_ink() => break,
                    Some(_) => latter.push(nv.pop().unwrap())
                }
            }
            if !nv.is_empty() {
                for v in nv { ng.push(v) }
                self.add(int, Whatsit::Grouped(ng))?;
            }
            for r in latter { self.add(int,r)? }
            Ok(())
        } else if top.priority() > wi.priority() {
            let wiopen = match self.base().buffer.iter().rev().find(|x| x.priority() == wi.priority()) {
                Some(StomachGroup::Other(w)) => w,
                _ => {
                    TeXErr!((int,None),"No group to close")
                }
            };
            let mut nwi = wiopen.new_from();
            self._close_stomach_group(int,wi)?;
            self.base_mut().buffer.push(top.new_from());
            let mut grv : Vec<Whatsit> = vec!();
            for w in top.get_d() {
                if grv.is_empty() && !w.has_ink() { self.add(int,w)? } else { grv.push(w) }
            }
            for w in grv { nwi.push(w) }
            self.base_mut().buffer.push(StomachGroup::Other(nwi));
            Ok(())
        } else {
            let mut ng = top.new_from();
            let mut nv = top.get_d();
            let mut latter : Vec<Whatsit> = vec!();
            loop {
                match nv.last() {
                    None => break,
                    Some(s) if s.has_ink() => break,
                    Some(_) => latter.push(nv.pop().unwrap())
                }
            }
            if !nv.is_empty() {
                for v in nv { ng.push(v) }
                self.add(int, match ng {
                    StomachGroup::Other(wg) => Whatsit::Grouped(wg),
                    _ => todo!()
                })?;
            }
            self._close_stomach_group(int,wi)?;
            for l in latter { self.add(int,l)? }
            Ok(())
        }
    }

    fn add_inner(&mut self,int:&Interpreter, wi: Whatsit) -> Result<(),TeXError> {
        match wi {
            Whatsit::Ls(ls) => {
                for wi in ls { self.add(int,wi)? }
                Ok(())
            }
            Whatsit::GroupOpen(ref g) => {
                match g {
                    WIGroup::FontChange(_, _, b, _) if !b => {
                        self.base_mut().buffer.push(StomachGroup::Other(g.clone()));
                        Ok(())
                    }
                    _ => {
                        self.base_mut().buffer.push(StomachGroup::Other(g.clone()));
                        Ok(())
                    },
                }
            }
            Whatsit::GroupClose(g) => {
                self._close_stomach_group(int,g)
            },
            Whatsit::Ext(e) if e.isGroup() => todo!(),
            o if o.has_ink() => {
                self.base_mut().buffer.last_mut().unwrap().push(o);
                Ok(())
            },
            _ => {
                let last_one = self.base_mut().buffer.iter_mut().rev().find(|x| match x {
                    StomachGroup::TeXGroup(GroupType::Box(_) | GroupType::Math,_) => true,
                    StomachGroup::Par(_) => true,
                    StomachGroup::Top(_) => true,
                    StomachGroup::Other(wg) if wg.opaque() => true,
                    o => !o.get().is_empty()
                }).unwrap();
                last_one.push(wi);
                Ok(())
            }
        }
    }

    fn drop_last(&mut self) {
        let buf = &mut self.base_mut().buffer;
        for ls in buf.iter_mut().rev() {
            for (i,v) in ls.get().iter().enumerate().rev() {
                match v {
                    Whatsit::Box(_) => {
                        ls.get_mut().remove(i);
                        return ()
                    },
                    Whatsit::Simple(_) => {
                        ls.get_mut().remove(i);
                        return ()
                    },
                    Whatsit::Grouped(_) => return (),
                    _ => ()
                }
            }
        }

    }

    fn last_whatsit(&self) -> Option<Whatsit> {
        let buf = &self.base().buffer;
        for ls in buf.iter().rev() {
            for v in ls.get().iter().rev() {
                match v {
                    w@Whatsit::Box(_) => return Some(w.clone()),
                    w@Whatsit::Simple(_) => return Some(w.clone()),
                    Whatsit::Grouped(_) => return None,
                    _ => ()
                }
            }
        }
        return None
    }
    fn on_begin_document_inner(&mut self, _: &Interpreter) {
        let base = self.base_mut();
        base.indocument = true;
        let mut groups : Vec<GroupType> = vec!();
        let stack = &mut base.buffer;
        loop {
            match stack.pop() {
                Some(StomachGroup::Top(v)) => {
                    stack.push(StomachGroup::Top(v));
                    break
                }
                Some(StomachGroup::TeXGroup(GroupType::Box(_) | GroupType::Math,_)) => panic!("This shouldn't happen!"),
                Some(StomachGroup::TeXGroup(gt,v)) => {
                    groups.push(gt);
                    for c in v {stack.last_mut().unwrap().push(c)}
                }
                Some(StomachGroup::Other(WIGroup::FontChange(f,_,_,v))) => {
                    base.basefont.get_or_insert(f);
                    for c in v {stack.last_mut().unwrap().push(c)}
                }
                Some(StomachGroup::Other(WIGroup::ColorChange(s,_,v))) => {
                    base.basecolor.get_or_insert(s);
                    for c in v {stack.last_mut().unwrap().push(c)}
                }
                _ => panic!("This shouldn't happen")
            }
        }
        for gt in groups.iter().rev() {
            stack.push(StomachGroup::TeXGroup(*gt,vec!()))
        }
    }
    fn reset_par(&mut self) {
        let base = self.base_mut();
        base.hangindent = 0;
        base.hangafter = 0;
        base.parshape = vec!();
    }
    fn page_height(&self) -> i32 {
        self.base().pageheight
    }
    fn close_all(&mut self,int:&Interpreter) -> Result<Vec<Whatsit>,TeXError> {
        loop {
            let last = self.base_mut().buffer.pop();
            match last {
                Some(StomachGroup::Top(v)) => {
                    //self.base_mut().buffer.push(t);
                    return Ok(v)
                },
                Some(StomachGroup::Other(g)) => self.add(int,Whatsit::Grouped(g))?,
                Some(p@StomachGroup::Par(_)) => {
                    self.base_mut().buffer.push(p);
                    self.end_paragraph(int)?
                },
                Some(g@StomachGroup::TeXGroup(_,_)) => {
                    self.base_mut().buffer.push(g);
                    for w in self.pop_group(int)? { self.add(int,w)? }
                }
                None => unreachable!()
            }
        }
    }
    fn final_xml(&mut self,int:&Interpreter) -> Result<String,TeXError> {
        let wis = self.close_all(int)?;
        let mut ret = "<doc>\n".to_string();
        for w in wis {
            ret += &w.as_xml_internal("  ".to_string())
        }
        ret += "\n</doc>";
        Ok(ret)
    }
    fn is_top(&self) -> bool {
        for b in self.base().buffer.iter().rev() {
            match b {
                StomachGroup::Top(_) => return true,
                StomachGroup::Other(g) if g.closesWithGroup() => (),
                StomachGroup::TeXGroup(GroupType::Token | GroupType::Begingroup,_) => (),
                _ => return false,
            }
        }
        unreachable!()
    }
}

pub struct StomachBase {
    pub buffer:Vec<StomachGroup>,
    pub indocument:bool,
    pub basefont:Option<Rc<Font>>,
    pub basecolor:Option<TeXStr>,
    pub hangindent:i32,
    pub hangafter:usize,
    pub pageheight:i32,
    pub parshape:Vec<(i32,i32)>
}

pub struct NoShipoutRoutine {
    base: StomachBase,
    floatlist: Vec<(TeXStr,i32)>,
    floatcmd:Option<DefMacro>
}
impl NoShipoutRoutine {
    pub fn new() -> NoShipoutRoutine {
        NoShipoutRoutine {
            base: StomachBase {
                buffer: vec!(StomachGroup::Top(vec!())),
                indocument: false,
                basefont: None,
                basecolor: None,
                hangindent: 0,
                hangafter: 0,
                parshape: vec!(),
                pageheight: 0
            },
            floatlist: vec!(),
            floatcmd:None
        }
    }
    fn do_floats(&mut self, int: &Interpreter) -> Result<(), TeXError> {
        use crate::commands::PrimitiveTeXCommand;
        use crate::catcodes::CategoryCode::*;
        //for s in &self.floatlist { println!("{}",s)}
        let inserts = int.state.borrow_mut().inserts.drain().map(|(_, x)| x).collect::<Vec<Vec<Whatsit>>>();
        let cmd = int.get_command(&"@freelist".into()).unwrap();
        let floatregs : Vec<i32> = match &*cmd.orig {
            PrimitiveTeXCommand::Def(df) if *df != *self.floatcmd.as_ref().unwrap() => {
                let mut freefloats: Vec<TeXStr> = vec!();
                for tk in &df.ret {
                    match (tk.catcode, tk.cmdname()) {
                        (_, s) if s == "@elt" => (),
                        (Escape, o) => freefloats.push(o.clone()),
                        _ => ()
                    }
                }
                self.floatlist.iter().filter(|(x,i)| !freefloats.contains(x)).map(|(_,i)| *i).collect()
            }
            _ => vec!()
        };
        if !inserts.is_empty() {
            self.add(int,Whatsit::Inserts(inserts))?
        }
        if !floatregs.is_empty() {
            int.change_state(StateChange::Cs("@freelist".into(),
                                             Some(PrimitiveTeXCommand::Def(self.floatcmd.as_ref().unwrap().clone()).as_command()),true))
        }
        for fnm in floatregs {
            self.add(int,Whatsit::Float(int.state_get_box(fnm)))?
        }
        Ok(())
    }
    fn get_float_list(&mut self, int: &Interpreter) -> Vec<(TeXStr,i32)> {
        use crate::commands::{PrimitiveTeXCommand,ProvidesWhatsit};
        use crate::catcodes::CategoryCode::*;
        let mut ret: Vec<(TeXStr,i32)> = vec!();
        let cmd = int.get_command(&"@freelist".into()).unwrap();
        match &*cmd.orig {
            PrimitiveTeXCommand::Def(df) => {
                for tk in &df.ret {
                    match (tk.catcode, tk.cmdname()) {
                        (_, s) if s == "@elt" => (),
                        (Escape, o) => {
                            let p = &*int.state_get_command(o).unwrap().orig;
                            match p {
                                PrimitiveTeXCommand::Char(tk) => ret.push((o.clone(),tk.char as i32)),
                                _ => panic!("Weird float setup")
                            }
                        },
                        _ => ()
                    }
                }
                if self.floatcmd.is_none() {
                    self.floatcmd = Some(df.clone())
                }
            }
            _ => unreachable!()
        }
        ret
    }
}

impl Stomach for NoShipoutRoutine {
    fn base_mut(&mut self) -> &mut StomachBase {
        self.base.borrow_mut()
    }
    fn base(&self) -> &StomachBase {
        &self.base
    }
    fn add(&mut self,int:&Interpreter, wi: Whatsit) -> Result<(),TeXError> {
        use crate::stomach::whatsits::SimpleWI;
        match wi {
            Whatsit::Simple(SimpleWI::Penalty(i)) if i <= -1000 && self.is_top() && self.base().indocument => {
                let last_one = self.base_mut().buffer.iter_mut().rev().find(|x| match x {
                    StomachGroup::TeXGroup(GroupType::Box(_) | GroupType::Math,_) => true,
                    StomachGroup::Par(_) => true,
                    StomachGroup::Top(_) => true,
                    o => !o.get().is_empty()
                }).unwrap();
                last_one.push(wi);
                self.do_floats(int);
                Ok(())
            }
            Whatsit::Exec(e) => (std::rc::Rc::try_unwrap(e).ok().unwrap()._apply)(int),
            _ => self.add_inner(int,wi)
        }
    }
    fn ship_whatsit(&mut self, _:Whatsit) {}
    fn on_begin_document(&mut self, int: &Interpreter) {
        self.on_begin_document_inner(int);
        self.floatlist = self.get_float_list(int);
        let maxval= i32::MAX;
        int.change_state(StateChange::Dimen(-(crate::commands::primitives::VSIZE.index as i32),(maxval / 3) * 2,true))
    }
}