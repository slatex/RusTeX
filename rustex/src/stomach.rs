use std::borrow::BorrowMut;
pub use crate::stomach::whatsits::Whatsit;
use crate::{Interpreter, TeXErr};
use crate::commands::DefMacro;
use crate::fonts::Font;
use crate::interpreter::state::{GroupType, State};
use crate::stomach::groups::{ColorChange, EndGroup, GroupClose, WIGroup, WIGroupCloseTrait, WIGroupTrait};
use crate::stomach::paragraph::Paragraph;
use crate::stomach::simple::{AlignBlock, HAlign, HSkip, SimpleWI};
use crate::utils::{TeXError, TeXStr};
use crate::stomach::whatsits::{Insert, PrintChar, WhatsitTrait};
use std::sync::{Arc, mpsc};
use std::sync::mpsc::{Receiver, Sender};
use crate::interpreter::params::InterpreterParams;
use crate::stomach::boxes::{HBox, TeXBox};

pub mod whatsits;
pub mod boxes;
pub mod math;
pub mod paragraph;
pub mod groups;
pub mod simple;
pub mod colon;
pub mod html;

pub fn split_vertical(vlist:Vec<Whatsit>,target:i32,int:&Interpreter) -> (Vec<Whatsit>,Vec<Whatsit>) {
    let mut currentheight : i32 = 0;
    //let mut marks: Vec<(Vec<Token>,Option<SourceFileReference>)> = vec!();
    let mut presplit : Vec<StomachGroup> = vec!(StomachGroup::Top(vec!()));
    let mut input : Vec<StomachGroup> = vec!(StomachGroup::Top(vlist));
    let first = loop {
        match input.last_mut() {
            None => break None,
            Some(StomachGroup::Top(sg)) if sg.is_empty() => {
                input.pop();
                break None
            },
            Some(sg) if sg.get().is_empty() => {
                let pop = presplit.pop();
                match pop {
                    Some(StomachGroup::Other(wg)) if wg.children().is_empty() => (),
                    Some(StomachGroup::Other(wg)) => presplit.last_mut().unwrap().push(Whatsit::Grouped(wg)),
                    _ => {
                        unreachable!()
                    }
                }
                input.pop();
            }
            Some(sg) => {
                let next = sg.get_mut().remove(0);
                match next {
                    Whatsit::Simple(SimpleWI::Mark(_)) => {
                        todo!()
                    },
                    Whatsit::Grouped(wg) => {
                        presplit.push(StomachGroup::Other(wg.new_from()));
                        input.push(StomachGroup::Other(wg))
                    },
                    Whatsit::Par(p) => {
                        let ht = p.height();
                        if currentheight + ht > target {
                            if currentheight + p.lineheight.unwrap() > target {
                                break Some(Whatsit::Par(p))
                            }
                            let (f,s) = p.split(target - currentheight,&int.state);
                            presplit.last_mut().unwrap().push(Whatsit::Par(f));
                            break Some(Whatsit::Par(s))
                        } else {
                            currentheight += ht;
                            presplit.last_mut().unwrap().push(Whatsit::Par(p))
                        }
                    },
                    next => {
                        let ht = next.height();
                        if currentheight + ht > target {
                            break Some(next)
                        } else {
                            currentheight += ht;
                            presplit.last_mut().unwrap().push(next)
                        }
                    }
                }
            }
        }
    };
    match first {
        None => {
            assert!(input.is_empty());
            match presplit.pop() {
                Some(StomachGroup::Top(v)) => (v,vec!()),
                _ => unreachable!()
            }
        }
        Some(_) => {
            while match presplit.last() {
                Some(StomachGroup::Top(_)) => false,
                _ => true
            } {
                let last = match presplit.pop().unwrap() {
                    StomachGroup::Other(wg) => wg,
                    _ => unreachable!()
                };
                presplit.last_mut().unwrap().push(Whatsit::Grouped(last))
            }
            let mut second : Vec<StomachGroup> = vec!(StomachGroup::Top(vec!()));
            for g in &input[1..] {
                second.push(g.new_from())
            }
            loop {
                match input.last_mut() {
                    None => break,
                    Some(StomachGroup::Top(sg)) if sg.is_empty() => {
                        input.pop();
                        break
                    },
                    Some(sg) if sg.get().is_empty() => {
                        let pop = second.pop();
                        match pop {
                            Some(StomachGroup::Other(wg)) if wg.children().is_empty() => (),
                            Some(StomachGroup::Other(wg)) => second.last_mut().unwrap().push(Whatsit::Grouped(wg)),
                            _ => {
                                unreachable!()
                            }
                        }
                        input.pop();
                    }
                    Some(sg) => {
                        let next = sg.get_mut().remove(0);
                        match next {
                            Whatsit::Simple(SimpleWI::Mark(_)) => {
                                todo!()
                            },
                            next => {
                                second.last_mut().unwrap().push(next)
                                }
                            }
                        }
                    }
                }
            let sec = match second.pop() {
                Some(StomachGroup::Top(v)) => v,
                _ => unreachable!()
            };
            match presplit.pop() {
                Some(StomachGroup::Top(v)) => (v,sec),
                _ => unreachable!()
            }
        }
    }
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
            Par(p) => Par(Paragraph::new(p.parskip)),
            Other(wig) => Other(wig.new_from())
        }
    }
    pub fn priority(&self) -> i16 {
        use StomachGroup::*;
        match self {
            Top(_) => 255,
            TeXGroup(GroupType::Box(_) | GroupType::Math | GroupType::LeftRight,_) => 250,
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
            Par(t) => {
                t.children.push(wi)
            },
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
        }
    }
    pub fn get_mut(&mut self) -> &mut Vec<Whatsit> {
        use StomachGroup::*;
        match self {
            Top(t) => t.borrow_mut(),
            TeXGroup(_,t) => t.borrow_mut(),
            Par(t) => t.children.borrow_mut(),
            Other(o) => o.children_mut()
        }
    }
    pub fn get_d(self) -> Vec<Whatsit> {
        use StomachGroup::*;
        match self {
            Top(t) => t,
            TeXGroup(_,t) => t,
            Par(t) => t.children,
            Other(w) => w.children_prim(),
        }
    }
}

