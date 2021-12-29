use std::cmp::min;
use std::path::PathBuf;
use std::rc::Rc;
use image::{DynamicImage, GenericImageView};
use crate::interpreter::dimensions::{dimtostr, MuSkip, Skip};
use crate::references::SourceFileReference;
use crate::stomach::boxes::TeXBox;
use crate::stomach::Whatsit;
use crate::stomach::whatsits::{HasWhatsitIter, WhatsitTrait};
use crate::Token;
use crate::utils::TeXStr;

#[derive(Clone)]
pub enum SimpleWI {
    PDFXImage(PDFXImage),
    VRule(VRule),
    HRule(HRule),
    VFil(VFil),
    VFill(VFill),
    HFil(HFil),
    HFill(HFill),
    Hss(Hss),
    Vss(Vss),
    VSkip(VSkip),
    HSkip(HSkip),
    MSkip(MSkip),
    Penalty(Penalty),
    PDFLiteral(PDFLiteral),
    PDFXForm(PDFXForm),
    Raise(Raise),
    MoveRight(MoveRight),
    VKern(VKern),
    HKern(HKern),
    PDFDest(PDFDest),
    HAlign(HAlign),
    VAlign(VAlign),
    Indent(Indent),
    Mark(Mark),
    Leaders(Leaders),
    PDFMatrix(PDFMatrix),
    Left(Left),
    Middle(Middle),
    Right(Right),
    External(Rc<dyn ExternalWhatsit>)
}

macro_rules! pass_on {
    ($s:tt,$e:ident$(,$tl:expr)*) => (match $s {
        SimpleWI::PDFXImage(g) => PDFXImage::$e(g $(,$tl)*),
        SimpleWI::VRule(g) => VRule::$e(g $(,$tl)*),
        SimpleWI::HRule(g) => HRule::$e(g $(,$tl)*),
        SimpleWI::VFil(g) => VFil::$e(g $(,$tl)*),
        SimpleWI::HFil(g) => HFil::$e(g $(,$tl)*),
        SimpleWI::VFill(g) => VFill::$e(g $(,$tl)*),
        SimpleWI::HFill(g) => HFill::$e(g $(,$tl)*),
        SimpleWI::Hss(g) => Hss::$e(g $(,$tl)*),
        SimpleWI::Vss(g) => Vss::$e(g $(,$tl)*),
        SimpleWI::VSkip(g) => VSkip::$e(g $(,$tl)*),
        SimpleWI::HSkip(g) => HSkip::$e(g $(,$tl)*),
        SimpleWI::MSkip(g) => MSkip::$e(g $(,$tl)*),
        SimpleWI::Penalty(g) => Penalty::$e(g $(,$tl)*),
        SimpleWI::PDFLiteral(g) => PDFLiteral::$e(g $(,$tl)*),
        SimpleWI::PDFXForm(g) => PDFXForm::$e(g $(,$tl)*),
        SimpleWI::Raise(g) => Raise::$e(g $(,$tl)*),
        SimpleWI::MoveRight(g) => MoveRight::$e(g $(,$tl)*),
        SimpleWI::VKern(g) => VKern::$e(g $(,$tl)*),
        SimpleWI::HKern(g) => HKern::$e(g $(,$tl)*),
        SimpleWI::PDFDest(g) => PDFDest::$e(g $(,$tl)*),
        SimpleWI::HAlign(g) => HAlign::$e(g $(,$tl)*),
        SimpleWI::VAlign(g) => VAlign::$e(g $(,$tl)*),
        SimpleWI::Indent(g) => Indent::$e(g $(,$tl)*),
        SimpleWI::Mark(g) => Mark::$e(g $(,$tl)*),
        SimpleWI::Leaders(g) => Leaders::$e(g $(,$tl)*),
        SimpleWI::PDFMatrix(g) => PDFMatrix::$e(g $(,$tl)*),
        SimpleWI::Left(g) => Left::$e(g $(,$tl)*),
        SimpleWI::Middle(g) => Middle::$e(g $(,$tl)*),
        SimpleWI::Right(g) => Right::$e(g $(,$tl)*),
        SimpleWI::External(e) => e.$e($($tl),*)
    })
}

