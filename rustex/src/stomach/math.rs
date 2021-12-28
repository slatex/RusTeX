use std::cmp::max;
use std::rc::Rc;
use crate::fonts::Font;
use crate::interpreter::dimensions::MuSkip;
use crate::references::SourceFileReference;
use crate::stomach::Whatsit;
use crate::stomach::whatsits::{HasWhatsitIter, WhatsitTrait};

#[derive(Clone)]
pub struct MathGroup {
    pub kernel : MathKernel,
    pub superscript : Option<MathKernel>,
    pub subscript : Option<MathKernel>,
    pub limits:bool
}

impl MathGroup {
    pub fn new(kernel:MathKernel,display:bool) -> MathGroup {
        MathGroup {
            kernel,subscript:None,superscript:None,limits:display
        }
    }
}

impl WhatsitTrait for MathGroup {
    fn as_whatsit(self) -> Whatsit { Whatsit::Math(self) }
    fn as_xml_internal(&self,prefix: String) -> String {
        let mut ret = "\n".to_string() + &prefix + "<math>\n  " + &prefix + "<kernel>";
        ret += &self.kernel.as_xml_internal(prefix.clone() + "    ");
        ret += "</kernel>";
        if self.subscript.is_some() {
            ret += "\n  ";
            ret += &prefix;
            ret += "<subscript>";
            ret += &self.subscript.as_ref().unwrap().as_xml_internal(prefix.clone() + "    ");
            ret += "</subscript>"
        }
        if self.superscript.is_some() {
            ret += "\n  ";
            ret += &prefix;
            ret += "<superscript>";
            ret += &self.superscript.as_ref().unwrap().as_xml_internal(prefix.clone() + "    ");
            ret += "</superscript>"
        }
        ret + "\n" + &prefix + "</math>"
    }
    fn width(&self) -> i32 {
        self.kernel.width() + match &self.superscript {
            None => 0,
            Some(k) => k.width()
        } + match &self.subscript {
            None => 0,
            Some(k) => k.width()
        }
    }
    fn height(&self) -> i32 {
        self.kernel.height() + match &self.superscript {
            None => 0,
            Some(k) => k.height() / 2
        } + match &self.subscript {
            None => 0,
            Some(k) => k.height() / 2
        }
    }
    fn depth(&self) -> i32 {
        match &self.subscript {
            Some(s) => max(s.height() / 2,self.kernel.depth()),
            None => self.kernel.depth()
        }
    }
    fn has_ink(&self) -> bool {
        self.kernel.has_ink() || match &self.superscript {
            None => false,
            Some(s) => s.has_ink()
        } || match &self.subscript {
            None => false,
            Some(s) => s.has_ink()
        }
    }
}

#[derive(Clone)]
pub enum MathKernel {
    Group(GroupedMath),
    MathChar(MathChar),
    MKern(MKern),
    Delimiter(Delimiter),
    MathOp(MathOp),
    MathOpen(MathOpen),
    MathClose(MathClose),
    MathBin(MathBin),
    MathOrd(MathOrd),
    MathPunct(MathPunct),
    MathRel(MathRel),
    MathInner(MathInner),
    Underline(Underline),
    Overline(Overline),
    MathAccent(MathAccent)
}

macro_rules! pass_on_kernel {
    ($s:tt,$e:ident$(,$tl:expr)*) => (match $s {
        MathKernel::Group(g) => GroupedMath::$e(g $(,$tl)*),
        MathKernel::MathChar(g) => MathChar::$e(g $(,$tl)*),
        MathKernel::MKern(g) => MKern::$e(g $(,$tl)*),
        MathKernel::Delimiter(g) => Delimiter::$e(g $(,$tl)*),
        MathKernel::MathOp(g) => MathOp::$e(g $(,$tl)*),
        MathKernel::MathOpen(g) => MathOpen::$e(g $(,$tl)*),
        MathKernel::MathClose(g) => MathClose::$e(g $(,$tl)*),
        MathKernel::MathBin(g) => MathBin::$e(g $(,$tl)*),
        MathKernel::MathOrd(g) => MathOrd::$e(g $(,$tl)*),
        MathKernel::MathPunct(g) => MathPunct::$e(g $(,$tl)*),
        MathKernel::MathRel(g) => MathRel::$e(g $(,$tl)*),
        MathKernel::MathInner(g) => MathInner::$e(g $(,$tl)*),
        MathKernel::Underline(g) => Underline::$e(g $(,$tl)*),
        MathKernel::Overline(g) => Overline::$e(g $(,$tl)*),
        MathKernel::MathAccent(g) => MathAccent::$e(g $(,$tl)*)
    })
}
impl WhatsitTrait for MathKernel {
    fn as_whatsit(self) -> Whatsit {
        Whatsit::Math(MathGroup {
            kernel:self,
            superscript:None,
            subscript:None,
            limits: false
        })
    }

