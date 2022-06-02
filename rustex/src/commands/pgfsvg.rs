use crate::commands::{PrimitiveExecutable, PrimitiveTeXCommand, ProvidesWhatsit, SimpleWhatsit};
use crate::{Interpreter, htmlliteral, htmlnode, TeXErr, htmlparent};
use crate::interpreter::dimensions::numtostr;
use crate::interpreter::TeXMode;
use crate::references::SourceFileReference;
use crate::stomach::boxes::TeXBox;
use crate::stomach::colon::ColonMode;
use crate::stomach::groups::WIGroupTrait;
use crate::stomach::html::{dimtohtml, HTML_NS, HTMLChild, HTMLColon, HTMLNode, HTMLParent, HTMLSCALE, HTMLStr, SVG_NS};
use crate::stomach::simple::{ExternalParam, ExternalWhatsit, SimpleWI};
use crate::stomach::Whatsit;
use crate::stomach::whatsits::WhatsitTrait;
use crate::utils::{TeXError, TeXStr};

pub static PGFSYSDRIVER : PrimitiveExecutable = PrimitiveExecutable {
    expandable:true,
    name:"pgfsysdriver",
    _apply:|xp, _| {
        xp.2 = crate::interpreter::string_to_tokens("pgfsys-rust.def".into());
        Ok(())
    }
};

