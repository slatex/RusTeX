use crate::commands::{PrimitiveExecutable, PrimitiveTeXCommand, ProvidesWhatsit, SimpleWhatsit};
use crate::{Interpreter, htmlliteral, htmlnode, TeXErr, htmlparent, Token};
use crate::interpreter::dimensions::numtostr;
use crate::interpreter::TeXMode;
use crate::references::SourceFileReference;
use crate::stomach::boxes::TeXBox;
use crate::stomach::colon::ColonMode;
use crate::stomach::groups::{ExternalWhatsitGroup, ExternalWhatsitGroupEnd, WIGroup, WIGroupTrait};
use crate::stomach::html::{dimtohtml, HTML_NS, HTMLChild, HTMLColon, HTMLNode, HTMLParent, HTMLSCALE, HTMLStr, numtohtml, SVG_NS};
use crate::stomach::simple::{ExternalParam, ExternalWhatsit, SimpleWI};
use crate::stomach::Whatsit;
use crate::stomach::whatsits::WhatsitTrait;
use crate::utils::{TeXError, TeXStr};

static FIX : bool = true;

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
    fn get_ref(&self) -> Option<SourceFileReference> { self.sourceref.clone()}
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
        if nret.is_empty() {return ()}
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
                    if FIX {
                        fo.style("width".into(),dimtohtml(wd + 655360));
                        fo.style("height".into(),dimtohtml(ht + 655360));
                        fo.style("translate".into(),dimtohtml(-305834) + " " +
                            dimtohtml(-ht - 218453))
                    } else {
                        fo.style("width".into(),dimtohtml(wd));
                        fo.style("height".into(),dimtohtml(ht));
                    }
                    htmlnode!(colon,HTML_NS:div,None,"foreign",htmlparent!(fo),div => {
                        self.bx.as_html(&ColonMode::H,colon,htmlparent!(div))
                    })
                })
            }
            _ => self.bx.as_html(mode,colon,node_top)
        }
    }
}
//unsafe impl Send for PGFEscape {}
//unsafe impl Sync for PGFEscape {}
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
//unsafe impl Send for PGFBox {}
impl PGFBox {
    fn normalize_i(wi : Whatsit,ret:&mut Vec<Whatsit>) {
        match wi {
            Whatsit::Space(_) => (),
            Whatsit::Simple(SimpleWI::Hss(_)) => (),
            Whatsit::Simple(SimpleWI::External(ext)) if ext.name().to_string() == "pgfescape" => {
                ext.normalize_dyn(&ColonMode::H,ret,None);
            }
            Whatsit::Box(tb) => {
                for c in tb.children().drain(..) {
                    PGFBox::normalize_i(c,ret)
                }
            },
            Whatsit::Grouped(WIGroup::External(e,mut ch)) if e.name().to_string() == "PGFGbegin" => {
                let mut nret : Vec<Whatsit> = vec!();
                for c in ch.drain(..) {
                    PGFBox::normalize_i(c,&mut nret)
                }
                ret.push(WIGroup::External(e,nret).as_whatsit())
            }
            Whatsit::Grouped(gr) => {
                for c in gr.children_prim() {
                    PGFBox::normalize_i(c,ret)
                }
            },
            o => ret.push(o)
        }
        /*
        match wi {

            //Whatsit::Simple(SimpleWI::External(ref ext)) if ext.name().to_string() == "pgfliteral" => { vec!(wi) }
            //Whatsit::Grouped(WIGroup::ColorChange(mut c)) => c.children.drain(..).map(PGFBox::normalize_i).flatten().collect(),
            //Whatsit::Space(_) => {vec!(wi) }
            o => {
                vec!(o)
            }
        }
         */
    }
}
impl WhatsitTrait for PGFBox {
    fn get_ref(&self) -> Option<SourceFileReference> { self.sourceref.clone()}
    fn normalize(mut self, _: &ColonMode, ret: &mut Vec<Whatsit>, _: Option<f32>) {
        let mut nc : Vec<Whatsit> = vec!();
        for x in self.content.drain(..) {
            PGFBox::normalize_i(x,&mut nc)
        }
        //self.content.drain(..).map(|x| PGFBox::normalize_i(x)).flatten().collect();
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
        str += &("<g transform=\"translate(0,".to_string() + &numtostr(if FIX {self.maxy + self.miny} else {self.maxy},"") + ")");
        if !FIX {
            str += &("scale=(1,-1) translate(0,".to_string() + &numtostr(-self.miny,"") + ")\">");
        }
        for s in &self.content {str += &s.as_xml_internal(prefix.clone() + "  ")}
        str + "\n" + &prefix + "</g></svg>"
    }
    fn as_html(self, _: &ColonMode, colon: &mut HTMLColon, node_top: &mut Option<HTMLParent>) {
        htmlnode!(colon,HTML_NS:div,None,"",node_top,div => {
            div.style("display".into(),"block".into());
            let currsize = colon.state.currsize;
            colon.state.currsize = self.maxx - self.minx;
            div.style("--current-width".into(),dimtohtml(self.maxx-self.minx).into());
        htmlnode!(colon,SVG_NS:svg,self.sourceref,"",htmlparent!(div),svg => {
            let mut vb : HTMLStr = numtohtml(self.minx); //numtostr(HTMLSCALE * self.minx,"").into();
            vb += " ";
            vb += numtohtml(self.miny);
            vb += " ";
            vb += numtohtml(self.maxx - self.minx);//numtostr(HTMLSCALE * (self.maxx-self.minx),"");
            vb += " ";
            vb += numtohtml(self.maxy - self.miny); //numtostr(HTMLSCALE * (self.maxy-self.miny),"");
            svg.attr("width".into(),dimtohtml(self.maxx-self.minx));
            svg.attr("height".into(),dimtohtml(self.maxy-self.miny));
            svg.attr("viewBox".into(),vb);
            htmlnode!(colon,g,None,"",htmlparent!(svg),g => {
                if FIX {
                    let mut tr : HTMLStr = "translate(0,".into();
                    tr += numtohtml(self.maxy + self.miny);
                    tr += ")";
                    g.attr("transform".into(),tr);
                } else {
                    let mut tr : HTMLStr = "translate(0,".into();
                    tr += numtohtml(self.maxy);
                    tr += ") scale(1,-1) translate(0,";
                    tr += numtohtml(-self.miny);
                    tr += ")";
                    g.attr("transform".into(),tr);
                }
                for c in self.content {
                    c.as_html(&ColonMode::External("svg".into()),colon,htmlparent!(g))
                }
            })
        });
            colon.state.currsize = currsize;
        });
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
    modes: |x| {true},
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

pub static PGF_FLUSH : PrimitiveExecutable = PrimitiveExecutable {
    name: "rustex!pgf!flushpath",
    expandable: true,
    _apply: |rf,int| {
        let cmd = int.get_command(&("pgf@sys@svgpath".into()))?.orig;
        match &*cmd {
            PrimitiveTeXCommand::Def(dm) => {
                rf.2 = dm.ret.clone();
                let empty =  int.get_command(&("pgfutil@empty".into()))?;
                int.state.commands.set("pgf@sys@svgpath".into(),Some(empty),true)
            }
            _ => TeXErr!("\\pgf@sys@svgpath wrongly defined")
        }
        Ok(())
    }
};

// <g>...</g>
// <svg
// overflow="visible"
// preserveAspectRatio=""
// id=""
// x="" y="" width="" height=""
// viewBox=". . . ."
// >

// <clipPath id=""><path id="" d=""/></clipPath>
// <path id="" d="" marker-start="url()" marker-end="url()"/>
// <use xlink:href="" marker-start="url(...)" marker-end="url(...)"/>
use std::collections::HashMap;
use std::sync::Arc;
use crate::catcodes::CategoryCode;
use crate::commands::pgfsvg::parsesvg::{parse_path, parse_transform, strtonum};
use crate::stomach::groups::GroupClose;


#[derive(PartialEq,Clone)]
struct PGFGBegin {
    sourceref:Option<SourceFileReference>,
    attrs:HashMap<TeXStr,TeXStr>,
    tag:TeXStr
}

impl ExternalWhatsitGroup for PGFGBegin {
    fn get_ref(&self,ch : &Vec<Whatsit>) -> Option<SourceFileReference> {
        match &self.sourceref {
            Some(cnt) => Some(cnt.clone()),
            None => SourceFileReference::from_wi_list(ch)
        }//SourceFileReference::from_wi_list(ch).or(self.sourceref.clone())
    }
    fn name(&self) -> TeXStr { "PGFGbegin".into() }
    fn params(&self,s:&str) -> Option<TeXStr> { match self.attrs.get(&s.into()) {
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
        match mode {
            ColonMode::External(s) if s == "svg" => {
                let mut nret : Vec<Whatsit> = vec!();
                for c in ch { c.normalize(mode,&mut nret,scale) }
                ret.push(Whatsit::Grouped(WIGroup::External(Arc::new(self.clone()),nret)))
            }
            _ => for c in ch { c.normalize(mode,ret,scale) }
        }
    }
    fn as_html(&self,ch:Vec<Whatsit>, mode: &ColonMode, colon:&mut HTMLColon, node_top: &mut Option<HTMLParent>) {
        match mode {
            ColonMode::External(s) if s.to_string() == "svg" => {}
            _ => return ()
        }
        let mut newnode = HTMLNode::new(colon.state.current_namespace,(&self.tag).into(),self.sourceref.clone());
        for (k,v) in &self.attrs {
            if FIX {
                match k {
                    k if k.to_string() == "stroke-width" => {
                        newnode.attr(k.into(), strtonum(v))
                    }
                    k if k.to_string() == "d" => {
                        newnode.attr(k.into(), parse_path(v))
                    }
                    k if k.to_string() == "transform" => {
                        newnode.attr(k.into(),parse_transform(v))
                    }
                    _ => newnode.attr(k.into(), v.into())
                }
            } else {
                newnode.attr(k.into(), v.into())
            }
        }
        for c in ch {
            c.as_html(mode,colon,&mut Some(newnode.as_parent()));
        }
        match node_top {
            Some(e) => {
                e.push(HTMLChild::Str("\n".into()));
                e.push(HTMLChild::Node(newnode));
                e.push(HTMLChild::Str("\n".into()));
            }
            _ => {
                colon.state.top.push(HTMLChild::Node(newnode))
            }
        }
    }
}
mod parsesvg {
    use crate::stomach::html::{HTMLSCALE, HTMLStr};
    use crate::utils::TeXStr;

    pub fn strtonum(ts : &TeXStr) -> HTMLStr {
        (ts.to_string().parse::<f32>().unwrap() * HTMLSCALE).to_string().into()
    }

    pub fn parse_transform(ts : &TeXStr) -> HTMLStr {
        fn parse_one(s : &str) -> String {
            if s.is_empty() { return "".into() };
            if s.starts_with("translate(") {
                let (n1,s1) = parse_get_num(&s[10..]);
                let (n2,s2) = parse_get_num(s1);
                "translate(".to_string() + scale(n1).as_str() +
                    "," + scale(-n2).as_str() + ")" + &parse_one(s2)
            } else if s.starts_with("matrix(") {
                let (n1, s1) = parse_get_num(s[7..].trim_start());
                let (n2, s2) = parse_get_num(s1.trim_start());
                let (n3, s3) = parse_get_num(s2.trim_start());
                let (n4, s4) = parse_get_num(s3.trim_start());
                let (n5, s5) = parse_get_num(s4.trim_start());
                let (n6, s6) = parse_get_num(s5.trim_start());
                "matrix(".to_string() +
                    n1.to_string().as_str() + "," +
                    n2.to_string().as_str() + "," +
                    n3.to_string().as_str() + "," +
                    n4.to_string().as_str() + "," +
                    scale(n5).as_str() + "," +
                    scale(-n6).as_str() + ")" + &parse_one(s6)
            } else {
                todo!("Foo!")
            }
        }
        parse_one(ts.to_string().trim()).into()
    }

    fn parse_get_num<'A>(s:&'A str) -> (f32,&'A str) {
        match s.find(|x| x == ' ' || x == ')' || x == ',') {
            Some(i) =>
                (s[..i].parse::<f32>().unwrap(),&s[i+1..]),
            None => (s.parse::<f32>().unwrap(),"")
        }
    }

    fn scale(f:f32) -> String {
        (HTMLSCALE * f).to_string()
    }

    pub fn parse_path(ts : &TeXStr) -> HTMLStr {
        fn parse_path_one(s : &str) -> String {
            if s.is_empty() { return "".into() };
            if s.starts_with("M") {
                let (n1, s1) = parse_get_num(s[1..].trim_start());
                let (n2, s2) = parse_get_num(s1.trim_start());
                "M ".to_string() +
                    scale(n1).as_str() + " " +
                    scale(-n2).as_str() + " " +
                    parse_path_one(s2.trim_start()).as_str()
            } else if s.starts_with("L") {
                let (n1, s1) = parse_get_num(s[1..].trim_start());
                let (n2, s2) = parse_get_num(s1.trim_start());
                "L ".to_string() +
                    scale(n1).as_str() + " " +
                    scale(-n2).as_str() + " " +
                    parse_path_one(s2.trim_start()).as_str()
            } else if s.starts_with("C") {
                let (n1, s1) = parse_get_num(s[1..].trim_start());
                let (n2, s2) = parse_get_num(s1.trim_start());
                let (n3, s3) = parse_get_num(s2.trim_start());
                let (n4, s4) = parse_get_num(s3.trim_start());
                let (n5, s5) = parse_get_num(s4.trim_start());
                let (n6, s6) = parse_get_num(s5.trim_start());
                "C ".to_string() +
                    scale(n1).as_str() + " " +
                    scale(-n2).as_str() + " " +
                    scale(n3).as_str() + " " +
                    scale(-n4).as_str() + " " +
                    scale(n5).as_str() + " " +
                    scale(-n6).as_str() + " " +
                    parse_path_one(s6.trim_start()).as_str()
            } else if s.starts_with("Z") {
                "Z ".to_string() + parse_path_one(s[1..].trim_start()).as_str()
            } else if s.starts_with("h") {
                let (n1, s1) = parse_get_num(s[1..].trim_start());
                "h ".to_string() +
                    scale(n1).as_str() + " " +
                    parse_path_one(s1.trim_start()).as_str()
            } else if s.starts_with("v") {
                let (n1, s1) = parse_get_num(s[1..].trim_start());
                "v ".to_string() +
                    scale(-n1).as_str() + " " +
                    parse_path_one(s1.trim_start()).as_str()
            } else {
                todo!("Foo!")
            }
        }
        parse_path_one(ts.to_string().trim()).into()
    }

}

#[derive(PartialEq,Clone)]
struct PGFGEnd {}
impl ExternalWhatsitGroupEnd for PGFGEnd {
    fn name(&self) -> TeXStr { "PGFGend".into() }
    fn params(&self,_:&str) -> Option<TeXStr> {None}
    fn priority(&self) -> i16 { 95 }
    fn sourceref(&self) -> &Option<SourceFileReference> {&None}
}

// <g
// clip-path="url(...)"
// fill-rule="evenodd|nonzero"
// opacity=""
// stroke-opacity=""
// fill-opacity=""
// transform="translate(. , .)"
// transform="matrix(. , . , . , . , . , .)"
// transform="scale(. , .)"
// stroke-dasharray=""
// stroke-dashoffset=""
// stroke-width=""
// visibility=""
// >

pub static PGF_G_BEGIN: SimpleWhatsit = SimpleWhatsit {
    name: "rustex!pgf!gbegin",
    modes: |_| { true },
    _get: |tk, int| {
        let mut attrs : HashMap<TeXStr,TeXStr> = HashMap::new();
        let mut key = int.read_argument()?;
        let keystr = int.tokens_to_string(&key);
        'attr: loop {
            match int.read_keyword(vec!(
                "about","datatype","href","inlist","prefix","property","rel",
                "resource","rev","src","typeof","content",
                "clip-path","fill-rule","opacity","stroke-opacity","fill-opacity",
                "transform","stroke-dasharray","stroke-dashoffset","stroke-width",
                "stroke","fill","id","marker-start","marker-end","d","fill",
                "visibility","stroke-linecap","stroke-linejoin","xlink:href",
                "fx","fy","stroke-miterlimit","patternUnits","patternTransform",
                "markerUnits","orient","overflow","attributeName","from","to",
                "animateTransform","animateMotion","type",
                "dur","restart","repeatCount","repeatDur","begin","end"))? {
                None => {
                    if !int.preview().to_string().starts_with("\\relax") {
                        println!("missing: >>{}",int.preview());
                        print!("");
                    }
                    break 'attr
                }
                Some(s) if s == "animateTransform" => {
                    attrs.insert("animateTransform".into(),"true".into());
                }
                Some(s) if s == "animateMotion" => {
                    attrs.insert("animateMotion".into(),"true".into());
                }
                Some(s) => {
                    int.read_eq();
                    let r = int.read_string()?;
                    attrs.insert(s.into(),r.into());
                }
            }
        }
        let bg = PGFGBegin {
            sourceref: int.update_reference(tk),
            attrs,tag:keystr.into()
        };
        Ok(Whatsit::GroupOpen(WIGroup::External(Arc::new(bg),vec!())))
    },
};

pub static PGF_G_END: SimpleWhatsit = SimpleWhatsit {
    name: "rustex!pgf!gend",
    modes: |_| { true },
    _get: |_, int| {
        let mut key = int.read_argument()?;
        let keystr = int.tokens_to_string(&key);
        Ok(Whatsit::GroupClose(GroupClose::External(Arc::new(PGFGEnd {}))))
    },
};

#[derive(PartialEq,Clone)]
struct PGFBegin {
    sourceref:Option<SourceFileReference>,
    attrs:HashMap<&'static str,TeXStr>
}

impl ExternalWhatsitGroup for PGFBegin {
    fn get_ref(&self,ch : &Vec<Whatsit>) -> Option<SourceFileReference> {
        match &self.sourceref {
            Some(cnt) => Some(cnt.clone()),
            None => SourceFileReference::from_wi_list(ch)
        }//SourceFileReference::from_wi_list(ch).or(self.sourceref.clone())
    }
    fn name(&self) -> TeXStr { "PGFbegin".into() }
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
        let mut newnode = HTMLNode::new(colon.state.current_namespace,"svg".into(),self.sourceref.clone());
        for (k,v) in &self.attrs {
            newnode.attr((*k).into(),v.into())
        }
        for c in ch {
            c.as_html(mode,colon,&mut Some(newnode.as_parent()))
        }
        match node_top {
            Some(e) => {
                e.push(HTMLChild::Node(newnode))
            }
            _ => {
                colon.state.top.push(HTMLChild::Node(newnode))
            }
        }
    }
}

