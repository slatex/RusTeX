use std::rc::Rc;
use std::sync::Arc;
use crate::commands::{PrimitiveExecutable, PrimitiveTeXCommand, ProvidesWhatsit, SimpleWhatsit};
use crate::{Interpreter, TeXErr};
use crate::interpreter::dimensions::numtostr;
use crate::references::SourceFileReference;
use crate::stomach::boxes::TeXBox;
use crate::stomach::groups::{ColorChange, ExternalWhatsitGroup, ExternalWhatsitGroupEnd, GroupClose, WIGroup};
use crate::stomach::simple::{ExternalParam, ExternalWhatsit, SimpleWI};
use crate::stomach::Whatsit;
use crate::stomach::whatsits::WhatsitTrait;
use crate::utils::{TeXError, TeXStr};

pub static PGFSYSDRIVER : PrimitiveExecutable = PrimitiveExecutable {
    expandable:true,
    name:"pgfsysdriver",
    _apply:|xp, int| {
        xp.2 = crate::interpreter::string_to_tokens("pgfsys-rust.def".into());
        Ok(())
    }
};

pub struct PGFColor {
    pub color:TeXStr,
    pub sourceref:Option<SourceFileReference>
}
unsafe impl Send for PGFColor {}
unsafe impl Sync for PGFColor {}
impl ExternalWhatsitGroup for PGFColor {
    fn name(&self) -> TeXStr { "pgfcolor".into() }
    fn params(&self, name: &str) -> Option<TeXStr> {
        if name == "color" {Some(self.color.clone())} else {None}
    }
    fn width(&self, ch: &Vec<Whatsit>) -> i32 { 0 }
    fn height(&self, ch: &Vec<Whatsit>) -> i32 { 0 }
    fn depth(&self, ch: &Vec<Whatsit>) -> i32 { 0 }
    fn as_xml_internal(&self, ch: &Vec<Whatsit>, prefix: String) -> String {
        let mut ret = "<g style=\"color:#".to_string() + &self.color.to_string() + "\">";
        for w in ch { ret += &w.as_xml_internal(prefix.clone())}
        ret + "</g>"
    }
    fn has_ink(&self, ch: &Vec<Whatsit>) -> bool { true }
    fn opaque(&self) -> bool { true }
    fn priority(&self) -> i16 { 75 }
    fn closes_with_group(&self) -> bool { false }
    fn sourceref(&self) -> &Option<SourceFileReference> { &self.sourceref }
}

pub static COLORPUSH : SimpleWhatsit = SimpleWhatsit {
    name:"rustex!pgf!colorpush",
    modes: |x| {true},
    _get: |tk, int| {
        let color = int.tokens_to_string(&int.read_balanced_argument(true,false,false,true)?);
        Ok(
            Whatsit::GroupOpen(WIGroup::External(Arc::new(PGFColor {
            color:ColorChange::as_html(color.into()).into(),
            sourceref:int.update_reference(tk)
        }),vec!())))
    },
};

pub struct PGFColorEnd {
    pub sourceref:Option<SourceFileReference>
}
unsafe impl Send for PGFColorEnd {}
impl ExternalWhatsitGroupEnd for PGFColorEnd {
    fn name(&self) -> TeXStr { "pgfcolor".into() }
    fn params(&self, _: &str) -> Option<TeXStr> { None }
    fn priority(&self) -> i16 { 75 }
    fn sourceref(&self) -> &Option<SourceFileReference> { &self.sourceref }
}

pub static COLORPOP : SimpleWhatsit = SimpleWhatsit {
    name:"rustex!pgf!colorpop",
    modes: |x| {true},
    _get: |tk, int| {
        Ok(Whatsit::GroupClose(GroupClose::External(Arc::new(PGFColorEnd { sourceref: int.update_reference(tk)}))))
    },
};

pub struct PGFEscape {
    pub sourceref:Option<SourceFileReference>,
    pub bx : TeXBox
}
impl WhatsitTrait for PGFEscape {
    fn has_ink(&self) -> bool { true }
    fn depth(&self) -> i32 { 0 }
    fn height(&self) -> i32 { 0 }
    fn width(&self) -> i32 { 0 }
    fn as_whatsit(self) -> Whatsit {
        Whatsit::Simple(SimpleWI::External(Arc::new(self)))
    }
    fn as_xml_internal(&self, prefix: String) -> String {
        "<escape>".to_string() + &self.bx.as_xml_internal(prefix) + "</escape>"
    }
}
unsafe impl Send for PGFEscape {}
unsafe impl Sync for PGFEscape {}
impl ExternalWhatsit for PGFEscape {
    fn name(&self) -> TeXStr { "pgfescape".into() }
    fn params(&self, name: &str) -> Option<ExternalParam> {
        if name == "box" {Some(ExternalParam::Whatsits(vec!(Whatsit::Box(self.bx.clone()))))} else {None}
    }
    fn sourceref(&self) -> &Option<SourceFileReference> { &self.sourceref }
}

pub static PGFHBOX: SimpleWhatsit = SimpleWhatsit {
    name:"rustex!pgf!hbox",
    modes: |x| {true},
    _get: |tk, int| {
        Ok(PGFEscape {
            bx:int.state_get_box(int.read_number()?),
            sourceref:int.update_reference(tk)
        }.as_whatsit())
    },
};

