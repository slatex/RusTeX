use std::collections::HashMap;
use std::sync::Arc;
use crate::commands::{Conditional, PrimitiveExecutable, PrimitiveTeXCommand, ProvidesWhatsit, SimpleWhatsit};
use crate::commands::conditionals::dotrue;
use crate::{htmlannotate, htmlliteral, htmlparent, TeXErr};
use crate::references::SourceFileReference;
use crate::stomach::colon::ColonMode;
use crate::stomach::groups::{ExternalWhatsitGroup, ExternalWhatsitGroupEnd, GroupClose, WIGroup};
use crate::stomach::html::{HTMLAnnotation, HTMLChild, HTMLColon, HTMLParent, HTMLStr};
use crate::stomach::simple::{ExternalParam, ExternalWhatsit, SimpleWI};
use crate::stomach::Whatsit;
use crate::stomach::whatsits::WhatsitTrait;
use crate::utils::TeXStr;


#[derive(PartialEq,Clone)]
pub struct HTMLLiteral {
    pub(crate) str : TeXStr
}
impl WhatsitTrait for HTMLLiteral {
    fn get_ref(&self) -> Option<SourceFileReference> { None }
    fn normalize(self, _: &ColonMode, ret: &mut Vec<Whatsit>, _: Option<f32>) {
        ret.push(self.as_whatsit())
    }
    fn as_whatsit(self) -> Whatsit { Whatsit::Simple(SimpleWI::External(Box::new(self)))}
    fn width(&self) -> i32 { 0 }
    fn height(&self) -> i32 { 0 }
    fn depth(&self) -> i32 { 0 }
    fn has_ink(&self) -> bool { true }
    fn as_xml_internal(&self, _: String) -> String { self.str.to_string() }
    fn as_html(self, _: &ColonMode, colon: &mut HTMLColon, node_top: &mut Option<HTMLParent>) {
        htmlliteral!(colon,node_top,self.str)
    }
}
impl ExternalWhatsit for HTMLLiteral {
    fn name(&self) -> TeXStr { "htmlliteral".into() }
    fn params(&self, name: &str) -> Option<ExternalParam> { if name == "string" { Some(ExternalParam::String(self.str.clone())) } else { None } }
    fn sourceref(&self) -> &Option<SourceFileReference> { &None }
    fn clone_box(&self) -> Box<dyn ExternalWhatsit> {
        Box::new(self.clone())
    }
    fn normalize_dyn(self:Box<Self>, mode: &ColonMode, ret: &mut Vec<Whatsit>, scale: Option<f32>) {
        self.clone().normalize(mode,ret,scale)
    }
    fn as_html_dyn(self:Box<Self>,mode:&ColonMode,colon:&mut HTMLColon,node_top:&mut Option<HTMLParent>) {
        self.clone().as_html(mode,colon,node_top)
    }
}

#[derive(PartialEq,Clone)]
pub struct HTMLNamespace {
    abbr:TeXStr,
    ns : TeXStr
}
impl WhatsitTrait for HTMLNamespace {
    fn get_ref(&self) -> Option<SourceFileReference> { None }
    fn normalize(self, _: &ColonMode, ret: &mut Vec<Whatsit>, _: Option<f32>) {
        ret.push(self.as_whatsit())
    }
    fn as_whatsit(self) -> Whatsit { Whatsit::Simple(SimpleWI::External(Box::new(self)))}
    fn width(&self) -> i32 { 0 }
    fn height(&self) -> i32 { 0 }
    fn depth(&self) -> i32 { 0 }
    fn has_ink(&self) -> bool { true }
    fn as_xml_internal(&self, _: String) -> String {
        "<namespace abbr=\"".to_string() + &self.abbr.to_string() + "\" target=\"" + &self.ns.to_string() + "\"/>"
    }
    fn as_html(self, _: &ColonMode, colon: &mut HTMLColon, _: &mut Option<HTMLParent>) {
        colon.namespaces.insert(self.abbr.to_string(),self.ns.to_string());
    }
}
impl ExternalWhatsit for HTMLNamespace {
    fn name(&self) -> TeXStr { "htmlnamespace".into() }
    fn params(&self, name: &str) -> Option<ExternalParam> {
        if name == "abbr" { Some(ExternalParam::String(self.abbr.clone())) }
        else if name == "ns" { Some(ExternalParam::String(self.ns.clone())) }
        else { None } }
    fn sourceref(&self) -> &Option<SourceFileReference> { &None }
    fn clone_box(&self) -> Box<dyn ExternalWhatsit> {
        Box::new(self.clone())
    }
    fn normalize_dyn(self:Box<Self>, mode: &ColonMode, ret: &mut Vec<Whatsit>, scale: Option<f32>) {
        self.clone().normalize(mode,ret,scale)
    }
    fn as_html_dyn(self:Box<Self>,mode:&ColonMode,colon:&mut HTMLColon,node_top:&mut Option<HTMLParent>) {
        self.clone().as_html(mode,colon,node_top)
    }
}

