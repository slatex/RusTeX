use std::cmp::{max, min, Ordering};
use std::path::PathBuf;
use crate::interpreter::Interpreter;
use crate::utils::{TeXError, TeXStr};
use std::rc::Rc;
use std::str::from_utf8;
use image::{DynamicImage, GenericImageView};
use crate::commands::MathWhatsit;
use crate::fonts::Font;
use crate::interpreter::dimensions::{dimtostr, MuSkip, Skip};
use crate::references::SourceFileReference;
use crate::stomach::StomachGroup;
use crate::Token;

pub trait HasWhatsitIter {
    fn iter_wi(&self) -> WhatsitIter;
}

impl HasWhatsitIter for Vec<Whatsit> {
    fn iter_wi(&self) -> WhatsitIter {
        WhatsitIter::new(self)
    }
}

pub struct WhatsitIter<'a> {
    children:&'a [Whatsit],
    parent:Option<Box<WhatsitIter<'a>>>
}

impl WhatsitIter<'_> {
    pub fn new(v:&Vec<Whatsit>) -> WhatsitIter {
        WhatsitIter {
            children:v.as_slice(),
            parent:None
        }
    }
}

impl <'a> Iterator for WhatsitIter<'a> {
    type Item = &'a Whatsit;
    fn next(&mut self) -> Option<Self::Item> {
        match self.children.get(0) {
            None => match self.parent.take() {
                Some(p) =>{
                    *self = *p;
                    self.next()
                }
                None => None
            }
            Some(Whatsit::Grouped(g)) if !g.opaque() => {
                self.children = &self.children[1..];
                *self = WhatsitIter {
                    children:g.children().as_slice(),
                    parent:Some(Box::new(std::mem::take(self)))
                };
                self.next()
            }
            Some(s) => {
                self.children = &self.children[1..];
                Some(s)
            }
        }
    }
}
impl<'a> Default for WhatsitIter<'a> {
    fn default() -> Self {
        WhatsitIter { children: &[], parent: None }
    }
}

pub static WIDTH_CORRECTION : i32 = 0;
pub static HEIGHT_CORRECTION : i32 = 0;

pub trait WhatsitTrait {
    fn as_whatsit(self) -> Whatsit;
    fn width(&self) -> i32;
    fn height(&self) -> i32;
    fn depth(&self) -> i32;
    fn as_xml_internal(&self,prefix: String) -> String;
    fn has_ink(&self) -> bool;

    fn as_xml(&self) -> String {
        self.as_xml_internal("".to_string())
    }
}

use crate::stomach::boxes::{BoxMode,TeXBox};
use crate::stomach::math::{MathGroup, MathInfix};
use crate::stomach::paragraph::Paragraph;

// -------------------------------------------------------------------------------------------------

#[derive(Clone)]
pub enum Whatsit {
    Exec(Rc<ExecutableWhatsit>),
    Box(TeXBox),
    Ext(Rc<dyn ExtWhatsit>),
    GroupOpen(WIGroup),
    GroupClose(WIGroup),
    Simple(SimpleWI),
    Char(u8,Rc<Font>,Option<SourceFileReference>),
    Math(MathGroup),
    MathInfix(MathInfix),
    Ls(Vec<Whatsit>),
    Grouped(WIGroup),
    Par(Paragraph),
    Inserts(Vec<Vec<Whatsit>>),
    Float(TeXBox)
}