#[derive(PartialEq,Clone)]
struct PGFEnd {}
impl ExternalWhatsitGroupEnd for PGFEnd {
    fn name(&self) -> TeXStr { "PGFend".into() }
    fn params(&self,_:&str) -> Option<TeXStr> {None}
    fn priority(&self) -> i16 { 95 }
    fn sourceref(&self) -> &Option<SourceFileReference> {&None}
}


pub static PGF_BEGIN: SimpleWhatsit = SimpleWhatsit {
    name: "rustex!pgf!begin",
    modes: |_| { true },
    _get: |tk, int| {
        let mut attrs : HashMap<&'static str,TeXStr> = HashMap::new();
        'attr: loop {
            match int.read_keyword(vec!("overflow","preserveAspectRatio","id","x","y","width","height","viewBox"))? {
                None => break 'attr,
                Some(s) => {
                    let r = int.read_string()?;
                    println!("SVG: {} = {}",s,r);
                    print!("")
                }
            }
        }
        let bg = PGFBegin {
            sourceref: int.update_reference(tk),
            attrs: Default::default()
        };
        Ok(Whatsit::GroupOpen(WIGroup::External(Arc::new(bg),vec!())))
    },
};

pub static PGF_END: SimpleWhatsit = SimpleWhatsit {
    name: "rustex!pgf!end",
    modes: |_| { true },
    _get: |_, _| {
        Ok(Whatsit::GroupClose(GroupClose::External(Arc::new(PGFEnd {}))))
    },
};

