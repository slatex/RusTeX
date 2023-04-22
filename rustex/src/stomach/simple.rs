use std::any::Any;
use std::cmp::{max, min};
use std::io::Cursor;
use std::path::PathBuf;
use base64::Engine;
use image::DynamicImage;
use image::imageops::FilterType;
use crate::interpreter::dimensions::{dimtostr, MuSkip, numtostr, round, Skip, SkipDim};
use crate::references::SourceFileReference;
use crate::stomach::boxes::{HBox, TeXBox, VBox};
use crate::stomach::colon::ColonMode;
use crate::stomach::html::{dimtohtml, HTML_NS, HTMLChild, HTMLColon, HTMLNode, HTMLParent, HTMLSCALE, HTMLStr};
use crate::stomach::math::MathChar;
use crate::stomach::Whatsit;
use crate::stomach::whatsits::{HasWhatsitIter, WhatsitTrait};
use crate::{htmlliteral, htmlnode, htmlparent, setwidth, Token, withlinescale};
use crate::fonts::ArcFont;
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
    PDFInfo(PDFInfo),
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
    External(Box<dyn ExternalWhatsit>)
}

impl Clone for Box<dyn ExternalWhatsit> {
    fn clone(&self) -> Self {
        self.clone_box()
    }
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
        SimpleWI::PDFInfo(g) => PDFInfo::$e(g $(,$tl)*),
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
    //fn get_par_width(&self) -> Option<i32> { pass_on!(self,get_par_width) }
    //fn get_par_widths(&self) -> Vec<i32> { pass_on!(self,get_par_widths) }
    fn get_ref(&self) -> Option<SourceFileReference> { pass_on!(self,get_ref) }
    fn as_whatsit(self) -> Whatsit { Whatsit::Simple(self) }
    fn width(&self) -> i32 { pass_on!(self,width) }
    fn height(&self) -> i32 { pass_on!(self,height) }
    fn depth(&self) -> i32 { pass_on!(self,depth) }
    fn as_xml_internal(&self, prefix: String) -> String {
        pass_on!(self,as_xml_internal,prefix)
    }
    fn has_ink(&self) -> bool { pass_on!(self,has_ink) }
    fn normalize(self, mode: &ColonMode, ret: &mut Vec<Whatsit>, scale: Option<f32>) {
        pass_on!(self,normalize,mode,ret,scale)
    }
    fn as_html(self, mode: &ColonMode, colon: &mut HTMLColon, node_top: &mut Option<HTMLParent>) {
        pass_on!(self,as_html,mode,colon,node_top)
    }
}

trait Normalizable {
    fn normalize(self, mode: &ColonMode, ret: &mut Vec<Whatsit>, scale: Option<f32>);
    fn as_html(self,mode:&ColonMode,colon:&mut HTMLColon,node_top:&mut Option<HTMLParent>);
}
impl Normalizable for Box<dyn ExternalWhatsit> {
    fn normalize(self, mode: &ColonMode, ret: &mut Vec<Whatsit>, scale: Option<f32>) {
        self.normalize_dyn(mode,ret,scale)
    }
    fn as_html(self,mode:&ColonMode,colon:&mut HTMLColon,node_top:&mut Option<HTMLParent>) {
        self.as_html_dyn(mode,colon,node_top)
    }
}

#[derive(Clone)]
pub enum AlignBlock {
    Noalign(Vec<Whatsit>),
    Block(Vec<(Vec<Whatsit>,Skip,usize)>)
}

// -------------------------------------------------------------------------------------------------
pub enum ExternalParam{
    String(TeXStr),
    Whatsits(Vec<Whatsit>),
    Num(i32)
}

pub trait ExternalWhatsit:Any+WhatsitTrait+Send+Sync {
    fn name(&self) -> TeXStr;
    fn params(&self,name:&str) -> Option<ExternalParam>;
    fn sourceref(&self) -> &Option<SourceFileReference>;
    fn clone_box(&self) -> Box<dyn ExternalWhatsit>;
    fn normalize_dyn(self:Box<Self>, mode: &ColonMode, ret: &mut Vec<Whatsit>, scale: Option<f32>);
    fn as_html_dyn(self:Box<Self>,mode:&ColonMode,colon:&mut HTMLColon,node_top:&mut Option<HTMLParent>);
}

#[derive(Clone)]
pub struct PDFImageRule {
    pub string : TeXStr,
    pub width : Option<i32>,
    pub height: Option<i32>,
    pub depth: Option<i32>
}

#[derive(Clone)]
pub struct PDFXImage{
    pub rule: PDFImageRule,
    pub attr:Option<TeXStr>,
    pub pagespec:Option<i32>,
    pub colorspace:Option<i32>,
    pub boxspec:Option<TeXStr>,
    pub filename:PathBuf,
    pub image:Option<DynamicImage>,
    pub sourceref:Option<SourceFileReference>,
    pub _width:Option<i32>,
    pub _height:Option<i32>
}
impl WhatsitTrait for PDFXImage {
    //fn get_par_width(&self) -> Option<i32> { None }
    //fn get_par_widths(&self) -> Vec<i32> { vec!() }
    fn get_ref(&self) -> Option<SourceFileReference> { self.sourceref.clone() }
    fn as_whatsit(self) -> Whatsit {
        Whatsit::Simple(SimpleWI::PDFXImage(self))
    }
    fn width(&self) -> i32 {
        match self._width {
            Some(w) => w,
            None => match &self.image {
                Some(img) => img.width() as i32 * 65536,
                _ => 65536
            }
        }
    }
    fn height(&self) -> i32 {
        match self._height {
            Some(h) => h,
            None => match &self.image {
                Some(img) => img.height() as i32 * 65536,
                _ => 65536
            }
        }
    }
    fn depth(&self) -> i32 { 0 }

