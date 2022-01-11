use crate::commands::{Conditional, PrimitiveExecutable, PrimitiveTeXCommand, ProvidesWhatsit, SimpleWhatsit};
use crate::commands::conditionals::dotrue;
use crate::references::SourceFileReference;
use crate::stomach::colon::ColonMode;
use crate::stomach::simple::{ExternalParam, ExternalWhatsit, SimpleWI};
use crate::stomach::Whatsit;
use crate::stomach::whatsits::WhatsitTrait;
use crate::utils::TeXStr;


#[derive(PartialEq,Clone)]
pub struct HTMLLiteral {
    str : TeXStr
}
impl WhatsitTrait for HTMLLiteral {
    fn normalize(mut self, _: &ColonMode, ret: &mut Vec<Whatsit>, _: Option<f32>) {
        ret.push(self.as_whatsit())
    }
    fn as_whatsit(self) -> Whatsit { Whatsit::Simple(SimpleWI::External(Box::new(self)))}
    fn width(&self) -> i32 { 0 }
    fn height(&self) -> i32 { 0 }
    fn depth(&self) -> i32 { 0 }
    fn has_ink(&self) -> bool { true }
    fn as_xml_internal(&self, prefix: String) -> String { self.str.to_string() }
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
}

#[derive(PartialEq,Clone)]
pub struct HTMLNamespace {
    abbr:TeXStr,
    ns : TeXStr
}
impl WhatsitTrait for HTMLNamespace {
    fn normalize(mut self, _: &ColonMode, ret: &mut Vec<Whatsit>, _: Option<f32>) {
        ret.push(self.as_whatsit())
    }
    fn as_whatsit(self) -> Whatsit { Whatsit::Simple(SimpleWI::External(Box::new(self)))}
    fn width(&self) -> i32 { 0 }
    fn height(&self) -> i32 { 0 }
    fn depth(&self) -> i32 { 0 }
    fn has_ink(&self) -> bool { true }
    fn as_xml_internal(&self, prefix: String) -> String {
        "<namespace abbr=\"".to_string() + &self.abbr.to_string() + "\" target=\"" + &self.ns.to_string() + "\"/>"
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
}

pub static IF_RUSTEX : Conditional = Conditional {
    name:"if@rustex",
    _apply: |int,cond,unless| {
        dotrue(int,cond,unless)
    }
};

pub static DIRECT_HTML: SimpleWhatsit = SimpleWhatsit {
    name: "rustex@directHTML",
    modes: |x| { true },
    _get: |tk, int| {
        let str = int.tokens_to_string(&int.read_balanced_argument(true,false,false,true)?);
        Ok(HTMLLiteral { str:str.into() }.as_whatsit())
    },
};

pub static NAMESPACE: SimpleWhatsit = SimpleWhatsit {
    name:"rustex@addNamespaceAbbrev",
    modes: |x| { true },
    _get: |tk, int| {
        let abbr = int.tokens_to_string(&int.read_balanced_argument(true,false,false,true)?);
        let ns = int.tokens_to_string(&int.read_balanced_argument(true,false,false,true)?);
        Ok(HTMLNamespace { abbr:abbr.into(),ns:ns.into() }.as_whatsit())
    }
};

pub static ANNOTATE_BEGIN: SimpleWhatsit = SimpleWhatsit {
    name: "rustex@annotateHTML",
    modes: |x| { true },
    _get: |tk, int| {
        let str = int.tokens_to_string(&int.read_balanced_argument(true,false,false,true)?);
        todo!()//Ok(PGFLiteral { str:str.into() }.as_whatsit())
    },
};

pub static ANNOTATE_END: SimpleWhatsit = SimpleWhatsit {
    name: "rustex@annotateHTMLEnd",
    modes: |x| { true },
    _get: |tk, int| {
        let str = int.tokens_to_string(&int.read_balanced_argument(true,false,false,true)?);
        todo!()//Ok(PGFLiteral { str:str.into() }.as_whatsit())
    },
};





pub fn rustex_special_commands() -> Vec<PrimitiveTeXCommand> {vec![
    PrimitiveTeXCommand::Cond(&IF_RUSTEX),
    PrimitiveTeXCommand::Whatsit(ProvidesWhatsit::Simple(&DIRECT_HTML)),
    //PrimitiveTeXCommand::Primitive(&NAMESPACE),
    PrimitiveTeXCommand::Whatsit(ProvidesWhatsit::Simple(&NAMESPACE)),
    PrimitiveTeXCommand::Whatsit(ProvidesWhatsit::Simple(&ANNOTATE_BEGIN)),
    PrimitiveTeXCommand::Whatsit(ProvidesWhatsit::Simple(&ANNOTATE_END)),
]}