pub enum StomachMessage {
    End,
    WI(Whatsit)
}

pub trait Stomach : Send {
    fn base_mut(&mut self) -> &mut StomachBase;
    fn base(&self) -> &StomachBase;
    fn add(&mut self,state:&mut State,params:&dyn InterpreterParams, wi: Whatsit) -> Result<(),TeXError>;
    fn on_begin_document_inner(&mut self, state: &mut State);

    // ---------------------------------------------------------------------------------------------

    fn start_paragraph(&mut self,parskip:i32) {
        self.flush().unwrap();
        self.base_mut().stomachgroups.push(StomachGroup::Par(Paragraph::new(parskip)))
    }

    fn end_paragraph(&mut self,state:&mut State) -> Result<(),TeXError> {
        self.flush()?;
        let mut p = self.end_paragraph_loop()?;
        let hangindent = self.base().hangindent;
        let hangafter = self.base().hangafter;
        let parshape = std::mem::take(&mut self.base_mut().parshape);
        p.close(state,hangindent,hangafter,parshape);
        self.add_inner_actually(Whatsit::Par(p))?;
        self.reset_par();
        Ok(())
    }

    fn end_paragraph_loop(&mut self) -> Result<Paragraph,TeXError> {
        if self.base().stomachgroups.len() < 2 {
            TeXErr!("Can't close paragraph in stomach!")
        } else {
            let ret = self.base_mut().stomachgroups.pop().unwrap();
            match ret {
                StomachGroup::Par(p) => Ok(p),
                StomachGroup::TeXGroup(gt,v) => {
                    for c in v { self.add_inner_actually(c)? }
                    let ret = self.end_paragraph_loop()?;
                    self.base_mut().stomachgroups.push(StomachGroup::TeXGroup(gt, vec!()));
                    Ok(ret)
                }
                StomachGroup::Other(g) => {
                    let ng = StomachGroup::Other(g.new_from());
                    self.add_inner_actually(Whatsit::Grouped(g))?;
                    let ret = self.end_paragraph_loop()?;
                    self.base_mut().stomachgroups.push(ng);
                    Ok(ret)
                }
                _ => todo!()
            }
        }
    }

