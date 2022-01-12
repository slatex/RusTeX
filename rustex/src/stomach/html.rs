use std::borrow::BorrowMut;
use std::collections::HashMap;
use std::fmt::{Debug, Display, Formatter};
use std::hash::{Hash, Hasher};
use std::ops::{Add, AddAssign};
use std::str::from_utf8;
use std::sync::Arc;
use image::{EncodableLayout, GenericImageView};
use itertools::Itertools;
use crate::fonts::Font;
use crate::fonts::fontchars::FontTableParam;
use crate::Interpreter;
use crate::interpreter::dimensions::numtostr;
use crate::references::SourceFileReference;
use crate::stomach::colon::{Colon, ColonBase, ColonMode};
use crate::stomach::groups::WIGroupTrait;
use crate::stomach::math::{GroupedMath, MathKernel};
use crate::stomach::simple::PDFMatrix;
use crate::stomach::Whatsit;
use crate::stomach::whatsits::WhatsitTrait;
use crate::utils::TeXStr;


pub fn dimtohtml(num:i32) -> HTMLStr {
    numtostr(num,"px").into()
}

pub static HTML_NS : &str = "http://www.w3.org/1999/xhtml";
pub static MATHML_NS : &str = "http://www.w3.org/1998/Math/MathML";
pub static SVG_NS : &str = "http://www.w3.org/2000/svg";
pub static RUSTEX_NS : &str = "http://kwarc.info/ns/RusTeX";

pub struct HTMLState {
    annotations_todo:Vec<(HTMLStr,HTMLStr)>,
    pub current_namespace:&'static str,
    pub top:Vec<HTMLChild>,
    pub currsize:i32,
    pub currcolor:Option<HTMLStr>
}
impl HTMLState {
    pub fn new() -> HTMLState { HTMLState {
        annotations_todo:vec!(),
        current_namespace:HTML_NS,
        top:vec!(),
        currsize:0,currcolor:None
    }}
}
#[macro_export]
macro_rules! htmlnode {
    ($sel:ident,$node:ident,$sref:expr,$name:tt,$node_parent:expr) => ({
        let mut _node_newnode = HTMLNode::new($sel.state.current_namespace,stringify!($node).into(),$sref);
        _node_newnode.classes.push($name.into());
        match $node_parent {
            Some(e) => {
                e.push(HTMLChild::Node(_node_newnode))
            }
            _ => {
                $sel.state.top.push(HTMLChild::Node(_node_newnode))
            }
        }
    });
    ($sel:ident,$ns:tt:$node:ident,$sref:expr,$name:tt,$node_parent:expr) => ({
        let mut _node_newnode = HTMLNode::new($ns,stringify!($node).into(),$sref);
        _node_newnode.classes.push($name.into());
        match $node_parent {
            Some(e) => {
                e.push(HTMLChild::Node(_node_newnode))
            }
            _ => {
                $sel.state.top.push(HTMLChild::Node(_node_newnode))
            }
        }
    });
    ($sel:ident,$node:ident,$sref:expr,$name:tt,$node_parent:expr,$nodename:ident => $e:expr) => (
        {
            let mut $nodename = HTMLNode::new($sel.state.current_namespace,stringify!($node).into(),$sref);
            $nodename.classes.push($name.into());
            $e
            match $node_parent {
                Some(e) => {
                    e.push(HTMLChild::Node($nodename))
                }
                _ => {
                    $sel.state.top.push(HTMLChild::Node($nodename))
                }
            }
        }
    );
    ($sel:ident,$ns:tt:$node:ident,$sref:expr,$name:tt,$node_parent:expr,$nodename:ident => $e:expr) => (
        {
            let mut $nodename = HTMLNode::new($ns,stringify!($node).into(),$sref);
            $nodename.classes.push($name.into());
            let _node_oldns = $sel.state.current_namespace;
            $sel.state.current_namespace = $ns;
            $e
            $sel.state.current_namespace = _node_oldns;
            match $node_parent {
                Some(e) => {
                    e.push(HTMLChild::Node($nodename))
                }
                _ => {
                    $sel.state.top.push(HTMLChild::Node($nodename))
                }
            }
        }
    );
}
#[macro_export]
macro_rules! htmlliteral {
    ($sel:ident,$node_parent:expr,$e:expr) => ({
        let _ret : HTMLStr = $e.into();
        match $node_parent {
            Some(e) => {
                e.push(HTMLChild::Str(_ret))
            }
            _ => {
                $sel.state.top.push(HTMLChild::Str(_ret))
            }
        }
    });
    ($sel:ident,$node_parent:expr,>$e:tt<) => ({
        let _ret : HTMLStr = $e.into();
        match $node_parent {
            Some(e) => {
                e.push(HTMLChild::Str(_ret.html_escape()))
            }
            _ => {
                $sel.state.top.push(HTMLChild::Str(_ret.html_escape()))
            }
        }
    })
}
#[macro_export]
macro_rules! htmlannotate {
    ($sel:ident,$node:ident,$sref:expr,$node_parent:expr,$nodename:ident => $e:expr) => (
        {
            let mut $nodename = HTMLAnnotation::new($sel.state.current_namespace,stringify!($node).into(),$sref);
            $e
            match $node_parent {
                Some(e) => {
                    e.push(HTMLChild::Annot($nodename))
                }
                _ => {
                    $sel.state.top.push(HTMLChild::Annot($nodename))
                }
            }
        }
    );
    ($sel:ident,$ns:tt:$node:ident,$sref:expr,$node_parent:expr,$nodename:ident => $e:expr) => (
        {
            $sel.state.current_namespace = $ns;
            let mut $nodename = HTMLAnnotation::new($ns,stringify!($node).into(),$sref);
            $e
            $sel.state.current_namespace = _node_oldns;
            match $node_parent {
                Some(e) => {
                    e.push(HTMLChild::Annot($nodename))
                }
                _ => {
                    $sel.state.top.push(HTMLChild::Annot($nodename))
                }
            }
        }
    );
}