impl WhatsitTrait for SimpleWI {
    fn as_whatsit(self) -> Whatsit { Whatsit::Simple(self) }
    fn width(&self) -> i32 { pass_on!(self,width) }
    fn height(&self) -> i32 { pass_on!(self,height) }
    fn depth(&self) -> i32 { pass_on!(self,depth) }
    fn as_xml_internal(&self, prefix: String) -> String {
        pass_on!(self,as_xml_internal,prefix)
    }
    fn has_ink(&self) -> bool { pass_on!(self,has_ink) }
}

#[derive(Clone)]
pub enum AlignBlock {
    Noalign(Vec<Whatsit>),
    Block(Vec<(Vec<Whatsit>,Skip,usize)>)
}

// -------------------------------------------------------------------------------------------------

pub trait ExternalWhatsit:WhatsitTrait {
    fn name(&self) -> &str;
    fn params(&self,name:&str) -> Option<&str>;
}

#[derive(Clone)]
pub struct PDFXImage{
    pub rule:TeXStr,
    pub attr:Option<TeXStr>,
    pub pagespec:Option<i32>,
    pub colorspace:Option<i32>,
    pub boxspec:Option<TeXStr>,
    pub filename:PathBuf,
    pub image:Option<DynamicImage>,
    pub sourceref:Option<SourceFileReference>
}
impl WhatsitTrait for PDFXImage {
    fn as_whatsit(self) -> Whatsit {
        Whatsit::Simple(SimpleWI::PDFXImage(self))
    }
    fn width(&self) -> i32 {
        match &self.image {
            Some(img) => img.width() as i32 * 65536,
            _ => 65536
        }
    }
    fn height(&self) -> i32 {
        match &self.image {
            Some(img) => img.height() as i32 * 65536,
            _ => 65536
        }
    }
    fn depth(&self) -> i32 { 0 }

    fn as_xml_internal(&self, prefix: String) -> String {
        let mut ret = "\n".to_string() + &prefix + "<pdfximage rule=\"" + &self.rule.to_string() + "\"";
        match &self.attr {
            None => (),
            Some(a) => {
                ret += " attr=\"";
                ret += &a.to_string();
                ret += "\""
            }
        }
        match &self.pagespec {
            None => (),
            Some(a) => {
                ret += " pagespec=\"";
                ret += &a.to_string();
                ret += "\""
            }
        }
        match &self.colorspace {
            None => (),
            Some(a) => {
                ret += " colorspace=\"";
                ret += &a.to_string();
                ret += "\""
            }
        }
        match &self.boxspec {
            None => (),
            Some(a) => {
                ret += " boxspec=\"";
                ret += &a.to_string();
                ret += "\""
            }
        }
        ret += " file=\"";
        ret += self.filename.to_str().unwrap();
        ret += "\"";
        ret + "/>"
    }
    fn has_ink(&self) -> bool { true }
}

#[derive(Clone)]
pub struct VRule {
    pub height:Option<i32>,
    pub width:Option<i32>,
    pub depth:Option<i32>,
    pub sourceref:Option<SourceFileReference>
}
impl WhatsitTrait for VRule {
    fn as_whatsit(self) -> Whatsit {
        Whatsit::Simple(SimpleWI::VRule(self))
    }
    fn width(&self) -> i32 { self.width.unwrap_or(26214) }
    fn height(&self) -> i32 { self.height.unwrap_or(0) }
    fn depth(&self) -> i32 { self.depth.unwrap_or(0) }
    fn as_xml_internal(&self, prefix: String) -> String {
        "\n".to_string() + &prefix + "<vrule width=\"" + &dimtostr(self.width()) +
            "\" height=\"" + &dimtostr(self.height()) + "\" depth=\"" + &dimtostr(self.depth()) + "\"/>"
    }
    fn has_ink(&self) -> bool {
        self.width() != 0 && (self.height() != 0 || self.depth() != 0)
    }
}