#[derive(Clone)]
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
        Whatsit::Simple(SimpleWI::External(Box::new(self)))
    }
    fn as_xml_internal(&self, prefix: String) -> String {
        "<escape>".to_string() + &self.bx.as_xml_internal(prefix) + "</escape>"
    }
    fn normalize(self, _: &ColonMode, ret: &mut Vec<Whatsit>, scale: Option<f32>) {
        let mut nret : Vec<Whatsit> = vec!();
        let _ = self.bx.width();
        let _ = self.bx.height();
        let _ = self.bx.depth();
        match self.bx {
            TeXBox::H(hb) => {
                //hb._width = Some(width); hb._height = Some(height); hb._depth = Some(depth);
                hb.normalize(&ColonMode::V,&mut nret,scale)
            },
            TeXBox::V(vb) => {
                //vb._width = Some(width); vb._height = Some(height); vb._depth = Some(depth);
                vb.normalize(&ColonMode::H,&mut nret,scale)
            },
            TeXBox::Void => return ()
        }
        assert_eq!(nret.len(),1);
        match nret.pop() {
            Some(Whatsit::Box(bx)) => ret.push(PGFEscape { bx, sourceref:self.sourceref}.as_whatsit()),
            _ => unreachable!()
        }
    }
    fn as_html(self, mode: &ColonMode, colon: &mut HTMLColon, node_top: &mut Option<HTMLParent>) {
        match mode {
            ColonMode::External(s) if s.to_string() == "svg" => {
                htmlnode!(colon,foreignObject,self.sourceref,"",node_top,fo => {
                    let wd = self.bx.width();
                    let ht = self.bx.height() + self.bx.depth();
                    fo.style("width".into(),dimtohtml(wd));
                    fo.style("height".into(),dimtohtml(ht));
                    htmlnode!(colon,HTML_NS:div,None,"foreign",htmlparent!(fo),div => {
                        self.bx.as_html(&ColonMode::H,colon,htmlparent!(div))
                    })
                })
            }
            _ => self.bx.as_html(mode,colon,node_top)
        }
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

pub static PGFHBOX: SimpleWhatsit = SimpleWhatsit {
    name:"rustex!pgf!hbox",
    modes: |_| {true},
    _get: |tk, int| {
        let num = int.read_number()?;
        Ok(PGFEscape {
            bx:int.state.boxes.take(num),
            sourceref:int.update_reference(tk)
        }.as_whatsit())
    },
};

#[derive(Clone)]
pub struct PGFBox {
    pub sourceref:Option<SourceFileReference>,
    pub content: Vec<Whatsit>,
    minx:i32,
    miny:i32,
    maxx:i32,
    maxy:i32
}
unsafe impl Send for PGFBox {}
impl PGFBox {
    fn normalize_i(wi : Whatsit) -> Vec<Whatsit> {
        match wi {
            Whatsit::Box(tb) => tb.children().drain(..).map(PGFBox::normalize_i).flatten().collect(),
            Whatsit::Simple(SimpleWI::External(ext)) if ext.name().to_string() == "pgfescape" => {
                let mut nv : Vec<Whatsit> = vec!();
                ext.normalize_dyn(&ColonMode::H,&mut nv,None);
                nv
            }
           // Whatsit::Simple(SimpleWI::Hss(_)) => vec!(),
            //Whatsit::Simple(SimpleWI::External(ref ext)) if ext.name().to_string() == "pgfliteral" => { vec!(wi) }
            //Whatsit::Grouped(WIGroup::ColorChange(mut c)) => c.children.drain(..).map(PGFBox::normalize_i).flatten().collect(),
            //Whatsit::Space(_) => {vec!(wi) }
            Whatsit::Grouped(gr) => gr.children_prim().drain(..).map(PGFBox::normalize_i).flatten().collect(),
            o => {
                vec!(o)
            }
        }
    }
}
impl WhatsitTrait for PGFBox {
    fn normalize(mut self, _: &ColonMode, ret: &mut Vec<Whatsit>, _: Option<f32>) {
        let nc : Vec<Whatsit> = self.content.drain(..).map(|x| PGFBox::normalize_i(x)).flatten().collect();
        self.content = nc;
        ret.push(self.as_whatsit())
    }
    fn has_ink(&self) -> bool { true }
    fn depth(&self) -> i32 { 0 }
    fn height(&self) -> i32 { self.maxy - self.miny }
    fn width(&self) -> i32 { self.maxx - self.minx }
    fn as_whatsit(self) -> Whatsit {
        Whatsit::Simple(SimpleWI::External(Box::new(self)))
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
    fn as_html(self, _: &ColonMode, colon: &mut HTMLColon, node_top: &mut Option<HTMLParent>) {
        htmlnode!(colon,SVG_NS:svg,self.sourceref,"",node_top,svg => {
            let scale = |i : i32| -> HTMLStr {numtostr((HTMLSCALE * (i as f32)).round() as i32,"").into()};
            let mut vb : HTMLStr = (scale)(self.minx); //numtostr(HTMLSCALE * self.minx,"").into();
            vb += " ";
            vb += (scale)(self.miny);
            vb += " ";
            vb += (scale)(self.maxx - self.minx);//numtostr(HTMLSCALE * (self.maxx-self.minx),"");
            vb += " ";
            vb += (scale)(self.maxy - self.miny); //numtostr(HTMLSCALE * (self.maxy-self.miny),"");
            svg.attr("width".into(),dimtohtml(self.maxx-self.minx));
            svg.attr("height".into(),dimtohtml(self.maxy-self.miny));
            svg.attr("viewBox".into(),vb);
            htmlnode!(colon,g,None,"",htmlparent!(svg),g => {
                let mut tr : HTMLStr = "translate(0,".into();
                tr += (scale)(self.maxy);
                tr += ") scale(1,-1) translate(0,";
                tr += (scale)(-self.miny);
                tr += ")";
                g.attr("transform".into(),tr);
                for c in self.content {
                    c.as_html(&ColonMode::External("svg".into()),colon,htmlparent!(g))
                }
            })
        })
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

pub fn get_dimen(s:&str,int:&Interpreter) -> Result<i32,TeXError> {
    use crate::commands::AssignableValue;
    let p = int.get_command(&s.into())?;
    match &*p.orig {
        PrimitiveTeXCommand::AV(AssignableValue::Dim(u)) => Ok(int.state.dimensions.get(&(*u as i32))),
        PrimitiveTeXCommand::AV(AssignableValue::PrimDim(r)) => Ok(int.state.dimensions.get(&-(r.index as i32))),
        _ => TeXErr!("Not a dimension: \\{}",s)
    }
}

pub static TYPESETPICTUREBOX: SimpleWhatsit = SimpleWhatsit {
    name:"rustex!pgf!typesetpicturebox",
    modes: |x| {x == TeXMode::Horizontal || x == TeXMode::RestrictedHorizontal},
    _get: |tk, int| {
        let num = int.read_number()?;
        Ok(PGFBox {
            content:int.state.boxes.take(num).children(),
            sourceref:int.update_reference(tk),
            minx:get_dimen("pgf@picminx",int)?,
            miny:get_dimen("pgf@picminy",int)?,
            maxx:get_dimen("pgf@picmaxx",int)?,
            maxy:get_dimen("pgf@picmaxy",int)?,
        }.as_whatsit())
    },
};

#[derive(PartialEq,Clone)]
pub struct PGFLiteral {
    str : TeXStr
}
impl WhatsitTrait for PGFLiteral {
    fn normalize(self, _: &ColonMode, ret: &mut Vec<Whatsit>, _: Option<f32>) {
        ret.push(self.as_whatsit())
    }
    fn as_whatsit(self) -> Whatsit { Whatsit::Simple(SimpleWI::External(Box::new(self)))}
    fn width(&self) -> i32 { 0 }
    fn height(&self) -> i32 { 0 }
    fn depth(&self) -> i32 { 0 }
    fn has_ink(&self) -> bool { true }
    fn as_xml_internal(&self, _: String) -> String { self.str.to_string() }
    fn as_html(self, mode: &ColonMode, colon: &mut HTMLColon, node_top: &mut Option<HTMLParent>) {
        match mode {
            ColonMode::External(s) if s.to_string()=="svg" => {
                htmlliteral!(colon,node_top,self.str)
            }
            _ => ()
        }
    }
}
impl ExternalWhatsit for PGFLiteral {
    fn name(&self) -> TeXStr { "pgfliteral".into() }
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

pub static PGFLITERAL : SimpleWhatsit = SimpleWhatsit {
    name: "rustex!pgf!literal",
    modes: |_| { true },
    _get: |_, int| {
        let tks = int.read_balanced_argument(true,false,false,true)?;
        let str = int.tokens_to_string(&tks);
        Ok(PGFLiteral { str:str.into() }.as_whatsit())
    },
};


// -------------------------------------------------------------------------------------------------

pub fn pgf_commands() -> Vec<PrimitiveTeXCommand> {vec![
    PrimitiveTeXCommand::Primitive(&PGFSYSDRIVER),
    PrimitiveTeXCommand::Whatsit(ProvidesWhatsit::Simple(&TYPESETPICTUREBOX)),
    PrimitiveTeXCommand::Whatsit(ProvidesWhatsit::Simple(&PGFLITERAL)),
    // PrimitiveTeXCommand::Whatsit(ProvidesWhatsit::Simple(&COLORPUSH)),
    // PrimitiveTeXCommand::Whatsit(ProvidesWhatsit::Simple(&COLORPOP)),
    PrimitiveTeXCommand::Whatsit(ProvidesWhatsit::Simple(&PGFHBOX))
]}