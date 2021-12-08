use std::borrow::BorrowMut;
use crate::stomach::whatsits::Whatsit;
use crate::{Interpreter, TeXErr};
use crate::utils::TeXError;
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
    TeXGroup(Vec<Whatsit>),
    Par(Paragraph),
    Other(WIGroup)
}
impl StomachGroup {
    pub fn push(&mut self,wi:Whatsit) {
        use StomachGroup::*;
        match self {
            Top(t) => t.push(wi),
            TeXGroup(t) => t.push(wi),
            Par(t) => t.children.push(wi),
            Other(WIGroup::FontChange(_,_,_,t)) => t.push(wi),
            Other(WIGroup::ColorChange(_,_,t)) => t.push(wi),
            _ => todo!()
        }
    }
    pub fn get(&self) -> &Vec<Whatsit> {
        use StomachGroup::*;
        match self {
            Top(t) => t,
            TeXGroup(t) => t,
            Par(t) => &t.children,
            Other(WIGroup::FontChange(_,_,_,t)) => t,
            Other(WIGroup::ColorChange(_,_,t)) => t,
            _ => todo!()
        }
    }
}

pub trait Stomach {
    fn buffer_mut(&mut self) -> &mut Vec<StomachGroup>;
    fn buffer(&self) -> &Vec<StomachGroup>;
    fn ship_whatsit(&mut self, wi:Whatsit);
    fn on_begin_document(&mut self, int:&Interpreter);
    fn in_document(&self) -> bool;

    // ---------------------------------------------------------------------------------------------

    fn start_paragraph(&mut self,indent: i64) {
        self.buffer_mut().push(StomachGroup::Par(Paragraph::new(Some(indent))))
    }

    fn new_group(&mut self) {
        self.buffer_mut().push(StomachGroup::TeXGroup(vec!()))
    }
    fn pop_group(&mut self,int:&Interpreter) -> Result<Vec<Whatsit>, TeXError> {
        let buf = self.buffer();
        if buf.len() < 2 {
            TeXErr!((int,None),"Can't close group in stomach!")
        } else {
            let ret = self.buffer_mut().pop().unwrap();
            match ret {
                StomachGroup::TeXGroup(v) => Ok(v),
                StomachGroup::Other(g) => {
                    let repushes = if g.closesWithGroup() {None} else {Some(g.new_from(vec!()))};
                    if g.has_ink() {
                        self.buffer_mut().last_mut().unwrap().push(Whatsit::Grouped(g));
                    } else {
                        let buf = self.buffer_mut().last_mut().unwrap();
                        for c in g.children_d() {buf.push(c)}
                    }
                    let ret = self.pop_group(int);
                    for c in repushes {
                        self.buffer_mut().push(StomachGroup::Other(c))
                    }
                    ret
                }
                _ => todo!()
            }
        }
    }
    fn close_group(&mut self,int:&Interpreter) -> Result<(),TeXError> {
        for w in self.pop_group(int)? {
            self.add(w)
        };
        Ok(())
    }
    fn add(&mut self, wi: Whatsit) {
        match wi {
            Whatsit::Ls(ls) => for wi in ls { self.add(wi) }
            Whatsit::GroupLike(ref g) => {
                match g {
                    WIGroup::FontChange(_,_,b,_) if !b => {
                        self.buffer_mut().push(StomachGroup::Other(g.clone()))
                    }
                    //WIGroup::ColorChange(_,_,_) if !self.in_document() && self.buffer().len() == 1 => self.buffer_mut().last_mut().unwrap().push(Whatsit::Grouped(g.clone())),
                    WIGroup::ColorChange(_,_,_) => {
                        self.buffer_mut().push(StomachGroup::Other(g.clone()))
                    },
                    //WIGroup::ColorEnd(_) if !self.in_document() && self.buffer().len() == 1 => self.buffer_mut().last_mut().unwrap().push(Whatsit::Grouped(g.clone())),
                    WIGroup::ColorEnd(_) => {
                        let mut ret = vec!(self.buffer_mut().pop().unwrap());
                        while match ret.last().unwrap() {
                            StomachGroup::Other(WIGroup::ColorChange(_,_,_)) => false,
                            _ => true
                        } {
                            ret.push(self.buffer_mut().pop().unwrap())
                        }
                        let head = match ret.pop() {
                            Some(StomachGroup::Other(c@WIGroup::ColorChange(_,_,_))) => c,
                            _ => unreachable!()
                        };
                        let (color,rf) = match &head {
                            WIGroup::ColorChange(cl,r,_) => (cl.clone(),r.clone()),
                            _ => unreachable!()
                        };
                        if head.has_ink() {
                            self.add(Whatsit::Grouped(head))
                        } else {
                            for t in head.children_d() { self.add(t) }
                        }
                        ret.reverse();
                        let stack = self.buffer_mut();
                        for sg in ret {
                            if sg.get().is_empty() {
                                stack.push(sg)
                            } else {
                                match sg {
                                    StomachGroup::TeXGroup(v) => {
                                        stack.push(StomachGroup::TeXGroup(vec!(
                                            Whatsit::Grouped(WIGroup::ColorChange(color.clone(),rf.clone(),v))
                                        )))
                                    },
                                    StomachGroup::Top(_) => unreachable!(),
                                    StomachGroup::Par(p) => {
                                        let mut np = Paragraph::new(p.indent);
                                        np.children = vec!(Whatsit::Grouped(WIGroup::ColorChange(color.clone(),rf.clone(),p.children)));
                                        stack.push(StomachGroup::Par(np))
                                    }
                                    StomachGroup::Other(wi) => {
                                        let v = vec!(Whatsit::Grouped(WIGroup::ColorChange(color.clone(),rf.clone(),wi.children().clone())));
                                        let new = wi.new_from(v);
                                        stack.push(StomachGroup::Other(new))
                                    }
                                }
                            }
                        }
                    }
                    _ => todo!()
                }
            },
            Whatsit::Ext(e) if e.isGroup() => todo!(),
            o if o.has_ink() => self.buffer_mut().last_mut().unwrap().push(o),
            _ => {
                let last_one = self.buffer_mut().iter_mut().rev().find(|x| match x {
                    StomachGroup::TeXGroup(_) => true,
                    StomachGroup::Par(_) => true,
                    StomachGroup::Top(_) => true,
                    o => !o.get().is_empty()
                }).unwrap();
                last_one.push(wi)
            }
        }
    }

    fn last_whatsit(&self) -> Option<Whatsit> {
        let buf = self.buffer();
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
}

pub struct EmptyStomach {
    buff : Vec<StomachGroup>,
    indocument:bool
}
impl EmptyStomach {
    pub fn new() -> EmptyStomach {
        EmptyStomach {
            buff:vec!(StomachGroup::Top(vec!())),
            indocument:false
        }
    }
}

impl Stomach for EmptyStomach {
    fn buffer_mut(&mut self) -> &mut Vec<StomachGroup> {
        self.buff.borrow_mut()
    }
    fn buffer(&self) -> &Vec<StomachGroup> {
        &self.buff
    }
    fn ship_whatsit(&mut self, _:Whatsit) {}
    fn on_begin_document(&mut self, int: &Interpreter) {
        self.indocument = true
    }
    fn in_document(&self) -> bool {
        self.indocument
    }
}