#[derive(Clone)]
pub struct HRule {
    pub height:Option<i32>,
    pub width:Option<i32>,
    pub depth:Option<i32>,
    pub sourceref:Option<SourceFileReference>
}
impl WhatsitTrait for HRule {
    fn as_whatsit(self) -> Whatsit {
        Whatsit::Simple(SimpleWI::HRule(self))
    }
    fn width(&self) -> i32 { self.width.unwrap_or(0) }
    fn height(&self) -> i32 { self.height.unwrap_or(26214) }
    fn depth(&self) -> i32 { self.depth.unwrap_or(0) }
    fn as_xml_internal(&self, prefix: String) -> String {
        "\n".to_string() + &prefix + "<hrule width=\"" + &dimtostr(self.width()) +
            "\" height=\"" + &dimtostr(self.height()) + "\" depth=\"" + &dimtostr(self.depth()) + "\"/>"
    }
    fn has_ink(&self) -> bool {
        self.width() != 0 && (self.height() != 0 || self.depth() != 0)
    }
}

#[derive(Clone)]
pub struct VSkip {
    pub skip:Skip,
    pub sourceref:Option<SourceFileReference>
}
impl WhatsitTrait for VSkip {
    fn as_whatsit(self) -> Whatsit {
        Whatsit::Simple(SimpleWI::VSkip(self))
    }
    fn width(&self) -> i32 { 0 }
    fn height(&self) -> i32 { self.skip.base }
    fn depth(&self) -> i32 { 0 }
    fn as_xml_internal(&self, prefix: String) -> String {
        "\n".to_string() + &prefix + "<vskip val=\"" + &self.skip.to_string() + "\"/>"
    }
    fn has_ink(&self) -> bool { false }
}

#[derive(Clone)]
pub struct HSkip {
    pub skip:Skip,
    pub sourceref:Option<SourceFileReference>
}
impl WhatsitTrait for HSkip {
    fn as_whatsit(self) -> Whatsit {
        Whatsit::Simple(SimpleWI::HSkip(self))
    }
    fn width(&self) -> i32 { self.skip.base }
    fn height(&self) -> i32 { 0 }
    fn depth(&self) -> i32 { 0 }
    fn as_xml_internal(&self, prefix: String) -> String {
        "\n".to_string() + &prefix + "<hskip val=\"" + &self.skip.to_string() + "\"/>"
    }
    fn has_ink(&self) -> bool { false }
}

#[derive(Clone)]
pub struct MSkip {
    pub skip:MuSkip,
    pub sourceref:Option<SourceFileReference>
}
impl WhatsitTrait for MSkip {
    fn as_whatsit(self) -> Whatsit {
        Whatsit::Simple(SimpleWI::MSkip(self))
    }
    fn width(&self) -> i32 { self.skip.base }
    fn height(&self) -> i32 { 0 }
    fn depth(&self) -> i32 { 0 }
    fn as_xml_internal(&self, prefix: String) -> String {
        "\n".to_string() + &prefix + "<mskip val=\"" + &self.skip.to_string() + "\"/>"
    }
    fn has_ink(&self) -> bool { false }
}

#[derive(Clone)]
pub struct Penalty {
    pub penalty:i32,
    pub sourceref:Option<SourceFileReference>
}
impl WhatsitTrait for Penalty {
    fn as_whatsit(self) -> Whatsit {
        Whatsit::Simple(SimpleWI::Penalty(self))
    }
    fn width(&self) -> i32 { 0 }
    fn height(&self) -> i32 { 0 }
    fn depth(&self) -> i32 { 0 }
    fn as_xml_internal(&self, prefix: String) -> String {
        "\n".to_string() + &prefix + "<penalty val=\"" + &self.penalty.to_string() + "\"/>"
    }
    fn has_ink(&self) -> bool { false }
}

#[derive(Clone)]
pub struct PDFLiteral {
    pub literal:TeXStr,
    pub sourceref:Option<SourceFileReference>
}
impl WhatsitTrait for PDFLiteral {
    fn as_whatsit(self) -> Whatsit {
        Whatsit::Simple(SimpleWI::PDFLiteral(self))
    }
    fn width(&self) -> i32 { 0 }
    fn height(&self) -> i32 { 0 }
    fn depth(&self) -> i32 { 0 }
    fn as_xml_internal(&self, _: String) -> String {
        "<pdfliteral value=\"".to_string() + &self.literal.to_string() + "\"/>"
    }
    fn has_ink(&self) -> bool { false }
}