// -------------------------------------------------------------------------------------------------

static CSS : &str = include_str!("../resources/html.css");

pub struct HTMLColon {
    pub base:ColonBase,
    ret : String,
    doheader:bool,
    pub state:HTMLState
}
unsafe impl Send for HTMLColon {}

impl Colon<String> for HTMLColon {
    fn base(&self) -> &ColonBase { &self.base }
    fn base_mut(&mut self) -> &mut ColonBase { &mut self.base }
    fn ship_whatsit(&mut self, w:Whatsit) {
        /*for w in self.normalize_whatsit(w) { self.ship_top(w,&mut None) */ w.as_html(&ColonMode::V,self,&mut None);
        for n in std::mem::take(&mut self.state.top) {
            self.ret += &n.make_string("  ".into(),&HTML_NS).to_string()
        }
    } //}
    fn initialize(&mut self, basefont: Arc<Font>, basecolor: TeXStr, int: &Interpreter) {
        if self.doheader {
            self.state.currsize = basefont.at.unwrap_or(655360);
            self.state.currcolor = match &basecolor {
                s if s.to_string() == "000000" => None,
                s => Some(s.clone().into())
            };
            self.ret = "<!DOCTYPE html PUBLIC \"-//W3C//DTD XHTML 1.1 plus MathML 2.0//EN\" \"http://www.w3.org/Math/DTD/mathml2/xhtml-math11-f.dtd\">\n".into();
            self.ret += "<html xmlns=\"";
            self.ret += HTML_NS;
            self.ret += "\" xmlns:mml=\"";
            self.ret += MATHML_NS;
            self.ret += "\" xmlns:svg=\"";
            self.ret += SVG_NS;
            self.ret += "\" xmlns:rustex=\"";
            self.ret += RUSTEX_NS;
            self.ret += "\">\n  <head>\n    <style>\n";
            self.ret += CSS;
            self.ret += "\n    </style>";
            //self.ret += "\n    <script type=\"text/javascript\" id=\"MathJax-script\" src=\"https://cdn.jsdelivr.net/npm/mathjax@3/es5/mml-chtml.js\"></script>";
            self.ret += "\n  </head>\n  <body style=\"width:";
            let pagewidth = int.state_dimension(-(crate::commands::pdftex::PDFPAGEWIDTH.index as i32));
            self.ret += &dimtohtml(pagewidth).to_string();
            self.ret += "\">\n    <div class=\"body\" style=\"font-size:";
            let fontsize = match &basefont.at {
                Some(i) => *i,
                None => 655360
            };
            self.ret += &dimtohtml(fontsize).to_string();
            self.ret += ";width:";
            let textwidth = int.state_dimension(-(crate::commands::primitives::HSIZE.index as i32));
            self.ret += &dimtohtml(textwidth).to_string();
            self.ret += ";padding:";
            self.ret += &dimtohtml(((pagewidth - textwidth) as f32 / 2.0).round() as i32).to_string();
            self.ret += ";line-height:";
            let lineheight = int.state_skip(-(crate::commands::primitives::BASELINESKIP.index as i32));
            self.ret += &(lineheight.base as f32 / fontsize as f32).to_string();
            self.ret += ";\"";
            self.ret += " rustex:font=\"";
            self.ret += basefont.file.name.to_string().as_str();
            self.ret += "\">\n";

            let base = self.base_mut();
            base.basefont = Some(basefont);
            base.basecolor = Some(basecolor);
        }
    }
    fn close(&mut self) -> String {
        if self.doheader {
            std::mem::take(&mut self.ret) + "\n    </div>\n  </body>\n</html>"
        } else { std::mem::take(&mut self.ret) }
    }
}
impl HTMLColon {
    pub fn new(doheader:bool) -> HTMLColon { HTMLColon {
        base:ColonBase::new(),
        ret:"".to_string(),
        state:HTMLState::new(),
        doheader
    }}
    /*
    fn ship_top<'a>(&mut self,w:Whatsit,node_top:&mut Option<HTMLParent<'a>>) {
        use Whatsit::*;
        use crate::stomach::simple::SimpleWI::*;
        use crate::stomach::boxes::TeXBox::*;
        use crate::interpreter::dimensions::SkipDim;
        use crate::stomach::groups::WIGroup::*;
        use crate::fonts::fontchars::FontTableParam;
        use crate::stomach::simple::ExternalParam;
        match w {
            Simple(VFil(v)) => htmlnode!(self,div,v.0,"vfil",node_top),
            Simple(VFill(v)) => htmlnode!(self,div,v.0,"vfill",node_top),
            Simple(PDFDest(d)) if d.dest.to_string() == "xyz" => {
            },
            Simple(VSkip(vsk)) => {
            },
            Simple(VKern(vsk)) => {
            },
            Simple(HRule(hr)) => {
                htmlnode!(self,div,hr.sourceref.clone(),"hrule",node_top,n => {
                    n.style("width".into(),dimtohtml(hr.width()));
                    n.style("min-width".into(),dimtohtml(hr.width()));
                    n.style("height".into(),dimtohtml(hr.height() + hr.depth()));
                    n.style("min-height".into(),dimtohtml(hr.height() + hr.height()));
                    n.style("background".into(),match &self.state.currcolor {
                        Some(c) => HTMLStr::from("#") + c,
                        None => "#000000".into()
                    });
                    if hr.depth() != 0 { n.style("margin-bottom".into(),dimtohtml(-hr.depth())) }
                })
            }
            Grouped(ColorChange(cc)) => {
            }
            Grouped(FontChange(fc)) => {
            }
            Par(p) => {
            }
            Simple(Vss(v)) => htmlnode!(self,div,v.0,"vss",node_top),
            Simple(HAlign(ha)) => (),
            Box(H(hb)) => {
                htmlnode!(self,div,None,"hbox",node_top,node => {
                    match hb._width {
                        Some(h) => {
                            node.style("width".into(),dimtohtml(h));
                            node.style("min-width".into(),dimtohtml(h))
                        }
                        _ => ()
                    }
                    match hb._height {
                        Some(h) => {
                            node.style("height".into(),dimtohtml(h));
                            node.style("min-height".into(),dimtohtml(h))
                        }
                        _ => ()
                    }
                    for c in hb.children { self.ship_h(c,&mut Some(HTMLParent::N(&mut node))) }
                })
            }
            Inserts(is) => {
                htmlliteral!(self,node_top,"<hr/>");
                for v in is.0 { for w in v { self.ship_top(w,node_top) }}
            }
            Simple(Penalty(_)) => (),
            Simple(crate::stomach::simple::SimpleWI::External(ext)) if ext.name().to_string() == "pgfbox" => {

            }
            Float(bx) => {
                htmlnode!(self,div,None,"vfill",node_top);
                self.ship_top(bx.as_whatsit(),node_top);
                htmlnode!(self,div,None,"vfill",node_top)
            }
            Box(V(vb)) if vb._height.is_none() => {
                for c in vb.children { self.ship_top(c,node_top) }
            }
            Box(V(vb)) => {

            }
            Simple(MoveRight(crate::stomach::simple::MoveRight { dim,content:bx,sourceref })) => {
                htmlannotate!(self,div,sourceref,node_top,a => {
                    a.classes.push("moveright".into());
                    a.style("margin-left".into(),dimtohtml(dim));
                    self.ship_top(bx.as_whatsit(),&mut Some(HTMLParent::A(&mut a)))
                })
            }
            Simple(crate::stomach::simple::SimpleWI::External(ext)) if ext.name().to_string() == "pgfliteral" => (),
            Simple(crate::stomach::simple::SimpleWI::External(ext)) => {
                println!("TODO: {}",ext.as_xml());
                htmlliteral!(self,node_top,"<!-- TODO:".to_string() + &ext.as_xml() +  "-->")
            }
            Simple(Mark(_)) | Box(Void) => (),
            _ => htmlliteral!(self,node_top,"<!-- TODO -->")//self.ret += &w.as_xml_internal("  ".to_string())
        }
        if node_top.is_none() {
            for n in std::mem::take(&mut self.state.top) {
                self.ret += &n.make_string("  ".into(),&HTML_NS).to_string()
            }
        }
    }
    fn ship_h<'a>(&mut self, w:Whatsit, node_top:&mut Option<HTMLParent<'a>>) {
        use Whatsit::*;
        use crate::stomach::simple::SimpleWI::*;
        use crate::stomach::boxes::TeXBox::*;
        use crate::stomach::groups::WIGroup::*;
        use crate::fonts::fontchars::FontTableParam;
        use crate::stomach::simple::ExternalParam;
        use crate::stomach::TeXBox;
        match w {
            Simple(PDFDest(d)) if d.dest.to_string() == "xyz" => {
                htmlnode!(self,a,d.sourceref,"pdfdest",node_top,node => {
                    node.attr("id".into(),d.target.clone().into());
                    node.attr("name".into(),d.target.into());
                })
            },
            Simple(Penalty(p)) if p.penalty <= -10000 => htmlliteral!(self,node_top,"<br/>"),
            Grouped(ColorChange(cc)) => {
                htmlannotate!(self,span,cc.sourceref,node_top,a => {
                    let mut color : HTMLStr = crate::stomach::groups::ColorChange::color_to_html(cc.color).into();
                    let hashcolor : HTMLStr = "#".into();
                    a.style("color".into(),hashcolor + &color);
                    let _oldcolor = std::mem::take(&mut self.state.currcolor);
                    self.state.currcolor = Some(color);
                    for c in cc.children { self.ship_h(c,&mut Some(HTMLParent::A(&mut a))) }
                    self.state.currcolor = _oldcolor;
                })
            }
            Grouped(PDFLink(lnk)) => {
            }
            Grouped(FontChange(fc)) => {
                match &fc.font.file.chartable {
                    Some(ft) => {
                        htmlannotate!(self,span,fc.sourceref,node_top,a => {
                            a.attr("rustex:font".into(),fc.font.file.name.clone().into());
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
                            let _oldsize = self.state.currsize;
                            match fc.font.at {
                                Some(at) if at != self.state.currsize => {
                                    let atstr = 100.0 * (at as f32) / (self.state.currsize as f32);
                                    a.style("font-size".into(),(atstr.to_string() + "%").into());
                                    self.state.currsize = at;
                                }
                                _ => ()
                            }
                            for c in fc.children { self.ship_h(c,&mut Some(HTMLParent::A(&mut a))) }
                            self.state.currsize = _oldsize;
                        })
                    }
                    _ => {
                        for c in fc.children { self.ship_h(c, node_top) }
                    }
                }
            }
            Grouped(PDFMatrixSave(sg)) => (),
            Simple(PDFMatrix(_)) => (),
            Simple(PDFXImage(pimg)) => {
                match pimg.image {
                    Some(ref img) => {

                    }
                    _ => ()
                }
            }
            Char(pc) => htmlliteral!(self,node_top,>{
                match &pc.font.file.chartable {
                    Some(ct) => ct.get_char(pc.char).to_string(),
                    None => pc.as_xml_internal("".to_string())
                }
            }<),
            Space(_) => htmlliteral!(self,node_top," "),
            Simple(VRule(vr)) => {
            }
            Simple(HRule(vr)) => { // from Leaders
            }
            Simple(HSkip(vsk)) => {
            },
            Simple(Indent(dim)) => {
                htmlnode!(self,span,dim.sourceref,"indent",node_top,node => {
                    node.style("margin-left".into(),dimtohtml(dim.dim));
                })
            },
            Simple(HKern(vsk)) => {
            },
            Box(V(vb)) => {
                htmlnode!(self,div,None,"vbox",node_top,node => {
                    use crate::stomach::boxes::VBoxType;
                    match vb.tp {
                        VBoxType::V => node.style("vertical-align".into(),"bottom".into()),
                        VBoxType::Center => node.style("vertical-align".into(),"middle".into()),
                        VBoxType::Top(_) => node.style("vertical-align".into(),"top".into())
                    }
                    match vb._height {
                        Some(h) => {
                            node.style("height".into(),dimtohtml(h));
                            node.style("min-height".into(),dimtohtml(h))
                        }
                        _ => ()
                    }
                    match vb._width {
                        Some(v) => {
                            node.style("width".into(),dimtohtml(v));
                            node.style("min-width".into(),dimtohtml(v))
                        }
                        _ => ()
                    }
                    for c in vb.children { self.ship_top(c,&mut Some(HTMLParent::N(&mut node))) }
                })
            }
            // TODO maybe? spread, center, vtop in general
            Simple(HFil(h)) => htmlnode!(self,span,h.0,"hfil",node_top),
            Simple(HFill(h)) => htmlnode!(self,span,h.0,"hfill",node_top),
            Simple(Hss(h)) => htmlnode!(self,span,h.0,"hss",node_top),
            Box(H(hb)) => {
            }
            Simple(Raise(r)) => (),
            Math(ref mg) if mg.limits => (),
            Math(ref mg) => (),
            Simple(Leaders(ld)) => {
                self.ship_h(ld.bx.clone().as_whatsit(),node_top);
                self.ship_h(ld.bx.clone().as_whatsit(),node_top);
                self.ship_h(ld.bx.as_whatsit(),node_top);
            }
            Simple(crate::stomach::simple::SimpleWI::External(ext)) if ext.name().to_string() == "pgfbox" => {
                htmlnode!(self,SVG_NS:svg,ext.sourceref().clone(),"",node_top,svg => {
                    let maxx = match ext.params("maxx") {
                        Some(ExternalParam::Num(i)) => i,
                        _ => unreachable!()
                    };
                    let maxy = match ext.params("maxy") {
                        Some(ExternalParam::Num(i)) => i,
                        _ => unreachable!()
                    };
                    let minx = match ext.params("minx") {
                        Some(ExternalParam::Num(i)) => i,
                        _ => unreachable!()
                    };
                    let miny = match ext.params("miny") {
                        Some(ExternalParam::Num(i)) => i,
                        _ => unreachable!()
                    };
                    let mut vb : HTMLStr = numtostr(minx,"").into();
                    vb += " ";
                    vb += numtostr(miny,"");
                    vb += " ";
                    vb += numtostr(maxx-minx,"");
                    vb += " ";
                    vb += numtostr(maxy-miny,"");
                    svg.attr("width".into(),dimtohtml(maxx-minx));
                    svg.attr("height".into(),dimtohtml(maxy-miny));
                    svg.attr("viewBox".into(),vb);
                    htmlnode!(self,g,None,"",&mut Some(HTMLParent::N(&mut svg)),g => {
                        let mut tr : HTMLStr = "translate(0,".into();
                        tr += numtostr(maxy,"");
                        tr += ") scale(1,-1) translate(0,";
                        tr += numtostr(-miny,"");
                        tr += ")";
                        g.attr("transform".into(),tr);
                        for c in match ext.params("content") {
                            Some(ExternalParam::Whatsits(v)) => v,
                            _ => unreachable!()
                        } {
                            self.ship_svg(c,&mut Some(HTMLParent::N(&mut g)))
                        }
                    })
                })
            }
            Simple(crate::stomach::simple::SimpleWI::External(ext)) if ext.name().to_string() == "pgfliteral" => (),
                Simple(crate::stomach::simple::SimpleWI::External(ext)) => {
                println!("TODO: {}",ext.as_xml());
                htmlliteral!(self,node_top,"<!-- TODO:".to_string() + &ext.as_xml() +  "-->")
            }
            Simple(Penalty(_)) => (), Simple(HAlign(_)) => self.ship_top(w,node_top),
            Simple(Mark(_)) | Box(TeXBox::Void) => (),
            _ => htmlliteral!(self,node_top,"<!-- TODO -->" )
        }
    }
    fn ship_svg<'a>(&mut self, w:Whatsit, node_top:&mut Option<HTMLParent<'a>>) {
        use Whatsit::*;
        use crate::stomach::simple::SimpleWI::*;
        use crate::stomach::boxes::TeXBox::*;
        use crate::stomach::groups::WIGroup::*;
        use crate::stomach::simple::ExternalParam;
        match w {
            Box(Void) | Space(_) | Simple(Hss(_)) => (),
            Simple(crate::stomach::simple::SimpleWI::External(ext)) if ext.name().to_string() == "pgfliteral" => {
                htmlliteral!(self,node_top,match ext.params("string") {
                    Some(ExternalParam::String(s)) => s,
                    _ => unreachable!()
                })
            }
            Simple(crate::stomach::simple::SimpleWI::External(ext)) if ext.name().to_string() == "pgfescape" => {

            }
            Grouped(ColorChange(cc)) => {
            }
            Grouped(wg) => for c in wg.children_prim() { self.ship_svg(c,node_top) }
            Box(H(hb)) => for c in hb.children { self.ship_svg(c,node_top) }
            Simple(crate::stomach::simple::SimpleWI::External(ext)) => {
                println!("TODO: {}",ext.as_xml());
                htmlliteral!(self,node_top,"<!-- TODO:".to_string() + &ext.as_xml() +  "-->")
            }
            _ => htmlliteral!(self,node_top,"<!-- TODO -->" )
        }
    }
    fn ship_kernel<'a>(&mut self, k:MathKernel, node_top:&mut Option<HTMLParent<'a>>) {
        use crate::stomach::math::MathKernel::*;
        match k {
            Group(gm) if gm.0.is_empty() => (),
            Group(mut gm) if gm.0.len() == 1 => self.ship_m(gm.0.pop().unwrap(),node_top),
            Group(GroupedMath(ls)) => (),
            MathChar(mc) => (),
            Delimiter(d) => self.ship_kernel(MathChar(d.small),node_top),
            MKern(m) => {
                htmlnode!(self,mspace,m.sourceref,"mkern",node_top,a => {
                    a.attr("width".into(),numtostr((m.sk.base as f32 / 1179648.0).round() as i32,"em").into())
                })
            }
            MathOp(crate::stomach::math::MathOp { content,sourceref }) => htmlannotate!(self,mrow,sourceref,node_top,node => {
                node.classes.push("mathop".into());
                self.ship_m(*content,&mut Some(HTMLParent::A(&mut node)))
            }),
            MathInner(crate::stomach::math::MathInner { content,sourceref }) => htmlannotate!(self,mrow,sourceref,node_top,node => {
                node.classes.push("inner".into());
                self.ship_m(*content,&mut Some(HTMLParent::A(&mut node)))
            }),
            MathRel(crate::stomach::math::MathRel { content,sourceref }) => htmlannotate!(self,mrow,sourceref,node_top,node => {
                node.classes.push("rel".into());
                self.ship_m(*content,&mut Some(HTMLParent::A(&mut node)))
            }),
            MathOpen(crate::stomach::math::MathOpen { content,sourceref }) => htmlannotate!(self,mrow,sourceref,node_top,node => {
                node.classes.push("open".into());
                self.ship_m(*content,&mut Some(HTMLParent::A(&mut node)))
            }),
            MathClose(crate::stomach::math::MathClose { content,sourceref }) => htmlannotate!(self,mrow,sourceref,node_top,node => {
                node.classes.push("close".into());
                self.ship_m(*content,&mut Some(HTMLParent::A(&mut node)))
            }),
            MathPunct(crate::stomach::math::MathPunct { content,sourceref }) => htmlannotate!(self,mrow,sourceref,node_top,node => {
                node.classes.push("punct".into());
                self.ship_m(*content,&mut Some(HTMLParent::A(&mut node)))
            }),
            MathBin(crate::stomach::math::MathBin { content,sourceref }) => htmlannotate!(self,mrow,sourceref,node_top,node => {
                node.classes.push("bin".into());
                self.ship_m(*content,&mut Some(HTMLParent::A(&mut node)))
            }),
            Underline(crate::stomach::math::Underline { content,sourceref }) => htmlnode!(self,munder,sourceref,"underline",node_top,node => {
                htmlannotate!(self,mrow,None,&mut Some(HTMLParent::N(&mut node)),mrow => {
                    self.ship_m(*content,&mut Some(HTMLParent::A(&mut mrow)))
                });
                htmlliteral!(self,&mut Some(HTMLParent::N(&mut node)),"&UnderBar;")
            }),
            Overline(crate::stomach::math::Overline { content,sourceref }) => htmlnode!(self,mover,sourceref,"overline",node_top,node => {
                htmlannotate!(self,mrow,None,&mut Some(HTMLParent::N(&mut node)),mrow => {
                    self.ship_m(*content,&mut Some(HTMLParent::A(&mut mrow)))
                });
                htmlliteral!(self,&mut Some(HTMLParent::N(&mut node)),"&OverBar;")
            }),
            MathAccent(crate::stomach::math::MathAccent { content, accent, sourceref}) =>
                (),
            _ => htmlliteral!(self,node_top,"<!-- TODO -->" )
        }
    }
    fn ship_m<'a>(&mut self, w:Whatsit, node_top:&mut Option<HTMLParent<'a>>) {
        use Whatsit::*;
        use crate::stomach::simple::SimpleWI::*;
        use crate::stomach::boxes::TeXBox::*;
        use crate::stomach::groups::WIGroup::*;
        use crate::stomach::math::MathGroup;
        match w {
            Math(MathGroup {kernel,superscript:Some(sup),subscript:None,limits:false}) => {

            }
            Math(MathGroup {kernel,superscript:Some(sup),subscript:None,limits:true}) => {

            }
            Math(MathGroup {kernel,superscript:None,subscript:Some(sub),limits:false}) => {

            }
            Math(MathGroup {kernel,superscript:None,subscript:Some(sub),limits:true}) => {

            }
            Math(MathGroup {kernel,superscript:Some(sup),subscript:Some(sub),limits:false}) => {

            }
            Math(MathGroup {kernel,superscript:Some(sup),subscript:Some(sub),limits:true}) => {

            }
            Simple(MSkip(m)) => {
            }
            Box(mut bx) if match &bx {
                Void => true,
                H(hb) => hb.children.iter().all(|x| match x {
                    Math(_) => true,
                    _ => false
                }),
                V(vb) => vb.children.iter().all(|x| match x {
                    Math(_) => true,
                    _ => false
                })
            }  => {
                for c in bx.children() { self.ship_m(c,node_top)}
            }
            Box(_) => htmlnode!(self,mtext,None,"box",node_top,mt => {
                htmlnode!(self,HTML_NS:span,None,"box",&mut Some(HTMLParent::N(&mut mt)),span => {
                    self.ship_h(w,&mut Some(HTMLParent::N(&mut span)))
                })
            }),
            Simple(HAlign(_)) => (),
            Simple(HKern(m)) => {
                htmlnode!(self,mspace,m.sourceref,"mskip",node_top,a => {
                    a.attr("width".into(),dimtohtml(m.dim))
                })
            }
            Grouped(PDFLink(lnk)) => htmlannotate!(self,mrow,lnk.sourceref,node_top,node => {
                node.attr("href".into(),lnk.action.as_link().into());
                for c in lnk.children{ self.ship_m(c,&mut Some(HTMLParent::A(&mut node))) }
            }),
            Grouped(ColorChange(cc)) => {
                htmlannotate!(self,mrow,cc.sourceref,node_top,a => {
                    let mut color : HTMLStr = crate::stomach::groups::ColorChange::color_to_html(cc.color).into();
                    let hashcolor : HTMLStr = "#".into();
                    a.style("color".into(),hashcolor + &color);
                    let _oldcolor = std::mem::take(&mut self.state.currcolor);
                    self.state.currcolor = Some(color);
                    for c in cc.children { self.ship_m(c,&mut Some(HTMLParent::A(&mut a))) }
                    self.state.currcolor = _oldcolor;
                })
            }
            Simple(HSkip(h)) => {
                htmlnode!(self,mspace,h.sourceref,"hskip",node_top,a => {
                    a.attr("width".into(),dimtohtml(h.skip.base))
                })
            }
            Simple(Left(l)) => for c in l.bx { self.ship_m(c.as_whatsit(),node_top)},
            Simple(Middle(l)) => for c in l.bx { self.ship_m(c.as_whatsit(),node_top)},
            Simple(Right(l)) => for c in l.bx { self.ship_m(c.as_whatsit(),node_top)},
            Above(o) => (),
            Simple(Leaders(ld)) => {
                self.ship_m(ld.bx.clone().as_whatsit(),node_top);
                self.ship_m(ld.bx.clone().as_whatsit(),node_top);
                self.ship_m(ld.bx.as_whatsit(),node_top);
            }
            Simple(HFil(_)|HFill(_)) => {
                htmlnode!(self,mspace,None,"hfil",node_top,a => {
                    a.attr("width".into(),dimtohtml(655360))
                })
            }
            Simple(Penalty(_)) => (),
            _ => htmlliteral!(self,node_top,"<!-- TODO -->" )
        }
    }
     */
}