impl Whatsit {
    pub fn as_xml(&self) -> String {
        self.as_xml_internal("".to_string())
    }
    pub fn as_xml_internal(&self,prefix: String) -> String {
        use Whatsit::*;
        match self {
            Exec(_) | GroupOpen(_) | GroupClose(_) => "".to_string(),
            Ext(e) => e.as_xml_internal(prefix),
            Math(m) => m.as_xml_internal(prefix),
            Simple(s) => s.as_xml_internal(prefix),
            Grouped(g) => g.as_xml_internal(prefix),
            Par(p) => p.as_xml_internal(prefix),
            Box(b) => b.as_xml_internal(prefix),
            Char(u,_,_) => {
                let ret = if *u == 60 {
                    TeXStr::new(&[38,108,116,59])
                } else if *u == 62 {
                    TeXStr::new(&[38,103,116,59])
                } else if *u == 38 {
                    TeXStr::new(&[38,97,109,112,59])
                } else {
                    TeXStr::new(&[*u])
                };
                ret.to_string()
            },
            MathInfix(i) => i.as_xml_internal(prefix),
            Inserts(vs) => {
                let mut ret = "\n".to_string() + &prefix + "<inserts>";
                for v in vs {
                    ret += "\n  ";
                    ret += &prefix;
                    ret += "<insert>";
                    for w in v {
                        ret  += &w.as_xml_internal(prefix.clone() + "    ")
                    }
                    ret += "\n  ";
                    ret += &prefix;
                    ret += "</insert>";
                }
                ret + "\n" + &prefix + "</inserts>"
            }
            Float(bx) => {
                let mut ret = "\n".to_string() + &prefix + "<float>";
                ret  += &bx.as_xml_internal(prefix.clone() + "  ");
                ret + "\n" + &prefix + "</float>"
            }
            Ls(_) => unreachable!(),
            _ => todo!()
        }

    }
    pub fn has_ink(&self) -> bool {
        use Whatsit::*;
        match self {
            Exec(_) | GroupClose(_) => false,
            Box(b) => b.has_ink(),
            Ext(e) => e.has_ink(),
            GroupOpen(w) => w.has_ink(),
            Grouped(w) => w.has_ink(),
            Simple(s) => s.has_ink(),
            Char(_,_,_) | Par(_) | Float(_) | Inserts(_) | MathInfix(_) => true,
            Math(m) => m.has_ink(),
            Ls(_) => unreachable!()
        }
    }
    pub fn width(&self) -> i32 {
        use Whatsit::*;
        match self {
            Exec(_) | GroupClose(_) => 0,
            Box(b) => b.width(),
            Ext(e) => e.width(),
            GroupOpen(w) => w.width(),
            Grouped(w) => w.width(),
            Simple(s) => s.width(),
            Char(u,f,_) => f.get_width(*u as u16),
            Math(m) => m.width(),
            Par(p) => p.width(),
            Float(s) => s.width(),
            Inserts(s) => todo!(),
            MathInfix(m) => m.width(),
            Ls(_) => unreachable!(),

        }
    }
    pub fn height(&self) -> i32 {
        use Whatsit::*;
        match self {
            Exec(_) | GroupClose(_) => 0,
            Box(b) => b.height(),
            Ext(e) => e.height(),
            GroupOpen(w) => w.height(),
            Grouped(w) => w.height(),
            Simple(s) => s.height(),
            Char(u,f,_) => f.get_height(*u as u16),
            Math(m) => m.height(),
            Par(p) => p.height(),
            Ls(_) => unreachable!(),
            Float(s) => s.height(),
            Inserts(s) => todo!(),
            MathInfix(m) => m.height(),
            _ => todo!()
        }
    }
    pub fn depth(&self) -> i32 {
        use Whatsit::*;
        match self {
            Exec(_) | GroupClose(_) => 0,
            Box(b) => b.depth(),
            Ext(e) => e.depth(),
            GroupOpen(w) => w.depth(),
            Grouped(w) => w.depth(),
            Simple(s) => s.depth(),
            Char(u,f,_) => f.get_depth(*u as u16),
            Math(m) => m.depth(),
            Par(p) => p.depth(),
            Ls(_) => unreachable!(),
            Float(s) => s.depth(),
            Inserts(s) => todo!(),
            MathInfix(m) => m.depth(),
            _ => todo!()
        }
    }
}

