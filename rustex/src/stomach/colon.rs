use std::sync::Arc;
use std::sync::mpsc::Receiver;
use std::thread::JoinHandle;
use crate::fonts::Font;
use crate::Interpreter;
use crate::interpreter::state::State;
use crate::stomach::{StomachMessage, Whatsit};
use crate::stomach::groups::{WIGroup, WIGroupTrait};
use crate::stomach::whatsits::WhatsitTrait;
use crate::utils::TeXStr;

pub struct ColonBase {
    pub basefont:Option<Arc<Font>>,
    pub basecolor:Option<TeXStr>,
    pub receiver:Option<Receiver<StomachMessage>>
}
impl ColonBase {
    pub fn new() -> ColonBase {
        ColonBase {
            basefont:None,
            basecolor:None,
            receiver:None
        }
    }
}

pub trait Colon<A> : Send {
    fn base_mut(&mut self) -> &mut ColonBase;
    fn base(&self) -> &ColonBase;
    fn ship_whatsit(&mut self, wi:Whatsit);
    fn close(self) -> A;

    fn normalize_whatsit(&self, wi:Whatsit) -> Vec<Whatsit> {
        let mut top : Vec<Whatsit> = vec!();
        normalize_top(wi,&mut top);
        top
    }
    fn initialize(&mut self,basefont:Arc<Font>,basecolor:TeXStr,_:&Interpreter) {
        let base = self.base_mut();
        base.basefont = Some(basefont);
        base.basecolor = Some(basecolor)
    }
}

// -------------------------------------------------------------------------------------------------

fn normalize_top(w : Whatsit,ret:&mut Vec<Whatsit>) {
    use Whatsit::*;
    use crate::stomach::simple::SimpleWI::*;
    use crate::stomach::boxes::TeXBox::*;
    match w {
        Whatsit::Simple(Penalty(_)) => (),
        Whatsit::Box(V(mut vb)) if vb._height.is_none() => {
            for c in vb.children { normalize_top(c,ret) }
        }
        Par(p) => {
            let (mut np,ch) = p.destroy();
            let mut hret : Vec<Whatsit> = vec!();
            for c in ch { normalize_h(c,&mut hret) }
            np.children = hret;
            ret.push(Par(np))
        }
        Simple(VSkip(sk)) if sk.skip.base == 0 => (),
        Simple(VKern(k)) if k.dim == 0 => (),
        Simple(VSkip(sk)) => {
            match ret.last_mut() {
                Some(Simple(VSkip(sk2))) => {
                    sk2.skip = sk.skip + sk2.skip;
                    if sk2.skip.base == 0 {
                        ret.pop();
                    }
                },
                _ => ret.push(Simple(VSkip(sk)))
            }
        }
        Simple(VFil(_)) | Simple(PDFDest(_)) => ret.push(w),
        _ => {
            ret.push(w)
        }
    }
}
/*
fn normalize_v(w:Whatsit) -> Vec<Whatsit> {
    use Whatsit::*;
    use crate::stomach::simple::SimpleWI::*;
    use crate::stomach::boxes::TeXBox::*;
    match w {
        Whatsit::Simple(Penalty(_)) => vec!(),
        Simple(VSkip(sk)) if sk.skip.base == 0 => vec!(),
        Simple(VKern(k)) if k.dim == 0 => vec!(),
        Simple(VFil(_)) | Simple(PDFDest(_)) | Simple(VSkip(_)) => vec!(w),
        _ => {
            vec!(w)
        }
    }
}

 */

fn normalize_h(w:Whatsit,ret:&mut Vec<Whatsit>) {
    use Whatsit::*;
    use crate::stomach::simple::SimpleWI::*;
    use crate::stomach::boxes::TeXBox::*;
    match w {
        Simple(Penalty(p)) if p.penalty > -10000 => (),
        Simple(HSkip(sk)) if sk.skip.base == 0 => (),
        Simple(HKern(k)) if k.dim == 0 => (),
        Simple(HSkip(sk)) => {
            match ret.last_mut() {
                Some(Simple(HSkip(sk2))) => {
                    sk2.skip = sk.skip + sk2.skip;
                    if sk2.skip.base == 0 {
                        ret.pop();
                    }
                },
                _ => ret.push(Simple(HSkip(sk)))
            }
        }
        Box(H(mut hb)) if hb._depth.is_none() && hb._height.is_none() && hb._width.is_none() => {
            for c in hb.children { normalize_h(c,ret) }
        }
        Box(H(hb)) if hb.children.is_empty() && hb.width() == 0 => (),
        Grouped(mut wg) => {
            let mut ng = wg.new_from();
            if wg.opaque() {
                for c in wg.children_prim() { normalize_h(c,ng.children_mut()) }
                ret.push(Grouped(ng));
                return ()
            }
            let mut in_ink = false;
            for c in wg.children_prim() {
                if c.has_ink() { in_ink = true }
                if in_ink { normalize_h(c, ng.children_mut()) } else { normalize_h(c, ret) }
            }
            let mut nret : Vec<Whatsit> = vec!();
            loop {
                match ng.children_mut().pop() {
                    None => break,
                    Some(w) if !w.has_ink() => nret.push(w),
                    Some(o) => {
                        ng.children_mut().push(o);
                        break
                    }
                }
            }
            if !ng.children().is_empty() {
                ret.push(Grouped(ng))
            }
            nret.reverse();
            ret.append(&mut nret);
        }
        Simple(HFil(_)) | Simple(PDFDest(_)) | Char(_) | Space(_) => ret.push(w),
        _ => {
            ret.push(w)
        }
    }
}

// -------------------------------------------------------------------------------------------------

pub struct NoColon {
    pub base:ColonBase
}
impl NoColon {
    pub fn new() -> NoColon {
        NoColon { base: ColonBase::new()}
    }
}
unsafe impl Send for NoColon {}

impl Colon<()> for NoColon {
    fn base(&self) -> &ColonBase { &self.base }
    fn base_mut(&mut self) -> &mut ColonBase { &mut self.base }
    fn ship_whatsit(&mut self, _:Whatsit) {}
    fn close(self) -> () {}
}

// -------------------------------------------------------------------------------------------------

pub struct XMLColon {
    pub base:ColonBase,
    ret : String
}
impl XMLColon {
    pub fn new() -> XMLColon { XMLColon {
        base: ColonBase::new(),
        ret:"<doc>\n".to_string()
    }}
}
unsafe impl Send for XMLColon {}
impl Colon<String> for XMLColon {
    fn base(&self) -> &ColonBase { &self.base }
    fn base_mut(&mut self) -> &mut ColonBase { &mut self.base }
    fn ship_whatsit(&mut self, w:Whatsit) { self.ret += &w.as_xml_internal("  ".to_string()) }
    fn close(self) -> String { self.ret + "\n</doc>"}
}