// -------------------------------------------------------------------------------------------------
#[derive(PartialEq,Clone)]
pub struct MiMoInfo {
    family:Option<HTMLStr>,
    weight:Option<HTMLStr>,
    style:Option<HTMLStr>,
    variant:Option<HTMLStr>,
    at:i32
}
impl MiMoInfo {
    pub fn new(font:&Arc<Font>) -> MiMoInfo {
        let mut ret = MiMoInfo { family:None,weight:None,style:None,variant:None,at:font.get_at()};
        match font.file.chartable {
            None => (),
            Some(ref tbl) => {
                if tbl.params.contains(&FontTableParam::Monospaced) {
                    ret.family = Some("monospace".into())
                }
                if tbl.params.contains(&FontTableParam::Italic) {
                    ret.style = Some("monospace".into())
                }
                if tbl.params.contains(&FontTableParam::Bold) {
                    ret.weight = Some("bold".into())
                }
                if tbl.params.contains(&FontTableParam::Blackboard) {
                    ret.family = Some("msbm".into())
                }
                if tbl.params.contains(&FontTableParam::Capital) {
                    ret.variant = Some("small-caps".into())
                }
                if tbl.params.contains(&FontTableParam::SansSerif) {
                    ret.family = Some("sans-serif".into())
                }
                if tbl.params.contains(&FontTableParam::Script) {
                    ret.family = Some("eusb".into())
                }
            }
        }
        ret
    }
}
pub struct HTMLNode {
    pub name:HTMLStr,
    pub namespace:&'static str,
    pub children:Vec<HTMLChild>,
    pub classes:Vec<HTMLStr>,
    attributes:HashMap<HTMLStr,HTMLStr>,
    styles:HashMap<HTMLStr,HTMLStr>,
    pub mimoinfo: Option<MiMoInfo>,
    pub sourceref:Option<SourceFileReference>
}