    fn new_group(&mut self,tp:GroupType) {
        self.base_mut().buffer.push(Whatsit::GroupOpen(WIGroup::GroupOpen(tp)))
    }
    fn pop_group(&mut self,state:&mut State) -> Result<Vec<Whatsit>, TeXError> {
        self.flush()?;
        if self.base().stomachgroups.len() < 2 {
            TeXErr!("Can't close group in stomach!")
        } else {
            let ret = self.base_mut().stomachgroups.pop().unwrap();
            match ret {
                StomachGroup::TeXGroup(_,v) => Ok(v),
                StomachGroup::Other(g) => {
                    let repushes = if g.closes_with_group() {None} else {Some(g.new_from())};
                    if g.has_ink() {
                        self.base_mut().stomachgroups.last_mut().unwrap().push(Whatsit::Grouped(g));
                    } else {
                        let buf = self.base_mut().stomachgroups.last_mut().unwrap();
                        for c in g.children_prim() {buf.push(c)}
                    }
                    let ret = self.pop_group(state)?;
                    for c in repushes {
                        self.base_mut().stomachgroups.push(StomachGroup::Other(c))
                    }
                    Ok(ret)
                }
                StomachGroup::Par(_) => {
                    self.base_mut().stomachgroups.push(ret);
                    self.end_paragraph(state)?;
                    self.pop_group(state)
                }
                _ => todo!()
            }
        }
    }

