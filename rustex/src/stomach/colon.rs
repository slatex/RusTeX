use std::sync::Arc;
use std::sync::mpsc::Receiver;
use crate::fonts::Font;
use crate::Interpreter;
use crate::stomach::{StomachMessage, Whatsit};
use crate::stomach::whatsits::WhatsitTrait;
use crate::utils::TeXStr;

#[derive(PartialEq,Clone)]
pub enum ColonMode { V,H,P,M,External(TeXStr) }

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
    fn close(&mut self) -> A;

    fn normalize_whatsit(&self, wi:Whatsit) -> Vec<Whatsit> {
        let mut top : Vec<Whatsit> = vec!();
        wi.normalize(&ColonMode::V,&mut top,None);
        top
    }
    fn initialize(&mut self,basefont:Arc<Font>,basecolor:TeXStr,_:&Interpreter) {
        let base = self.base_mut();
        base.basefont = Some(basefont);
        base.basecolor = Some(basecolor)
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
//unsafe impl Send for NoColon {}

impl Colon<()> for NoColon {
    fn base(&self) -> &ColonBase { &self.base }
    fn base_mut(&mut self) -> &mut ColonBase { &mut self.base }
    fn ship_whatsit(&mut self, _:Whatsit) {}
    fn close(&mut self) -> () {}
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
//unsafe impl Send for XMLColon {}
impl Colon<String> for XMLColon {
    fn base(&self) -> &ColonBase { &self.base }
    fn base_mut(&mut self) -> &mut ColonBase { &mut self.base }
    fn ship_whatsit(&mut self, w:Whatsit) { self.ret += &w.as_xml_internal("  ".to_string()) }
    fn close(&mut self) -> String { std::mem::take(&mut self.ret) + "\n</doc>"}
}