#[macro_export]
macro_rules! htmlparent {
    ($e:tt) => (&mut Some($e.as_parent()))
}
impl HTMLNode {
    pub fn as_parent(&mut self) -> HTMLParent {
        HTMLParent::N(self)
    }
    pub fn new(namespace:&'static str,name:HTMLStr,sourceref:Option<SourceFileReference>) -> HTMLNode { HTMLNode {
        name,namespace,children:vec!(),classes:vec!(),
        attributes:HashMap::new(),styles:HashMap::new(),mimoinfo:None,sourceref
    }}
    pub fn attr(&mut self,name:HTMLStr,value:HTMLStr) {
        self.attributes.borrow_mut().insert(name,value);
    }
    pub fn style(&mut self,name:HTMLStr,value:HTMLStr) {
        self.styles.borrow_mut().insert(name,value);
    }
    pub fn make_string(&mut self,prefix:HTMLStr,namespace:&str) -> HTMLStr {
        match self.mimoinfo.take() {
            None => (),
            Some(mi) => {
                for s in mi.family { self.style("family".into(),s)}
                for s in mi.weight { self.style("weight".into(),s)}
                for s in mi.style { self.style("style".into(),s)}
                for s in mi.variant { self.style("variant".into(),s)}
                // TODO size
            }
        }
        if self.children.len() == 1 {
            match self.children.first_mut() {
                Some(HTMLChild::Annot(a)) => {
                    let mut works : bool = true;
                    for k in self.attributes.keys() {
                        if a.attributes.keys().contains(k) {
                            works = false;
                            break
                        }
                    }
                    if works {
                        for (k,v) in std::mem::take(&mut a.attributes) { self.attributes.insert(k,v); }
                        for (k,v) in std::mem::take(&mut a.styles) { self.styles.insert(k,v); }
                    }
                    self.children = std::mem::take(&mut a.children)
                }
                _ => ()
            }
        }
        let mut ret : HTMLStr = "".into();
        let mut body : HTMLStr = "".into();
        for c in self.children.drain(..) {
            body += match c {
                HTMLChild::Node(mut n) => n.make_string(prefix.clone(),self.namespace),
                HTMLChild::Annot(mut a) => a.make_string(prefix.clone(),self.namespace),
                HTMLChild::Str(mut s) => s.clone(),
            }
        }
        ret += "<" + &self.name;
        for (a,v) in &self.attributes {
            ret += " " + a + "=\"" + v + "\""
        }
        self.classes = self.classes.drain(..).filter(|x| x.to_string() != "").collect();
        if !self.classes.is_empty() {
            ret += " class=\"" + &self.classes[0];
            for c in &self.classes[1..] { ret += " " + c}
            ret += "\""
        }
        if !self.styles.is_empty() {
            ret += " style=\"";
            for (s,v) in &self.styles { ret += s + ":" + v + ";" }
            ret += "\"";
        }
        if namespace != self.namespace {
            ret += " xmlns=\"";ret += self.namespace;ret += "\""
        }
        match &self.sourceref {
            Some(s) => {
                ret += " rustex:sourceref=\"";
                ret += HTMLStr::from(s.as_string()).html_escape();
                ret += "\""
            }
            _ => ()
        }
        ret += ">";
        ret += body;
        ret + "</" + &self.name + ">"
    }
}

