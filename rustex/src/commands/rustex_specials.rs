use std::collections::HashMap;
use std::sync::Arc;
use crate::commands::{Conditional, NumericCommand, PrimitiveExecutable, PrimitiveTeXCommand, ProvidesWhatsit, SimpleWhatsit};
use crate::commands::conditionals::dotrue;
use crate::{htmlannotate, htmlliteral, htmlnode, htmlparent, log, TeXErr};
use crate::interpreter::dimensions::{Numeric, pt};
use crate::references::SourceFileReference;
use crate::stomach::colon::ColonMode;
use crate::stomach::groups::{ExternalWhatsitGroup, ExternalWhatsitGroupEnd, GroupClose, WIGroup};
use crate::stomach::html::{dimtohtml, HTMLAnnotation, HTMLChild, HTMLColon, HTMLNode, HTMLParent, HTMLSCALE, HTMLStr};
use crate::stomach::simple::{ExternalParam, ExternalWhatsit, SimpleWI};
use crate::stomach::Whatsit;
use crate::stomach::whatsits::WhatsitTrait;
use crate::utils::TeXStr;


#[derive(PartialEq,Clone)]
pub struct HTMLLiteral {
    pub(crate) str : TeXStr
}
impl WhatsitTrait for HTMLLiteral {
    fn as_whatsit(self) -> Whatsit { Whatsit::Simple(SimpleWI::External(Box::new(self)))}
    fn width(&self) -> i32 { 0 }
    fn height(&self) -> i32 { 0 }
    fn depth(&self) -> i32 { 0 }
    fn as_xml_internal(&self, _: String) -> String { self.str.to_string() }
    fn has_ink(&self) -> bool { true }
    fn normalize(self, _: &ColonMode, ret: &mut Vec<Whatsit>, _: Option<f32>) {
        ret.push(self.as_whatsit())
    }
    fn as_html(self, _: &ColonMode, colon: &mut HTMLColon, node_top: &mut Option<HTMLParent>) {
        htmlliteral!(colon,node_top,self.str)
    }
    fn get_ref(&self) -> Option<SourceFileReference> { None }
    //fn get_par_width(&self) -> Option<i32> { None }
    //fn get_par_widths(&self) -> Vec<i32> { vec!() }
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
    fn as_whatsit(self) -> Whatsit { Whatsit::Simple(SimpleWI::External(Box::new(self)))}
    fn width(&self) -> i32 { 0 }
    fn height(&self) -> i32 { 0 }
    fn depth(&self) -> i32 { 0 }
    fn as_xml_internal(&self, _: String) -> String {
        "<namespace abbr=\"".to_string() + &self.abbr.to_string() + "\" target=\"" + &self.ns.to_string() + "\"/>"
    }
    fn has_ink(&self) -> bool { true }
    fn normalize(self, _: &ColonMode, ret: &mut Vec<Whatsit>, _: Option<f32>) {
        ret.push(self.as_whatsit())
    }
    fn as_html(self, _: &ColonMode, colon: &mut HTMLColon, _: &mut Option<HTMLParent>) {
        colon.namespaces.insert(self.abbr.to_string(),self.ns.to_string());
    }
    fn get_ref(&self) -> Option<SourceFileReference> { None }
    //fn get_par_width(&self) -> Option<i32> { None }
    //fn get_par_widths(&self) -> Vec<i32> { vec!() }
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
pub enum Sized {
    None,
    Plus(i32),
    Minus(i32),
    Times(f32)
}

#[derive(PartialEq,Clone)]
pub struct AnnotateBegin {
    pub sourceref:Option<SourceFileReference>,
    pub attrs:HashMap<String,String>,
    pub styles:HashMap<String,String>,
    pub classes:Vec<String>,
    pub block:bool,pub sized:Sized
}
impl ExternalWhatsitGroup for AnnotateBegin {
    fn get_ref(&self,ch : &Vec<Whatsit>) -> Option<SourceFileReference> {
        match &self.sourceref {
            Some(cnt) => Some(cnt.clone()),
            None => SourceFileReference::from_wi_list(ch)
        }//SourceFileReference::from_wi_list(ch).or(self.sourceref.clone())
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
    fn as_xml_internal(&self,chs:&Vec<Whatsit>, prefix: String) -> String {
        let mut ret = "\n".to_string() + &prefix + "<annotate ";
        for (k,v) in &self.attrs {
            ret += k;
            ret += "=\"";
            ret += v;
            ret += "\" ";
        }
        for (k,v) in &self.styles {
            ret += "style/";
            ret += k;
            ret += "=\"";
            ret += v;
            ret += "\" ";
        }
        ret += ">";
        for c in chs {
            ret += &c.as_xml_internal(prefix.clone() + "  ")
        }
        ret + "\n" + &prefix + "</annotate>"
    }
    fn normalize(&self,ch:Vec<Whatsit>, mode: &ColonMode, ret: &mut Vec<Whatsit>, scale: Option<f32>) {
        let mut nret : Vec<Whatsit> = vec!();
        for c in ch { c.normalize(mode,&mut nret,scale) }
        ret.push(Whatsit::Grouped(WIGroup::External(Arc::new(self.clone()),nret)))
    }
    fn as_html(&self,ch:Vec<Whatsit>, mode: &ColonMode, colon:&mut HTMLColon, node_top: &mut Option<HTMLParent>) {
        //println!("-----------------------------------------------------------------------------\n\n{}",self.as_xml_internal(&ch,"".to_string()));
        match mode {
            ColonMode::H | ColonMode::V | ColonMode::P if self.block => htmlnode!(colon,div,self.get_ref(&ch),"",node_top,d => {
                for (k,v) in &self.attrs {
                    d.attr(k.into(),v.into())
                }
                for (k,v) in &self.styles {
                    d.style(k.into(),v.into())
                }
                for c in &self.classes {
                    d.classes.push(c.into());
                }
                let str : Option<HTMLStr> = match self.sized {
                    Sized::None => None,
                    Sized::Plus(i) => Some(<&str as Into<HTMLStr>>::into("calc(var(--document-width) + ") + dimtohtml(i) + ")"),
                    Sized::Minus(i) => Some(<&str as Into<HTMLStr>>::into("calc(var(--document-width) - ") + dimtohtml(i) + ")"),
                    Sized::Times(f) => Some(<&str as Into<HTMLStr>>::into("calc(var(--document-width) * ") + f.to_string().as_str() + ")")
                };
                htmlnode!(colon,div,None,"rustex-withwidth",htmlparent!(d),d2 => {
                    if let Some(str) = str { d2.style("--temp-width".into(),str) }
                    htmlnode!(colon,span,None,"rustex-contents",htmlparent!(d2),inner => {
                        for c in ch { c.as_html(mode,colon,htmlparent!(inner))}
                    });
                });
            }),
            ColonMode::H | ColonMode::V | ColonMode::P => htmlannotate!(colon,span,self.get_ref(&ch),node_top,a => {
                for (k,v) in &self.attrs {
                    a.attr(k.into(),v.into())
                }
                for (k,v) in &self.styles {
                    a.style(k.into(),v.into())
                }
                for c in &self.classes {
                    a.classes.push(c.into());
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
                for c in &self.classes {
                    a.classes.push(c.into());
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
        let tks = int.read_balanced_argument(true,false,false,false)?;
        let str = int.tokens_to_string(&tks).to_string().trim().to_string();
        let mut annotate = AnnotateBegin {sourceref:int.update_reference(tk),attrs:HashMap::new(),styles:HashMap::new(),classes:vec!(),block:false,sized:Sized::None};
        let mut index = 0;
        'outer: loop {
            if str.as_bytes().get(index).is_none() { break }
            let mut attr : Vec<u8> = vec!();
            let mut isstyle = false;
            let mut isclass = false;
            loop {
                match str.as_bytes().get(index) {
                    None => break 'outer,
                    Some(58) /* : */ if attr == vec!(115,116,121,108,101) => {
                        index += 1;
                        isstyle = true;
                        attr = vec!()
                    }
                    Some(61) if isstyle && attr == vec!(99,108,97,115,115) => {
                        index += 1;
                        isclass = true;
                        match str.as_bytes().get(index) {
                            Some(34) /* " */ => {
                                index += 1;
                                break
                            }
                            _ => TeXErr!("Expected \" after = in \\rustex@annotateHTML")
                        }
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
                    Some(32) => index += 1,
                    Some(o) => {
                        attr.push(*o);
                        index += 1
                    }
                }
            }
            if attr== vec!(114,117,115,116,101,120,58,115,105,122,101,100) { // rustex:sized
                //while *str.as_bytes().get(index) == Some(32) { index += 1}
                let sign = str.as_bytes().get(index);
                index += 1;
                let mut value: Vec<u8> = vec!();
                loop {
                    match str.as_bytes().get(index) {
                        None => TeXErr!("Expected value after rustex:sized= in \\rustex@annotateHTML"),
                        Some(34) => {
                            index += 1;
                            break
                        }
                        Some(o) => {
                            value.push(*o);
                            index += 1
                        }
                    }
                }
                let num = std::str::from_utf8(value.as_slice()).unwrap();
                match (sign,num.parse::<f64>()) {
                    (Some(43),Ok(f)) => annotate.sized = Sized::Plus((pt(f) / (HTMLSCALE as f64)).round() as i32),
                    (Some(45),Ok(f)) => annotate.sized = Sized::Minus((pt(f) / (HTMLSCALE as f64)).round() as i32),
                    (Some(42),Ok(f)) => annotate.sized = Sized::Times(f as f32),
                    (_,Err(_)) => TeXErr!("Expected numeric value after rustex:sized= in \\rustex@annotateHTML"),
                    (_,_) => TeXErr!("Expected '+','-' or '*' after rustex:sized= in \\rustex@annotateHTML")
                }
            }
            else {
                let mut value: Vec<u8> = vec!();
                loop {
                    match str.as_bytes().get(index) {
                        None => break 'outer,
                        Some(34) => {
                            index += 1;
                            break
                        }
                        Some(o) => {
                            value.push(*o);
                            index += 1
                        }
                    }
                }
                if attr == vec!(114,117,115,116,101,120,58,98,108,111,99,107) { // rustex:block
                    if value == vec!(116,114,117,101) { annotate.block = true}
                } else if isclass {
                    annotate.classes.push(std::str::from_utf8(value.as_slice()).unwrap().to_string());
                } else if isstyle {
                    annotate.styles.insert(std::str::from_utf8(attr.as_slice()).unwrap().to_string(), std::str::from_utf8(value.as_slice()).unwrap().to_string());
                } else {
                    let key = std::str::from_utf8(attr.as_slice()).unwrap().to_string();
                    annotate.attrs.insert(key, std::str::from_utf8(value.as_slice()).unwrap().to_string());
                }
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
        use std::io::{self, Write};
        let prev = int.preview();
        unsafe {crate::LOG = true}
        log!("BREAK! {}",prev);
        Ok(())
    },
    expandable: false,
    name: "rustexBREAK"
};

pub static SCALE:NumericCommand = NumericCommand {
    name: "rustex@scale",
    _getvalue: |_| {
        Ok(Numeric::Dim((HTMLSCALE * 65536.0) as i32))
    }
};

pub fn rustex_special_commands() -> Vec<PrimitiveTeXCommand> {vec![
    PrimitiveTeXCommand::Cond(&IF_RUSTEX),
    PrimitiveTeXCommand::Whatsit(ProvidesWhatsit::Simple(&DIRECT_HTML)),
    PrimitiveTeXCommand::Primitive(&BREAK),
    PrimitiveTeXCommand::Whatsit(ProvidesWhatsit::Simple(&NAMESPACE)),
    PrimitiveTeXCommand::Whatsit(ProvidesWhatsit::Simple(&ANNOTATE_BEGIN)),
    PrimitiveTeXCommand::Whatsit(ProvidesWhatsit::Simple(&ANNOTATE_END)),
    PrimitiveTeXCommand::Num(&SCALE),
]}