// -------------------------------------------------------------------------------------------------

pub fn pgf_commands() -> Vec<PrimitiveTeXCommand> {vec![
    PrimitiveTeXCommand::Primitive(&PGFSYSDRIVER),
    PrimitiveTeXCommand::Primitive(&PGF_FLUSH),
    PrimitiveTeXCommand::Whatsit(ProvidesWhatsit::Simple(&TYPESETPICTUREBOX)),
    PrimitiveTeXCommand::Whatsit(ProvidesWhatsit::Simple(&PGFLITERAL)),
    // PrimitiveTeXCommand::Whatsit(ProvidesWhatsit::Simple(&COLORPUSH)),
    // PrimitiveTeXCommand::Whatsit(ProvidesWhatsit::Simple(&COLORPOP)),
    PrimitiveTeXCommand::Whatsit(ProvidesWhatsit::Simple(&PGFHBOX)),

    PrimitiveTeXCommand::Whatsit(ProvidesWhatsit::Simple(&PGF_BEGIN)),
    PrimitiveTeXCommand::Whatsit(ProvidesWhatsit::Simple(&PGF_END)),
    PrimitiveTeXCommand::Whatsit(ProvidesWhatsit::Simple(&PGF_G_BEGIN)),
    PrimitiveTeXCommand::Whatsit(ProvidesWhatsit::Simple(&PGF_G_END)),
]}