    fn close_group(&mut self) -> Result<(),TeXError> {
        self.flush()?;
        let mut cwgs: Vec<WIGroup> = vec!();
        for bg in self.base().stomachgroups.iter().rev() {
            match bg {
                StomachGroup::Other(w) if w.closes_with_group() => cwgs.push(w.new_from()),
                StomachGroup::TeXGroup(_,_) => break,
                /*StomachGroup::Par(_) => {
                    println!("Here!")
                }*/
                _ => ()
            }
        }
        for _ in cwgs { self._close_stomach_group(GroupClose::EndGroup(EndGroup{sourceref:None}))?; }

        let top = self.base_mut().stomachgroups.pop();
        match top {
            None => {
                TeXErr!("Stomach empty")
            },
            Some(StomachGroup::TeXGroup(_,v)) => {
                for c in v { self.add_inner_actually(c)? }
                Ok(())
            }
            Some(StomachGroup::Other(g)) if g.closes_with_group() => {
                self.close_group()?;
                let mut ng = g.new_from();
                let mut nv = g.children_prim();
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
                    self.add_inner_actually( Whatsit::Grouped(ng))?;
                }
                for r in latter { self.add_inner_actually(r)? }
                Ok(())
            }
            Some(p) => {
                self.close_group()?;
                self.base_mut().stomachgroups.push(p);
                Ok(())
            }
        }
    }

    fn _close_stomach_group(&mut self, wi:GroupClose) -> Result<(),TeXError> {
        let top = match self.base_mut().stomachgroups.pop() {
            None => TeXErr!("Error in Stomach: Stomach empty"),
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
                self.add_inner_actually(Whatsit::Grouped(ng))?;
            }
            for r in latter { self.add_inner_actually(r)? }
            Ok(())
        } else if top.priority() > wi.priority() {
            let wiopen = match self.base().stomachgroups.iter().rev().find(|x| x.priority() == wi.priority()) {
                Some(StomachGroup::Other(w)) => w,
                _ => {
                    TeXErr!("No group to close")
                }
            };
            let mut nwi = wiopen.new_from();
            self._close_stomach_group(wi)?;
            self.base_mut().stomachgroups.push(top.new_from());
            *nwi.children_mut() = top.get_d();
            self.base_mut().stomachgroups.last_mut().unwrap().push(Whatsit::Grouped(nwi));
            Ok(())
        } else {
            match top {
                StomachGroup::Other(mut wg) => {
                    let ng = StomachGroup::Other(wg.new_from());
                    self.add_inner_actually(Whatsit::Grouped(wg))?;
                    self._close_stomach_group(wi)?;
                    self.base_mut().stomachgroups.push(ng);
                    Ok(())
                }
                StomachGroup::TeXGroup(gt@(GroupType::Token|GroupType::Begingroup),nv) => {
                    for w in nv {self.add_inner_actually(w)?;}
                    self._close_stomach_group(wi)?;
                    self.base_mut().stomachgroups.push(StomachGroup::TeXGroup(gt,vec!()));
                    Ok(())
                }
                _ => {
                    TeXErr!("No group to close")
                }
            }
        }
    }

    fn add_inner(&mut self,state:&mut State,params:&dyn InterpreterParams, wi: Whatsit) -> Result<(),TeXError> {
        match wi {
            Whatsit::Ls(ls) => {
                for wi in ls { self.add(state,params,wi)? }
                Ok(())
            }
            Whatsit::GroupOpen(_) => {
                self.base_mut().buffer.push(wi);
                Ok(())
            }
            Whatsit::Char(ref pc) => match self.base().buffer.last() {
                Some(Whatsit::Char(ref pc2)) if *pc.font == *pc2.font => {
                    match pc.font.file.ligs.get(&(pc2.char,pc.char)) {
                        Some(u) => {
                            let nc = PrintChar {
                                char:*u,
                                sourceref:pc.sourceref.clone(),
                                font:pc2.font.clone()
                            };
                            self.drop_last();
                            self.base_mut().buffer.push(nc.as_whatsit());
                            Ok(())
                        }
                        None => {
                            self.flush()?;
                            self.base_mut().buffer.push(wi);
                            Ok(())
                        }
                    }
                }
                _ => {
                    self.flush()?;
                    self.base_mut().buffer.push(wi);
                    Ok(())
                }
            }
            Whatsit::Simple(SimpleWI::Penalty(_)) => match self.base().buffer.last() {
                Some(Whatsit::Simple(SimpleWI::Penalty(_))) => {
                    self.drop_last();
                    self.base_mut().buffer.push(wi);
                    Ok(())
                },
                _ => {
                    self.base_mut().buffer.push(wi);
                    Ok(())
                }
            }
            Whatsit::Box(_) => {
                self.base_mut().buffer.push(wi);
                Ok(())
            },
            Whatsit::Simple(SimpleWI::HAlign(_)) => {
                self.base_mut().buffer.push(wi);
                Ok(())
            },
            _ => {
                self.flush()?;
                self.add_inner_actually(wi)
            }
        }
    }

    fn flush(&mut self) -> Result<(),TeXError> {
        let buffer : Vec<Whatsit> = std::mem::take(self.base_mut().buffer.as_mut());
        for w in buffer {
            self.add_inner_actually(w)?
        }
        Ok(())
    }

    fn add_inner_actually(&mut self, wi: Whatsit) -> Result<(),TeXError> {
        match wi {
            Whatsit::GroupOpen(WIGroup::GroupOpen(tp)) => {
                self.base_mut().stomachgroups.push(StomachGroup::TeXGroup(tp, vec!()));
                Ok(())
            }
            Whatsit::GroupOpen(ref g) => {
                self.base_mut().stomachgroups.push(StomachGroup::Other(g.clone()));
                Ok(())
            }
            Whatsit::GroupClose(g) => {
                self._close_stomach_group(g)
            },
            o => {
                if self.base().stomachgroups.is_empty() {
                    return Ok(()) // TODO - temporary for final_xml
                }
                let base = self.base_mut();
                match base.stomachgroups.last_mut().unwrap() {
                    StomachGroup::Top(v) if base.sender.is_some() => {
                        let sender = base.sender.as_ref().unwrap();
                        for w in v.drain(..) { sender.send(StomachMessage::WI(w)).unwrap(); }
                        sender.send(StomachMessage::WI(o)).unwrap();
                    },
                    sg => sg.push(o)
                }
                Ok(())
            }
        }
    }

    fn drop_last(&mut self) {
        let mut repush : Vec<Whatsit> = vec!();
        loop {
            match self.base_mut().buffer.pop() {
                Some(r@Whatsit::GroupOpen(_)) => repush.push(r),
                Some(_) | None => break
            }
        }
        repush.reverse();
        for r in repush { self.base_mut().buffer.push(r) }
    }

    fn last_halign(&mut self,mut halign:HAlign) -> bool {
        match halign.rows.pop() {
            Some(AlignBlock::Block(v)) => {
                let mut ch : Vec<Whatsit> = vec!();
                for (c,_,_) in v {
                    ch.push(HBox {
                        spread:0,
                        _width:None,_height:None,_depth:None,rf:None,
                        children:c
                    }.as_whatsit());
                    ch.push(HSkip {
                        skip:halign.skip,sourceref:None
                    }.as_whatsit())
                }
                /*loop {
                    match rows.last() {
                        _ => break
                    }
                }*/
                self.base_mut().buffer.push(HAlign{
                    skip:halign.skip,template:halign.template,rows:halign.rows,sourceref:halign.sourceref
                }.as_whatsit());
                let tb = TeXBox::H(HBox {
                    children: ch,
                    spread: 0,
                    _width: None,
                    _height: None,
                    _depth: None,
                    rf: None
                });
                self.base_mut().buffer.push(tb.as_whatsit());
                true
            }
            Some(AlignBlock::Noalign(mut v)) => {
                self.base_mut().buffer.push(HAlign{
                    skip:halign.skip,template:halign.template,rows:halign.rows,sourceref:halign.sourceref
                }.as_whatsit());
                for w in v.drain(..).rev() {
                    /*match w {
                        Whatsit::Simple(SimpleWI::Penalty(_)) => (),
                        _ =>*/ self.base_mut().buffer.push(w)
                    //}
                }
                true
            }
            _ => false
        }
    }

    fn last_box(&mut self) -> Result<Option<TeXBox>,TeXError> {
        let mut repush : Vec<Whatsit> = vec!();
        loop {
            match self.base_mut().buffer.pop() {
                Some(r@Whatsit::GroupOpen(WIGroup::GroupOpen(_))) => {
                    repush.push(r);
                    break
                }
                Some(r@Whatsit::GroupOpen(_)) => repush.push(r),
                Some(Whatsit::Box(tb)) => {
                    repush.reverse();
                    for r in repush { self.base_mut().buffer.push(r) }
                    return Ok(Some(tb))
                }
                Some(Whatsit::Simple(SimpleWI::HAlign(h))) => {
                    let done = self.last_halign(h);
                    repush.reverse();
                    for r in repush { self.base_mut().buffer.push(r) }
                    if done { return self.last_box() } else { return Ok(None) }
                },
                Some(r) => {
                    repush.push(r);
                    break
                }
                None => break
            }
        }
        repush.reverse();
        for r in repush { self.base_mut().buffer.push(r) }
        Ok(None)
    }

    fn last_whatsit(&mut self) -> Option<Whatsit> {
        let mut repush : Vec<Whatsit> = vec!();
        loop {
            match self.base_mut().buffer.pop() {
                Some(r@Whatsit::GroupOpen(WIGroup::GroupOpen(_))) => {
                    repush.push(r);
                    break
                },
                Some(r@Whatsit::GroupOpen(_)) => {
                    repush.push(r);
                },
                Some(Whatsit::Simple(SimpleWI::HAlign(h))) => {
                    let ret = self.last_halign(h);
                    repush.reverse();
                    for r in repush { self.base_mut().buffer.push(r) }
                    if ret { return self.last_whatsit() } else {return None}
                },
                Some(r) => {
                    repush.push(r.clone());
                    repush.reverse();
                    for r in repush { self.base_mut().buffer.push(r) }
                    return Some(r)
                }
                _ => break
            }
        }
        repush.reverse();
        for r in repush { self.base_mut().buffer.push(r) }
        None
    }

    fn on_begin_document(&mut self, state:&mut State) -> (Receiver<StomachMessage>,Arc<Font>,TeXStr) {
        self.flush().unwrap();
        state.indocument = true;
        let base = self.base_mut();
        base.indocument = true;
        let mut basefont: Option<Arc<Font>> = None;
        let mut basecolor: TeXStr = "000000".into();
        let mut rets : Vec<Whatsit> = vec!();
        for s in &mut base.stomachgroups { match s {
            StomachGroup::Top(_) => (),
            StomachGroup::TeXGroup(GroupType::Box(_) | GroupType::Math,_) | StomachGroup::Par(_) => break,
            StomachGroup::Other(WIGroup::FontChange(f)) => {
                basefont = Some(f.font.clone());
                for c in std::mem::take(&mut f.children) {rets.push(c)}
            }
            StomachGroup::Other(WIGroup::ColorChange(cc)) => {
                basecolor = ColorChange::color_to_html(cc.color.clone()).into();
                for c in std::mem::take(&mut cc.children) {rets.push(c)}
            }
            g => {
                for c in std::mem::take(g.get_mut()) {rets.push(c)}
            }
        }}
        let mut i = 1;
        loop {
            match base.stomachgroups.get(i) {
                Some(StomachGroup::TeXGroup(GroupType::Box(_) | GroupType::Math,_) | StomachGroup::Par(_)) => break,
                Some(StomachGroup::Other(WIGroup::ColorChange(_) | WIGroup::FontChange(_))) => {base.stomachgroups.remove(i);}
                Some(o) => i +=1,
                None => break
            }
        }
        for r in rets {base.stomachgroups.first_mut().unwrap().push(r)}
        if basefont.is_none() {
            basefont = Some(state.currfont.get(&()))
        }
        self.on_begin_document_inner(state);
        let (sender,receiver) = mpsc::channel::<StomachMessage>();
        self.base_mut().sender = Some(sender);
        (receiver,basefont.unwrap(),basecolor)
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
    fn close_all(&mut self,state:&mut State) -> Result<Vec<Whatsit>,TeXError> {
        self.flush()?;
        loop {
            let last = self.base_mut().stomachgroups.pop();
            match last {
                Some(StomachGroup::Top(v)) => {
                    return Ok(v)
                },
                Some(StomachGroup::Other(g)) => self.add_inner_actually(Whatsit::Grouped(g))?,
                Some(p@StomachGroup::Par(_)) => {
                    self.base_mut().stomachgroups.push(p);
                    self.end_paragraph(state)?
                },
                Some(g@StomachGroup::TeXGroup(_,_)) => {
                    self.base_mut().stomachgroups.push(g);
                    for w in self.pop_group(state)? { self.add_inner_actually(w)? }
                }
                None => return Ok(vec!())
            }
        }
    }
    fn finish(&mut self,state:&mut State) {
        let wis = self.close_all(state);
        match self.base().sender.as_ref() {
            Some(sender) => {
                match wis {
                    Ok(v) => for w in v { sender.send(StomachMessage::WI(w)).unwrap() }
                    _ => ()
                }
                sender.send(StomachMessage::End).unwrap();
            }
            _ => ()
        }
    }
    fn final_xml(&mut self,state:&mut State) -> Result<String,TeXError> {
        let wis = self.close_all(state)?;
        self.finish(state);
        let mut ret = "<doc>\n".to_string();
        for w in wis {
            ret += &w.as_xml_internal("  ".to_string())
        }
        ret += "\n</doc>";
        Ok(ret)
    }
    fn is_top(&self) -> bool {
        for b in self.base().stomachgroups.iter().rev() {
            match b {
                StomachGroup::Top(_) => return true,
                StomachGroup::Other(g) if g.closes_with_group() => (),
                StomachGroup::TeXGroup(GroupType::Token | GroupType::Begingroup,_) => (),
                _ => return false,
            }
        }
        unreachable!()
    }
}