#[derive(Clone)]
pub struct PDFXForm {
    pub attr:Option<TeXStr>,
    pub resource:Option<TeXStr>,
    pub content:TeXBox,
    pub sourceref:Option<SourceFileReference>
}
impl WhatsitTrait for PDFXForm {
    fn as_whatsit(self) -> Whatsit {
        Whatsit::Simple(SimpleWI::PDFXForm(self))
    }
    fn width(&self) -> i32 { 0 }
    fn height(&self) -> i32 { 0 }
    fn depth(&self) -> i32 { 0 }
    fn as_xml_internal(&self, _: String) -> String {
        todo!()
    }
    fn has_ink(&self) -> bool { false }
}

#[derive(Clone)]
pub struct Raise {
    pub dim:i32,
    pub content:TeXBox,
    pub sourceref:Option<SourceFileReference>
}
impl WhatsitTrait for Raise {
    fn as_whatsit(self) -> Whatsit {
        Whatsit::Simple(SimpleWI::Raise(self))
    }
    fn width(&self) -> i32 { self.content.width() }
    fn height(&self) -> i32 { min(self.content.height() + self.dim,0) }
    fn depth(&self) -> i32 { min(self.content.depth() - self.dim,0) }
    fn as_xml_internal(&self, prefix: String) -> String {
        let mut ret = "\n".to_string() + &prefix + "<raise by=\"" + &dimtostr(self.dim) + "\">";
        ret += &self.content.as_xml_internal(prefix.clone() + "  ");
        ret + "\n" + &prefix + "</raise>"
    }
    fn has_ink(&self) -> bool { self.content.has_ink() }
}

#[derive(Clone)]
pub struct MoveRight {
    pub dim:i32,
    pub content:TeXBox,
    pub sourceref:Option<SourceFileReference>
}
impl WhatsitTrait for MoveRight {
    fn as_whatsit(self) -> Whatsit {
        Whatsit::Simple(SimpleWI::MoveRight(self))
    }
    fn width(&self) -> i32 { self.content.width() + self.dim }
    fn height(&self) -> i32 { self.content.height() }
    fn depth(&self) -> i32 { self.content.depth() }
    fn as_xml_internal(&self, prefix: String) -> String {
        let mut ret = "\n".to_string() + &prefix + "<moveright by=\"" + &dimtostr(self.dim) + "\">";
        ret += &self.content.as_xml_internal(prefix.clone() + "  ");
        ret + "\n" + &prefix + "</moveright>"
    }
    fn has_ink(&self) -> bool { self.content.has_ink() }
}

#[derive(Clone)]
pub struct VKern {
    pub dim:i32,
    pub sourceref:Option<SourceFileReference>
}
impl WhatsitTrait for VKern {
    fn as_whatsit(self) -> Whatsit {
        Whatsit::Simple(SimpleWI::VKern(self))
    }
    fn width(&self) -> i32 { 0 }
    fn height(&self) -> i32 { self.dim }
    fn depth(&self) -> i32 { 0 }
    fn as_xml_internal(&self, prefix: String) -> String {
        "\n".to_string() + &prefix + "<vkern val=\"" + &dimtostr(self.dim) + "\"/>"
    }
    fn has_ink(&self) -> bool { false }
}

#[derive(Clone)]
pub struct HKern {
    pub dim:i32,
    pub sourceref:Option<SourceFileReference>
}
impl WhatsitTrait for HKern {
    fn as_whatsit(self) -> Whatsit {
        Whatsit::Simple(SimpleWI::HKern(self))
    }
    fn width(&self) -> i32 { self.dim }
    fn height(&self) -> i32 { 0 }
    fn depth(&self) -> i32 { 0 }
    fn as_xml_internal(&self, prefix: String) -> String {
        "\n".to_string() + &prefix + "<hkern val=\"" + &dimtostr(self.dim) + "\"/>"
    }
    fn has_ink(&self) -> bool { false }
}

#[derive(Clone)]
pub struct Indent {
    pub dim:i32,
    pub sourceref:Option<SourceFileReference>
}
impl WhatsitTrait for Indent {
    fn as_whatsit(self) -> Whatsit {
        Whatsit::Simple(SimpleWI::Indent(self))
    }
    fn width(&self) -> i32 { self.dim }
    fn height(&self) -> i32 { 0 }
    fn depth(&self) -> i32 { 0 }
    fn as_xml_internal(&self, prefix: String) -> String {
        "\n".to_string() + &prefix + "<indent val=\"" + &dimtostr(self.dim) + "\"/>"
    }
    fn has_ink(&self) -> bool { false }
}

