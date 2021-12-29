use std::collections::HashMap;
use std::rc::Rc;
use crate::fonts::Font;
use crate::references::SourceFileReference;
use crate::stomach::simple::SimpleWI;
use crate::stomach::Whatsit;
use crate::stomach::whatsits::{ActionSpec, HasWhatsitIter, WhatsitTrait};
use crate::utils::TeXStr;

#[derive(Clone)]
pub enum WIGroup {
    FontChange(FontChange),
    ColorChange(ColorChange),
    //       rule   attr  action
    PDFLink(PDFLink),
    PDFMatrixSave(PDFMatrixSave),
    External(Rc<dyn ExternalWhatsitGroup>,Vec<Whatsit>)
}
macro_rules! pass_on {
    ($s:tt,$e:ident,($ext:ident,$ch:ident) => $exp:expr $(,$tl:expr)*) => (match $s {
        WIGroup::FontChange(g) => FontChange::$e(g $(,$tl)*),
        WIGroup::ColorChange(g) => ColorChange::$e(g $(,$tl)*),
        WIGroup::PDFLink(g) => PDFLink::$e(g $(,$tl)*),
        WIGroup::PDFMatrixSave(g) => PDFMatrixSave::$e(g $(,$tl)*),
        WIGroup::External($ext,$ch) => $exp
    })
}

pub trait WIGroupTrait : WhatsitTrait {
    fn children(&self) -> &Vec<Whatsit>;
    fn children_mut(&mut self) -> &mut Vec<Whatsit>;
    fn children_prim(self) -> Vec<Whatsit>;
    fn opaque(&self) -> bool;
    fn priority(&self) -> i16;
    //fn new_from(&self) -> Self;
    fn closes_with_group(&self) -> bool;
    fn as_wi_group(self) -> WIGroup;

    fn push(&mut self,wi:Whatsit) {
        self.children_mut().push(wi)
    }
}

pub trait ExternalWhatsitGroup {
    fn name(&self) -> &str;
    fn params(&self,name:&str) -> Option<&str>;
    fn width(&self,ch:&Vec<Whatsit>) -> i32;
    fn height(&self,ch:&Vec<Whatsit>) -> i32;
    fn depth(&self,ch:&Vec<Whatsit>) -> i32;
    fn as_xml_internal(&self,ch:&Vec<Whatsit>, prefix: String) -> String;
    fn has_ink(&self,ch:&Vec<Whatsit>) -> bool;
    fn opaque(&self) -> bool;
    fn priority(&self) -> i16;
    fn closes_with_group(&self) -> bool;
}

impl WhatsitTrait for WIGroup {
    fn as_whatsit(self) -> Whatsit {
        Whatsit::GroupOpen(self)
    }
    fn width(&self) -> i32 {
        pass_on!(self,width,(e,ch)=>e.width(ch))
    }
    fn height(&self) -> i32 { pass_on!(self,height,(e,ch)=>e.height(ch)) }
    fn depth(&self) -> i32 { pass_on!(self,depth,(e,ch)=>e.depth(ch)) }
    fn as_xml_internal(&self, prefix: String) -> String {
        pass_on!(self,as_xml_internal,(e,ch)=>e.as_xml_internal(ch,prefix),prefix)
    }
    fn has_ink(&self) -> bool {
        pass_on!(self,has_ink,(e,ch)=>e.has_ink(ch))
    }
}
impl WIGroup {
    pub fn new_from(&self) -> Self {
        match self {
            WIGroup::FontChange(g) => WIGroup::FontChange(g.new_from()),
            WIGroup::ColorChange(g) => WIGroup::ColorChange(g.new_from()),
            WIGroup::PDFLink(g) => WIGroup::PDFLink(g.new_from()),
            WIGroup::PDFMatrixSave(g) => WIGroup::PDFMatrixSave(g.new_from()),
            WIGroup::External(e,_) => WIGroup::External(e.clone(),vec!())
        }
    }
}
impl WIGroupTrait for WIGroup {
    fn children(&self) -> &Vec<Whatsit> {
        pass_on!(self,children,(e,ch)=>ch)
    }
    fn children_mut(&mut self) -> &mut Vec<Whatsit> {
        pass_on!(self,children_mut,(e,ch)=>ch)
    }
    fn children_prim(self) -> Vec<Whatsit> {
        pass_on!(self,children_prim,(e,ch)=>ch)
    }
    fn opaque(&self) -> bool {
        pass_on!(self,opaque,(e,ch)=>e.opaque())
    }
    fn priority(&self) -> i16 {
        pass_on!(self,priority,(e,ch)=>e.priority())
    }
    fn as_wi_group(self) -> WIGroup { self }
    fn closes_with_group(&self) -> bool {
        pass_on!(self,closes_with_group,(e,ch)=>e.closes_with_group())
    }
}