pub struct PGFBox {
    pub sourceref:Option<SourceFileReference>,
    pub content: Vec<Whatsit>,
    minx:i32,
    miny:i32,
    maxx:i32,
    maxy:i32
}
unsafe impl Send for PGFBox {}
impl WhatsitTrait for PGFBox {
    fn has_ink(&self) -> bool { true }
    fn depth(&self) -> i32 { 0 }
    fn height(&self) -> i32 { self.maxy - self.miny }
    fn width(&self) -> i32 { self.maxx - self.minx }
    fn as_whatsit(self) -> Whatsit {
        Whatsit::Simple(SimpleWI::External(Arc::new(self)))
    }
    fn as_xml_internal(&self, prefix: String) -> String {
        let mut str ="\n".to_string() + &prefix + "<svg xmlns=\"http://www.w3.org/2000/svg\"";
        str += &(" width=\"".to_string() + &numtostr(self.maxx - self.minx,"px") + "\"");
        str += &(" height=\"".to_string() + &numtostr(self.maxy - self.miny,"px") + "\"");
        str += &(" viewBox=\"".to_string() + &numtostr(self.minx,"") +
            " " + &numtostr(self.miny,"") +
            " " + &numtostr(self.maxx - self.minx,"") +
            " " + &numtostr(self.maxy - self.miny,"") +
            "\">");
        str += &("<g transform=\"translate(0,".to_string() + &numtostr(self.maxy,"") + ") scale=(1,-1) translate(0," +
            &numtostr(-self.miny,"") + ")\">");
        for s in &self.content {str += &s.as_xml_internal(prefix.clone() + "  ")}
        str + "\n" + &prefix + "</g></svg>"
    }
}
impl ExternalWhatsit for PGFBox {
    fn name(&self) -> TeXStr { "pgfbox".into() }
    fn params(&self, name: &str) -> Option<ExternalParam> {
        if name == "content" {Some(ExternalParam::Whatsits(self.content.clone()))}
        else if name == "minx" { Some(ExternalParam::Num(self.minx)) }
        else if name == "miny" { Some(ExternalParam::Num(self.miny)) }
        else if name == "maxx" { Some(ExternalParam::Num(self.maxx)) }
        else if name == "maxy" { Some(ExternalParam::Num(self.maxy)) }
        else { None }
    }
    fn sourceref(&self) -> &Option<SourceFileReference> { &self.sourceref }
}

pub fn get_dimen(s:&str,int:&Interpreter) -> Result<i32,TeXError> {
    use crate::commands::AssignableValue;
    let p = int.get_command(&s.into())?;
    match &*p.orig {
        PrimitiveTeXCommand::AV(AssignableValue::Dim(u)) => Ok(int.state_dimension(*u as i32)),
        PrimitiveTeXCommand::AV(AssignableValue::PrimDim(r)) => Ok(int.state_dimension(- (r.index as i32))),
        _ => TeXErr!((int,None),"Not a dimension: \\{}",s)
    }
}

pub static TYPESETPICTUREBOX: SimpleWhatsit = SimpleWhatsit {
    name:"rustex!pgf!typesetpicturebox",
    modes: |x| {true},
    _get: |tk, int| {
        Ok(PGFBox {
            content:int.state_get_box(int.read_number()?).children(),
            sourceref:int.update_reference(tk),
            minx:get_dimen("pgf@picminx",int)?,
            miny:get_dimen("pgf@picminy",int)?,
            maxx:get_dimen("pgf@picmaxx",int)?,
            maxy:get_dimen("pgf@picmaxy",int)?,
        }.as_whatsit())
    },
};

pub struct PGFLiteral {
    str : TeXStr
}
impl WhatsitTrait for PGFLiteral {
    fn as_whatsit(self) -> Whatsit { Whatsit::Simple(SimpleWI::External(Arc::new(self)))}
    fn width(&self) -> i32 { 0 }
    fn height(&self) -> i32 { 0 }
    fn depth(&self) -> i32 { 0 }
    fn has_ink(&self) -> bool { true }
    fn as_xml_internal(&self, prefix: String) -> String { self.str.to_string() }
}
impl ExternalWhatsit for PGFLiteral {
    fn name(&self) -> TeXStr { "pgfliteral".into() }
    fn params(&self, name: &str) -> Option<ExternalParam> { if name == "string" { Some(ExternalParam::String(self.str.clone())) } else { None } }
    fn sourceref(&self) -> &Option<SourceFileReference> { &None }
}

pub static PGFLITERAL : SimpleWhatsit = SimpleWhatsit {
    name: "rustex!pgf!literal",
    modes: |x| { true },
    _get: |tk, int| {
        let str = int.tokens_to_string(&int.read_balanced_argument(true,false,false,true)?);
        Ok(PGFLiteral { str:str.into() }.as_whatsit())
    },
};


// -------------------------------------------------------------------------------------------------

pub fn pgf_commands() -> Vec<PrimitiveTeXCommand> {vec![
    PrimitiveTeXCommand::Primitive(&PGFSYSDRIVER),
    PrimitiveTeXCommand::Whatsit(ProvidesWhatsit::Simple(&TYPESETPICTUREBOX)),
    PrimitiveTeXCommand::Whatsit(ProvidesWhatsit::Simple(&PGFLITERAL)),
    PrimitiveTeXCommand::Whatsit(ProvidesWhatsit::Simple(&COLORPUSH)),
    PrimitiveTeXCommand::Whatsit(ProvidesWhatsit::Simple(&COLORPOP)),
    PrimitiveTeXCommand::Whatsit(ProvidesWhatsit::Simple(&PGFHBOX))
]}