#[derive(Clone)]
pub struct PDFDest {
    pub target:TeXStr,
    pub dest:TeXStr,
    pub sourceref:Option<SourceFileReference>
}
impl WhatsitTrait for PDFDest {
    fn as_whatsit(self) -> Whatsit {
        Whatsit::Simple(SimpleWI::PDFDest(self))
    }
    fn width(&self) -> i32 { 0 }
    fn height(&self) -> i32 { 0 }
    fn depth(&self) -> i32 { 0 }
    fn as_xml_internal(&self, prefix: String) -> String {
        "\n".to_string() + &prefix + "<pdfdest target=\"" + &self.target.to_string() + "\" dest=\"" + &self.dest.to_string() + "\"/>"
    }
    fn has_ink(&self) -> bool { false }
}

#[derive(Clone)]
pub struct HAlign {
    pub skip:Skip,
    pub template:Vec<(Vec<Token>,Vec<Token>,Skip)>,
    pub rows:Vec<AlignBlock>,
    pub sourceref:Option<SourceFileReference>
}
impl WhatsitTrait for HAlign {
    fn as_whatsit(self) -> Whatsit {
        Whatsit::Simple(SimpleWI::HAlign(self))
    }
    fn width(&self) -> i32 {
        let mut width:i32 = 0;
        for b in &self.rows {
            match b {
                AlignBlock::Noalign(v) => {
                    let mut max = 0;
                    for c in v.iter_wi() {
                        let w = c.width();
                        if w > max {max = w}
                    }
                    if max > width { width = max }
                }
                AlignBlock::Block(ls) => {
                    let mut w:i32 = 0;
                    for (v,s,_) in ls {
                        w += s.base;
                        for c in v.iter_wi() { w += c.width() }
                    }
                    if w > width { width = w }
                }
            }
        }
        width + self.skip.base
    }
    fn height(&self) -> i32 {
        let mut height:i32 = 0;
        for b in &self.rows {
            match b {
                AlignBlock::Noalign(v) => {
                    for c in v.iter_wi() {
                        height += c.height();
                    }
                }
                AlignBlock::Block(ls) => {
                    let mut ht:i32 = 0;
                    for (v,_,_) in ls {
                        for c in v.iter_wi() {
                            let h = c.height();
                            if h > ht { ht = h }
                        }
                    }
                    height += ht
                }
            }
        }
        height
    }
    fn depth(&self) -> i32 { 0 }
    fn as_xml_internal(&self, prefix: String) -> String {
        let mut ret = "\n".to_string() + &prefix + "<halign>";
        for block in &self.rows { match block {
            AlignBlock::Noalign(nas) => {
                ret += "\n  ";
                ret += &prefix;
                ret += "<noalign>";
                for w in nas { ret += &w.as_xml_internal(prefix.clone() + "    ") }
                ret += "\n  ";
                ret += &prefix;
                ret += "</noalign>";
            }
            AlignBlock::Block(ls) => {
                ret += "\n  ";
                ret += &prefix;
                ret += "<row>";
                for (l,_,_) in ls {
                    ret += "\n  ";
                    ret += &prefix;
                    ret += "<cell>";
                    for w in l {
                        ret += &w.as_xml_internal(prefix.clone() + "    ")
                    }
                    ret += "\n  ";
                    ret += &prefix;
                    ret += "</cell>";
                }
                ret += "\n  ";
                ret += &prefix;
                ret += "</row>";
            }
        }}
        ret + "\n" + &prefix + "</halign>"
    }
    fn has_ink(&self) -> bool {
        for v in &self.rows {
            match v {
                AlignBlock::Noalign(v) => for c in v { if c.has_ink() {return true} }
                AlignBlock::Block(v) => for (iv,_,_) in v { for c in iv { if c.has_ink() {return true} } }
            }
        }
        false
    }
}