// -------------------------------------------------------------------------------------------------

#[derive(Clone)]
pub struct FontChange {
    pub font:Rc<Font>,
    pub closes_with_group:bool,
    pub children:Vec<Whatsit>,
    pub sourceref:Option<SourceFileReference>
}
impl WhatsitTrait for FontChange {
    fn as_whatsit(self) -> Whatsit {
        Whatsit::GroupOpen(WIGroup::FontChange(self))
    }
    fn width(&self) -> i32 { todo!() }
    fn height(&self) -> i32 { todo!() }
    fn depth(&self) -> i32 { todo!() }
    fn as_xml_internal(&self, prefix: String) -> String {
        let mut ret = "\n".to_string() + &prefix + "<font TODO=\"\">";
        for c in &self.children {
            ret += &c.as_xml_internal(prefix.clone() + "  ")
        }
        ret + "\n" + &prefix + "</font>"
    }
    fn has_ink(&self) -> bool {
        for w in self.children.iter_wi() {
            if w.has_ink() { return true }
        }
        return false
    }
}
impl FontChange {
    pub fn new_from(&self) -> Self {
        FontChange {
            font: self.font.clone(),
            closes_with_group: self.closes_with_group,
            children: vec![],
            sourceref: self.sourceref.clone()
        }
    }

}
impl WIGroupTrait for FontChange {
    fn children(&self) -> &Vec<Whatsit> { &self.children }
    fn children_prim(self) -> Vec<Whatsit> { self.children }
    fn as_wi_group(self) -> WIGroup { WIGroup::FontChange(self) }
    fn children_mut(&mut self) -> &mut Vec<Whatsit> { self.children.as_mut() }
    fn opaque(&self) -> bool { false }
    fn priority(&self) -> i16 {
        if self.closes_with_group { 25 } else { 2 }
    }
    fn closes_with_group(&self) -> bool {
        self.closes_with_group
    }
}

#[derive(Clone)]
pub struct ColorChange {
    pub color:TeXStr,
    pub children:Vec<Whatsit>,
    pub sourceref:Option<SourceFileReference>
}
impl WhatsitTrait for ColorChange {
    fn as_whatsit(self) -> Whatsit {
        WIGroup::ColorChange(self).as_whatsit()
    }
    fn width(&self) -> i32 { todo!() }
    fn height(&self) -> i32 { todo!() }
    fn depth(&self) -> i32 { todo!() }