pub struct HTMLAnnotation {
    pub name:HTMLStr,
    pub namespace:&'static str,
    pub children:Vec<HTMLChild>,
    pub classes:Vec<HTMLStr>,
    attributes:HashMap<HTMLStr,HTMLStr>,
    styles:HashMap<HTMLStr,HTMLStr>,
    pub sourceref:Option<SourceFileReference>
}
impl HTMLAnnotation {
    pub fn as_parent(&mut self) -> HTMLParent {
        HTMLParent::A(self)
    }
    pub fn new(namespace:&'static str,name:HTMLStr,sourceref:Option<SourceFileReference>) -> HTMLAnnotation { HTMLAnnotation {
        name,namespace,children:vec!(),classes:vec!(),
        attributes:HashMap::new(),styles:HashMap::new(),sourceref
    }}
    pub fn attr(&mut self,name:HTMLStr,value:HTMLStr) {
        self.attributes.borrow_mut().insert(name,value);
    }
    pub fn style(&mut self,name:HTMLStr,value:HTMLStr) {
        self.styles.borrow_mut().insert(name,value);
    }
    pub fn make_string(&mut self,prefix:HTMLStr,namespace:&str) -> HTMLStr {
        if self.children.len() == 1 {
            match self.children.first_mut() {
                Some(HTMLChild::Node(n)) => {
                    let mut works : bool = true;
                    for k in self.attributes.keys() {
                        if n.attributes.keys().contains(k) {
                            works = false;
                            break
                        }
                    }
                    if works {
                        for (k,v) in std::mem::take(&mut self.attributes) {
                            n.attributes.insert(k,v);
                        }
                        for (k,v) in std::mem::take(&mut self.styles) {
                            if n.styles.get(&k).is_none() { n.styles.insert(k,v); }
                        }
                        return n.make_string(prefix,namespace)
                    }
                }
                Some(HTMLChild::Annot(a)) => {
                    let mut works : bool = true;
                    for k in self.attributes.keys() {
                        if a.attributes.keys().contains(k) {
                            works = false;
                            break
                        }
                    }
                    if works {
                        for (k,v) in std::mem::take(&mut self.attributes) {
                            a.attributes.insert(k,v);
                        }
                        for (k,v) in std::mem::take(&mut self.styles) {
                            if a.styles.get(&k).is_none() { a.styles.insert(k,v); }
                        }
                        return a.make_string(prefix,namespace)
                    }
                }
                _ => ()
            }
        }
        HTMLNode {
            name: self.name.clone(),
            namespace:self.namespace,
            children: std::mem::take(&mut self.children),
            classes: std::mem::take(&mut self.classes),
            attributes: std::mem::take(&mut self.attributes),
            styles: std::mem::take(&mut self.styles),
            mimoinfo: None,
            sourceref: std::mem::take(&mut self.sourceref)
        }.make_string(prefix,namespace)
    }
}