#[derive(Clone)]
pub struct VAlign {
    pub skip:Skip,
    pub template:Vec<(Vec<Token>,Vec<Token>,Skip)>,
    pub columns:Vec<AlignBlock>,
    pub sourceref:Option<SourceFileReference>
}
impl WhatsitTrait for VAlign {
    fn as_whatsit(self) -> Whatsit {
        Whatsit::Simple(SimpleWI::VAlign(self))
    }
    fn width(&self) -> i32 {
        let mut width:i32 = 0;
        for b in &self.columns {
            match b {
                AlignBlock::Noalign(v) => {
                    for c in v.iter_wi() {
                        width += c.width();
                    }
                }
                AlignBlock::Block(ls) => {
                    let mut wd:i32 = 0;
                    for (v,_,_) in ls {
                        for c in v.iter_wi() {
                            let w = c.width();
                            if w > wd { wd = w }
                        }
                    }
                    width += wd
                }
            }
        }
        width
    }
    fn height(&self) -> i32 {
        let mut height:i32 = 0;
        for b in &self.columns {
            match b {
                AlignBlock::Noalign(v) => {
                    let mut max = 0;
                    for c in v.iter_wi() {
                        let w = c.height();
                        if w > max {max = w}
                    }
                    if max > height { height = max }
                }
                AlignBlock::Block(ls) => {
                    let mut w:i32 = 0;
                    for (v,s,_) in ls {
                        w += s.base;
                        for c in v.iter_wi() { w += c.height()}
                    }
                    if w > height { height = w }
                }
            }
        }
        height + self.skip.base
    }
    fn depth(&self) -> i32 { 0 }
    fn as_xml_internal(&self, prefix: String) -> String {
        let mut ret = "\n".to_string() + &prefix + "<valign>";
        for block in &self.columns { match block {
            AlignBlock::Noalign(nas) => {
                ret += "\n  ";
                ret += &prefix;
                ret += "<noalign>";
                for w in nas { ret += &w.as_xml_internal(prefix.clone() + "    ") }
                ret += "\n  ";
                ret += &prefix;
                ret += "</noalign>";
            }
            AlignBlock::Block(ls) => {
                ret += "\n  ";
                ret += &prefix;
                ret += "<column>";
                for (l,_,_) in ls {
                    ret += "\n  ";
                    ret += &prefix;
                    ret += "<cell>";
                    for w in l {
                        ret += &w.as_xml_internal(prefix.clone() + "    ")
                    }
                    ret += "\n  ";
                    ret += &prefix;
                    ret += "</cell>";
                }
                ret += "\n  ";
                ret += &prefix;
                ret += "</column>";
            }
        }}
        ret + "\n" + &prefix + "</valign>"
    }
    fn has_ink(&self) -> bool {
        for v in &self.columns {
            match v {
                AlignBlock::Noalign(v) => for c in v { if c.has_ink() {return true} }
                AlignBlock::Block(v) => for (iv,_,_) in v { for c in iv { if c.has_ink() {return true} } }
            }
        }
        false
    }
}

#[derive(Clone)]
pub struct Mark {
    pub toks:Vec<Token>,
    pub sourceref:Option<SourceFileReference>
}
impl WhatsitTrait for Mark {
    fn as_whatsit(self) -> Whatsit {
        Whatsit::Simple(SimpleWI::Mark(self))
    }
    fn width(&self) -> i32 { 0 }
    fn height(&self) -> i32 { 0 }
    fn depth(&self) -> i32 { 0 }
    fn as_xml_internal(&self, _: String) -> String {
        "<mark/>".to_string()
    }
    fn has_ink(&self) -> bool { false }
}

#[derive(Clone)]
pub struct Leaders {
    pub bx:Box<Whatsit>,
    pub sourceref:Option<SourceFileReference>
}
impl WhatsitTrait for Leaders {
    fn as_whatsit(self) -> Whatsit {
        Whatsit::Simple(SimpleWI::Leaders(self))
    }
    fn width(&self) -> i32 { self.bx.width() }
    fn height(&self) -> i32 { self.bx.height() }
    fn depth(&self) -> i32 { self.bx.depth() }
    fn as_xml_internal(&self, prefix: String) -> String {
        "<leaders>".to_string() + &self.bx.as_xml_internal(prefix) + "</leaders>"
    }
    fn has_ink(&self) -> bool { self.bx.has_ink() }
}