    fn as_xml_internal(&self, prefix: String) -> String {
        let mut ret = "\n".to_string() + &prefix + "<color color=\"" + self.color.to_string().as_str() + "\">";
        for c in &self.children {
            ret += &c.as_xml_internal(prefix.clone() + "  ")
        }
        ret + "\n" + &prefix + "</color>"
    }
    fn has_ink(&self) -> bool {
        for w in self.children.iter_wi() {
            if w.has_ink() { return true }
        }
        return false
    }
}
impl ColorChange {
    pub fn new_from(&self) -> Self {
        ColorChange {
            color: self.color.clone(),
            children: vec![],
            sourceref: self.sourceref.clone()
        }
    }
}
impl WIGroupTrait for ColorChange {
    fn children(&self) -> &Vec<Whatsit> { &self.children }
    fn children_prim(self) -> Vec<Whatsit> { self.children }
    fn children_mut(&mut self) -> &mut Vec<Whatsit> { self.children.as_mut() }
    fn opaque(&self) -> bool { false }
    fn priority(&self) -> i16 { 50 }
    fn closes_with_group(&self) -> bool { false }
    fn as_wi_group(self) -> WIGroup {
        WIGroup::ColorChange(self)
    }
}

#[derive(Clone)]
pub struct PDFLink {
    pub rule: TeXStr,
    pub attr: TeXStr,
    pub action: ActionSpec,
    pub children: Vec<Whatsit>,
    pub sourceref:Option<SourceFileReference>
}
impl WhatsitTrait for PDFLink {
    fn as_whatsit(self) -> Whatsit {
        WIGroup::PDFLink(self).as_whatsit()
    }
    fn width(&self) -> i32 { todo!() }
    fn height(&self) -> i32 { todo!() }
    fn depth(&self) -> i32 { todo!() }
    fn as_xml_internal(&self, prefix: String) -> String {
        let mut ret = "\n".to_string() + &prefix + "<link rule=\"" +
            self.rule.to_string().as_str() + "\" attr=\"" +
            self.attr.to_string().as_str() + &self.action.as_xml() + "\">";
        for c in &self.children {
            ret += &c.as_xml_internal(prefix.clone() + "  ")
        }
        ret + "\n" + &prefix + "</link>"
    }
    fn has_ink(&self) -> bool {
        for w in self.children.iter_wi() {
            if w.has_ink() { return true }
        }
        return false
    }
}
impl PDFLink {
    pub fn new_from(&self) -> Self {
        PDFLink {
            rule:self.rule.clone(),
            attr:self.attr.clone(),
            action:self.action.clone(),
            children: vec![],
            sourceref: self.sourceref.clone()
        }
    }
}
impl WIGroupTrait for PDFLink {
    fn children(&self) -> &Vec<Whatsit> { &self.children }
    fn children_prim(self) -> Vec<Whatsit> { self.children }
    fn children_mut(&mut self) -> &mut Vec<Whatsit> { self.children.as_mut() }
    fn opaque(&self) -> bool { false }
    fn priority(&self) -> i16 { 60 }
    fn closes_with_group(&self) -> bool { false }
    fn as_wi_group(self) -> WIGroup {
        WIGroup::PDFLink(self)
    }
}

#[derive(Clone)]
pub struct PDFMatrixSave {
    pub is_vertical:bool,
    pub children: Vec<Whatsit>,
    pub sourceref:Option<SourceFileReference>
}
impl WhatsitTrait for PDFMatrixSave {
    fn as_whatsit(self) -> Whatsit {
        WIGroup::PDFMatrixSave(self).as_whatsit()
    }
    fn width(&self) -> i32 { todo!() }
    fn height(&self) -> i32 { todo!() }
    fn depth(&self) -> i32 { todo!() }
    fn as_xml_internal(&self, prefix: String) -> String {
        let mut ret = "\n".to_string() + &prefix + "<pdfmatrix>";
        for c in &self.children {
            ret += &c.as_xml_internal(prefix.clone() + "  ")
        }
        ret + "\n" + &prefix + "</pdfmatrix>"
    }
    fn has_ink(&self) -> bool {
        for w in self.children.iter_wi() {
            if w.has_ink() { return true }
        }
        return false
    }
}
impl PDFMatrixSave {
    pub fn new_from(&self) -> Self {
        match self.children.iter_wi().find(|x| match x {
            Whatsit::Simple(SimpleWI::PDFMatrix(_)) => true,
            _ => false
        }) {
            None => PDFMatrixSave {
                is_vertical:self.is_vertical,
                children:vec!(),
                sourceref:self.sourceref.clone()
            },
            Some(p) => PDFMatrixSave {
                is_vertical:self.is_vertical,
                children:vec!(p.clone()),
                sourceref:self.sourceref.clone()
            }
        }
    }
}
impl WIGroupTrait for PDFMatrixSave {
    fn children(&self) -> &Vec<Whatsit> { &self.children }
    fn children_prim(self) -> Vec<Whatsit> { self.children }
    fn children_mut(&mut self) -> &mut Vec<Whatsit> { self.children.as_mut() }
    fn opaque(&self) -> bool { true }
    fn priority(&self) -> i16 { 70 }
    fn closes_with_group(&self) -> bool { false }
    fn as_wi_group(self) -> WIGroup { WIGroup::PDFMatrixSave(self) }
}