    fn as_xml_internal(&self, prefix: String) -> String {
        let mut ret = "\n".to_string() + &prefix + "<pdfximage rule=\"" + &self.rule.string.to_string() + "\"";
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
    fn normalize(mut self, _: &ColonMode, ret: &mut Vec<Whatsit>, scale: Option<f32>) {
        match scale {
            Some(f) => {
                self._width = Some(((self.width() as f32) * f).round() as i32);
                self._height = Some(((self.height() as f32) * f).round() as i32);
                ret.push(self.as_whatsit())
            }
            _ => ret.push(self.as_whatsit())
        }
    }
    fn as_html(self, mode: &ColonMode, colon: &mut HTMLColon, node_top: &mut Option<HTMLParent>) {
        if *mode == ColonMode::M {
            return HBox {
                children: vec!(self.as_whatsit()),
                spread: 0,
                _width: None,
                _height: None,
                _depth: None,
                _to: None,
                rf: None,
                lineheight:None
            }.as_html(mode, colon, node_top)
        }
        match self.image {
            Some(ref img) => {
                let target_width = 5 * ((HTMLSCALE as f64) * round(self.width())).round() as u32;
                let target_height = 5 * ((HTMLSCALE as f64) * round(self.height())).round() as u32;
                /*println!("Original width: {}\nOriginal height:{}\nthis.width:{}\nthis.height:{}\ntarget_width:{}\ntarget_height:{}",
                    img.width(),img.height(),dimtohtml(self.width()),dimtohtml(self.height()),target_width,target_height
                );*/
                let nimg = if img.width() > target_width || img.height() > target_height {
                    image::imageops::resize(
                        &img.clone().into_rgba8(),
                        target_width,
                        target_height,
                        FilterType::Gaussian
                    )
                } else { img.clone().into_rgba8() };
                let mut buf = Cursor::new(vec!());//Vec<u8> = vec!();
                match nimg.write_to(&mut buf, image::ImageOutputFormat::Png/*Jpeg(254)*/) {
                    Ok(_) => {
                        let res_base64 = "data:image/png;base64,".to_string() + &base64::engine::general_purpose::STANDARD.encode(&buf.into_inner());
                        htmlnode!(colon,img,self.sourceref.clone(),"",node_top,i => {
                            i.attr("src".into(),res_base64.into());
                            i.attr("width".into(),dimtohtml(self.width()));
                            i.attr("height".into(),dimtohtml(self.height()));
                        })
                    }
                    Err(e) => ()
                        //println!("{}",e)
                }
            }
            _ => ()
        }
    }
}

#[derive(Clone)]
pub struct VRule {
    pub height:Option<i32>,
    pub width:Option<i32>,
    pub depth:Option<i32>,
    pub font:ArcFont,
    pub sourceref:Option<SourceFileReference>
}
impl WhatsitTrait for VRule {
    //fn get_par_width(&self) -> Option<i32> { Some(self.width()) }
    //fn get_par_widths(&self) -> Vec<i32> { vec!(self.width()) }
    fn get_ref(&self) -> Option<SourceFileReference> { self.sourceref.clone() }
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
        true//self.width() != 0 && (self.height() != 0 || self.depth() != 0)
    }
    fn normalize(self, md: &ColonMode, ret: &mut Vec<Whatsit>, _: Option<f32>) {
        match md {
            ColonMode::H | ColonMode::P | ColonMode::M => ret.push(self.as_whatsit()),
            _ => ret.push(HRule {
                height: self.height,
                width: self.width,
                depth: self.depth,
                sourceref: self.sourceref,
            }.as_whatsit())
        }
        /*if self.width() != 0 && (self.height.unwrap_or(10) != 0 || self.depth() != 0) {*/ //}
    }
    fn as_html(mut self, mode: &ColonMode, colon: &mut HTMLColon, node_top: &mut Option<HTMLParent>) {
        match mode {
            ColonMode::M => htmlnode!(colon,mspace,self.sourceref.clone(),"vrule",node_top,n => {
                if self.height() != 0 {n.attr("height".into(),dimtohtml(self.height()))}
                if self.width() != 0 {n.attr("width".into(),dimtohtml(self.height()))}
                if self.depth() != 0 {n.attr("depth".into(),dimtohtml(self.height()))}
                n.style("background".into(),match &colon.state.currcolor {
                    Some(c) => HTMLStr::from("#") + c,
                    None => "#000000".into()
                });
            }),
            /*_ => htmlnode!(colon,div,self.sourceref.clone(),"rustex-vrule",node_top,n => {
                let width = self.width();
                if 3.1*(width as f32) > (colon.textwidth as f32) {
                    setwidth!(colon,width,n);
                } else {
                    n.style("width".into(),dimtohtml(self.width()));
                }
                n.style("background".into(),match &colon.state.currcolor {
                    Some(c) => HTMLStr::from("#") + c,
                    None => "#000000".into()
                });
                n.style("vertical-align".into(),"text-bottom".into());
                match (self.depth,self.height) {
                    (None,None) => {
                        let ht : HTMLStr = <&str as Into<HTMLStr>>::into("min(100%,") + dimtohtml(self.font.get_at() )+ ")";
                        n.style("height".into(),ht.clone());
                    }
                    (None,Some(ht)) => {
                        n.style("margin-bottom".into(),"-0.5ex".into());
                        let retstr: HTMLStr = <&str as Into<HTMLStr>>::into("calc(0.5ex + ") + dimtohtml(self.height() + self.depth()) + ")";
                        n.style("height".into(),retstr.clone());
                    }
                    _ => {
                        n.style("height".into(),dimtohtml(self.height() + self.depth()));
                        n.style("margin-bottom".into(),dimtohtml(-self.depth()));
                    }
                }
            }),*/
            _ if self.depth.is_none() && self.height.is_none() => match mode {
                ColonMode::P => htmlnode!(colon,div,self.sourceref.clone(),"rustex-vrule",node_top,n => {
                    let width = self.width();
                    if 3.1*(width as f32) > (colon.textwidth as f32) {
                        setwidth!(colon,width,n);
                    } else {
                        n.style("width".into(),dimtohtml(self.width()));
                        n.style("min-width".into(),dimtohtml(self.width()));
                    }
                    n.style("background".into(),match &colon.state.currcolor {
                        Some(c) => HTMLStr::from("#") + c,
                        None => "#000000".into()
                    });
                    n.style("min-height".into(),dimtohtml(self.font.get_at()))
                }),
                _ => htmlnode!(colon,div,self.sourceref.clone(),"rustex-vrule",node_top,n => {
                    let width = self.width();
                    if 3.1*(width as f32) > (colon.textwidth as f32) {
                        setwidth!(colon,width,n);
                    } else {
                        n.style("width".into(),dimtohtml(self.width()));
                        n.style("min-width".into(),dimtohtml(self.width()));
                    }
                    n.style("background".into(),match &colon.state.currcolor {
                        Some(c) => HTMLStr::from("#") + c,
                        None => "#000000".into()
                    });
                    n.style("align-self".into(),"stretch".into());
                }),
            }
            /*_ => htmlnode!(colon,div,self.sourceref.clone(),"rustex-vrule-container",node_top,m => {
                let width = self.width();
                if 3.1*(width as f32) > (colon.textwidth as f32) {
                    setwidth!(colon,width,m);
                } else {
                    m.style("width".into(),dimtohtml(self.width()));
                    m.style("min-width".into(),dimtohtml(self.width()));
                }
                htmlnode!(colon,div,self.sourceref.clone(),"rustex-vrule",htmlparent!(m),n => {
                    n.style("width".into(),"100%".into());
                    n.style("background".into(),match &colon.state.currcolor {
                        Some(c) => HTMLStr::from("#") + c,
                        None => "#000000".into()
                    });
                    n.style("vertical-align".into(),"text-bottom".into());
                    match (self.depth,self.height) {
                        (None,None) => {
                            let ht : HTMLStr = <&str as Into<HTMLStr>>::into("min(100%,") + dimtohtml(self.font.get_at() )+ ")";
                            n.style("height".into(),"100%".into());
                            m.style("height".into(),ht);
                        }
                        (None,Some(ht)) => {
                            n.style("margin-bottom".into(),"-0.5ex".into());
                            let retstr: HTMLStr = <&str as Into<HTMLStr>>::into("calc(0.5ex + ") + dimtohtml(self.height() + self.depth()) + ")";
                            n.style("height".into(),retstr.clone());
                            m.style("height".into(),retstr.clone());
                        }
                        _ => {
                            n.style("height".into(),dimtohtml(self.height() + self.depth()));
                            m.style("height".into(),dimtohtml(self.height() + self.depth()));
                            n.style("margin-bottom".into(),dimtohtml(-self.depth()));
                        }
                    }
                });
            }),*/
            _ => htmlnode!(colon,div,self.sourceref.clone(),"rustex-vrule-container",node_top,m => {
                m.style("height".into(),dimtohtml(self.height() + self.depth()));
                let width = self.width();
                if 3.1*(width as f32) > (colon.textwidth as f32) {
                    setwidth!(colon,width,m);
                } else {
                    m.style("width".into(),dimtohtml(self.width()));
                    m.style("min-width".into(),dimtohtml(self.width()));
                }
                htmlnode!(colon,div,self.sourceref.clone(),"rustex-vrule",htmlparent!(m),n => {
                    n.style("width".into(),"100%".into());
                    n.style("background".into(),match &colon.state.currcolor {
                        Some(c) => HTMLStr::from("#") + c,
                        None => "#000000".into()
                    });
                    n.style("vertical-align".into(),"baseline".into());
                    match (mode,self.depth) {
                        (ColonMode::P,None) => {
                            n.style("margin-bottom".into(),"-0.5ex".into());
                            let retstr: HTMLStr = "calc(0.5ex + ".into();
                            n.style("height".into(),retstr + dimtohtml(self.height() + self.depth()) + ")");
                        }
                        _ => {
                            n.style("height".into(),dimtohtml(self.height() + self.depth()));
                            n.style("margin-bottom".into(),dimtohtml(-self.depth()));
                        }
                    }
                })
            })
        }
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
    /*fn get_par_width(&self) -> Option<i32> { if self.width() == 0 {None} else {Some(self.width())} }
    fn get_par_widths(&self) -> Vec<i32> { match self.get_par_width() {
        Some(w) => vec!(w),
        _ => vec!()
    } }*/
    fn get_ref(&self) -> Option<SourceFileReference> { self.sourceref.clone() }
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
    fn normalize(self, mode: &ColonMode, ret: &mut Vec<Whatsit>, _: Option<f32>) {
        if self.width.unwrap_or(10) != 0 && (self.height() != 0 || self.depth() != 0) {
            match mode {
                ColonMode::V =>
                    ret.push(self.as_whatsit()),
                _ => ret.push(VRule {
                    height: self.height,
                    width: self.width,
                    depth: self.depth,
                    font: Default::default(),
                    sourceref: None,
                }.as_whatsit())
            }
        }
    }
    fn as_html(self, _: &ColonMode, colon: &mut HTMLColon, node_top: &mut Option<HTMLParent>) {
        htmlnode!(colon,div,self.sourceref.clone(),"rustex-hrule-container",node_top,m => {
            m.style("height".into(),dimtohtml(self.height() + self.depth()));
            if self.width() != 0 {
                setwidth!(colon,self.width(),m);
            } else {
                m.style("width".into(),"100%".into());
            }
        htmlnode!(colon,div,self.sourceref.clone(),"rustex-hrule",htmlparent!(m),n => {
            n.style("width".into(),"100%".into());
            n.style("height".into(),dimtohtml(self.height() + self.depth()));
            n.style("min-height".into(),dimtohtml(self.height() + self.depth()));
            n.style("background".into(),match &colon.state.currcolor {
                Some(c) => HTMLStr::from("#") + c,
                None => "#000000".into()
            });
            if self.depth() != 0 {
                n.style("margin-bottom".into(),dimtohtml(-self.depth()))
            }
        })})
    }
}

#[derive(Clone)]
pub struct VSkip {
    pub skip:Skip,
    pub sourceref:Option<SourceFileReference>
}
impl WhatsitTrait for VSkip {
    //fn get_par_width(&self) -> Option<i32> { None }
    //fn get_par_widths(&self) -> Vec<i32> { vec!() }
    fn get_ref(&self) -> Option<SourceFileReference> { self.sourceref.clone() }
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
    fn normalize(self, _: &ColonMode, ret: &mut Vec<Whatsit>, _: Option<f32>) {
        match ret.last_mut() {
            Some(Whatsit::Simple(SimpleWI::VSkip(sk2))) => {
                sk2.skip = self.skip + sk2.skip;
                if sk2.skip.base == 0 {
                    ret.pop();
                }
            },
            _ => ret.push(self.as_whatsit())
        }
    }
    fn as_html(self, _: &ColonMode, colon: &mut HTMLColon, node_top: &mut Option<HTMLParent>) {
        htmlnode!(colon,div,self.sourceref,"rustex-vskip",node_top,node => {
            node.style("margin-bottom".into(),dimtohtml(self.skip.base));
        })
    }
}

#[derive(Clone)]
pub struct HSkip {
    pub skip:Skip,
    pub sourceref:Option<SourceFileReference>
}
impl WhatsitTrait for HSkip {
    //fn get_par_width(&self) -> Option<i32> { None }
    //fn get_par_widths(&self) -> Vec<i32> { vec!() }
    fn get_ref(&self) -> Option<SourceFileReference> { self.sourceref.clone() }
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
    fn normalize(self, _: &ColonMode, ret: &mut Vec<Whatsit>, _: Option<f32>) {
        match ret.last_mut() {
            Some(Whatsit::Simple(SimpleWI::HSkip(sk2))) => {
                sk2.skip = self.skip + sk2.skip;
                if sk2.skip.base == 0 {
                    ret.pop();
                }
            },
            _ => ret.push(self.as_whatsit())
        }
    }
    fn as_html(self, mode: &ColonMode, colon: &mut HTMLColon, node_top: &mut Option<HTMLParent>) {
        match mode {
            ColonMode::H | ColonMode::P =>
                htmlnode!(colon,div,self.sourceref,"rustex-hskip",node_top,node => {
                    node.style("margin-left".into(),dimtohtml(self.skip.base));
                }),
            ColonMode::M =>
                htmlnode!(colon,mspace,self.sourceref,"rustex-mskip",node_top,a => {
                    if self.skip.base < 0 {
                        a.style("margin-left".into(),dimtohtml(self.skip.base));
                    }
                    a.attr("width".into(),dimtohtml(self.skip.base))
                }),
            _ => ()//TeXErr!("TODO")
        }
    }
}

#[derive(Clone)]
pub struct MSkip {
    pub skip:MuSkip,
    pub sourceref:Option<SourceFileReference>
}
impl WhatsitTrait for MSkip {
    //fn get_par_width(&self) -> Option<i32> { None }
    //fn get_par_widths(&self) -> Vec<i32> { vec!() }
    fn get_ref(&self) -> Option<SourceFileReference> { self.sourceref.clone() }
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
    fn normalize(self, _: &ColonMode, ret: &mut Vec<Whatsit>, _: Option<f32>) {
        match ret.last_mut() {
            Some(Whatsit::Simple(SimpleWI::MSkip(sk2))) => {
                sk2.skip = self.skip + sk2.skip;
                if sk2.skip.base == 0 {
                    ret.pop();
                }
            },
            _ => ret.push(self.as_whatsit())
        }
    }
    fn as_html(self, mode: &ColonMode, colon: &mut HTMLColon, node_top: &mut Option<HTMLParent>) {
        match mode {
            ColonMode::M =>
                htmlnode!(colon,mspace,self.sourceref,"rustex-mskip",node_top,a => {
                    if self.skip.base < 0 {
                        a.style("margin-left".into(),(self.skip.get_em().to_string() + "em").into());
                    }
                    a.attr("width".into(),(self.skip.get_em().to_string() + "em").into()) // 1179648
                }),
            ColonMode::H | ColonMode::P =>
                htmlnode!(colon,div,self.sourceref,"rustex-hskip",node_top,node => {
                    node.style("margin-left".into(),dimtohtml(self.skip.base));
                }),
            _ => ()//TeXErr!("TODO")
        }
    }
}

#[derive(Clone)]
pub struct Penalty {
    pub penalty:i32,
    pub sourceref:Option<SourceFileReference>
}
impl WhatsitTrait for Penalty {
    //fn get_par_width(&self) -> Option<i32> { None }
    //fn get_par_widths(&self) -> Vec<i32> { vec!() }
    fn get_ref(&self) -> Option<SourceFileReference> { self.sourceref.clone() }
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
    fn normalize(self, mode: &ColonMode, ret: &mut Vec<Whatsit>, _: Option<f32>) {
        match (self.penalty,mode) {
            (p,ColonMode::H | ColonMode::P) if p <= -10000 => ret.push(self.as_whatsit()),
            _ => ()
        }
    }
    fn as_html(self, mode: &ColonMode, colon: &mut HTMLColon, node_top: &mut Option<HTMLParent>) {
        match mode {
            ColonMode::H | ColonMode::P if self.penalty <= -10000 =>
                htmlliteral!(colon,node_top,"<br/>"),
            _ => ()
        }
    }
}

#[derive(Clone)]
pub struct PDFLiteral {
    pub literal:TeXStr,
    pub sourceref:Option<SourceFileReference>
}
impl WhatsitTrait for PDFLiteral {
    //fn get_par_width(&self) -> Option<i32> { None }
    //fn get_par_widths(&self) -> Vec<i32> { vec!() }
    fn get_ref(&self) -> Option<SourceFileReference> { self.sourceref.clone() }
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
    fn normalize(self, _: &ColonMode, ret: &mut Vec<Whatsit>, _: Option<f32>) {
        ret.push(self.as_whatsit())
    }
    fn as_html(self, _: &ColonMode, _: &mut HTMLColon, _: &mut Option<HTMLParent>) {}
}

#[derive(Clone)]
pub struct PDFInfo {
    pub info:TeXStr,
    pub sourceref:Option<SourceFileReference>
}
impl WhatsitTrait for PDFInfo {
    //fn get_par_width(&self) -> Option<i32> { None }
    //fn get_par_widths(&self) -> Vec<i32> { vec!() }
    fn get_ref(&self) -> Option<SourceFileReference> { self.sourceref.clone() }
    fn as_whatsit(self) -> Whatsit {
        Whatsit::Simple(SimpleWI::PDFInfo(self))
    }
    fn width(&self) -> i32 { 0 }
    fn height(&self) -> i32 { 0 }
    fn depth(&self) -> i32 { 0 }
    fn as_xml_internal(&self, _: String) -> String {
        "<pdfinfo value=\"".to_string() + &self.info.to_string() + "\"/>"
    }
    fn has_ink(&self) -> bool { false }
    fn normalize(self, _: &ColonMode, ret: &mut Vec<Whatsit>, _: Option<f32>) {
        ret.push(self.as_whatsit())
    }
    fn as_html(self, _: &ColonMode, _: &mut HTMLColon, _: &mut Option<HTMLParent>) {}
}


#[derive(Clone)]
pub struct PDFXForm {
    pub attr:Option<TeXStr>,
    pub resource:Option<TeXStr>,
    pub content:TeXBox,
    pub sourceref:Option<SourceFileReference>
}
impl WhatsitTrait for PDFXForm {
    //fn get_par_width(&self) -> Option<i32> { self.content.get_par_width() }
    //fn get_par_widths(&self) -> Vec<i32> { self.content.get_par_widths() }
    fn get_ref(&self) -> Option<SourceFileReference> { self.sourceref.clone() }
    fn as_whatsit(self) -> Whatsit {
        Whatsit::Simple(SimpleWI::PDFXForm(self))
    }
    fn width(&self) -> i32 { 0 }
    fn height(&self) -> i32 { 0 }
    fn depth(&self) -> i32 { 0 }
    fn as_xml_internal(&self, _: String) -> String {
        "<pdfxform/>".to_string()//TeXErr!("TODO")
    }
    fn has_ink(&self) -> bool { false }
    fn normalize(self, _: &ColonMode, ret: &mut Vec<Whatsit>, _: Option<f32>) {
        ret.push(self.as_whatsit())
    }
    fn as_html(self, _: &ColonMode, _: &mut HTMLColon, _: &mut Option<HTMLParent>) {}
}

#[derive(Clone)]
pub struct Raise {
    pub dim:i32,
    pub content:TeXBox,
    pub sourceref:Option<SourceFileReference>
}
impl WhatsitTrait for Raise {
    //fn get_par_width(&self) -> Option<i32> { self.content.get_par_width() }
    //fn get_par_widths(&self) -> Vec<i32> { self.content.get_par_widths() }
    fn get_ref(&self) -> Option<SourceFileReference> { self.sourceref.clone() }
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
    fn normalize(self, mode: &ColonMode, ret: &mut Vec<Whatsit>, scale: Option<f32>) {
        if self.dim == 0 {
            return self.content.normalize(mode, ret, scale)
        }
        let mut nch: Vec<Whatsit> = vec!();
        let bx = self.content;
        match bx {
            TeXBox::H(bx) => {
                for c in bx.children { c.normalize(&ColonMode::H, &mut nch, scale) }
                if nch.is_empty() && (bx._width.is_none() || bx._width == Some(0)) { return }
                ret.push(Raise {
                    content: TeXBox::H(HBox {
                        children: nch,
                        spread: bx.spread,
                        _width: bx._width,
                        _height: bx._height,
                        _depth: bx._depth,
                        _to: bx._to,
                        rf: bx.rf,
                        lineheight:bx.lineheight
                    }),
                    dim:self.dim,
                    sourceref:self.sourceref
                }.as_whatsit())
            }
            TeXBox::V(bx) => {
                for c in bx.children { c.normalize(&ColonMode::V, &mut nch, scale) }
                if nch.is_empty() && (bx._height.is_none() || bx._height == Some(0)) { return }
                ret.push(Raise {
                    content: TeXBox::V(VBox {
                        children: nch,
                        spread: bx.spread,
                        _width: bx._width,
                        _height: bx._height,
                        _depth: bx._depth,
                        _to: bx._to,
                        rf: bx.rf,
                        tp:bx.tp
                    }),
                    dim:self.dim,
                    sourceref:self.sourceref
                }.as_whatsit())
            }
            _ => ()
        }
    }
    fn as_html(self, mode: &ColonMode, colon: &mut HTMLColon, node_top: &mut Option<HTMLParent>) {
        match mode {
            ColonMode::H | ColonMode::V | ColonMode::P =>
                htmlnode!(colon,div,self.sourceref,"rustex-raise",node_top,node => {
                node.style("bottom".into(),dimtohtml(self.dim));
                node.style("margin-top".into(),dimtohtml(self.dim));
                node.style("margin-bottom".into(),dimtohtml(-self.dim));
                self.content.as_html(mode,colon,htmlparent!(node))
            }),
            ColonMode::M =>
                htmlnode!(colon,mrow,self.sourceref,"rustex-raise",node_top,node => {
                node.style("bottom".into(),dimtohtml(self.dim));
                node.style("margin-top".into(),dimtohtml(self.dim));
                node.style("margin-bottom".into(),dimtohtml(-self.dim));
                self.content.as_html(mode,colon,htmlparent!(node))
            }),
            _ => ()//TeXErr!("TODO")
        }
    }
}

#[derive(Clone)]
pub struct MoveRight {
    pub dim:i32,
    pub content:TeXBox,
    pub sourceref:Option<SourceFileReference>
}
impl WhatsitTrait for MoveRight {
    //fn get_par_width(&self) -> Option<i32> { self.content.get_par_width() }
    //fn get_par_widths(&self) -> Vec<i32> { self.content.get_par_widths() }
    fn get_ref(&self) -> Option<SourceFileReference> { self.sourceref.clone() }
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
    fn normalize(self, mode: &ColonMode, ret: &mut Vec<Whatsit>, scale: Option<f32>) {
        if self.dim == 0 {
            return self.content.normalize(mode, ret, scale)
        }
        let mut nch: Vec<Whatsit> = vec!();
        let bx = self.content;
        match bx {
            TeXBox::H(bx) => {
                for c in bx.children { c.normalize(&ColonMode::H, &mut nch, scale) }
                if nch.is_empty() && (bx._width.is_none() || bx._width == Some(0)) { return }
                ret.push(MoveRight {
                    content: TeXBox::H(HBox {
                        children: nch,
                        spread: bx.spread,
                        _width: bx._width,
                        _height: bx._height,
                        _depth: bx._depth,
                        _to: bx._to,
                        rf: bx.rf,
                        lineheight:bx.lineheight
                    }),
                    dim:self.dim,
                    sourceref:self.sourceref
                }.as_whatsit())
            }
            TeXBox::V(bx) => {
                for c in bx.children { c.normalize(&ColonMode::V, &mut nch, scale) }
                if nch.is_empty() && (bx._height.is_none() || bx._height == Some(0)) { return }
                ret.push(MoveRight {
                    content: TeXBox::V(VBox {
                        children: nch,
                        spread: bx.spread,
                        _width: bx._width,
                        _height: bx._height,
                        _depth: bx._depth,
                        _to: bx._to,
                        rf: bx.rf,
                        tp:bx.tp
                    }),
                    dim:self.dim,
                    sourceref:self.sourceref
                }.as_whatsit())
            }
            _ => ()
        }
    }
    fn as_html(self, mode: &ColonMode, colon: &mut HTMLColon, node_top: &mut Option<HTMLParent>) {
        htmlnode!(colon,div,self.sourceref,"rustex-moveright",node_top,node => {
            node.style("margin-left".into(),dimtohtml(self.dim));
            self.content.as_html(mode,colon,htmlparent!(node))
        })
    }
}

#[derive(Clone)]
pub struct VKern {
    pub dim:i32,
    pub sourceref:Option<SourceFileReference>
}
impl WhatsitTrait for VKern {
    //fn get_par_width(&self) -> Option<i32> { None }
    //fn get_par_widths(&self) -> Vec<i32> { vec!() }
    fn get_ref(&self) -> Option<SourceFileReference> { self.sourceref.clone() }
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
    fn normalize(self, _: &ColonMode, ret: &mut Vec<Whatsit>, _: Option<f32>) {
        match ret.last_mut() {
            Some(Whatsit::Simple(SimpleWI::VKern(sk2))) => {
                sk2.dim = self.dim + sk2.dim;
                if sk2.dim == 0 {
                    ret.pop();
                }
            },
            _ => ret.push(self.as_whatsit())
        }
    }
    fn as_html(self, _: &ColonMode, colon: &mut HTMLColon, node_top: &mut Option<HTMLParent>) {
        htmlnode!(colon,div,self.sourceref,"rustex-kern",node_top,node => {
            node.style("margin-bottom".into(),dimtohtml(self.dim));
        })
    }
}

#[derive(Clone)]
pub struct HKern {
    pub dim:i32,
    pub sourceref:Option<SourceFileReference>
}
impl WhatsitTrait for HKern {
    //fn get_par_width(&self) -> Option<i32> { None }
    //fn get_par_widths(&self) -> Vec<i32> { vec!() }
    fn get_ref(&self) -> Option<SourceFileReference> { self.sourceref.clone() }
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
    fn normalize(self, _: &ColonMode, ret: &mut Vec<Whatsit>, _: Option<f32>) {
        match ret.last_mut() {
            Some(Whatsit::Simple(SimpleWI::HKern(sk2))) => {
                sk2.dim = self.dim + sk2.dim;
                if sk2.dim == 0 {
                    ret.pop();
                }
            },
            _ => ret.push(self.as_whatsit())
        }
    }
    fn as_html(self, mode: &ColonMode, colon: &mut HTMLColon, node_top: &mut Option<HTMLParent>) {
        //colon.state.add_kern(self.dim);
        htmlnode!(colon,div,self.get_ref(),"rustex-kern",node_top,node => {
            node.style("margin-left".into(),dimtohtml(self.dim));
        });
    }
}

#[derive(Clone)]
pub struct Indent {
    pub dim:i32,
    pub sourceref:Option<SourceFileReference>
}
impl WhatsitTrait for Indent {
    //fn get_par_width(&self) -> Option<i32> { None }
    //fn get_par_widths(&self) -> Vec<i32> { vec!() }
    fn get_ref(&self) -> Option<SourceFileReference> { self.sourceref.clone() }
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
    fn normalize(self, _: &ColonMode, ret: &mut Vec<Whatsit>, _: Option<f32>) {
        match ret.last_mut() {
            Some(Whatsit::Simple(SimpleWI::Indent(sk2))) => {
                sk2.dim = self.dim + sk2.dim;
                if sk2.dim == 0 {
                    ret.pop();
                }
            },
            _ => ret.push(self.as_whatsit())
        }
    }
    fn as_html(self, _: &ColonMode, colon: &mut HTMLColon, node_top: &mut Option<HTMLParent>) {
        htmlnode!(colon,div,self.sourceref,"indent",node_top,node => {
            node.style("margin-left".into(),dimtohtml(self.dim));
        })
    }
}

#[derive(Clone)]
pub struct PDFDest {
    pub target:TeXStr,
    pub dest:TeXStr,
    pub sourceref:Option<SourceFileReference>
}
impl WhatsitTrait for PDFDest {
    //fn get_par_width(&self) -> Option<i32> { None }
    //fn get_par_widths(&self) -> Vec<i32> { vec!() }
    fn get_ref(&self) -> Option<SourceFileReference> { self.sourceref.clone() }
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
    fn normalize(self, _: &ColonMode, ret: &mut Vec<Whatsit>, _: Option<f32>) {
        ret.push(self.as_whatsit())
    }
    fn as_html(self, _: &ColonMode, colon: &mut HTMLColon, node_top: &mut Option<HTMLParent>) {
        htmlnode!(colon,a,self.sourceref.clone(),"pdfdest",node_top,node => {
            node.attr("id".into(),self.target.clone().into());
            node.attr("name".into(),self.target.into());
        })
    }
}

#[derive(Clone)]
pub struct HAlign {
    pub skip:Skip,
    pub template:Vec<(Vec<Token>,Vec<Token>,Skip)>,
    pub rows:Vec<AlignBlock>,
    pub lineheight:Option<i32>,
    pub sourceref:Option<SourceFileReference>
}
impl WhatsitTrait for HAlign {
    //fn get_par_width(&self) -> Option<i32> { None }
    //fn get_par_widths(&self) -> Vec<i32> { vec!() }
    fn get_ref(&self) -> Option<SourceFileReference> { self.sourceref.clone() }
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
                    if let Some(lht) = self.lineheight {
                        ht = max(ht,(lht * 3) / 2);
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
    fn normalize(self, _: &ColonMode, ret: &mut Vec<Whatsit>, scale: Option<f32>) {
        let mut nrows : Vec<AlignBlock> = vec!();
        for block in self.rows {
            match block {
                AlignBlock::Noalign(v) => {
                    let mut na : Vec<Whatsit> = vec!();
                    for w in v { w.normalize(&ColonMode::V, &mut na, scale)}
                    if !na.is_empty() { nrows.push(AlignBlock::Noalign(na))}
                }
                AlignBlock::Block(vv) => {
                    let mut nb : Vec<(Vec<Whatsit>,Skip,usize)> = vec!();
                    for (v,sk,num) in vv {
                        let mut nv : Vec<Whatsit> = vec!();
                        for w in v { w.normalize(&ColonMode::H,&mut nv,scale) }
                        nb.push((nv,sk,num))
                    }
                    nrows.push(AlignBlock::Block(nb))
                }
            }
        }
        ret.push(HAlign {
            skip:self.skip,
            template:self.template,
            rows:nrows,lineheight:self.lineheight,
            sourceref:self.sourceref
        }.as_whatsit())
    }
    fn as_html(self, mode: &ColonMode, colon: &mut HTMLColon, node_top: &mut Option<HTMLParent>) {
        match mode {
            ColonMode::H | ColonMode::V | ColonMode::P => {
                let width = self.width();
                let height = self.height();
                htmlnode!(colon,table,self.sourceref,"rustex-halign",node_top,table => {
                    if crate::INSERT_RUSTEX_ATTRS {
                        table.attr("rustex:width".into(),dimtohtml(width));
                        table.attr("rustex:height".into(),dimtohtml(height));
                    }
                    withlinescale!(colon,self.lineheight,table,{
                    if self.skip.base != 0 {
                        table.style("margin-top".into(),dimtohtml(self.skip.base))
                    }
                    for row in self.rows {
                        HAlign::do_row(mode, colon, &mut table,row)
                    }
                    })
                })
            }
            ColonMode::M => htmlnode!(colon,mtext,self.get_ref(),"",node_top,mt => {
                //let oldwd = colon.state.currsize;
                let mut wd = self.width();
                if wd == 0 {wd = 2048};
                //colon.state.currsize = wd;
                mt.style("width".into(),dimtohtml(wd));
                htmlnode!(colon,HTML_NS:span,None,"rustex-contents",htmlparent!(mt),span => {
                    span.forcefont = true;
                    htmlliteral!(colon,htmlparent!(span),"\n");
                    self.as_html(&ColonMode::H,colon,htmlparent!(span));
                    htmlliteral!(colon,htmlparent!(span),"\n");
                });
            }),
                /*htmlnode!(colon,mtable,self.sourceref,"halign",node_top,table => {
                    table.style("align".into(),"center".into());
                    if self.skip.base != 0 {
                        table.style("margin-top".into(),dimtohtml(self.skip.base))
                    }
                    for row in self.rows {
                        HAlign::do_row(mode, colon, &mut table,row)
                    }
                }),*/
            _ => ()//TeXErr!("TODO")
        }
    }
}
macro_rules! docell {
    ($mode:expr,$sel:ident,$node_parent:expr,$nodename:ident => $e:expr) => ({
        match $mode {
            ColonMode::M => htmlnode!($sel,mtd,None,"rustex-cell",htmlparent!($node_parent),$nodename => $e),
            _ => htmlnode!($sel,td,None,"rustex-cell",htmlparent!($node_parent),$nodename => $e)
        }
    })
}
macro_rules! dorow {
    ($mode:expr,$sel:ident,$node_parent:expr,$nodename:ident => $e:expr) => ({
        match $mode {
            ColonMode::M => htmlnode!($sel,mtr,None,"rustex-row",htmlparent!($node_parent),$nodename => $e),
            _ => htmlnode!($sel,tr,None,"rustex-row",htmlparent!($node_parent),$nodename => $e)
        }
    })
}
macro_rules! dobox {
    ($mode:expr,$sel:ident,$node_parent:expr,$nodename:ident => $e:expr) => ({
        match $mode {
            ColonMode::M => htmlnode!($sel,mrow,None,"",htmlparent!($node_parent),$nodename => $e),
            _ => htmlnode!($sel,div,None,"rustex-hbox",htmlparent!($node_parent),$nodename => {
                /*$nodename.style("height".into(),"0".into());
                $nodename.style("max-height".into(),"0".into());*/
                $e
            })
        }
    })
}
impl HAlign {
    fn do_cell(mode: &ColonMode, colon: &mut HTMLColon, row:&mut HTMLNode,mut vs:Vec<Whatsit>,skip:Skip,num:usize) {
        docell!(mode,colon,row,cell => {
            if num > 1 { cell.attr("colspan".into(),num.to_string().into()) }
            let mut alignment = (0,0);
            let mut repush = vec!();
            loop {
                match vs.pop() {
                    Some(Whatsit::Simple(SimpleWI::VRule(v))) if v.width() <= 393216 => {
                        let wd = dimtohtml(v.width());
                        cell.style("border-right".into(),wd + " solid")
                    },
                    Some(o@Whatsit::Simple(SimpleWI::VRule(_))) => {vs.push(o);break}
                    Some(Whatsit::Simple(SimpleWI::HFil(_) | SimpleWI::Hss(_))) => alignment.1 = 1,
                    Some(Whatsit::Simple(SimpleWI::HFill(_))) => alignment.1 = 2,
                    Some(ref o@Whatsit::Simple(SimpleWI::HSkip(ref sk))) => {
                        match sk.skip.stretch {
                            Some(SkipDim::Fil(_)) => alignment.1 = 1,
                            Some(SkipDim::Fill(_) | SkipDim::Filll(_)) => alignment.1 = 2,
                            _ => ()
                        }
                        repush.push(o.clone());
                    },
                    Some(o) if !o.has_ink() => repush.push(o),
                    Some(o) => {vs.push(o);break}
                    None => break
                }
            }
            for c in repush.into_iter().rev() {vs.push(c)}
            let mut incell : bool = false;
            dobox!(mode,colon,cell,bx => {
                if colon.state.line_scale <= 0.0 {
                    bx.style("height".into(),"0".into());
                }
                let mut inspace = false;
                for w in vs { match w {
                    Whatsit::Simple(SimpleWI::VRule(v)) if !incell && v.width() <= 393216 => cell.style("border-left".into(),dimtohtml(v.width()) + " solid"),
                    o@Whatsit::Simple(SimpleWI::VRule(_)) => {
                        inspace = false;
                        incell = true;
                        o.as_html(mode,colon,htmlparent!(bx))
                    }
                    Whatsit::Simple(SimpleWI::HFil(_) | SimpleWI::Hss(_)) => alignment.0 = 1,
                    Whatsit::Simple(SimpleWI::HFill(_)) => alignment.0 = 2,
                    ref o@Whatsit::Simple(SimpleWI::HSkip(ref sk)) => {
                        match sk.skip.stretch {
                            Some(SkipDim::Fil(_)) => alignment.0 = 1,
                            Some(SkipDim::Fill(_) | SkipDim::Filll(_)) => alignment.0 = 2,
                            _ => ()
                        }
                        o.clone().as_html(mode,colon,htmlparent!(bx));
                    }
                    Whatsit::Space(_) if !inspace => {
                        incell = true;
                        inspace = true;
                        htmlliteral!(colon,htmlparent!(bx),"&nbsp;")
                    }
                    Whatsit::Space(_) => {}
                    Whatsit::Char(ref pc) => {
                        match pc.font.file.chartable.as_ref().map(|ct| ct.table.get(&pc.char)) {
                            Some(Some(s)) if *s == " " && !inspace => {
                                incell = true;
                                inspace = true;
                                htmlliteral!(colon,htmlparent!(bx),"&nbsp;")
                            }
                            Some(Some(s)) if *s == " " && inspace => {}
                            _ => {
                                incell = true;
                                inspace = false;
                                w.as_html(mode,colon,htmlparent!(bx))
                            }
                        }
                    }
                    o if !o.has_ink() => {o.as_html(mode,colon,htmlparent!(bx))}
                    o => {
                        inspace = false;
                        incell = true;
                        o.as_html(mode,colon,htmlparent!(bx))
                    }
                }}
                HSkip {skip,sourceref:None}.as_html(mode,colon,htmlparent!(bx));
            });
            match alignment {
                (a,b) if a == b && a != 0 => {
                    cell.style("text-align".into(),"center".into());
                    //cell.style("justify-items".into(),"center".into());
                }
                (a,b) if a > b => {
                    cell.style("text-align".into(),"right".into());
                    //cell.style("justify-items".into(),"right".into());
                },
                _ => {
                    cell.style("text-align".into(),"left".into());
                    //cell.style("justify-items".into(),"left".into())
                }
            }
            //cell.style("margin-right".into(),dimtohtml(skip.base));
        })
    }
    fn do_row(mode: &ColonMode, colon: &mut HTMLColon, table:&mut HTMLNode,row:AlignBlock) {
        match row {
            AlignBlock::Noalign(mut v) => {
                let mut aboveborder = true;
                for c in v.into_iter() {
                    match c {
                        Whatsit::Simple(SimpleWI::HRule(hr)) => {
                            aboveborder = false;
                            if table.children.is_empty() {
                                table.style("border-top".into(),dimtohtml(hr.height()) + " solid")
                            } else {
                                match table.children.last_mut() {
                                    Some(HTMLChild::Node(row)) => row.style("border-bottom".into(),dimtohtml(hr.height()) + " solid"),
                                    _ => ()//TeXErr!("Should be unreachable!")
                                }
                            }
                        }
                        Whatsit::Simple(SimpleWI::VSkip(sk)) if aboveborder && !table.children.is_empty() => {
                            match table.children.last_mut() {
                                Some(HTMLChild::Node(row)) => row.style("margin-bottom".into(),dimtohtml(sk.height())),
                                _ => ()//TeXErr!("Should be unreachable!")
                            }
                        }
                        Whatsit::Simple(SimpleWI::VSkip(sk)) if aboveborder => {
                            table.style("margin-top".into(),dimtohtml(sk.height()))
                        }
                        o => {
                        }
                    }
                }
            }
            AlignBlock::Block(cells) => {
                if cells.iter().any(|c| {c.0.iter().any(|e| e.has_ink())}) {
                    dorow!(mode,colon,table,row => {
                        for (mut vs,skip,num) in cells {
                            HAlign::do_cell(mode,colon,&mut row,vs,skip,num)
                        }
                    })}
            }
        }
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
    //fn get_par_width(&self) -> Option<i32> { None }
    //fn get_par_widths(&self) -> Vec<i32> { vec!() }
    fn get_ref(&self) -> Option<SourceFileReference> { self.sourceref.clone() }
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

    fn normalize(self, _: &ColonMode, ret: &mut Vec<Whatsit>, scale: Option<f32>) {
        let mut nrows : Vec<AlignBlock> = vec!();
        for block in self.columns {
            match block {
                AlignBlock::Noalign(v) => {
                    let mut na : Vec<Whatsit> = vec!();
                    for w in v { w.normalize(&ColonMode::H, &mut na, scale)}
                    if !na.is_empty() { nrows.push(AlignBlock::Noalign(na))}
                }
                AlignBlock::Block(vv) => {
                    let mut nb : Vec<(Vec<Whatsit>,Skip,usize)> = vec!();
                    for (v,sk,num) in vv {
                        let mut nv : Vec<Whatsit> = vec!();
                        for w in v { w.normalize(&ColonMode::V,&mut nv,scale) }
                        nb.push((nv,sk,num))
                    }
                    nrows.push(AlignBlock::Block(nb))
                }
            }
        }
        ret.push(VAlign {
            skip:self.skip,
            template:self.template,
            columns:nrows,
            sourceref:self.sourceref
        }.as_whatsit())
    }
    fn as_html(self, _: &ColonMode, _: &mut HTMLColon, _: &mut Option<HTMLParent>) {
        ()//TeXErr!("TODO")
    }
}

#[derive(Clone)]
pub struct Mark {
    pub toks:Vec<Token>,
    pub sourceref:Option<SourceFileReference>
}
impl WhatsitTrait for Mark {
    //fn get_par_width(&self) -> Option<i32> { None }
    //fn get_par_widths(&self) -> Vec<i32> { vec!() }
    fn get_ref(&self) -> Option<SourceFileReference> { self.sourceref.clone() }
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
    fn normalize(self, _: &ColonMode, _: &mut Vec<Whatsit>, _: Option<f32>) {}
    fn as_html(self, _: &ColonMode, _: &mut HTMLColon, _: &mut Option<HTMLParent>) {}
}

#[derive(Clone)]
pub struct Leaders {
    pub bx:Box<Whatsit>,pub glue:Box<Whatsit>,
    pub sourceref:Option<SourceFileReference>
}
impl WhatsitTrait for Leaders {
    //fn get_par_width(&self) -> Option<i32> { None }
    //fn get_par_widths(&self) -> Vec<i32> { vec!() }
    fn get_ref(&self) -> Option<SourceFileReference> { self.sourceref.clone() }
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
    fn normalize(mut self, mode: &ColonMode, ret: &mut Vec<Whatsit>, scale: Option<f32>) {
        match &mut *self.bx {
            Whatsit::Box(bx) => {
                match bx {
                    TeXBox::H(hb) => hb._width = Some(655360),
                    TeXBox::V(vb) => vb._width = Some(655360),
                    _ => ()
                }
                self.bx.normalize(mode,ret,scale);
                self.glue.normalize(mode,ret,scale);
            }
            _ => self.bx.normalize(mode,ret,scale)
        }
        /*
        let mut nret : Vec<Whatsit> = vec!();
        self.bx.normalize(mode,&mut nret,scale);
        if nret.is_empty() {} else if nret.len() == 1 {
            ret.push(Leaders { bx:Box::new(nret.pop().unwrap()), sourceref:self.sourceref }.as_whatsit())
        } else {
            ret.push(Leaders { bx:Box::new(nret.pop().unwrap()), sourceref:self.sourceref }.as_whatsit())
        }
         */
    }
    fn as_html(self, mode: &ColonMode, colon: &mut HTMLColon, node_top: &mut Option<HTMLParent>) {
        self.bx.clone().as_html(mode,colon,node_top);
        self.bx.clone().as_html(mode,colon,node_top);
        self.bx.as_html(mode,colon,node_top);
    }
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
    //fn get_par_width(&self) -> Option<i32> { None }
    //fn get_par_widths(&self) -> Vec<i32> { vec!() }
    fn get_ref(&self) -> Option<SourceFileReference> { self.sourceref.clone() }
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
    fn normalize(self, _: &ColonMode, ret: &mut Vec<Whatsit>, _: Option<f32>) {
        ret.push(self.as_whatsit())
    }
    fn as_html(self, _: &ColonMode, _: &mut HTMLColon, _: &mut Option<HTMLParent>) {}
}

#[derive(Clone)]
pub struct Left {
    pub bx:Option<MathChar>,
    pub sourceref:Option<SourceFileReference>
}
impl WhatsitTrait for Left {
    //fn get_par_width(&self) -> Option<i32> { None }
    //fn get_par_widths(&self) -> Vec<i32> { vec!() }
    fn get_ref(&self) -> Option<SourceFileReference> { self.sourceref.clone() }
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
    fn normalize(self, _: &ColonMode, ret: &mut Vec<Whatsit>, _: Option<f32>) {
        ret.push(self.as_whatsit())
    }
    fn as_html(self, mode: &ColonMode, colon: &mut HTMLColon, node_top: &mut Option<HTMLParent>) {
        if let Some(c) = self.bx { c.as_html_inner(mode,colon,node_top,true)}
    }
}

#[derive(Clone)]
pub struct Middle {
    pub bx:Option<MathChar>,
    pub sourceref:Option<SourceFileReference>
}
impl WhatsitTrait for Middle {
    //fn get_par_width(&self) -> Option<i32> { None }
    //fn get_par_widths(&self) -> Vec<i32> { vec!() }
    fn get_ref(&self) -> Option<SourceFileReference> { self.sourceref.clone() }
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
    fn normalize(self, _: &ColonMode, ret: &mut Vec<Whatsit>, _: Option<f32>) {
        ret.push(self.as_whatsit())
    }
    fn as_html(self, mode: &ColonMode, colon: &mut HTMLColon, node_top: &mut Option<HTMLParent>) {
        if let Some(c) = self.bx { c.as_html_inner(mode,colon,node_top,true)}
    }
}

#[derive(Clone)]
pub struct Right {
    pub bx:Option<MathChar>,
    pub sourceref:Option<SourceFileReference>
}
impl WhatsitTrait for Right {
    //fn get_par_width(&self) -> Option<i32> { None }
    //fn get_par_widths(&self) -> Vec<i32> { vec!() }
    fn get_ref(&self) -> Option<SourceFileReference> { self.sourceref.clone() }
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
    fn normalize(self, _: &ColonMode, ret: &mut Vec<Whatsit>, _: Option<f32>) {
        ret.push(self.as_whatsit())
    }
    fn as_html(self, mode: &ColonMode, colon: &mut HTMLColon, node_top: &mut Option<HTMLParent>) {
        if let Some(c) = self.bx { c.as_html_inner(mode,colon,node_top,true)}
    }
}

macro_rules! trivial {
    ($e:ident) => (
        #[derive(Clone)]
        pub struct $e(pub Option<SourceFileReference>);
        impl WhatsitTrait for $e {
            fn get_ref(&self) -> Option<SourceFileReference> { self.0.clone() }
            fn as_whatsit(self) -> Whatsit {
                Whatsit::Simple(SimpleWI::$e(self))
            }
            fn width(&self) -> i32 { 0 }
            fn height(&self) -> i32 { 0 }
            fn depth(&self) -> i32 { 0 }
            fn as_xml_internal(&self, _: String) -> String {
                "<".to_string() + &stringify!($e) + "/>"
            }
            fn has_ink(&self) -> bool { false }
            fn normalize(self, _: &ColonMode, ret: &mut Vec<Whatsit>, _: Option<f32>) {
                ret.push(self.as_whatsit())
            }
            fn as_html(self, mode: &ColonMode, colon: &mut HTMLColon, node_top: &mut Option<HTMLParent>) {
                match mode {
                    ColonMode::H | ColonMode::V | ColonMode::P => {
                        htmlnode!(colon,div,self.0,("rustex-".to_string() + &stringify!($e)),node_top)
                    }
                    _ => ()
                }
            }
            //fn get_par_width(&self) -> Option<i32> { None }
            //fn get_par_widths(&self) -> Vec<i32> { vec!() }
        }
    )
}
trivial!(HFil);
trivial!(HFill);
trivial!(VFil);
trivial!(VFill);
trivial!(Hss);
trivial!(Vss);
