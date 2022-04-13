use std::cmp::max;
use std::sync::Arc;
use crate::fonts::Font;
use crate::{htmlannotate, htmlliteral, htmlnode, htmlparent};
use crate::interpreter::dimensions::{MuSkip, numtostr};
use crate::references::SourceFileReference;
use crate::stomach::colon::ColonMode;
use crate::stomach::html::{dimtohtml, HTMLAnnotation, HTMLChild, HTMLColon, HTMLNode, HTMLParent, HTMLStr, MATHML_NS, FontInfo};
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
    fn normalize(self, mode: &ColonMode, ret: &mut Vec<Whatsit>, _: Option<f32>) {
        let subscript = match self.subscript {
            Some(k) => normalize_kernel(k),
            _ => None
        };
        let superscript = match self.superscript {
            Some(k) => normalize_kernel(k),
            _ => None
        };
        let mut kernel = match normalize_kernel(self.kernel) {
            None if subscript.is_none() && superscript.is_none() => return,
            None => MathKernel::Group(GroupedMath(vec!())),
            Some(k) => k
        };
        if superscript.is_none() && subscript.is_none() {
            use crate::stomach::simple::SimpleWI;
            match &mut kernel {
                MathKernel::Group(GroupedMath(ref mut v)) if v.len() == 1 => {
                    match v.pop().unwrap() {
                        o@Whatsit::Simple(SimpleWI::HAlign(_)) => {
                            ret.push(o);
                            return
                        }
                        Whatsit::Box(b) if *mode != ColonMode::M => ret.push(b.as_whatsit()),
                        o if *mode == ColonMode::M => {
                            ret.push(o);
                            return
                        }
                        o => {
                            v.push(o)
                        }
                    }
                }
                _ => (),
            };
        }
        ret.push(MathGroup { kernel, subscript, superscript, limits: self.limits }.as_whatsit());
    }
    fn as_html(self, mode: &ColonMode, colon: &mut HTMLColon, node_top: &mut Option<HTMLParent>) {
        match mode {
            ColonMode::H if self.limits => htmlnode!(colon,div,None,"displaymathcontainer",node_top,div =>{
                div.style("width".into(),"100%".into());
                div.style("min-width".into(),"100%".into());
                if crate::INSERT_RUSTEX_ATTRS {
                    div.attr("rustex:width".into(),dimtohtml(self.width()));
                    div.attr("rustex:height".into(),dimtohtml(self.height()));
                }
                htmlnode!(colon,MATHML_NS:math,None,"",htmlparent!(div),node=> {
                    node.attr("displaystyle".into(),"true".into());
                    htmlnode!(colon,mrow,None,"",htmlparent!(node),mrow => {
                        self.as_html(&ColonMode::M,colon,htmlparent!(mrow));
                        /*if mrow.children.len() == 1 {
                            match mrow.children.pop().unwrap() {
                                HTMLChild::Node(n) if n.name == "mrow".into() => mrow.children = n.children,
                                o => mrow.children.push(o)
                            }
                        }*/
                    })
                })
            }),
            ColonMode::H => htmlnode!(colon,MATHML_NS:math,None,"",node_top,node=> {
                htmlnode!(colon,mrow,None,"",htmlparent!(node),mrow => {
                    self.as_html(&ColonMode::M,colon,htmlparent!(mrow));
                    /*if mrow.children.len() == 1 {
                        match mrow.children.pop().unwrap() {
                            HTMLChild::Node(n) if n.name == "mrow".into() => mrow.children = n.children,
                            o => mrow.children.push(o)
                        }
                    }*/
                })
            }),
            ColonMode::M => match (self.subscript,self.superscript) {
                (None,None) => self.kernel.as_html(mode,colon,node_top),
                (None,Some(ss)) if self.limits && self.kernel.is_largeop() => htmlnode!(colon,mover,None,"",node_top,msup => {
                    msup.attr("displaystyle".into(),"true".into());
                    self.kernel.as_html(mode,colon,htmlparent!(msup));
                    if msup.children.is_empty() { htmlnode!(colon,mrow,None,"",htmlparent!(msup)) }
                    msup.children.push(HTMLChild::Str("".into()));
                    ss.as_html(mode,colon,htmlparent!(msup));
                    if msup.children.len() < 3 { htmlnode!(colon,mrow,None,"",htmlparent!(msup)) }
                }),
                (None,Some(ss)) => htmlnode!(colon,msup,None,"",node_top,msup => {
                    msup.attr("displaystyle".into(),"false".into());
                    self.kernel.as_html(mode,colon,htmlparent!(msup));
                    if msup.children.is_empty() { htmlnode!(colon,mrow,None,"",htmlparent!(msup)) }
                    msup.children.push(HTMLChild::Str("".into()));
                    ss.as_html(mode,colon,htmlparent!(msup));
                    if msup.children.len() < 3 { htmlnode!(colon,mrow,None,"",htmlparent!(msup)) }
                }),
                (Some(ss),None) if self.limits && self.kernel.is_largeop() => htmlnode!(colon,munder,None,"",node_top,msup => {
                    msup.attr("displaystyle".into(),"true".into());
                    self.kernel.as_html(mode,colon,htmlparent!(msup));
                    if msup.children.is_empty() { htmlnode!(colon,mrow,None,"",htmlparent!(msup)) }
                    msup.children.push(HTMLChild::Str("".into()));
                    ss.as_html(mode,colon,htmlparent!(msup));
                    if msup.children.len() < 3 { htmlnode!(colon,mrow,None,"",htmlparent!(msup)) }
                }),
                (Some(ss),None) => htmlnode!(colon,msub,None,"",node_top,msup => {
                    msup.attr("displaystyle".into(),"false".into());
                    self.kernel.as_html(mode,colon,htmlparent!(msup));
                    if msup.children.is_empty() { htmlnode!(colon,mrow,None,"",htmlparent!(msup)) }
                    msup.children.push(HTMLChild::Str("".into()));
                    ss.as_html(mode,colon,htmlparent!(msup));
                    if msup.children.len() < 3 { htmlnode!(colon,mrow,None,"",htmlparent!(msup)) }
                }),
                (Some(subk),Some(supk)) if self.limits && self.kernel.is_largeop() => htmlnode!(colon,munderover,None,"",node_top,msub => {
                    msub.attr("displaystyle".into(),"true".into());
                    self.kernel.as_html(mode,colon,htmlparent!(msub));
                    if msub.children.is_empty() { htmlnode!(colon,mrow,None,"",htmlparent!(msub)) }
                    msub.children.push(HTMLChild::Str("".into()));
                    subk.as_html(mode,colon,htmlparent!(msub));
                    if msub.children.len() < 3 { htmlnode!(colon,mrow,None,"",htmlparent!(msub)) }
                    msub.children.push(HTMLChild::Str("".into()));
                    supk.as_html(mode,colon,htmlparent!(msub));
                    if msub.children.len() < 5 { htmlnode!(colon,mrow,None,"",htmlparent!(msub)) }
                }),
                (Some(subk),Some(supk)) => htmlnode!(colon,msubsup,None,"",node_top,msub => {
                    msub.attr("displaystyle".into(),"false".into());
                    self.kernel.as_html(mode,colon,htmlparent!(msub));
                    if msub.children.is_empty() { htmlnode!(colon,mrow,None,"",htmlparent!(msub)) }
                    msub.children.push(HTMLChild::Str("".into()));
                    subk.as_html(mode,colon,htmlparent!(msub));
                    if msub.children.len() < 3 { htmlnode!(colon,mrow,None,"",htmlparent!(msub)) }
                    msub.children.push(HTMLChild::Str("".into()));
                    supk.as_html(mode,colon,htmlparent!(msub));
                    if msub.children.len() < 5 { htmlnode!(colon,mrow,None,"",htmlparent!(msub)) }
                })
            }
            _ => todo!()
        }
    }
}