#[derive(Clone)]
pub enum WIGroup {
    FontChange(Rc<Font>,Option<SourceFileReference>,bool,Vec<Whatsit>),
    ColorChange(TeXStr,Option<SourceFileReference>,Vec<Whatsit>),
    //       rule   attr  action
    PDFLink(TeXStr,TeXStr,ActionSpec,Option<SourceFileReference>,Vec<Whatsit>),
    PdfMatrixSave(Option<SourceFileReference>,bool,Vec<Whatsit>),
    PdfRestore(Option<SourceFileReference>),
    LinkEnd(Option<SourceFileReference>),
    ColorEnd(Option<SourceFileReference>),
}
impl WIGroup {
    pub fn as_xml_internal(&self,prefix: String) -> String {
        use WIGroup::*;
        match self {
            FontChange(_,_,_,v) => {
                let mut ret = "\n".to_string() + &prefix + "<font TODO=\"\">";
                for c in v {
                    ret += &c.as_xml_internal(prefix.clone() + "  ")
                }
                ret + "\n" + &prefix + "</font>"
            }
            ColorChange(c,_,v) => {
                let mut ret = "\n".to_string() + &prefix + "<color color=\"" + c.to_string().as_str() + "\">";
                for c in v {
                    ret += &c.as_xml_internal(prefix.clone() + "  ")
                }
                ret + "\n" + &prefix + "</color>"
            }
            PDFLink(a,b,_,_,v) => {
                let mut ret = "\n".to_string() + &prefix + "<link a=\"" + a.to_string().as_str() + "\" b=\"" + b.to_string().as_str() + "\">";
                for c in v {
                    ret += &c.as_xml_internal(prefix.clone() + "  ")
                }
                ret + "\n" + &prefix + "</link>"
            }
            PdfMatrixSave(_,_,v) => {
                let mut ret = "\n".to_string() + &prefix + "<pdfmatrix>";
                for c in v {
                    ret += &c.as_xml_internal(prefix.clone() + "  ")
                }
                ret + "\n" + &prefix + "</pdfmatrix>"
            }
            _ => todo!()
        }
    }
    pub fn opaque(&self) -> bool {
        use WIGroup::*;
        match self {
            PdfMatrixSave(_,_,_) => true,
            _ => false
        }
    }
    pub fn push(&mut self,wi:Whatsit) {
        use WIGroup::*;
        match self {
            FontChange(_,_,_,v) => v.push(wi),
            ColorChange(_,_,v) => v.push(wi),
            PDFLink(_,_,_,_,v) => v.push(wi),
            PdfMatrixSave(_,_,v) => v.push(wi),
            ColorEnd(_) | LinkEnd(_) | PdfRestore(_) => unreachable!(),
        }
    }
    pub fn priority(&self) -> i16 {
        use WIGroup::*;
        match self {
            FontChange(_,_,true,_) => 25,
            FontChange(_,_,_,_) => 2,
            ColorChange(_,_,_) | ColorEnd(_) => 50,
            PDFLink(_,_,_,_,_) | LinkEnd(_) => 60,
            PdfMatrixSave(_,_,_) | PdfRestore(_) => 70
        }
    }
    pub fn has_ink(&self) -> bool {
        use WIGroup::*;
        match self {
            ColorEnd(_) | LinkEnd(_) | PdfRestore(_) => false,
            _ => {
                for x in self.children() { if x.has_ink() {return true} }
                false
            }
        }
    }
    pub fn children_d(self) -> Vec<Whatsit> {
        use WIGroup::*;
        match self {
            FontChange(_,_,_,v) => v,
            ColorChange(_,_,v) => v,
            PDFLink(_,_,_,_,v) => v,
            PdfMatrixSave(_,_,v) => v,
            ColorEnd(_) | LinkEnd(_) | PdfRestore(_) => unreachable!()
        }
    }
    pub fn children(&self) -> &Vec<Whatsit> {
        use WIGroup::*;
        match self {
            FontChange(_,_,_,v) => v,
            ColorChange(_,_,v) => v,
            PDFLink(_,_,_,_,v) => v,
            PdfMatrixSave(_,_,v) => v,
            ColorEnd(_) | LinkEnd(_) | PdfRestore(_) => unreachable!()
        }
    }
    pub fn new_from(&self) -> WIGroup {
        use WIGroup::*;
        match self {
            FontChange(f,r,b,_) => FontChange(f.clone(),r.clone(),*b,vec!()),
            ColorChange(c,r,_) => ColorChange(c.clone(),r.clone(),vec!()),
            PDFLink(a,b,c,d,_) => PDFLink(a.clone(),b.clone(),c.clone(),d.clone(),vec!()),
            PdfMatrixSave(r,b,v) => {
                match v.iter().find(|x| match x {
                    Whatsit::Simple(SimpleWI::PdfMatrix(a,b,c,d,o)) => true,
                    _ => false
                }) {
                    None => PdfMatrixSave(r.clone(),*b,vec!()),
                    Some(p) => PdfMatrixSave(r.clone(),*b,vec!(p.clone()))
                }
            }
            ColorEnd(_) | LinkEnd(_) | PdfRestore(_) => unreachable!()
        }
    }
    pub fn width(&self) -> i32 {
        todo!()
    }
    pub fn height(&self) -> i32 {
        todo!( )
    }
    pub fn depth(&self) -> i32 { todo!( )}
    pub fn closesWithGroup(&self) -> bool {
        match self {
            WIGroup::FontChange(_,_,b,_) => !*b,
            _ => false
        }
    }
}