#[derive(Clone)]
pub struct PDFMatrix {
    pub scale:f32,
    pub rotate:f32,
    pub skewx:f32,
    pub skewy:f32,
    pub sourceref:Option<SourceFileReference>
}
impl WhatsitTrait for PDFMatrix {
    fn as_whatsit(self) -> Whatsit {
        Whatsit::Simple(SimpleWI::PDFMatrix(self))
    }
    fn width(&self) -> i32 { 0 }
    fn height(&self) -> i32 { 0 }
    fn depth(&self) -> i32 { 0 }
    fn as_xml_internal(&self, _: String) -> String {
        "<pdfmatrix scale=\"".to_string() + &self.scale.to_string() +
            "\" rotate=\"" + &self.rotate.to_string() +
            "\" skewx=\"" + &self.skewx.to_string() +
            "\" sskewy=\"" + &self.skewy.to_string() + "\"/>"
    }
    fn has_ink(&self) -> bool { false }
}

#[derive(Clone)]
pub struct Left {
    pub bx:Option<Box<Whatsit>>,
    pub sourceref:Option<SourceFileReference>
}
impl WhatsitTrait for Left {
    fn as_whatsit(self) -> Whatsit {
        Whatsit::Simple(SimpleWI::Left(self))
    }
    fn width(&self) -> i32 { self.bx.as_ref().map_or(0,|x| x.width()) }
    fn height(&self) -> i32 { self.bx.as_ref().map_or(0,|x| x.height()) }
    fn depth(&self) -> i32 { self.bx.as_ref().map_or(0,|x| x.depth()) }
    fn as_xml_internal(&self, prefix: String) -> String {
        "<left>".to_string() + self.bx.as_ref().map(|x| x.as_xml_internal(prefix)).unwrap_or("".to_string()).as_str() + "</left>"
    }
    fn has_ink(&self) -> bool { self.bx.is_some() }
}

#[derive(Clone)]
pub struct Middle {
    pub bx:Option<Box<Whatsit>>,
    pub sourceref:Option<SourceFileReference>
}
impl WhatsitTrait for Middle {
    fn as_whatsit(self) -> Whatsit {
        Whatsit::Simple(SimpleWI::Middle(self))
    }
    fn width(&self) -> i32 { self.bx.as_ref().map_or(0,|x| x.width()) }
    fn height(&self) -> i32 { self.bx.as_ref().map_or(0,|x| x.height()) }
    fn depth(&self) -> i32 { self.bx.as_ref().map_or(0,|x| x.depth()) }
    fn as_xml_internal(&self, prefix: String) -> String {
        "<middle>".to_string() + self.bx.as_ref().map(|x| x.as_xml_internal(prefix)).unwrap_or("".to_string()).as_str() + "</middle>"
    }
    fn has_ink(&self) -> bool { self.bx.is_some() }
}

#[derive(Clone)]
pub struct Right {
    pub bx:Option<Box<Whatsit>>,
    pub sourceref:Option<SourceFileReference>
}
impl WhatsitTrait for Right {
    fn as_whatsit(self) -> Whatsit {
        Whatsit::Simple(SimpleWI::Right(self))
    }
    fn width(&self) -> i32 { self.bx.as_ref().map_or(0,|x| x.width()) }
    fn height(&self) -> i32 { self.bx.as_ref().map_or(0,|x| x.height()) }
    fn depth(&self) -> i32 { self.bx.as_ref().map_or(0,|x| x.depth()) }
    fn as_xml_internal(&self, prefix: String) -> String {
        "<right>".to_string() + self.bx.as_ref().map(|x| x.as_xml_internal(prefix)).unwrap_or("".to_string()).as_str() + "</right>"
    }
    fn has_ink(&self) -> bool { self.bx.is_some() }
}

macro_rules! trivial {
    ($e:ident) => (
        #[derive(Clone)]
        pub struct $e(pub Option<SourceFileReference>);
        impl WhatsitTrait for $e {
            fn as_whatsit(self) -> Whatsit {
                Whatsit::Simple(SimpleWI::$e(self))
            }
            fn width(&self) -> i32 { 0 }
            fn height(&self) -> i32 { 0 }
            fn depth(&self) -> i32 { 0 }
            fn as_xml_internal(&self, prefix: String) -> String {
                "<".to_string() + &stringify!($e) + "/>"
            }
            fn has_ink(&self) -> bool { false }
        }
    )
}
trivial!(HFil);
trivial!(HFill);
trivial!(VFil);
trivial!(VFill);
trivial!(Hss);
trivial!(Vss);