fn normalize_kernel(k:MathKernel) -> Option<MathKernel> {
    let mut nret : Vec<Whatsit> = vec!();
    k.normalize(&ColonMode::M,&mut nret,None);
    if nret.is_empty() { return None } else if nret.len() == 1 {
        match nret.pop() {
            Some(Whatsit::Math(MathGroup { kernel,subscript:None,superscript:None,limits:_})) => {
                return Some(kernel)
            }
            Some(o) => nret.push(o),
            _ => unreachable!()
        }
    }
    Some(MathKernel::Group(GroupedMath(nret)))
}

#[derive(Clone)]
pub enum MathKernel {
    Group(GroupedMath),
    MathChar(MathChar),
    MKern(MKern),
    Delimiter(Delimiter),
    Radical(Radical),
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
    MathAccent(MathAccent),
    CustomMath(CustomMathChar)
}

macro_rules! pass_on_kernel {
    ($s:tt,$e:ident$(,$tl:expr)*) => (match $s {
        MathKernel::Group(g) => GroupedMath::$e(g $(,$tl)*),
        MathKernel::MathChar(g) => MathChar::$e(g $(,$tl)*),
        MathKernel::MKern(g) => MKern::$e(g $(,$tl)*),
        MathKernel::Delimiter(g) => Delimiter::$e(g $(,$tl)*),
        MathKernel::Radical(g) => Radical::$e(g $(,$tl)*),
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
        MathKernel::MathAccent(g) => MathAccent::$e(g $(,$tl)*),
        MathKernel::CustomMath(g) => CustomMathChar::$e(g $(,$tl)*),
    })
}
impl MathKernel {
    pub fn is_largeop(&self) -> bool {
        match self {
            MathKernel::MathOp(_) => true,
            MathKernel::MathChar(mc) if mc.class == 1 => true,
            _ => false
        }
    }
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
    fn normalize(self, mode: &ColonMode, ret: &mut Vec<Whatsit>, scale: Option<f32>) {
        pass_on_kernel!(self,normalize,mode,ret,scale)
    }
    fn as_html(self, mode: &ColonMode, colon: &mut HTMLColon, node_top: &mut Option<HTMLParent>) {
        pass_on_kernel!(self,as_html,mode,colon,node_top)
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
    fn normalize(self, mode: &ColonMode, ret: &mut Vec<Whatsit>, scale: Option<f32>) {
        let mut nret : Vec<Whatsit> = vec!();
        for w in self.0 { w.normalize(mode,&mut nret,scale) }
        if nret.is_empty() { return }
        ret.push(GroupedMath(nret).as_whatsit())
    }
    fn as_html(mut self, mode: &ColonMode, colon: &mut HTMLColon, node_top: &mut Option<HTMLParent>) {
        if self.0.is_empty() {}
        else if self.0.len() == 1 {
            self.0.pop().unwrap().as_html(mode,colon,node_top)
        }
        else {
            htmlannotate!(colon,mrow,None,node_top,node => {
                for w in self.0 { w.as_html(mode,colon,htmlparent!(node)) }
            })
        }
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
    fn as_xml_internal(&self, _: String) -> String {
        "<mkern value=\"".to_string() + &self.sk.to_string() + "\"/>"
    }
    fn has_ink(&self) -> bool { false }
    fn normalize(self, _: &ColonMode, ret: &mut Vec<Whatsit>, _: Option<f32>) {
        match ret.last_mut() {
            Some(Whatsit::Math(MathGroup { kernel:MathKernel::MKern(ref mut mk),subscript:None,superscript:None,limits:_})) =>
                mk.sk = mk.sk + self.sk,
            _ if self.sk.base == 0 => (),
            _ => ret.push(self.as_whatsit())
        }
    }
    fn as_html(self, _: &ColonMode, colon: &mut HTMLColon, node_top: &mut Option<HTMLParent>) {
        htmlnode!(colon,mspace,self.sourceref,"mkern",node_top,a => {
            a.attr("width".into(),numtostr((self.sk.base as f32 / 1179648.0).round() as i32,"em").into())
        })
    }
}

#[derive(Clone)]
pub struct CustomMathChar {
    pub str:String,
    pub font:Arc<Font>,
    pub sourceref:Option<SourceFileReference>
}
impl WhatsitTrait for CustomMathChar {
    fn as_whatsit(self) -> Whatsit {
        MathKernel::CustomMath(self).as_whatsit()
    }
    fn width(&self) -> i32 { self.font.get_width(32) }
    fn height(&self) -> i32 { self.font.get_height(32) }
    fn depth(&self) -> i32 { self.font.get_depth(32) }
    fn as_xml_internal(&self, prefix: String) -> String {
        "\n".to_owned() + &prefix + "<mathchar value=\"" + &self.str + "\"/>"
    }
    fn has_ink(&self) -> bool { true }
    fn normalize(self, _: &ColonMode, ret: &mut Vec<Whatsit>, _: Option<f32>) {
        ret.push(self.as_whatsit())
    }
    fn as_html(self, _: &ColonMode, colon: &mut HTMLColon, node_top: &mut Option<HTMLParent>) {
        let maybemimo = match match node_top {
            Some(HTMLParent::N(n)) => n.children.last_mut(),
            Some(HTMLParent::A(n)) => n.children.last_mut(),
            _ => None
        } {
            Some(HTMLChild::Node(n)) => Some(n),
            _ => None
        };
        let mut charstr : HTMLStr = self.str.into();
        charstr = charstr.html_escape();
        let mimoinfo = FontInfo::new(&self.font);
        let clsstr : HTMLStr = "binop".into();
        match maybemimo {
            Some(n) if String::from(&n.name) == "mo" && n.fontinfo.is_some() && n.fontinfo.as_ref().unwrap() == &mimoinfo => {
                n.children.push(HTMLChild::Str(charstr))
            },
            _ => {
                htmlnode!(colon,mo,self.sourceref,clsstr,node_top,a => {
                    a.fontinfo = Some(mimoinfo);
                    if crate::INSERT_RUSTEX_ATTRS { a.attr("rustex:font".into(),(&self.font.file.name).into()) }
                    htmlliteral!(colon,htmlparent!(a),>charstr<)
                })
            }
        }
    }
}

#[derive(Clone)]
pub struct MathChar {
    pub class:u32,
    pub family:u32,
    pub position:u32,
    pub font:Arc<Font>,
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
    fn normalize(self, _: &ColonMode, ret: &mut Vec<Whatsit>, _: Option<f32>) {
        ret.push(self.as_whatsit())
    }
    fn as_html(self, _: &ColonMode, colon: &mut HTMLColon, node_top: &mut Option<HTMLParent>) {
        let maybemimo = match match node_top {
            Some(HTMLParent::N(n)) => n.children.last_mut(),
            Some(HTMLParent::A(n)) => n.children.last_mut(),
            _ => None
        } {
            Some(HTMLChild::Node(n)) => Some(n),
            _ => None
        };
        let mut charstr : HTMLStr = match &self.font.file.chartable {
            Some(ct) => ct.get_char(self.position as u8).into(),
            None => {
                //println!("Here! {} in {}",mc.position,mc.font.name);
                "???".into()
            }
        };
        charstr = charstr.html_escape();
        let mimoinfo = FontInfo::new(&self.font);
        let clsstr : HTMLStr = (match self.class {
            1 => "largeop",
            2 => "binop",
            3 => "rel",
            4 => "opening",
            5 => "closing",
            6 => "punctuation",
            _ => "",
        }).into();
        match (maybemimo,self.class) {
            (Some(n),0|7) if String::from(&n.name) == "mi" && n.fontinfo.is_some() && n.fontinfo.as_ref().unwrap() == &mimoinfo => {
                n.children.push(HTMLChild::Str(charstr))
            }
            (Some(n),i) if 0<i && i<7 && String::from(&n.name) == "mo" && n.fontinfo.is_some() && n.fontinfo.as_ref().unwrap() == &mimoinfo => {
                n.children.push(HTMLChild::Str(charstr))
            }
            (_,0|7) => {
                htmlnode!(colon,mi,self.sourceref,clsstr,node_top,a => {
                    a.fontinfo = Some(mimoinfo);
                    if crate::INSERT_RUSTEX_ATTRS { a.attr("rustex:font".into(),(&self.font.file.name).into()) }
                    htmlliteral!(colon,htmlparent!(a),>charstr<)
                })
            }
            (_,_) => {
                htmlnode!(colon,mo,self.sourceref,clsstr,node_top,a => {
                    a.fontinfo = Some(mimoinfo);
                    if crate::INSERT_RUSTEX_ATTRS { a.attr("rustex:font".into(),(&self.font.file.name).into()) }
                    htmlliteral!(colon,htmlparent!(a),>charstr<)
                })
            }
        }
    }
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
    fn normalize(self, _: &ColonMode, ret: &mut Vec<Whatsit>, _: Option<f32>) {
        ret.push(self.as_whatsit())
    }
    fn as_html(self, mode: &ColonMode, colon: &mut HTMLColon, node_top: &mut Option<HTMLParent>) {
        self.small.as_html(mode,colon,node_top)
    }
}

#[derive(Clone)]
pub struct Radical {
    pub small:MathChar,
    pub large:MathChar,
    pub sourceref:Option<SourceFileReference>
}
impl WhatsitTrait for Radical {
    fn as_whatsit(self) -> Whatsit {
        MathKernel::Radical(self).as_whatsit()
    }
    fn width(&self) -> i32 { self.small.width() }
    fn height(&self) -> i32 { self.small.height() }
    fn depth(&self) -> i32 { self.small.depth() }
    fn as_xml_internal(&self, prefix: String) -> String {
        "\n".to_owned() + &prefix + "<radical>" + &self.small.as_xml_internal(prefix.clone()) + &self.large.as_xml_internal(prefix) + "</delimiter>"
    }
    fn has_ink(&self) -> bool { true }
    fn normalize(self, _: &ColonMode, ret: &mut Vec<Whatsit>, _: Option<f32>) {
        ret.push(self.as_whatsit())
    }
    fn as_html(self, mode: &ColonMode, colon: &mut HTMLColon, node_top: &mut Option<HTMLParent>) {
        self.small.as_html(mode,colon,node_top)
    }
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
            fn normalize(self, mode: &ColonMode, ret: &mut Vec<Whatsit>, scale: Option<f32>) {
                let mut nret : Vec<Whatsit> = vec!();
                self.content.normalize(mode,&mut nret,scale);
                let nw = match nret.len() {
                    1 => {
                        nret.pop().unwrap()
                    },
                    _ => GroupedMath(nret).as_whatsit()
                };
                ret.push($e { content:std::boxed::Box::new(nw), sourceref:self.sourceref }.as_whatsit())
            }
            fn as_html(self, mode: &ColonMode, colon: &mut HTMLColon, node_top: &mut Option<HTMLParent>) {
                htmlannotate!(colon,mrow,self.sourceref,node_top,node => {
                    node.classes.push(stringify!($e).into());
                    self.content.clone().as_html(mode,colon,htmlparent!(node))
                })
            }
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
    fn normalize(self, mode: &ColonMode, ret: &mut Vec<Whatsit>, scale: Option<f32>) {
        let mut nret : Vec<Whatsit> = vec!();
        self.content.normalize(mode,&mut nret,scale);
        let nw = match nret.len() {
            1 => {
                nret.pop().unwrap()
            },
            _ => GroupedMath(nret).as_whatsit()
        };
        ret.push(MathAccent { content:std::boxed::Box::new(nw), sourceref:self.sourceref,accent:self.accent }.as_whatsit())
    }
    fn as_html(self, mode: &ColonMode, colon: &mut HTMLColon, node_top: &mut Option<HTMLParent>) {
        htmlnode!(colon,mover,self.sourceref,"mathaccent",node_top,node => {
            node.attr("accent".into(),"true".into());
            htmlannotate!(colon,mrow,None,htmlparent!(node),mrow => {
                self.content.clone().as_html(mode,colon,htmlparent!(mrow))
            });
            htmlannotate!(colon,mrow,None,htmlparent!(node),mrow => {
                self.accent.as_html(mode,colon,htmlparent!(mrow))
            });
        })
    }
}

#[derive(Clone)]
pub struct Above {
    pub top:Vec<Whatsit>,
    pub bottom:Vec<Whatsit>,
    pub thickness:Option<i32>,
    pub delimiters:(Option<MathChar>,Option<MathChar>),
    pub sourceref:Option<SourceFileReference>
}
impl Above {
    pub fn set(&mut self,first:Vec<Whatsit>,second:Vec<Whatsit>) {
        self.top = first;
        self.bottom = second
    }
}
impl WhatsitTrait for Above {
    fn as_whatsit(self) -> Whatsit {
        Whatsit::Above(self)
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
        for w in &self.bottom {lower = max(lower,w.height())}
        upper + lower + self.thickness.unwrap_or(0) + (5*65536)
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
    fn normalize(self, mode: &ColonMode, ret: &mut Vec<Whatsit>, scale: Option<f32>) {
        let mut ntop: Vec<Whatsit> = vec!();
        let mut nbottom: Vec<Whatsit> = vec!();
        for c in self.top { c.normalize(mode,&mut ntop,scale)}
        for c in self.bottom { c.normalize(mode,&mut nbottom,scale)}
        ret.push(crate::stomach::math::Above {
            top:ntop,bottom:nbottom,delimiters:self.delimiters,sourceref:self.sourceref,thickness:self.thickness
        }.as_whatsit())
    }
    fn as_html(self, mode: &ColonMode, colon: &mut HTMLColon, node_top: &mut Option<HTMLParent>) {
        htmlannotate!(colon,mrow,None,node_top,mrow => {
            match self.delimiters.0 {
                Some(d) => d.as_html(mode,colon,htmlparent!(mrow)),
                _ => ()
            }
            htmlnode!(colon,mfrac,self.sourceref,"over",htmlparent!(mrow),over => {
                match self.thickness {
                    Some(i) => over.attr("linethickness".into(),dimtohtml(i)),
                    _ => ()
                }
                htmlannotate!(colon,mrow,None,htmlparent!(over),a => {
                    for c in self.top { c.as_html(mode,colon,htmlparent!(a)) }
                });
                htmlannotate!(colon,mrow,None,htmlparent!(over),a => {
                    for c in self.bottom { c.as_html(mode,colon,htmlparent!(a)) }
                })
            });
            match self.delimiters.1 {
                Some(d) => d.as_html(mode,colon,htmlparent!(mrow)),
                _ => ()
            }
        })
    }
}