pub static IF_RUSTEX : Conditional = Conditional {
    name:"if@rustex",
    _apply: |int,cond,unless| {
        dotrue(int,cond,unless)
    }
};

pub static DIRECT_HTML: SimpleWhatsit = SimpleWhatsit {
    name: "rustex@directHTML",
    modes: |_| { true },
    _get: |_, int| {
        let tks = int.read_balanced_argument(true,false,false,true)?;
        let str = int.tokens_to_string(&tks);
        Ok(HTMLLiteral { str:str.into() }.as_whatsit())
    },
};

pub static NAMESPACE: SimpleWhatsit = SimpleWhatsit {
    name:"rustex@addNamespaceAbbrev",
    modes: |_| { true },
    _get: |_, int| {
        let mut tks = int.read_balanced_argument(true,false,false,true)?;
        let abbr = int.tokens_to_string(&tks);
        tks = int.read_balanced_argument(true,false,false,true)?;
        let ns = int.tokens_to_string(&tks);
        Ok(HTMLNamespace { abbr:abbr.into(),ns:ns.into() }.as_whatsit())
    }
};

#[derive(PartialEq,Clone)]
struct AnnotateBegin {
    sourceref:Option<SourceFileReference>,
    attrs:HashMap<String,String>,
    styles:HashMap<String,String>
}
impl ExternalWhatsitGroup for AnnotateBegin {
    fn get_ref(&self,ch : &Vec<Whatsit>) -> Option<SourceFileReference> {
        SourceFileReference::from_wi_list(ch).or(self.sourceref.clone())
    }
    fn name(&self) -> TeXStr { "HTMLannotate".into() }
    fn params(&self,s:&str) -> Option<TeXStr> { match self.attrs.get(s) {
        Some(x) => Some(x.clone().into()),
        _ => None
    } }
    fn width(&self,_:&Vec<Whatsit>) -> i32 { 0 }
    fn height(&self,_:&Vec<Whatsit>) -> i32 { 0 }
    fn depth(&self,_:&Vec<Whatsit>) -> i32 { 0 }
    fn has_ink(&self,ch:&Vec<Whatsit>) -> bool {
        for c in ch {
            if c.has_ink() {return true }
        }
        false
    }
    fn opaque(&self) -> bool { false }
    fn priority(&self) -> i16 { 95 }
    fn closes_with_group(&self) -> bool { false }
    fn sourceref(&self) -> &Option<SourceFileReference> { &self.sourceref }
    fn as_xml_internal(&self,_:&Vec<Whatsit>, _: String) -> String {
        "".to_string()
    }
    fn normalize(&self,ch:Vec<Whatsit>, mode: &ColonMode, ret: &mut Vec<Whatsit>, scale: Option<f32>) {
        let mut nret : Vec<Whatsit> = vec!();
        for c in ch { c.normalize(mode,&mut nret,scale) }
        ret.push(Whatsit::Grouped(WIGroup::External(Arc::new(self.clone()),nret)))
    }
    fn as_html(&self,ch:Vec<Whatsit>, mode: &ColonMode, colon:&mut HTMLColon, node_top: &mut Option<HTMLParent>) {
        match mode {
            ColonMode::H | ColonMode::V | ColonMode::P => htmlannotate!(colon,span,self.get_ref(&ch),node_top,a => {
                for (k,v) in &self.attrs {
                    a.attr(k.into(),v.into())
                }
                for (k,v) in &self.styles {
                    a.style(k.into(),v.into())
                }
                for c in ch {
                    c.as_html(mode,colon,htmlparent!(a))
                }
            }),
            ColonMode::M => htmlannotate!(colon,mrow,self.get_ref(&ch),node_top,a => {
                for (k,v) in &self.attrs {
                    a.attr(k.into(),v.into())
                }
                for (k,v) in &self.styles {
                    a.style(k.into(),v.into())
                }
                for c in ch {
                    c.as_html(mode,colon,htmlparent!(a))
                }
            }),
            _ => () //TeXErr!("TODO")
        }
    }
}
#[derive(PartialEq,Clone)]
struct AnnotateEnd {}
impl ExternalWhatsitGroupEnd for AnnotateEnd {
    fn name(&self) -> TeXStr { "HTMLannotateEnd".into() }
    fn params(&self,_:&str) -> Option<TeXStr> {None}
    fn priority(&self) -> i16 { 95 }
    fn sourceref(&self) -> &Option<SourceFileReference> {&None}
}


