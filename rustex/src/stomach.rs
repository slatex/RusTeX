use std::borrow::{Borrow, BorrowMut};
use std::ops::Deref;
use std::rc::Rc;
use crate::stomach::whatsits::Whatsit;
use crate::{Interpreter, TeXErr};
use crate::fonts::{Font, Nullfont};
use crate::interpreter::state::GroupType;
use crate::utils::{TeXError, TeXStr};
use crate::stomach::whatsits::WIGroup;

pub mod whatsits;

pub struct Paragraph {
    pub indent:Option<i64>,
    pub children:Vec<Whatsit>,
}

impl Paragraph {
    pub fn new(indent:Option<i64>) -> Paragraph { Paragraph {
        indent,children:vec!()
    }}
}

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
            Par(p) => Par(Paragraph::new(p.indent)),
            Other(wig) => Other(wig.new_from())
        }
    }
    pub fn priority(&self) -> i16 {
        use StomachGroup::*;
        match self {
            Top(_) => 255,
            TeXGroup(GroupType::Box(_),_) => 250,
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
            Other(WIGroup::FontChange(_,_,_,t)) => t,
            Other(WIGroup::ColorChange(_,_,t)) => t,
            _ => todo!()
        }
    }
    pub fn get_d(self) -> Vec<Whatsit> {
        use StomachGroup::*;
        match self {
            Top(t) => t,
            TeXGroup(_,t) => t,
            Par(t) => t.children,
            Other(WIGroup::FontChange(_,_,_,t)) => t,
            Other(WIGroup::ColorChange(_,_,t)) => t,
            _ => todo!()
        }
    }
}

pub trait Stomach {
    fn base_mut(&mut self) -> &mut StomachBase;
    fn base(&self) -> &StomachBase;
    fn ship_whatsit(&mut self, wi:Whatsit);
    fn add(&mut self,int:&Interpreter, wi: Whatsit) -> Result<(),TeXError>;

    // ---------------------------------------------------------------------------------------------

    fn start_paragraph(&mut self,indent: i64) {
        self.base_mut().buffer.push(StomachGroup::Par(Paragraph::new(Some(indent))))
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
            let mut nwi = wi.new_from();
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
                    WIGroup::ColorChange(_, _, _) => {
                        self.base_mut().buffer.push(StomachGroup::Other(g.clone()));
                        Ok(())
                    },
                    _ => todo!()
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
                    StomachGroup::TeXGroup(GroupType::Box(_),_) => true,
                    StomachGroup::Par(_) => true,
                    StomachGroup::Top(_) => true,
                    o => !o.get().is_empty()
                }).unwrap();
                last_one.push(wi);
                Ok(())
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
                    _ => ()
                }
            }
        }
        return None
    }
    fn on_begin_document(&mut self, _: &Interpreter) {
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
                Some(StomachGroup::TeXGroup(GroupType::Box(_),_)) => panic!("This shouldn't happen!"),
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
}

pub struct StomachBase {
    pub buffer:Vec<StomachGroup>,
    pub indocument:bool,
    pub basefont:Option<Rc<Font>>,
    pub basecolor:Option<TeXStr>
}

pub struct NoShipoutRoutine {
    base: StomachBase
}
impl NoShipoutRoutine {
    pub fn new() -> NoShipoutRoutine {
        NoShipoutRoutine {
            base:StomachBase {
                buffer:vec!(StomachGroup::Top(vec!())),
                indocument:false,
                basefont:None,
                basecolor:None
            }
        }
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
        match wi {
            Whatsit::Exec(e) => (std::rc::Rc::try_unwrap(e).ok().unwrap()._apply)(int),
            _ => self.add_inner(int,wi)
        }
    }
    fn ship_whatsit(&mut self, _:Whatsit) {}
}