pub struct StomachBase {
    pub stomachgroups:Vec<StomachGroup>,
    pub buffer:Vec<Whatsit>,
    pub indocument:bool,
    pub hangindent:i32,
    pub hangafter:usize,
    pub pageheight:i32,
    pub parshape:Vec<(i32,i32)>,
    pub sender:Option<Sender<StomachMessage>>
}
impl StomachBase {
    pub fn new() -> StomachBase {
        StomachBase {
            stomachgroups: vec!(StomachGroup::Top(vec!())),
            buffer:vec!(),
            indocument: false,
            hangindent: 0,
            hangafter: 0,
            parshape: vec!(),
            pageheight: 0,
            sender:None,
            //sender:None
        }
    }
}

pub struct NoShipoutRoutine {
    base: StomachBase,
    floatlist: Vec<(TeXStr,i32)>,
    floatcmd:Option<DefMacro>
}
impl NoShipoutRoutine {
    pub fn new() -> NoShipoutRoutine {
        NoShipoutRoutine {
            base: StomachBase::new(),
            floatlist: vec!(),
            floatcmd:None
        }
    }
    fn do_floats(&mut self, state: &mut State, params:&dyn InterpreterParams) -> Result<(), TeXError> {
        use crate::commands::PrimitiveTeXCommand;
        use crate::catcodes::CategoryCode::*;
        //for s in &self.floatlist { println!("{}",s)}
        let inserts = state.inserts.drain().map(|(_, x)| x).collect::<Vec<Vec<Whatsit>>>();
        let cmd = state.commands.get(&"@freelist".into()).unwrap();
        let floatregs : Vec<i32> = match &*cmd.orig {
            PrimitiveTeXCommand::Def(df) if *df != *self.floatcmd.as_ref().unwrap() => {
                let mut freefloats: Vec<TeXStr> = vec!();
                for tk in &df.ret {
                    match (tk.catcode, tk.cmdname()) {
                        (_, s) if &s == "@elt" => (),
                        (Escape, o) => freefloats.push(o.clone()),
                        _ => ()
                    }
                }
                self.floatlist.iter().filter(|(x,_)| !freefloats.contains(x)).map(|(_,i)| *i).collect()
            }
            _ => vec!()
        };
        if !inserts.is_empty() {
            self.add(state,params,Whatsit::Inserts(Insert(inserts)))?
        }
        if !floatregs.is_empty() {
            state.commands.set("@freelist".into(),
                                             Some(PrimitiveTeXCommand::Def(self.floatcmd.as_ref().unwrap().clone()).as_command()),true)
        }
        for fnm in floatregs {
            let bx = state.boxes.take(fnm);
            self.add(state,params,Whatsit::Float(bx))?
        }
        Ok(())
    }
    fn get_float_list(&mut self, state: &State) -> Vec<(TeXStr,i32)> {
        use crate::commands::PrimitiveTeXCommand;
        use crate::catcodes::CategoryCode::*;
        let mut ret: Vec<(TeXStr,i32)> = vec!();
        let cmd = state.commands.get(&"@freelist".into()).unwrap();
        match &*cmd.orig {
            PrimitiveTeXCommand::Def(df) => {
                for tk in &df.ret {
                    match (tk.catcode, tk.cmdname()) {
                        (_, s) if &s == "@elt" => (),
                        (Escape, o) => {
                            let p = &*state.commands.get(&o).unwrap().orig;
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

unsafe impl Send for NoShipoutRoutine {}

impl Stomach for NoShipoutRoutine {
    fn base_mut(&mut self) -> &mut StomachBase {
        self.base.borrow_mut()
    }
    fn base(&self) -> &StomachBase {
        &self.base
    }
    fn add(&mut self,state:&mut State,params:&dyn InterpreterParams, wi: Whatsit) -> Result<(),TeXError> {
        match wi {
            Whatsit::Simple(SimpleWI::Penalty(ref p)) if p.penalty <= -1000 && self.is_top() && self.base().indocument => {
                self.add_inner(state,params,wi)?;
                self.do_floats(state,params)?;
                Ok(())
            }
            Whatsit::Exec(e) => (std::sync::Arc::try_unwrap(e).ok().unwrap()._apply)(state,params),
            _ => self.add_inner(state,params,wi)
        }
    }
    fn on_begin_document_inner(&mut self, state: &mut State) {
        self.floatlist = self.get_float_list(state);
        let maxval= i32::MAX;
        state.dimensions.set(-(crate::commands::primitives::VSIZE.index as i32),(maxval / 3) * 2,true);
    }
}

impl Interpreter<'_> {
    pub fn stomach_add(&mut self,wi:Whatsit) -> Result<(),TeXError> {
        self.stomach.add(&mut self.state,self.params,wi)
    }
}