pub enum HTMLParent<'a> {
    N(&'a mut HTMLNode),
    A(&'a mut HTMLAnnotation)
}
impl HTMLParent<'_> {
    pub fn push(&mut self,wi:HTMLChild) {
        match self {
            HTMLParent::N(p) => p.children.push(wi),
            HTMLParent::A(p) => p.children.push(wi)
        }
    }
}

pub enum HTMLChild {
    Str(HTMLStr),
    Node(HTMLNode),
    Annot(HTMLAnnotation)
}
impl HTMLChild {
    pub fn make_string(mut self,prefix:HTMLStr,namespace:&str) -> HTMLStr {
        match self {
            HTMLChild::Str(mut s) => s,
            HTMLChild::Node(mut n) => n.make_string(prefix,namespace),
            HTMLChild::Annot(mut a) => a.make_string(prefix,namespace)
        }
    }
    pub fn stringify(&mut self,prefix:HTMLStr,namespace:&str) {
        match self {
            HTMLChild::Str(_) => (),
            HTMLChild::Node(n) => {
                *self = HTMLChild::Str(n.make_string(prefix,namespace))
            }
            HTMLChild::Annot(n) => {
                *self = HTMLChild::Str(n.make_string(prefix,namespace))
            }
        }
    }
}

// -------------------------------------------------------------------------------------------------