    fn width(&self) -> i32 { pass_on_kernel!(self,width) }
    fn height(&self) -> i32 { pass_on_kernel!(self,height) }
    fn depth(&self) -> i32 { pass_on_kernel!(self,depth) }
    fn as_xml_internal(&self, prefix: String) -> String {
        pass_on_kernel!(self,as_xml_internal,prefix)
    }
    fn has_ink(&self) -> bool { pass_on_kernel!(self,has_ink) }
}

#[derive(Clone)]
pub enum MathInfix {
    Over(Over),
    Above(Above),
}
macro_rules! pass_on_infix {
    ($s:tt,$e:ident$(,$tl:expr)*) => (match $s {
        MathInfix::Over(g) => Over::$e(g $(,$tl)*),
        MathInfix::Above(g) => Above::$e(g $(,$tl)*)
    })
}
impl WhatsitTrait for MathInfix {
    fn as_whatsit(self) -> Whatsit {
        Whatsit::MathInfix(self)
    }
    fn width(&self) -> i32 { pass_on_infix!(self,width) }
    fn height(&self) -> i32 { pass_on_infix!(self,height) }
    fn depth(&self) -> i32 { pass_on_infix!(self,depth) }
    fn as_xml_internal(&self, prefix: String) -> String {
        pass_on_infix!(self,as_xml_internal,prefix)
    }
    fn has_ink(&self) -> bool { pass_on_infix!(self,has_ink) }
}
impl MathInfix {
    pub fn set(&mut self,first:Vec<Whatsit>,second:Vec<Whatsit>) {
        use MathInfix::*;
        match self {
            Over(o) => {
                o.top = first;
                o.bottom = second
            }
            Above(o) => {
                o.top = first;
                o.bottom = second
            }
        }
    }
}

// -------------------------------------------------------------------------------------------------

#[derive(Clone)]
pub struct GroupedMath(pub Vec<Whatsit>);
impl WhatsitTrait for GroupedMath {
    fn as_whatsit(self) -> Whatsit {
        MathKernel::Group(self).as_whatsit()
    }
    fn width(&self) -> i32 {
        let mut ret = 0;
        for c in self.0.iter_wi() { ret += c.width() }
        ret
    }
    fn height(&self) -> i32 {
        let mut ret = 0;
        for c in self.0.iter_wi() {
            let w = c.height();
            if w > ret { ret = w }
        }
        ret
    }
    fn depth(&self) -> i32 {
        let mut ret = 0;
        for c in self.0.iter_wi() {
            let w = c.depth();
            if w > ret { ret = w }
        }
        ret
    }
    fn as_xml_internal(&self, prefix: String) -> String {
        let mut ret = "".to_string();
        for w in &self.0 {ret += &w.as_xml_internal(prefix.clone())}
        ret
    }
    fn has_ink(&self) -> bool {
        for c in &self.0 { if c.has_ink() { return true } }
        false
    }
}

#[derive(Clone)]
pub struct MKern {
    pub sk:MuSkip,
    pub sourceref:Option<SourceFileReference>
}
impl WhatsitTrait for MKern {
    fn as_whatsit(self) -> Whatsit {
        MathKernel::MKern(self).as_whatsit()
    }
    fn width(&self) -> i32 { self.sk.base }
    fn height(&self) -> i32 { 0}
    fn depth(&self) -> i32 { 0 }
    fn as_xml_internal(&self, prefix: String) -> String {
        "<mkern value=\"".to_string() + &self.sk.to_string() + "\"/>"
    }
    fn has_ink(&self) -> bool { false }
}

#[derive(Clone)]
pub struct MathChar {
    pub class:u32,
    pub family:u32,
    pub position:u32,
    pub font:Rc<Font>,
    pub sourceref:Option<SourceFileReference>
}
impl WhatsitTrait for MathChar {
    fn as_whatsit(self) -> Whatsit {
        MathKernel::MathChar(self).as_whatsit()
    }
    fn width(&self) -> i32 { self.font.get_width(self.position as u16) }
    fn height(&self) -> i32 { self.font.get_height(self.position as u16) }
    fn depth(&self) -> i32 { self.font.get_depth(self.position as u16) }
    fn as_xml_internal(&self, prefix: String) -> String {
        "\n".to_owned() + &prefix + "<mathchar value=\"" + &self.position.to_string() + "\"/>"
    }
    fn has_ink(&self) -> bool { true }
}

#[derive(Clone)]
pub struct Delimiter {
    pub small:MathChar,
    pub large:MathChar,
    pub sourceref:Option<SourceFileReference>
}
impl WhatsitTrait for Delimiter {
    fn as_whatsit(self) -> Whatsit {
        MathKernel::Delimiter(self).as_whatsit()
    }
    fn width(&self) -> i32 { self.small.width() }
    fn height(&self) -> i32 { self.small.height() }
    fn depth(&self) -> i32 { self.small.depth() }
    fn as_xml_internal(&self, prefix: String) -> String {
        "\n".to_owned() + &prefix + "<delimiter>" + &self.small.as_xml_internal(prefix.clone()) + &self.large.as_xml_internal(prefix) + "</delimiter>"
    }
    fn has_ink(&self) -> bool { true }
}