pub static ANNOTATE_BEGIN: SimpleWhatsit = SimpleWhatsit {
    name: "rustex@annotateHTML",
    modes: |_| { true },
    _get: |tk, int| {
        let tks = int.read_balanced_argument(true,false,false,true)?;
        let str = int.tokens_to_string(&tks).to_string().trim().to_string();
        let mut annotate = AnnotateBegin {sourceref:int.update_reference(tk),attrs:HashMap::new(),styles:HashMap::new()};
        let mut index = 0;
        'outer: loop {
            if str.as_bytes().get(index).is_none() { break }
            let mut attr : Vec<u8> = vec!();
            let mut isstyle = false;
            loop {
                match str.as_bytes().get(index) {
                    None => break 'outer,
                    Some(58) /* : */ if attr == vec!(115,116,121,108,101) => {
                        index += 1;
                        isstyle = true;
                        attr = vec!()
                    }
                    Some(61) /* = */ => {
                        index += 1;
                        match str.as_bytes().get(index) {
                            Some(34) /* " */ => {
                                index += 1;
                                break
                            }
                            _ => TeXErr!("Expected \" after = in \\rustex@annotateHTML")
                        }
                    }
                    Some(32) if attr.is_empty() => index += 1,
                    Some(o) => {
                        attr.push(*o);
                        index += 1
                    }
                }
            }
            let mut value : Vec<u8> = vec!();
            loop {
                match str.as_bytes().get(index) {
                    None => break 'outer,
                    Some(34) => {
                        index +=1;
                        break
                    }
                    Some(o) => {
                        value.push(*o);
                        index += 1
                    }
                }
            }
            if isstyle {
                annotate.styles.insert(std::str::from_utf8(attr.as_slice()).unwrap().to_string(),std::str::from_utf8(value.as_slice()).unwrap().to_string());
            } else {
                annotate.attrs.insert(std::str::from_utf8(attr.as_slice()).unwrap().to_string(),std::str::from_utf8(value.as_slice()).unwrap().to_string());
            }
        }
        Ok(Whatsit::GroupOpen(WIGroup::External(Arc::new(annotate),vec!())))
    },
};

pub static ANNOTATE_END: SimpleWhatsit = SimpleWhatsit {
    name: "rustex@annotateHTMLEnd",
    modes: |_| { true },
    _get: |_, _| {
        Ok(Whatsit::GroupClose(GroupClose::External(Arc::new(AnnotateEnd {}))))
    },
};

pub static BREAK: PrimitiveExecutable = PrimitiveExecutable {
    _apply: |_,int| {
        let prev = int.preview();
        unsafe {crate::LOG = true}
        println!("BREAK! {}",prev);
        Ok(())
    },
    expandable: true,
    name: "rustexBREAK"
};




pub fn rustex_special_commands() -> Vec<PrimitiveTeXCommand> {vec![
    PrimitiveTeXCommand::Cond(&IF_RUSTEX),
    PrimitiveTeXCommand::Whatsit(ProvidesWhatsit::Simple(&DIRECT_HTML)),
    PrimitiveTeXCommand::Primitive(&BREAK),
    PrimitiveTeXCommand::Whatsit(ProvidesWhatsit::Simple(&NAMESPACE)),
    PrimitiveTeXCommand::Whatsit(ProvidesWhatsit::Simple(&ANNOTATE_BEGIN)),
    PrimitiveTeXCommand::Whatsit(ProvidesWhatsit::Simple(&ANNOTATE_END)),
]}