#[derive(Clone)]
pub enum HTMLStr {
    Imm(&'static str),
    Mut(String)
}
impl HTMLStr {
    pub fn to_string(self) -> String { self.into() }
}
impl Display for HTMLStr {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        use HTMLStr::*;
        match self {
            Imm(s) => core::fmt::Display::fmt(s,f),
            Mut(s) => core::fmt::Display::fmt(s,f)
        }
    }
}
impl HTMLStr {
    fn vec(&self) -> &[u8] {
        use HTMLStr::*;
        match self {
            Imm(a) => a.as_bytes(),
            Mut(s) => s.as_bytes(),
        }
    }
    pub fn take(self) -> HTMLStr {
        use HTMLStr::*;
        match self {
            Mut(s) => Mut(s),
            Imm(s) => Mut(s.to_string())
        }
    }
    pub fn html_escape(self) -> HTMLStr {
        let retstr : String = self.to_string().chars().map(|c| match c {
            '>' => "&gt;".to_string(),
            '<' => "&lt;".to_string(),
            '&' => "&amp;".to_string(),
            o => o.to_string()
        }).collect();
        retstr.into()
    }
}
impl Hash for HTMLStr {
    fn hash<H: Hasher>(&self, state: &mut H) {
        (*self.vec()).hash(state)
    }
}
impl PartialEq for HTMLStr {
    fn eq(&self, other: &Self) -> bool {
        *self.vec() == *other.vec()
    }
}
impl Eq for HTMLStr {}
impl From<&'static str> for HTMLStr {
    fn from(s: &'static str) -> Self {
        HTMLStr::Imm(s)
    }
}
impl From<String> for HTMLStr {
    fn from(s: String) -> Self {
        HTMLStr::Mut(s)
    }
}
impl From<TeXStr> for HTMLStr {
    fn from(s: TeXStr) -> Self {
        HTMLStr::Mut(s.to_utf8())
    }
}
impl AddAssign<HTMLStr> for HTMLStr {
    fn add_assign(&mut self, rhs: HTMLStr) {
        self.add_assign(&rhs)
    }
}
impl AddAssign<&HTMLStr> for HTMLStr {
    fn add_assign(&mut self, rhs: &HTMLStr) {
        use HTMLStr::*;
        match (&self,rhs) {
            (Mut(s),Imm(t)) => *self = Mut(s.to_owned() + t),
            (Mut(s),Mut(t)) => *self = Mut(s.to_owned() + t),
            (Imm(s),Imm(t)) => *self = Mut(s.to_string() + t),
            (Imm(s),Mut(t)) => *self = Mut(s.to_string() + &t),
        }
    }
}
impl AddAssign<&mut HTMLStr> for HTMLStr {
    fn add_assign(&mut self, rhs: &mut HTMLStr) {
        use HTMLStr::*;
        match (&self,rhs) {
            (Mut(s),Imm(t)) => *self = Mut(s.to_owned() + t),
            (Mut(s),Mut(t)) => *self = Mut(s.to_owned() + t),
            (Imm(s),Imm(t)) => *self = Mut(s.to_string() + t),
            (Imm(s),Mut(t)) => *self = Mut(s.to_string() + &t),
        }
    }
}
impl AddAssign<&str> for HTMLStr {
    fn add_assign(&mut self, rhs: &str) {
        use HTMLStr::*;
        match &self {
            Mut(s) => *self = Mut(s.to_owned() + rhs),
            Imm(s) => *self = Mut(s.to_string() + rhs)
        }
    }
}
impl AddAssign<String> for HTMLStr {
    fn add_assign(&mut self, rhs: String) {
        use HTMLStr::*;
        match &self {
            Mut(s) => *self = Mut(s.to_owned() + &rhs),
            Imm(s) => *self = Mut(s.to_string() + &rhs)
        }
    }
}