macro_rules! mathgroupkernel {
    ($e:ident) => (
        #[derive(Clone)]
        pub struct $e {
            pub content:Box<Whatsit>,
            pub sourceref:Option<SourceFileReference>
        }
        impl WhatsitTrait for $e {
            fn as_whatsit(self) -> Whatsit {
                MathKernel::$e(self).as_whatsit()
            }
            fn width(&self) -> i32 { self.content.width() }
            fn height(&self) -> i32 { self.content.height() }
            fn depth(&self) -> i32 { self.content.depth() }
            fn as_xml_internal(&self, prefix: String) -> String {
                "\n".to_owned() + &prefix + "<" + stringify!($e) + ">" + &self.content.as_xml_internal(prefix) + "</" + stringify!($e) + ">"
            }
            fn has_ink(&self) -> bool { self.content.has_ink() }
        }
    )
}

mathgroupkernel!(MathOp);
mathgroupkernel!(MathOpen);
mathgroupkernel!(MathClose);
mathgroupkernel!(MathBin);
mathgroupkernel!(MathOrd);
mathgroupkernel!(MathPunct);
mathgroupkernel!(MathRel);
mathgroupkernel!(MathInner);
mathgroupkernel!(Underline);
mathgroupkernel!(Overline);

#[derive(Clone)]
pub struct MathAccent {
    pub content:Box<Whatsit>,
    pub accent:MathChar,
    pub sourceref:Option<SourceFileReference>
}
impl WhatsitTrait for MathAccent {
    fn as_whatsit(self) -> Whatsit {
        MathKernel::MathAccent(self).as_whatsit()
    }
    fn width(&self) -> i32 { max(self.content.width(),self.accent.width()) }
    fn height(&self) -> i32 { self.content.height() + self.accent.height() }
    fn depth(&self) -> i32 { self.content.depth() }
    fn as_xml_internal(&self, prefix: String) -> String {
        "<mathaccent>".to_owned() + &self.content.as_xml_internal(prefix.clone()) +
            &self.accent.as_xml_internal(prefix) + "</mathaccent>"
    }
    fn has_ink(&self) -> bool { true }
}

#[derive(Clone)]
pub struct Over {
    pub top:Vec<Whatsit>,
    pub bottom:Vec<Whatsit>,
    pub delimiters:Option<Box<(Whatsit,Whatsit)>>,
    pub sourceref:Option<SourceFileReference>
}
impl WhatsitTrait for Over {
    fn as_whatsit(self) -> Whatsit {
        MathInfix::Over(self).as_whatsit()
    }
    fn width(&self) -> i32 {
        let mut upper : i32 = 0;
        let mut lower : i32 = 0;
        for w in &self.top {upper += w.width()}
        for w in &self.bottom {lower += w.width()}
        max(upper,lower)
    }
    fn height(&self) -> i32 {
        let mut upper : i32 = 0;
        let mut lower : i32 = 0;
        for w in &self.top {upper = max(upper,w.height())}
        for w in &self.bottom {lower += max(lower,w.height())}
        upper + lower
    }
    fn depth(&self) -> i32 { 0 }
    fn as_xml_internal(&self, prefix: String) -> String {
        let mut ret = "<over><first>".to_string();
        for w in &self.top { ret += &w.as_xml_internal(prefix.clone())}
        ret += "</first><second>";
        for w in &self.bottom { ret += &w.as_xml_internal(prefix.clone())}
        ret += "</second></over>";
        ret
        // TODO delimiters
    }
    fn has_ink(&self) -> bool { true }
}

#[derive(Clone)]
pub struct Above {
    pub top:Vec<Whatsit>,
    pub bottom:Vec<Whatsit>,
    pub thickness:i32,
    pub delimiters:Option<Box<(Whatsit,Whatsit)>>,
    pub sourceref:Option<SourceFileReference>
}
impl WhatsitTrait for Above {
    fn as_whatsit(self) -> Whatsit {
        MathInfix::Above(self).as_whatsit()
    }
    fn width(&self) -> i32 {
        let mut upper : i32 = 0;
        let mut lower : i32 = 0;
        for w in &self.top {upper += w.width()}
        for w in &self.bottom {lower += w.width()}
        max(upper,lower)
    }
    fn height(&self) -> i32 {
        let mut upper : i32 = 0;
        let mut lower : i32 = 0;
        for w in &self.top {upper = max(upper,w.height())}
        for w in &self.bottom {lower += max(lower,w.height())}
        upper + lower + self.thickness
    }
    fn depth(&self) -> i32 { 0 }
    fn as_xml_internal(&self, prefix: String) -> String {
        let mut ret = "<above><first>".to_string();
        for w in &self.top { ret += &w.as_xml_internal(prefix.clone())}
        ret += "</first><second>";
        for w in &self.bottom { ret += &w.as_xml_internal(prefix.clone())}
        ret += "</second></above>";
        ret
    }
    fn has_ink(&self) -> bool { true }
}