// -------------------------------------------------------------------------------------------------

pub trait ExternalWhatsitGroupEnd {
    fn name(&self) -> &str;
    fn params(&self,name:&str) -> Option<&str>;
    fn priority(&self) -> i16;
}

#[derive(Clone)]
pub enum GroupClose {
    PDFRestore(PDFRestore),
    LinkEnd(LinkEnd),
    ColorEnd(ColorEnd),
    EndGroup(EndGroup),
    External(Rc<dyn ExternalWhatsitGroupEnd>)
}
macro_rules! pass_on_close {
    ($s:tt,$e:ident,$ext:ident => $exp:expr $(,$tl:expr)*) => (match $s {
        GroupClose::PDFRestore(g) => PDFRestore::$e(g $(,$tl)*),
        GroupClose::LinkEnd(g) => LinkEnd::$e(g $(,$tl)*),
        GroupClose::ColorEnd(g) => ColorEnd::$e(g $(,$tl)*),
        GroupClose::EndGroup(g) => EndGroup::$e(g $(,$tl)*),
        GroupClose::External($ext) => $exp
    })
}
pub trait WIGroupCloseTrait : WhatsitTrait {
    fn priority(&self) -> i16;
    fn as_whatsit_i(self) -> Whatsit;
}
impl WIGroupCloseTrait for GroupClose {
    fn priority(&self) -> i16 {
        pass_on_close!(self,priority,e => e.priority())
    }
    fn as_whatsit_i(self) -> Whatsit {
        pass_on_close!(self,as_whatsit_i,e=>Whatsit::GroupClose(GroupClose::External(e)))
    }
}
impl WhatsitTrait for GroupClose {
    fn as_whatsit(self) -> Whatsit {
        WIGroupCloseTrait::as_whatsit_i(self)
    }
    fn width(&self) -> i32 { 0 }
    fn height(&self) -> i32 { 0 }
    fn depth(&self) -> i32 { 0 }
    fn as_xml_internal(&self, _: String) -> String { "".to_string() }
    fn has_ink(&self) -> bool { false }
}

macro_rules! groupclose {
    ($e:ident,$p:expr) => (
        #[derive(Clone)]
        pub struct $e {
            pub sourceref:Option<SourceFileReference>
        }
        impl WIGroupCloseTrait for $e {
            fn priority(&self) -> i16 { $p }
            fn as_whatsit_i(self) -> Whatsit {
                Whatsit::GroupClose(GroupClose::$e(self))
            }
        }
        impl WhatsitTrait for $e {
            fn as_whatsit(self) -> Whatsit {
                WIGroupCloseTrait::as_whatsit_i(self)
            }
            fn width(&self) -> i32 { 0 }
            fn height(&self) -> i32 { 0 }
            fn depth(&self) -> i32 { 0 }
            fn as_xml_internal(&self, prefix: String) -> String { "".to_string() }
            fn has_ink(&self) -> bool { false }
        }
    )
}
groupclose!(PDFRestore,70);
groupclose!(LinkEnd,60);
groupclose!(ColorEnd,50);
groupclose!(EndGroup,25);