use std::sync::Arc;
use crate::fonts::Font;
use crate::fonts::fontchars::FontTableParam;
use crate::{htmlannotate, htmlnode, htmlparent};
use crate::interpreter::state::GroupType;
use crate::references::SourceFileReference;
use crate::stomach::colon::ColonMode;
use crate::stomach::html::{HTMLAnnotation, HTMLChild, HTMLColon, HTMLNode, HTMLParent, HTMLStr};
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
    External(Arc<dyn ExternalWhatsitGroup>,Vec<Whatsit>),
    GroupOpen(GroupType)
}
macro_rules! pass_on {
    ($s:tt,$e:ident,($ext:ident,$ch:ident) => $exp:expr $(,$tl:expr)*) => (match $s {
        WIGroup::FontChange(g) => FontChange::$e(g $(,$tl)*),
        WIGroup::ColorChange(g) => ColorChange::$e(g $(,$tl)*),
        WIGroup::PDFLink(g) => PDFLink::$e(g $(,$tl)*),
        WIGroup::PDFMatrixSave(g) => PDFMatrixSave::$e(g $(,$tl)*),
        WIGroup::External($ext,$ch) => $exp,
        WIGroup::GroupOpen(_) => unreachable!()
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

pub trait ExternalWhatsitGroup : Send+Sync {
    fn name(&self) -> TeXStr;
    fn params(&self,name:&str) -> Option<TeXStr>;
    fn width(&self,ch:&Vec<Whatsit>) -> i32;
    fn height(&self,ch:&Vec<Whatsit>) -> i32;
    fn depth(&self,ch:&Vec<Whatsit>) -> i32;
    fn as_xml_internal(&self,ch:&Vec<Whatsit>, prefix: String) -> String;
    fn has_ink(&self,ch:&Vec<Whatsit>) -> bool;
    fn opaque(&self) -> bool;
    fn priority(&self) -> i16;
    fn closes_with_group(&self) -> bool;
    fn sourceref(&self) -> &Option<SourceFileReference>;
    fn normalize(&self,ch:Vec<Whatsit>, mode: &ColonMode, ret: &mut Vec<Whatsit>, scale: Option<f32>);
    fn as_html(&self,ch:Vec<Whatsit>, mode: &ColonMode, colon:&mut HTMLColon, node_top: &mut Option<HTMLParent>);
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
    fn normalize(self, mode: &ColonMode, ret: &mut Vec<Whatsit>, scale: Option<f32>) {
        pass_on!(self,normalize,(e,ch) => e.normalize(ch,mode,ret,scale),mode,ret,scale);
    }
    fn as_html(self, mode: &ColonMode, colon: &mut HTMLColon, node_top: &mut Option<HTMLParent>) {
        pass_on!(self,as_html,(e,ch) => e.as_html(ch,mode,colon,node_top),mode,colon,node_top)
    }
}
impl WIGroup {
    pub fn new_from(&self) -> Self {
        match self {
            WIGroup::FontChange(g) => WIGroup::FontChange(g.new_from()),
            WIGroup::ColorChange(g) => WIGroup::ColorChange(g.new_from()),
            WIGroup::PDFLink(g) => WIGroup::PDFLink(g.new_from()),
            WIGroup::PDFMatrixSave(g) => WIGroup::PDFMatrixSave(g.new_from()),
            WIGroup::External(e,_) => WIGroup::External(e.clone(),vec!()),
            WIGroup::GroupOpen(_) => unreachable!()
        }
    }
}
impl WIGroupTrait for WIGroup {
    fn children(&self) -> &Vec<Whatsit> {
        pass_on!(self,children,(_e,ch)=>ch)
    }
    fn children_mut(&mut self) -> &mut Vec<Whatsit> {
        pass_on!(self,children_mut,(_e,ch)=>ch)
    }
    fn children_prim(self) -> Vec<Whatsit> {
        pass_on!(self,children_prim,(_e,ch)=>ch)
    }
    fn opaque(&self) -> bool {
        pass_on!(self,opaque,(e,_ch)=>e.opaque())
    }
    fn priority(&self) -> i16 {
        pass_on!(self,priority,(e,_ch)=>e.priority())
    }
    fn as_wi_group(self) -> WIGroup { self }
    fn closes_with_group(&self) -> bool {
        pass_on!(self,closes_with_group,(e,_ch)=>e.closes_with_group())
    }
}

// -------------------------------------------------------------------------------------------------

#[derive(Clone)]
pub struct FontChange {
    pub font:Arc<Font>,
    pub closes_with_group:bool,
    pub children:Vec<Whatsit>,
    pub sourceref:Option<SourceFileReference>
}
impl WhatsitTrait for FontChange {
    fn normalize(self, mode: &ColonMode, ret: &mut Vec<Whatsit>, scale: Option<f32>) {
        let mut ng = self.new_from();
        let mut in_ink = false;
        for c in self.children {
            if c.has_ink() { in_ink = true }
            if in_ink { c.normalize(mode, ng.children_mut(), scale) } else { c.normalize(mode, ret, scale) }
        }
        let mut nret : Vec<Whatsit> = vec!();
        loop {
            match ng.children.pop() {
                None => break,
                Some(w) if !w.has_ink() => nret.push(w),
                Some(o) => {
                    ng.children_mut().push(o);
                    break
                }
            }
        }
        if !ng.children().is_empty() {
            if ng.children.len() == 1 {
                match ng.children.pop().unwrap() {
                    o@Whatsit::Grouped(WIGroup::FontChange(_)) => ret.push(o),
                    o => {
                        ng.children.push(o);
                        ret.push(Whatsit::Grouped(WIGroup::FontChange(ng)))
                    }
                }
            } else {
                ret.push(Whatsit::Grouped(WIGroup::FontChange(ng)))
            }
        }
        nret.reverse();
        ret.append(&mut nret);
    }
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
    fn as_html(self, mode: &ColonMode, colon: &mut HTMLColon, node_top: &mut Option<HTMLParent>) {
        match &self.font.file.chartable {
            Some(ft) => {
                htmlannotate!(colon,span,self.sourceref,node_top,a => {
                    a.attr("rustex:font".into(),self.font.file.name.clone().into());
                    for prop in &ft.params {
                        match prop {
                            FontTableParam::Text | FontTableParam::Math | FontTableParam::CapitalLetters => (),
                            FontTableParam::SansSerif => a.style("font-family".into(),"sans-serif".into()),
                            FontTableParam::Italic => a.style("font-style".into(),"italic".into()),
                            FontTableParam::Bold => a.style("font-weight".into(),"bold".into()),
                            FontTableParam::Script => a.style("font-family".into(),"eusb".into()),
                            FontTableParam::Capital => a.style("font-variant".into(),"small-caps".into()),
                            FontTableParam::Monospaced => a.style("font-family".into(),"monospace".into()),
                            FontTableParam::Blackboard => a.style("font-family".into(),"msbm".into()),
                                // ret ::= ("mathvariant","double-struck")
                            FontTableParam::Fraktur => todo!()
                        }
                    }
                    let _oldsize = colon.state.currsize;
                    match self.font.at {
                        Some(at) if at != colon.state.currsize => {
                            let atstr = 100.0 * (at as f32) / (colon.state.currsize as f32);
                            a.style("font-size".into(),(atstr.to_string() + "%").into());
                            colon.state.currsize = at;
                        }
                        _ => ()
                    }
                    for c in self.children {
                        c.as_html(mode,colon,htmlparent!(a))
                    }
                    colon.state.currsize = _oldsize;
                })
            }
            _ => {
                for c in self.children { c.as_html(mode,colon,node_top) }
            }
        }
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
    fn normalize(self, mode: &ColonMode, ret: &mut Vec<Whatsit>, scale: Option<f32>) {
        let mut ng = self.new_from();
        let mut in_ink = false;
        for c in self.children {
            if c.has_ink() { in_ink = true }
            if in_ink { c.normalize(mode, ng.children_mut(), scale) } else { c.normalize(mode, ret, scale) }
        }
        let mut nret : Vec<Whatsit> = vec!();
        loop {
            match ng.children.pop() {
                None => break,
                Some(w) if !w.has_ink() => nret.push(w),
                Some(o) => {
                    ng.children.push(o);
                    break
                }
            }
        }
        if !ng.children.is_empty() {
            if ng.children.len() == 1 {
                match ng.children.pop().unwrap() {
                    Whatsit::Grouped(WIGroup::FontChange(mut fc)) => {
                        if fc.children.len() == 1 {
                            match fc.children.last().unwrap() {
                                Whatsit::Grouped(WIGroup::ColorChange(_)) => {
                                    ret.push(Whatsit::Grouped(WIGroup::FontChange(fc)))
                                }
                                _ => {
                                    ng.children = std::mem::take(&mut fc.children);
                                    fc.children = vec!(Whatsit::Grouped(WIGroup::ColorChange(ng)));
                                    ret.push(Whatsit::Grouped(WIGroup::FontChange(fc)))
                                }
                            }
                        } else {
                            ng.children = std::mem::take(&mut fc.children);
                            fc.children = vec!(Whatsit::Grouped(WIGroup::ColorChange(ng)));
                            ret.push(Whatsit::Grouped(WIGroup::FontChange(fc)))
                        }
                    }
                    o@Whatsit::Grouped(WIGroup::ColorChange(_)) => ret.push(o),
                    o => {
                        ng.children.push(o);
                        ret.push(Whatsit::Grouped(WIGroup::ColorChange(ng)))
                    }
                }
            } else {
                ret.push(Whatsit::Grouped(WIGroup::ColorChange(ng)))
            }
        }
        nret.reverse();
        ret.append(&mut nret);
    }
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
    fn as_html(self, mode: &ColonMode, colon: &mut HTMLColon, node_top: &mut Option<HTMLParent>) {
        match mode {
            ColonMode::H | ColonMode::V => htmlannotate!(colon,span,self.sourceref,node_top,a => {
                let color : HTMLStr = ColorChange::color_to_html(self.color).into();
                let hashcolor : HTMLStr = "#".into();
                a.style("color".into(),hashcolor + &color);
                let _oldcolor = std::mem::take(&mut colon.state.currcolor);
                colon.state.currcolor = Some(color);
                for c in self.children { c.as_html(mode,colon,htmlparent!(a)) }
                colon.state.currcolor = _oldcolor;
            }),
            ColonMode::M => htmlannotate!(colon,mrow,self.sourceref,node_top,a => {
                let color : HTMLStr = ColorChange::color_to_html(self.color).into();
                let hashcolor : HTMLStr = "#".into();
                a.style("color".into(),hashcolor + &color);
                let _oldcolor = std::mem::take(&mut colon.state.currcolor);
                colon.state.currcolor = Some(color);
                for c in self.children { c.as_html(mode,colon,htmlparent!(a)) }
                colon.state.currcolor = _oldcolor;
            }),
            _ => for c in self.children { c.as_html(mode,colon,node_top) }
        }
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
    pub fn color_to_html(color:TeXStr) -> String {
        fn tostr(c:f32) -> String {
            let s = format!("{:X}", (c.round() as i32));
            if s.len() == 1 {"0".to_string() + &s} else { s }
        }
        let ls: Vec<String> = color.to_string().split(' ').map(|x| x.to_string()).collect();
        if ls.contains(&"k".to_string()) {
            let third = 1.0 - str::parse::<f32>(ls[3].as_str()).unwrap();
            let r = 255.0*(1.0 - str::parse::<f32>(ls[0].as_str()).unwrap()) * third;
            let g = 255.0*(1.0 - str::parse::<f32>(ls[1].as_str()).unwrap()) * third;
            let b = 255.0*(1.0 - str::parse::<f32>(ls[2].as_str()).unwrap()) * third;
            tostr(r) + &tostr(g) + &tostr(b)
        } else if ls.contains(&"rg".to_string()) {
            let r = 255.0 * str::parse::<f32>(ls[0].as_str()).unwrap();
            let g = 255.0 * str::parse::<f32>(ls[1].as_str()).unwrap();
            let b = 255.0 * str::parse::<f32>(ls[2].as_str()).unwrap();
            tostr(r) + &tostr(g) + &tostr(b)
        } else if ls.contains(&"g".to_string()) {
            let r = 255.0 * str::parse::<f32>(ls[0].as_str()).unwrap();
            tostr(r) + &tostr(r) + &tostr(r)
        } else {
            panic!("Malformed color string: {}",color)
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
    fn normalize(self, mode: &ColonMode, ret: &mut Vec<Whatsit>, scale: Option<f32>) {
        let mut ng = self.new_from();
        for c in self.children { c.normalize(mode, ng.children_mut(), scale) }
        ret.push(Whatsit::Grouped(WIGroup::PDFLink(ng)))
    }
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
    fn as_html(self, mode: &ColonMode, colon: &mut HTMLColon, node_top: &mut Option<HTMLParent>) {
        match mode {
            ColonMode::H | ColonMode::V => htmlnode!(colon,a,self.sourceref,"pdflink",node_top,a => {
                a.attr("href".into(),self.action.as_link().into());
                for c in self.children { c.as_html(mode,colon,htmlparent!(a)) }
            }),
            ColonMode::M => htmlannotate!(colon,mrow,self.sourceref,node_top,a => {
                a.attr("href".into(),self.action.as_link().into());
                for c in self.children { c.as_html(mode,colon,htmlparent!(a)) }
            }),
            _ => for c in self.children { c.as_html(mode,colon,node_top) }
        }
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
    fn normalize(mut self, mode: &ColonMode, ret: &mut Vec<Whatsit>, scale: Option<f32>) {
        let matrix = match self.children.iter().filter(|x| match x {
            Whatsit::Simple(SimpleWI::PDFMatrix(_)) => true,
            _ => false
        }).next() {
            Some(Whatsit::Simple(SimpleWI::PDFMatrix(g))) => g.clone(),
            _ => {
                for c in self.children {c.normalize(mode,ret,scale)}
                return
            }
        };
        let mut ng = self.new_from();
        let nch : Vec<Whatsit> = self.children.drain(..).filter(|x| match x {
            Whatsit::Simple(SimpleWI::PDFMatrix(_)) => false,
            _ => true
        }).collect();
        if matrix.scale == matrix.skewy && matrix.rotate == 0.0 && matrix.skewx == 0.0 {
            for c in nch { c.normalize(mode,ret,Some(matrix.scale)) }
        } else {
            ng.children.push(matrix.as_whatsit());
            for c in nch { c.normalize(mode, &mut ng.children,scale) }
            ret.push(Whatsit::Grouped(WIGroup::PDFMatrixSave(ng)))
        }
    }
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
    fn as_html(self, mode: &ColonMode, colon: &mut HTMLColon, node_top: &mut Option<HTMLParent>) {
        match self.children.iter().filter(|x| match x {
            Whatsit::Simple(SimpleWI::PDFMatrix(_)) => true,
            _ => false
        }).next() {
            Some(Whatsit::Simple(SimpleWI::PDFMatrix(matrix))) => {
                htmlnode!(colon,span,self.sourceref,"pdfmatrix",node_top,m => {
                    m.style("transform-origin".into(),"top left".into());
                    let mut tf : HTMLStr = "matrix(".into();
                    tf += matrix.scale.to_string();
                    tf += ",";
                    tf += matrix.rotate.to_string();
                    tf += ",";
                    tf += matrix.skewx.to_string();
                    tf += ",";
                    tf += matrix.skewy.to_string();
                    tf += ",0,0)";
                    m.style("transform".into(),tf);
                    for c in self.children {
                        c.as_html(mode,colon,htmlparent!(m))
                    }
                })
            }
            _ => {
                for c in self.children { c.as_html(mode,colon,node_top) }
            }
        }
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

pub trait ExternalWhatsitGroupEnd : Send + Sync {
    fn name(&self) -> TeXStr;
    fn params(&self,name:&str) -> Option<TeXStr>;
    fn priority(&self) -> i16;
    fn sourceref(&self) -> &Option<SourceFileReference>;
}

#[derive(Clone)]
pub enum GroupClose {
    PDFRestore(PDFRestore),
    LinkEnd(LinkEnd),
    ColorEnd(ColorEnd),
    EndGroup(EndGroup),
    External(Arc<dyn ExternalWhatsitGroupEnd>)
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
    fn normalize(self, _: &ColonMode, _: &mut Vec<Whatsit>, _: Option<f32>) {}
    fn as_html(self, _: &ColonMode, _: &mut HTMLColon, _: &mut Option<HTMLParent>) {}
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
            fn as_xml_internal(&self, _: String) -> String { "".to_string() }
            fn has_ink(&self) -> bool { false }
            fn normalize(self, _: &ColonMode, _: &mut Vec<Whatsit>, _: Option<f32>) {}
            fn as_html(self, _: &ColonMode, _: &mut HTMLColon, _: &mut Option<HTMLParent>) {}
        }
    )
}
groupclose!(PDFRestore,70);
groupclose!(LinkEnd,60);
groupclose!(ColorEnd,50);
groupclose!(EndGroup,25);