impl Add<&str> for HTMLStr {
    type Output = HTMLStr;
    fn add(self, rhs: &str) -> Self::Output {
        use HTMLStr::*;
        match self {
            Mut(s) => Mut(s + rhs),
            Imm(s) => Mut(s.to_string() + rhs)
        }
    }
}
impl Add<HTMLStr> for HTMLStr {
    type Output = HTMLStr;
    fn add(self, rhs: HTMLStr) -> Self::Output {
        use HTMLStr::*;
        match (self,rhs) {
            (Mut(s),Mut(t)) => Mut(s + &t),
            (Mut(s),Imm(t)) => Mut(s + t),
            (Imm(s),Mut(t)) => Mut(s.to_string() + &t),
            (Imm(s),Imm(t)) => Mut(s.to_string() + t),
        }
    }
}
impl Add<&str> for &HTMLStr {
    type Output = HTMLStr;
    fn add(self, rhs: &str) -> Self::Output {
        use HTMLStr::*;
        match self {
            Mut(s) => Mut(s.to_owned() + rhs),
            Imm(s) => Mut(s.to_string() + rhs)
        }
    }
}
impl Add<HTMLStr> for &HTMLStr {
    type Output = HTMLStr;
    fn add(self, rhs: HTMLStr) -> Self::Output {
        use HTMLStr::*;
        match (self,rhs) {
            (Mut(s),Mut(t)) => Mut(s.to_owned() + &t),
            (Mut(s),Imm(t)) => Mut(s.to_owned() + t),
            (Imm(s),Mut(t)) => Mut(s.to_string() + &t),
            (Imm(s),Imm(t)) => Mut(s.to_string() + t),
        }
    }
}
impl Add<&HTMLStr> for HTMLStr {
    type Output = HTMLStr;
    fn add(self, rhs: &HTMLStr) -> Self::Output {
        use HTMLStr::*;
        match (self,rhs) {
            (Mut(s),Mut(t)) => Mut(s + &t),
            (Mut(s),Imm(t)) => Mut(s + t),
            (Imm(s),Mut(t)) => Mut(s.to_string() + &t),
            (Imm(s),Imm(t)) => Mut(s.to_string() + t),
        }
    }
}
impl Add<HTMLStr> for &str {
    type Output = HTMLStr;
    fn add(self, rhs: HTMLStr) -> Self::Output {
        use HTMLStr::*;
        match rhs {
            Mut(s) => Mut(self.to_string() + &s),
            Imm(s) => Mut(self.to_string() + s)
        }
    }

}
impl Add<&HTMLStr> for &str {
    type Output = HTMLStr;
    fn add(self, rhs: &HTMLStr) -> Self::Output {
        use HTMLStr::*;
        match rhs {
            Mut(s) => Mut(self.to_string() + &s),
            Imm(s) => Mut(self.to_string() + s)
        }
    }
}
impl From<HTMLStr> for String {
    fn from(s: HTMLStr) -> Self {
        match s {
            HTMLStr::Mut(s) => s,
            HTMLStr::Imm(s) => s.to_string()
        }
    }
}
impl From<&HTMLStr> for String {
    fn from(s: &HTMLStr) -> Self {
        match s {
            HTMLStr::Mut(s) => s.clone(),
            HTMLStr::Imm(s) => s.to_string()
        }
    }
}