#[derive(Clone)]
pub enum ActionSpec {
    User(TeXStr),
    GotoNum(i32),
    //    file   name    window
    File(TeXStr,TeXStr,Option<TeXStr>),
    FilePage(TeXStr,i32,Option<TeXStr>),
    Name(TeXStr),
    Page(i32)
}

#[derive(Clone)]
pub enum AlignBlock {
    Noalign(Vec<Whatsit>),
    Block(Vec<(Vec<Whatsit>,Skip)>)
}

//                      rule           attr            pagespec      colorspace         boxspec          file          image
#[derive(Clone)]
pub struct Pdfximage(pub TeXStr,pub Option<TeXStr>,pub Option<i32>,pub Option<i32>,pub Option<TeXStr>,pub PathBuf,pub Option<DynamicImage>);

#[derive(Clone)]
pub enum SimpleWI {
    Img(Pdfximage,Option<SourceFileReference>),
    //                                  height       width      depth
    VRule(Option<SourceFileReference>,Option<i32>,Option<i32>,Option<i32>),
    HRule(Option<SourceFileReference>,Option<i32>,Option<i32>,Option<i32>),
    VFil(Option<SourceFileReference>),
    VFill(Option<SourceFileReference>),
    VSkip(Skip,Option<SourceFileReference>),
    HSkip(Skip,Option<SourceFileReference>),
    MSkip(MuSkip,Option<SourceFileReference>),
    HFil(Option<SourceFileReference>),
    HFill(Option<SourceFileReference>),
    Penalty(i32),
    PdfLiteral(TeXStr,Option<SourceFileReference>),
    //          attr            resource
    Pdfxform(Option<TeXStr>,Option<TeXStr>,TeXBox,Option<SourceFileReference>),
    Raise(i32,TeXBox,Option<SourceFileReference>),
    MoveRight(i32,TeXBox,Option<SourceFileReference>),
    VKern(i32,Option<SourceFileReference>),
    HKern(i32,Option<SourceFileReference>),
    PdfDest(TeXStr,TeXStr,Option<SourceFileReference>),
    Halign(Skip,Vec<(Vec<Token>,Vec<Token>,Skip)>,Vec<AlignBlock>,Option<SourceFileReference>),
    Valign(Skip,Vec<(Vec<Token>,Vec<Token>,Skip)>,Vec<AlignBlock>,Option<SourceFileReference>),
    Hss(Option<SourceFileReference>),
    Vss(Option<SourceFileReference>),
    Indent(i32,Option<SourceFileReference>),
    Mark(Vec<Token>,Option<SourceFileReference>),
    Leaders(Box<Whatsit>,Option<SourceFileReference>),
    PdfMatrix(f32,f32,f32,f32,Option<SourceFileReference>),
    Left(Box<Whatsit>,Option<SourceFileReference>),
    Middle(Box<Whatsit>,Option<SourceFileReference>),
    Right(Box<Whatsit>,Option<SourceFileReference>),
}
impl SimpleWI {
    pub fn as_xml_internal(&self,prefix: String) -> String {
        use SimpleWI::*;
        match self {
            VRule(_,_,_,_) =>
                "\n".to_string() + &prefix + "<vrule width=\"" + &dimtostr(self.width()) +
                    "\" height=\"" + &dimtostr(self.height()) + "\" depth=\"" + &dimtostr(self.depth()) + "\"/>",
            HRule(_,_,_,_) =>
                "\n".to_string() + &prefix + "<hrule width=\"" + &dimtostr(self.width()) +
                    "\" height=\"" + &dimtostr(self.height()) + "\" depth=\"" + &dimtostr(self.depth()) + "\"/>",
            Penalty(i) => "\n".to_string() + &prefix + "<penalty val=\"" + &i.to_string() + "\"/>",
            VFil(_) => "\n".to_string() + &prefix + "<vfil/>",
            VFill(_) => "\n".to_string() + &prefix + "<vfill/>",
            HFil(_) => "\n".to_string() + &prefix + "<hfil/>",
            HFill(_) => "\n".to_string() + &prefix + "<hfill/>",
            PdfDest(a,b,_) => "\n".to_string() + &prefix + "<pdfdest a=\"" + a.to_string().as_str() + "\" b=\"" + b.to_string().as_str() + "\"/>",
            VSkip(i,_) => "\n".to_string() + &prefix + "<vskip val=\"" + &i.to_string() + "\"/>",
            HSkip(i,_) => "\n".to_string() + &prefix + "<hskip val=\"" + &i.to_string() + "\"/>",
            VKern(i,_) => "\n".to_string() + &prefix + "<vkern val=\"" + &dimtostr(*i) + "\"/>",
            HKern(i,_) => "\n".to_string() + &prefix + "<hkern val=\"" + &dimtostr(*i) + "\"/>",
            Indent(i,_) => "\n".to_string() + &prefix + "<indent val=\"" + &dimtostr(*i) + "\"/>",
            Halign(_,_,v,_) => {
                let mut ret = "\n".to_string() + &prefix + "<halign>";
                for block in v { match block {
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
                        for (l,_) in ls {
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
            },
            Valign(_,_,v,_) => {
                let mut ret = "\n".to_string() + &prefix + "<valign>";
                for block in v { match block {
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
                        for (l,_) in ls {
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
            },
            Raise(d,bx,_) => {
                let mut ret = "\n".to_string() + &prefix + "<raise by=\"" + &dimtostr(*d) + "\">";
                ret += &bx.as_xml_internal(prefix.clone() + "  ");
                ret + "\n" + &prefix + "</raise>"
            },
            MoveRight(d,bx,_) => {
                let mut ret = "\n".to_string() + &prefix + "<moveright by=\"" + &dimtostr(*d) + "\">";
                ret += &bx.as_xml_internal(prefix.clone() + "  ");
                ret + "\n" + &prefix + "</moveright>"
            },
            Hss(_) => "<hss/>".to_string(),
            Vss(_) => "<vss/>".to_string(),
            MSkip(sk,_) => "<mskip skip=\"".to_string() + &sk.to_string() + "\"/>",
            Mark(_,_) => "".to_string(),
            Leaders(bx,_) => "<leaders>".to_string() + &bx.as_xml_internal(prefix) + "</leaders>",
            PdfLiteral(s,_) => "<pdfliteral value=\"".to_string() + &s.to_string() + "\"/>",
            PdfMatrix(a,b,c,d,_) => "<pdfmatrix a=\"".to_string() + &a.to_string() + "\" b=\"" + &b.to_string() + "\" c=\"" + &c.to_string() + "\" d=\"" + &d.to_string() + "\"/>",
            Img(Pdfximage(rule,attr,pagespec,colorspace,boxspec,file,_),_) => {
                let mut ret = "\n".to_string() + &prefix + "<pdfximage rule=\"" + &rule.to_string() + "\"";
                match attr {
                    None => (),
                    Some(a) => {
                        ret += " attr=\"";
                        ret += &a.to_string();
                        ret += "\""
                    }
                }
                match pagespec {
                    None => (),
                    Some(a) => {
                        ret += " pagespec=\"";
                        ret += &a.to_string();
                        ret += "\""
                    }
                }
                match colorspace {
                    None => (),
                    Some(a) => {
                        ret += " colorspace=\"";
                        ret += &a.to_string();
                        ret += "\""
                    }
                }
                match boxspec {
                    None => (),
                    Some(a) => {
                        ret += " boxspec=\"";
                        ret += &a.to_string();
                        ret += "\""
                    }
                }
                ret += " file=\"";
                ret += file.to_str().unwrap();
                ret += "\"";
                ret + "/>"
            },
            Pdfxform(_,_,_,_) => todo!(),
            Left(w,_) => "<left>".to_string() + &w.as_xml_internal(prefix) + "</left>",
            Middle(w,_) => "<middle>".to_string() + &w.as_xml_internal(prefix) + "</middle>",
            Right(w,_) => "<right>".to_string() + &w.as_xml_internal(prefix) + "</right>"
        }
    }
    //                      rule           attr            pagespec      colorspace         boxspec          file          image
    //pub struct Pdfximage(pub TeXStr,pub Option<TeXStr>,pub Option<i32>,pub Option<i32>,pub Option<TeXStr>,pub PathBuf,pub DynamicImage);
    pub fn has_ink(&self) -> bool {
        use SimpleWI::*;
        match self {
            VRule(_,_,_,_) | HRule(_,_,_,_) | Img(_,_) | Left(_,_) | Right(_,_) | Middle(_,_) => true,
            VFil(_) | VFill(_) | VSkip(_,_) | HSkip(_,_) | HFil(_) | HFill(_) | Penalty(_) |
            PdfLiteral(_,_) | Pdfxform(_,_,_,_) | VKern(_,_) | HKern(_,_) | PdfDest(_,_,_)
            | Hss(_) | Vss(_) | Indent(_,_) | MSkip(_,_) | Mark(_,_) | PdfMatrix(_,_,_,_,_) => false,
            Raise(_,bx,_) => bx.has_ink(),
            MoveRight(_,bx,_) => bx.has_ink(),
            Leaders(w,_) => w.has_ink(),
            Halign(_,_,ab,_) => {
                for v in ab {
                    match v {
                        AlignBlock::Noalign(v) => for c in v { if c.has_ink() {return true} }
                        AlignBlock::Block(v) => for (iv,_) in v { for c in iv { if c.has_ink() {return true} } }
                    }
                }
                false
            }
            Valign(_,_,ab,_) => {
                for v in ab {
                    match v {
                        AlignBlock::Noalign(v) => for c in v { if c.has_ink() {return true} }
                        AlignBlock::Block(v) => for (iv,_) in v { for c in iv { if c.has_ink() {return true} } }
                    }
                }
                false
            }
        }
    }
    pub fn width(&self) -> i32 {
        use SimpleWI::*;
        match self {
            VKern(_,_) | Penalty(_) | VSkip(_,_) | HFill(_) | HFil(_) | VFil(_) | VFill(_)
                | Hss(_) | Vss(_) | PdfDest(_,_,_) | Mark(_,_) | PdfMatrix(_,_,_,_,_)
                | PdfLiteral(_,_) | Pdfxform(_,_,_,_) => 0,
            HKern(i,_) => *i,
            VRule(_,_,w,_) => w.unwrap_or(26214),
            HRule(_,_,w,_) => w.unwrap_or(0),
            HSkip(sk,_) => sk.base,
            MSkip(sk,_) => sk.base,
            Indent(i,_) => *i,
            Img(Pdfximage(_,_,_,_,_,_,Some(img)),_) => img.width() as i32 * 65536,
            Img(Pdfximage(_,_,_,_,_,_,_),_) => 65536,
            Halign(sk,_,bxs,_) => {
                let mut width:i32 = 0;
                for b in bxs {
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
                            for (v,s) in ls {
                                w += s.base;
                                for c in v.iter_wi() { w += c.width() }
                            }
                            if w > width { width = w }
                        }
                    }
                }
                width + sk.base
            }
            Valign(_,_,bxs,_) => {
                let mut width:i32 = 0;
                for b in bxs {
                    match b {
                        AlignBlock::Noalign(v) => {
                            for c in v.iter_wi() {
                                width += c.width();
                            }
                        }
                        AlignBlock::Block(ls) => {
                            let mut wd:i32 = 0;
                            for (v,s) in ls {
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
            Raise(_,b,_) => b.width(),
            MoveRight(i,n,_) => min(0,n.width() + i),
            Leaders(b,_) => b.width(), // TODO maybe
            Left(w,_) => w.width(),
            Right(w,_) => w.width(),
            Middle(w,_) => w.width(),
            _ => {
                todo!()
            }
        }
    }
    pub fn height(&self) -> i32 {
        use SimpleWI::*;
        match self {
            HKern(_,_) | Penalty(_) | HSkip(_,_) | HFill(_) | HFil(_) | VFil(_) | VFill(_)
                | Hss(_) | Vss(_) | Indent(_,_) | MSkip(_,_) | PdfDest(_,_,_) | Mark(_,_)
                | PdfMatrix(_,_,_,_,_)| PdfLiteral(_,_)| Pdfxform(_,_,_,_) => 0,
            Img(Pdfximage(_,_,_,_,_,_,Some(img)),_) => img.height() as i32 * 65536,
            Img(Pdfximage(_,_,_,_,_,_,_),_) => 65536,
            VRule(_,h,_,_) => h.unwrap_or(0),
            HRule(_,h,_,_) => h.unwrap_or(26214),
            VKern(i,_) => *i,
            Leaders(b,_) => b.height(),
            VSkip(sk,_) => sk.base,
            Halign(_,_,bxs,_) => {
                let mut height:i32 = 0;
                for b in bxs {
                    match b {
                        AlignBlock::Noalign(v) => {
                            for c in v.iter_wi() {
                                height += c.height();
                            }
                        }
                        AlignBlock::Block(ls) => {
                            let mut ht:i32 = 0;
                            for (v,s) in ls {
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
            Valign(sk,_,bxs,_) => {
                let mut height:i32 = 0;
                for b in bxs {
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
                            for (v,s) in ls {
                                w += s.base;
                                for c in v.iter_wi() { w += c.height()}
                            }
                            if w > height { height = w }
                        }
                    }
                }
                height + sk.base
            }
            Raise(r,b,_) => min(0,b.height() + r),
            MoveRight(_,b,_) => b.height(),
            Left(w,_) => w.height(),
            Right(w,_) => w.height(),
            Middle(w,_) => w.height(),
            _ => {
                todo!()
            }
        }
    }
    pub fn depth(&self) -> i32 {
        use SimpleWI::*;
        match self {
            HKern(_,_) | VKern(_,_) | Penalty(_) | HSkip(_,_) | VSkip(_,_)
                | HFill(_) | HFil(_) | VFil(_) | VFill(_) | Halign(_,_,_,_) | Valign(_,_,_,_)
                | Hss(_) | Vss(_) | Indent(_,_) | MSkip(_,_) | PdfDest(_,_,_) | Mark(_,_)
                | Img(_,_) | PdfMatrix(_,_,_,_,_) | PdfLiteral(_,_)| Pdfxform(_,_,_,_) => 0,
            VRule(_,_,_,d) => d.unwrap_or(0),
            HRule(_,_,_,d) => d.unwrap_or(0),
            Raise(r,b,_) => max(b.depth() - r,0),
            MoveRight(_,b,_) => b.depth(),
            Leaders(b,_) => b.depth(),
            Left(w,_) => w.depth(),
            Right(w,_) => w.depth(),
            Middle(w,_) => w.depth(),
            _ => todo!()
        }
    }
}


pub struct ExecutableWhatsit {
    pub _apply : Box<dyn FnOnce(&Interpreter) -> Result<(),TeXError>>
}

pub trait ExtWhatsit {
    fn name(&self) -> TeXStr;
    fn reference(&self) -> Option<SourceFileReference>;
    fn children(&self) -> Vec<Whatsit>;
    fn isGroup(&self) -> bool;
    fn height(&self) -> i32;
    fn width(&self) -> i32;
    fn depth(&self) -> i32;
    fn has_ink(&self) -> bool;
    fn as_xml_internal(&self,prefix:String) -> String;
}