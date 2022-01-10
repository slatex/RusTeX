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
use crate::stomach::colon::{Colon, ColonBase};
use crate::stomach::groups::WIGroupTrait;
use crate::stomach::math::{GroupedMath, MathKernel};
use crate::stomach::simple::PDFMatrix;
use crate::stomach::Whatsit;
use crate::stomach::whatsits::WhatsitTrait;
use crate::utils::TeXStr;


fn dimtohtml(num:i32) -> HTMLStr {
    numtostr(num,"px").into()
}

static HTML_NS : &str = "http://www.w3.org/1999/xhtml";
static MATHML_NS : &str = "http://www.w3.org/1998/Math/MathML";
static SVG_NS : &str = "http://www.w3.org/2000/svg";
static RUSTEX_NS : &str = "http://kwarc.info/ns/RusTeX";

pub struct HTMLState {
    annotations_todo:Vec<(HTMLStr,HTMLStr)>,
    current_namespace:&'static str,
    top:Vec<HTMLChild>,
    currsize:i32,
    currcolor:Option<HTMLStr>
}
impl HTMLState {
    pub fn new() -> HTMLState { HTMLState {
        annotations_todo:vec!(),
        current_namespace:HTML_NS,
        top:vec!(),
        currsize:0,currcolor:None
    }}
}
macro_rules! node {
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

macro_rules! literal {
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

macro_rules! annotate {
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
    state:HTMLState
}
unsafe impl Send for HTMLColon {}

impl Colon<String> for HTMLColon {
    fn base(&self) -> &ColonBase { &self.base }
    fn base_mut(&mut self) -> &mut ColonBase { &mut self.base }
    fn ship_whatsit(&mut self, w:Whatsit) { /*for w in self.normalize_whatsit(w) { */self.ship_top(w,&mut None) } //}
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
    fn ship_top<'a>(&mut self,w:Whatsit,node_top:&mut Option<HTMLParent<'a>>) {
        use Whatsit::*;
        use crate::stomach::simple::SimpleWI::*;
        use crate::stomach::boxes::TeXBox::*;
        use crate::interpreter::dimensions::SkipDim;
        use crate::stomach::groups::WIGroup::*;
        use crate::fonts::fontchars::FontTableParam;
        use crate::stomach::simple::ExternalParam;
        match w {
            Simple(VFil(v)) => node!(self,div,v.0,"vfil",node_top),
            Simple(VFill(v)) => node!(self,div,v.0,"vfill",node_top),
            Simple(PDFDest(d)) if d.dest.to_string() == "xyz" => {
                node!(self,a,d.sourceref,"pdfdest",node_top,node => {
                    node.attr("id".into(),d.target.clone().into());
                    node.attr("name".into(),d.target.into());
                })
            },
            Simple(VSkip(vsk)) => {
                node!(self,div,vsk.sourceref,"vskip",node_top,node => {
                    node.style("margin-bottom".into(),dimtohtml(vsk.skip.base));
                })
            },
            Simple(VKern(vsk)) => {
                node!(self,div,vsk.sourceref,"vkern",node_top,node => {
                    node.style("margin-bottom".into(),dimtohtml(vsk.dim));
                })
            },
            Simple(HRule(hr)) => {
                node!(self,div,hr.sourceref.clone(),"hrule",node_top,n => {
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
                annotate!(self,span,cc.sourceref,node_top,a => {
                    let mut color : HTMLStr = crate::stomach::groups::ColorChange::as_html(cc.color).into();
                    let hashcolor : HTMLStr = "#".into();
                    a.style("color".into(),hashcolor + &color);
                    let _oldcolor = std::mem::take(&mut self.state.currcolor);
                    self.state.currcolor = Some(color);
                    for c in cc.children { self.ship_top(c,&mut Some(HTMLParent::A(&mut a))) }
                    self.state.currcolor = _oldcolor;
                })
            }
            Grouped(FontChange(fc)) => {
                match &fc.font.file.chartable {
                    Some(ft) => {
                        annotate!(self,span,fc.sourceref,node_top,a => {
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
                            for c in fc.children { self.ship_top(c,&mut Some(HTMLParent::A(&mut a))) }
                            self.state.currsize = _oldsize;
                        })
                    }
                    _ => {
                        for c in fc.children { self.ship_top(c,node_top) }
                    }
                }
            }
            Par(p) => {
                node!(self,div,None,"paragraph",node_top,node => {
                    match p.leftskip {
                        Some(sk) if match sk.stretch {
                            Some(SkipDim::Fil(_) | SkipDim::Fill(_) | SkipDim::Filll(_)) => true,
                            _ => false
                        } => match p.rightskip {
                            Some(sk) if match sk.stretch {
                                Some(SkipDim::Fil(_) | SkipDim::Fill(_) | SkipDim::Filll(_)) => true,
                                _ => false
                            } => {
                                node.style("text-align".into(),"center".into());
                                node.style("justify-content".into(),"center".into());
                                node.style("align-items".into(),"center".into());
                            }
                            _ => {
                                node.style("text-align".into(),"right".into());
                                node.style("justify-content".into(),"right".into());
                                node.style("align-items".into(),"right".into());
                            }
                        }
                        _ => match p.rightskip {
                            Some(sk) if match sk.stretch {
                                Some(SkipDim::Fil(_) | SkipDim::Fill(_) | SkipDim::Filll(_)) => true,
                                _ => false
                            } => {
                                node.style("text-align".into(),"left".into());
                                node.style("justify-content".into(),"left".into());
                                node.style("align-items".into(),"left".into());
                            },
                            _ => ()
                        }
                    }
                    match p.leftskip {
                        Some(sk) if sk.base != 0 => node.style("margin-left".into(),dimtohtml(sk.base)),
                        _ => ()
                    }
                    match p.rightskip {
                        Some(sk) if sk.base != 0 => node.style("margin-right".into(),dimtohtml(sk.base)),
                        _ => ()
                    }
                    if p.parskip != 0 {
                        node.style("margin-top".into(),dimtohtml(p.parskip))
                    }
                    node.style("width".into(),dimtohtml(p.width()));
                    node.style("min-width".into(),dimtohtml(p.width()));
                    for c in p.children { self.ship_h(c,&mut Some(HTMLParent::N(&mut node))) }
                });
            }
            Simple(Vss(v)) => node!(self,div,v.0,"vss",node_top),
            Simple(HAlign(ha)) => {
                use crate::stomach::simple::AlignBlock;
                node!(self,table,ha.sourceref,"halign",node_top,table => {
                    if ha.skip.base != 0 {
                        table.style("margin-top".into(),dimtohtml(ha.skip.base))
                    }
                    for row in ha.rows {
                        match row {
                            AlignBlock::Noalign(mut v) => {
                                if v.len() == 1 {
                                    match v.pop() {
                                        Some(Simple(HRule(hr))) => {
                                            if table.children.is_empty() {
                                                table.style("border-top".into(),dimtohtml(hr.height()) + " solid")
                                            } else {
                                                match table.children.last_mut() {
                                                    Some(HTMLChild::Node(row)) => row.style("border-bottom".into(),dimtohtml(hr.height()) + " solid"),
                                                    _ => unreachable!()
                                                }
                                            }
                                        }
                                        _ => ()
                                    }
                                } else {
                                    print!("")
                                }
                            }
                            AlignBlock::Block(cells) => {
                                node!(self,tr,None,"row",&mut Some(HTMLParent::N(&mut table)),row => {
                                    for (mut vs,skip,num) in cells {
                                        node!(self,td,None,"cell",&mut Some(HTMLParent::N(&mut row)),cell => {
                                            cell.style("margin-right".into(),dimtohtml(skip.base));
                                            if num > 1 { cell.attr("colspan".into(),num.to_string().into()) }
                                            let mut alignment = (false,false);
                                            loop {
                                                match vs.pop() {
                                                    Some(Simple(VRule(v))) => cell.style("border-right".into(),dimtohtml(v.width()) + " solid"),
                                                    Some(Simple(HFil(_) | HFill(_))) => alignment.1 = true,
                                                    Some(o) => {vs.push(o);break}
                                                    None => break
                                                }
                                            }
                                            let mut incell : bool = false;
                                            node!(self,div,None,"hbox",&mut Some(HTMLParent::N(&mut cell)),bx => {
                                                for w in vs { match w {
                                                    Simple(VRule(v)) if !incell => cell.style("border-left".into(),dimtohtml(v.width()) + " solid"),
                                                    Simple(HFil(_) | HFill(_)) if !incell => alignment.0 = true,
                                                    o => {
                                                        incell = true;
                                                        self.ship_h(o,&mut Some(HTMLParent::N(&mut bx)))
                                                    }
                                                }}
                                            });
                                            match alignment {
                                                (true,true) => cell.style("text-align".into(),"center".into()),
                                                (true,false) => cell.style("text-align".into(),"right".into()),
                                                _ => cell.style("text-align".into(),"left".into()),
                                            }
                                        })
                                    }
                                })
                            }
                        }
                    }
                })
            }
            Box(H(hb)) => {
                node!(self,div,None,"hbox",node_top,node => {
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
                literal!(self,node_top,"<hr/>");
                for v in is.0 { for w in v { self.ship_top(w,node_top) }}
            }
            Simple(Penalty(_)) => (),
            Simple(crate::stomach::simple::SimpleWI::External(ext)) if ext.name().to_string() == "pgfbox" => {
                node!(self,SVG_NS:svg,ext.sourceref().clone(),"",node_top,svg => {
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
                    node!(self,g,None,"",&mut Some(HTMLParent::N(&mut svg)),g => {
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
            Float(bx) => {
                node!(self,div,None,"vfill",node_top);
                self.ship_top(bx.as_whatsit(),node_top);
                node!(self,div,None,"vfill",node_top)
            }
            Box(V(vb)) if vb._height.is_none() => {
                for c in vb.children { self.ship_top(c,node_top) }
            }
            Box(V(vb)) => {
                node!(self,div,None,"vbox",node_top,node => {
                    match vb._height {
                        Some(v) => {
                            node.style("height".into(),dimtohtml(v));
                            node.style("min-height".into(),dimtohtml(v))
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
            Simple(MoveRight(crate::stomach::simple::MoveRight { dim,content:bx,sourceref })) => {
                annotate!(self,div,sourceref,node_top,a => {
                    a.classes.push("moveright".into());
                    a.style("margin-left".into(),dimtohtml(dim));
                    self.ship_top(bx.as_whatsit(),&mut Some(HTMLParent::A(&mut a)))
                })
            }
            Simple(crate::stomach::simple::SimpleWI::External(ext)) if ext.name().to_string() == "pgfliteral" => (),
            Simple(crate::stomach::simple::SimpleWI::External(ext)) => {
                println!("TODO: {}",ext.as_xml());
                literal!(self,node_top,"<!-- TODO:".to_string() + &ext.as_xml() +  "-->")
            }
            Simple(Mark(_)) | Box(Void) => (),
            _ => literal!(self,node_top,"<!-- TODO -->")//self.ret += &w.as_xml_internal("  ".to_string())
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
                node!(self,a,d.sourceref,"pdfdest",node_top,node => {
                    node.attr("id".into(),d.target.clone().into());
                    node.attr("name".into(),d.target.into());
                })
            },
            Simple(Penalty(p)) if p.penalty <= -10000 => literal!(self,node_top,"<br/>"),
            Grouped(ColorChange(cc)) => {
                annotate!(self,span,cc.sourceref,node_top,a => {
                    let mut color : HTMLStr = crate::stomach::groups::ColorChange::as_html(cc.color).into();
                    let hashcolor : HTMLStr = "#".into();
                    a.style("color".into(),hashcolor + &color);
                    let _oldcolor = std::mem::take(&mut self.state.currcolor);
                    self.state.currcolor = Some(color);
                    for c in cc.children { self.ship_h(c,&mut Some(HTMLParent::A(&mut a))) }
                    self.state.currcolor = _oldcolor;
                })
            }
            Grouped(PDFLink(lnk)) => {
                node!(self,a,lnk.sourceref,"pdflink",node_top,a => {
                    a.attr("href".into(),lnk.action.as_link().into());
                    for c in lnk.children { self.ship_h(c,&mut Some(HTMLParent::N(&mut a))) }
                })
            }
            Grouped(FontChange(fc)) => {
                match &fc.font.file.chartable {
                    Some(ft) => {
                        annotate!(self,span,fc.sourceref,node_top,a => {
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
            Grouped(PDFMatrixSave(sg)) => match sg.children.iter().filter(|x| match x {
                    Simple(PDFMatrix(_)) => true,
                    _ => false
                }).next() {
                Some(Simple(PDFMatrix(matrix))) => {
                    node!(self,span,sg.sourceref,"pdfmatrix",node_top,m => {
                        m.style("transform-origin".into(),"top left".into());
                        let mut tf : HTMLStr = "matrix(".into();
                        tf += matrix.scale.to_string();
                        tf += ",";
                        tf += matrix.rotate.to_string();
                        tf += ",";
                        tf += matrix.skewx.to_string();
                        tf += ",";
                        tf += matrix.skewy.to_string();
                        tf += ",0,0)";
                        m.style("transform".into(),tf);
                        for c in sg.children {
                            self.ship_h(c,&mut Some(HTMLParent::N(&mut m)))
                        }
                    })
                }
                _ => {
                    for c in sg.children { self.ship_h(c, node_top) }
                }
            }
            Simple(PDFMatrix(_)) => (),
            Simple(PDFXImage(pimg)) => {
                match pimg.image {
                    Some(ref img) => {
                        let mut buf:Vec<u8> = vec!();
                        img.write_to(&mut buf, image::ImageOutputFormat::Png);
                        let res_base64 = "data:image/png;base64,".to_string() + &base64::encode(&buf);
                        node!(self,img,pimg.sourceref.clone(),"",node_top,i => {
                                i.attr("src".into(),res_base64.into());
                                i.attr("width".into(),dimtohtml(pimg.width()));
                                let ht : HTMLStr = img.height().to_string().into();
                                i.attr("height".into(),dimtohtml(pimg.height()));
                            })
                    }
                    _ => ()
                }
            }
            Char(pc) => literal!(self,node_top,>{
                match &pc.font.file.chartable {
                    Some(ct) => ct.get_char(pc.char).to_string(),
                    None => pc.as_xml_internal("".to_string())
                }
            }<),
            Space(_) => literal!(self,node_top," "),
            Simple(VRule(vr)) => {
                node!(self,span,vr.sourceref.clone(),"vrule",node_top,n => {
                    n.style("width".into(),dimtohtml(vr.width()));
                    n.style("min-width".into(),dimtohtml(vr.width()));
                    n.style("height".into(),dimtohtml(vr.height() + vr.depth()));
                    n.style("min-height".into(),dimtohtml(vr.height() + vr.height()));
                    n.style("background".into(),match &self.state.currcolor {
                        Some(c) => HTMLStr::from("#") + c,
                        None => "#000000".into()
                    });
                    if vr.depth() != 0 { n.style("margin-bottom".into(),dimtohtml(-vr.depth())) }
                })
            }
            Simple(HRule(vr)) => { // from Leaders
                node!(self,span,vr.sourceref.clone(),"vrule",node_top,n => {
                    n.style("width".into(),dimtohtml(vr.width()));
                    n.style("min-width".into(),dimtohtml(vr.width()));
                    n.style("height".into(),dimtohtml(vr.height() + vr.depth()));
                    n.style("min-height".into(),dimtohtml(vr.height() + vr.height()));
                    n.style("background".into(),match &self.state.currcolor {
                        Some(c) => HTMLStr::from("#") + c,
                        None => "#000000".into()
                    });
                    if vr.depth() != 0 { n.style("margin-bottom".into(),dimtohtml(-vr.depth())) }
                })
            }
            Simple(HSkip(vsk)) => {
                node!(self,span,vsk.sourceref,"hskip",node_top,node => {
                    node.style("margin-left".into(),dimtohtml(vsk.skip.base));
                })
            },
            Simple(Indent(dim)) => {
                node!(self,span,dim.sourceref,"indent",node_top,node => {
                    node.style("margin-left".into(),dimtohtml(dim.dim));
                })
            },
            Simple(HKern(vsk)) => {
                node!(self,span,vsk.sourceref,"hkern",node_top,node => {
                    node.style("margin-left".into(),dimtohtml(vsk.dim));
                })
            },
            Box(V(vb)) => {
                node!(self,div,None,"vbox",node_top,node => {
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
            Simple(HFil(h)) => node!(self,span,h.0,"hfil",node_top),
            Simple(HFill(h)) => node!(self,span,h.0,"hfill",node_top),
            Simple(Hss(h)) => node!(self,span,h.0,"hss",node_top),
            Box(H(hb)) => {
                node!(self,div,None,"hbox",node_top,node => {
                    match hb._width {
                        Some(h) => {
                            node.style("width".into(),dimtohtml(h));
                            node.style("min-width".into(),dimtohtml(h))
                        }
                        _ => ()
                    }
                    match hb._height {
                        Some(v) => {
                            node.style("height".into(),dimtohtml(v));
                            node.style("min-height".into(),dimtohtml(v))
                        }
                        _ => ()
                    }
                    for c in hb.children { self.ship_h(c,&mut Some(HTMLParent::N(&mut node))) }
                })
            }
            Simple(Raise(r)) => node!(self,span,r.sourceref,"raise",node_top,node => {
                node.style("bottom".into(),dimtohtml(r.dim));
                self.ship_h(r.content.as_whatsit(),&mut Some(HTMLParent::N(&mut node)))
            }),
            Math(ref mg) if mg.limits => node!(self,div,None,"displaymathcontainer",node_top,div =>{
                div.style("width".into(),"100%".into());
                div.style("min-width".into(),"100%".into());
                node!(self,MATHML_NS:math,None,"",&mut Some(HTMLParent::N(&mut div)),node=> {
                    node.attr("displaystyle".into(),"true".into());
                    node!(self,mrow,None,"math",&mut Some(HTMLParent::N(&mut node)),mrow => {
                        self.ship_m(w,&mut Some(HTMLParent::N(&mut mrow)));
                        if mrow.children.len() == 1 {
                            match mrow.children.pop().unwrap() {
                                HTMLChild::Node(n) if n.name == "mrow".into() => mrow.children = n.children,
                                o => mrow.children.push(o)
                            }
                        }
                    })
                })
            }),
            Math(ref mg) => node!(self,MATHML_NS:math,None,"math",node_top,node=> {
                node!(self,mrow,None,"math",&mut Some(HTMLParent::N(&mut node)),mrow => {
                    self.ship_m(w,&mut Some(HTMLParent::N(&mut mrow)));
                    if mrow.children.len() == 1 {
                        match mrow.children.pop().unwrap() {
                            HTMLChild::Node(n) if n.name == "mrow".into() => mrow.children = n.children,
                            o => mrow.children.push(o)
                        }
                    }
                })
            }),
            Simple(Leaders(ld)) => {
                self.ship_h(ld.bx.clone().as_whatsit(),node_top);
                self.ship_h(ld.bx.clone().as_whatsit(),node_top);
                self.ship_h(ld.bx.as_whatsit(),node_top);
            }
            Simple(crate::stomach::simple::SimpleWI::External(ext)) if ext.name().to_string() == "pgfbox" => {
                node!(self,SVG_NS:svg,ext.sourceref().clone(),"",node_top,svg => {
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
                    node!(self,g,None,"",&mut Some(HTMLParent::N(&mut svg)),g => {
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
                literal!(self,node_top,"<!-- TODO:".to_string() + &ext.as_xml() +  "-->")
            }
            Simple(Penalty(_)) => (), Simple(HAlign(_)) => self.ship_top(w,node_top),
            Simple(Mark(_)) | Box(TeXBox::Void) => (),
            _ => literal!(self,node_top,"<!-- TODO -->" )
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
                literal!(self,node_top,match ext.params("string") {
                    Some(ExternalParam::String(s)) => s,
                    _ => unreachable!()
                })
            }
            Simple(crate::stomach::simple::SimpleWI::External(ext)) if ext.name().to_string() == "pgfescape" => {
                let bx = match ext.params("box") {
                    Some(ExternalParam::Whatsits(mut v)) => v.pop().unwrap(),
                    _ => unreachable!()
                };
                //node!(self,g,"",node_top,g => {
                //    g.attr("transform".into(),"scale(1,-1)".into());
                    node!(self,foreignObject,ext.sourceref().clone(),"",node_top/*&mut Some(HTMLParent::N(&mut g))*/,fo => {
                        let wd = bx.width();
                        let ht = bx.height() + bx.depth();
                        fo.style("width".into(),dimtohtml(wd));
                        fo.style("height".into(),dimtohtml(ht));
                        /*let mut trans : HTMLStr = "translate(0,".into();
                        trans += dimtohtml(-ht);
                        trans += ")";
                        fo.style("transform".into(),trans.into());*/
                        node!(self,HTML_NS:div,None,"foreign",&mut Some(HTMLParent::N(&mut fo)),div => {
                            //div.style("transform-origin".into(),"top left".into());
                            self.ship_h(bx,&mut Some(HTMLParent::N(&mut div)))
                        })
                    })
               // })
            }
            Grouped(ColorChange(cc)) => {
                annotate!(self,g,cc.sourceref,node_top,a => {
                    let mut color : HTMLStr = crate::stomach::groups::ColorChange::as_html(cc.color).into();
                    let hashcolor : HTMLStr = "#".into();
                    a.style("color".into(),hashcolor + &color);
                    let _oldcolor = std::mem::take(&mut self.state.currcolor);
                    self.state.currcolor = Some(color);
                    for c in cc.children { self.ship_h(c,&mut Some(HTMLParent::A(&mut a))) }
                    self.state.currcolor = _oldcolor;
                })
            }
            Grouped(wg) => for c in wg.children_prim() { self.ship_svg(c,node_top) }
            Box(H(hb)) => for c in hb.children { self.ship_svg(c,node_top) }
            Simple(crate::stomach::simple::SimpleWI::External(ext)) => {
                println!("TODO: {}",ext.as_xml());
                literal!(self,node_top,"<!-- TODO:".to_string() + &ext.as_xml() +  "-->")
            }
            _ => literal!(self,node_top,"<!-- TODO -->" )
        }
    }
    fn ship_kernel<'a>(&mut self, k:MathKernel, node_top:&mut Option<HTMLParent<'a>>) {
        use crate::stomach::math::MathKernel::*;
        match k {
            Group(gm) if gm.0.is_empty() => (),
            Group(mut gm) if gm.0.len() == 1 => self.ship_m(gm.0.pop().unwrap(),node_top),
            Group(GroupedMath(ls)) => annotate!(self,mrow,None,node_top,node => {
                for w in ls { self.ship_m(w,&mut Some(HTMLParent::A(&mut node))) }
            }),
            MathChar(mc) => {
                let maybemimo = match match node_top {
                    Some(HTMLParent::N(n)) => n.children.last_mut(),
                    Some(HTMLParent::A(n)) => n.children.last_mut(),
                    _ => None
                } {
                    Some(HTMLChild::Node(n)) => Some(n),
                    _ => None
                };
                let charstr : HTMLStr = match &mc.font.file.chartable {
                    Some(ct) => ct.get_char(mc.position as u8).into(),
                    None => {
                        //println!("Here! {} in {}",mc.position,mc.font.name);
                        "???".into()
                    }
                };
                let mimoinfo = MiMoInfo::new(&mc.font);
                let clsstr : HTMLStr = (match mc.class {
                    1 => "largeop",
                    2 => "op",
                    3 => "rel",
                    4 => "opening",
                    5 => "closing",
                    6 => "punctuation",
                    _ => "",
                }).into();
                match (maybemimo,mc.class) {
                    (Some(n),0|7) if String::from(&n.name) == "mi" && n.mimoinfo.is_some() && n.mimoinfo.as_ref().unwrap() == &mimoinfo => {
                        // TODO
                        n.children.push(HTMLChild::Str(charstr))
                    }
                    (Some(n),i) if 0<i && i<7 && String::from(&n.name) == "mo" && n.mimoinfo.is_some() && n.mimoinfo.as_ref().unwrap() == &mimoinfo => {
                        n.children.push(HTMLChild::Str(charstr))
                    }
                    (_,0|7) => {
                        node!(self,mi,mc.sourceref,clsstr,node_top,a => {
                            a.mimoinfo = Some(mimoinfo);
                            literal!(self,&mut Some(HTMLParent::N(&mut a)),>charstr<)
                        })
                    }
                    (_,_) => {
                        node!(self,mo,mc.sourceref,clsstr,node_top,a => {
                            a.mimoinfo = Some(mimoinfo);
                            literal!(self,&mut Some(HTMLParent::N(&mut a)),>charstr<)
                        })
                    }
                }
            }
            Delimiter(d) => self.ship_kernel(MathChar(d.small),node_top),
            MKern(m) => {
                node!(self,mspace,m.sourceref,"mkern",node_top,a => {
                    a.attr("width".into(),numtostr((m.sk.base as f32 / 1179648.0).round() as i32,"em").into())
                })
            }
            MathOp(crate::stomach::math::MathOp { content,sourceref }) => annotate!(self,mrow,sourceref,node_top,node => {
                node.classes.push("mathop".into());
                self.ship_m(*content,&mut Some(HTMLParent::A(&mut node)))
            }),
            MathInner(crate::stomach::math::MathInner { content,sourceref }) => annotate!(self,mrow,sourceref,node_top,node => {
                node.classes.push("inner".into());
                self.ship_m(*content,&mut Some(HTMLParent::A(&mut node)))
            }),
            MathRel(crate::stomach::math::MathRel { content,sourceref }) => annotate!(self,mrow,sourceref,node_top,node => {
                node.classes.push("rel".into());
                self.ship_m(*content,&mut Some(HTMLParent::A(&mut node)))
            }),
            MathOpen(crate::stomach::math::MathOpen { content,sourceref }) => annotate!(self,mrow,sourceref,node_top,node => {
                node.classes.push("open".into());
                self.ship_m(*content,&mut Some(HTMLParent::A(&mut node)))
            }),
            MathClose(crate::stomach::math::MathClose { content,sourceref }) => annotate!(self,mrow,sourceref,node_top,node => {
                node.classes.push("close".into());
                self.ship_m(*content,&mut Some(HTMLParent::A(&mut node)))
            }),
            MathPunct(crate::stomach::math::MathPunct { content,sourceref }) => annotate!(self,mrow,sourceref,node_top,node => {
                node.classes.push("punct".into());
                self.ship_m(*content,&mut Some(HTMLParent::A(&mut node)))
            }),
            MathBin(crate::stomach::math::MathBin { content,sourceref }) => annotate!(self,mrow,sourceref,node_top,node => {
                node.classes.push("bin".into());
                self.ship_m(*content,&mut Some(HTMLParent::A(&mut node)))
            }),
            Underline(crate::stomach::math::Underline { content,sourceref }) => node!(self,munder,sourceref,"underline",node_top,node => {
                annotate!(self,mrow,None,&mut Some(HTMLParent::N(&mut node)),mrow => {
                    self.ship_m(*content,&mut Some(HTMLParent::A(&mut mrow)))
                });
                literal!(self,&mut Some(HTMLParent::N(&mut node)),"&UnderBar;")
            }),
            Overline(crate::stomach::math::Overline { content,sourceref }) => node!(self,mover,sourceref,"overline",node_top,node => {
                annotate!(self,mrow,None,&mut Some(HTMLParent::N(&mut node)),mrow => {
                    self.ship_m(*content,&mut Some(HTMLParent::A(&mut mrow)))
                });
                literal!(self,&mut Some(HTMLParent::N(&mut node)),"&OverBar;")
            }),
            MathAccent(crate::stomach::math::MathAccent { content, accent, sourceref}) =>
                node!(self,mover,sourceref,"mathaccent",node_top,node => {
                    node.attr("accent".into(),"true".into());
                    annotate!(self,mrow,None,&mut Some(HTMLParent::N(&mut node)),mrow => {
                        self.ship_m(*content,&mut Some(HTMLParent::A(&mut mrow)))
                    });
                    annotate!(self,mrow,None,&mut Some(HTMLParent::N(&mut node)),mrow => {
                        self.ship_kernel(MathKernel::MathChar(accent),&mut Some(HTMLParent::A(&mut mrow)))
                    });
            }),
            _ => literal!(self,node_top,"<!-- TODO -->" )
        }
    }
    fn ship_m<'a>(&mut self, w:Whatsit, node_top:&mut Option<HTMLParent<'a>>) {
        use Whatsit::*;
        use crate::stomach::simple::SimpleWI::*;
        use crate::stomach::boxes::TeXBox::*;
        use crate::stomach::groups::WIGroup::*;
        use crate::stomach::math::MathGroup;
        match w {
            Math(MathGroup {kernel,superscript:None,subscript:None,limits:_}) => {
                self.ship_kernel(kernel,node_top)
            }
            Math(MathGroup {kernel,superscript:Some(sup),subscript:None,limits:false}) => {
                node!(self,msup,None,"superscript",node_top,msup => {
                    msup.attr("displaystyle".into(),"false".into());
                    self.ship_kernel(kernel,&mut Some(HTMLParent::N(&mut msup)));
                    if msup.children.is_empty() { node!(self,mrow,None,"",&mut Some(HTMLParent::N(&mut msup))) }
                    msup.children.push(HTMLChild::Str("".into()));
                    self.ship_kernel(sup,&mut Some(HTMLParent::N(&mut msup)));
                    if msup.children.len() < 3 { node!(self,mrow,None,"",&mut Some(HTMLParent::N(&mut msup))) }
                })
            }
            Math(MathGroup {kernel,superscript:Some(sup),subscript:None,limits:true}) => {
                node!(self,mover,None,"superscript",node_top,msup => {
                    msup.attr("displaystyle".into(),"true".into());
                    self.ship_kernel(kernel,&mut Some(HTMLParent::N(&mut msup)));
                    if msup.children.is_empty() { node!(self,mrow,None,"",&mut Some(HTMLParent::N(&mut msup))) }
                    msup.children.push(HTMLChild::Str("".into()));
                    self.ship_kernel(sup,&mut Some(HTMLParent::N(&mut msup)));
                    if msup.children.len() < 3 { node!(self,mrow,None,"",&mut Some(HTMLParent::N(&mut msup))) }
                })
            }
            Math(MathGroup {kernel,superscript:None,subscript:Some(sub),limits:false}) => {
                node!(self,msub,None,"subscript",node_top,msub => {
                    msub.attr("displaystyle".into(),"false".into());
                    self.ship_kernel(kernel,&mut Some(HTMLParent::N(&mut msub)));
                    if msub.children.is_empty() { node!(self,mrow,None,"",&mut Some(HTMLParent::N(&mut msub))) }
                    msub.children.push(HTMLChild::Str("".into()));
                    self.ship_kernel(sub,&mut Some(HTMLParent::N(&mut msub)));
                    if msub.children.len() < 3 { node!(self,mrow,None,"",&mut Some(HTMLParent::N(&mut msub))) }
                })
            }
            Math(MathGroup {kernel,superscript:None,subscript:Some(sub),limits:true}) => {
                node!(self,munder,None,"subscript",node_top,msub => {
                    msub.attr("displaystyle".into(),"true".into());
                    self.ship_kernel(kernel,&mut Some(HTMLParent::N(&mut msub)));
                    if msub.children.is_empty() { node!(self,mrow,None,"",&mut Some(HTMLParent::N(&mut msub))) }
                    msub.children.push(HTMLChild::Str("".into()));
                    self.ship_kernel(sub,&mut Some(HTMLParent::N(&mut msub)));
                    if msub.children.len() < 3 { node!(self,mrow,None,"",&mut Some(HTMLParent::N(&mut msub))) }
                })
            }
            Math(MathGroup {kernel,superscript:Some(sup),subscript:Some(sub),limits:false}) => {
                node!(self,msubsup,None,"subscript superscript",node_top,msub => {
                    msub.attr("displaystyle".into(),"false".into());
                    self.ship_kernel(kernel,&mut Some(HTMLParent::N(&mut msub)));
                    if msub.children.is_empty() { node!(self,mrow,None,"",&mut Some(HTMLParent::N(&mut msub))) }
                    msub.children.push(HTMLChild::Str("".into()));
                    self.ship_kernel(sub,&mut Some(HTMLParent::N(&mut msub)));
                    if msub.children.len() < 3 { node!(self,mrow,None,"",&mut Some(HTMLParent::N(&mut msub))) }
                    msub.children.push(HTMLChild::Str("".into()));
                    self.ship_kernel(sup,&mut Some(HTMLParent::N(&mut msub)));
                    if msub.children.len() < 5 { node!(self,mrow,None,"",&mut Some(HTMLParent::N(&mut msub))) }
                })
            }
            Math(MathGroup {kernel,superscript:Some(sup),subscript:Some(sub),limits:true}) => {
                node!(self,munderover,None,"subscript superscript",node_top,msub => {
                    msub.attr("displaystyle".into(),"true".into());
                    self.ship_kernel(kernel,&mut Some(HTMLParent::N(&mut msub)));
                    if msub.children.is_empty() { node!(self,mrow,None,"",&mut Some(HTMLParent::N(&mut msub))) }
                    msub.children.push(HTMLChild::Str("".into()));
                    self.ship_kernel(sub,&mut Some(HTMLParent::N(&mut msub)));
                    if msub.children.len() < 3 { node!(self,mrow,None,"",&mut Some(HTMLParent::N(&mut msub))) }
                    msub.children.push(HTMLChild::Str("".into()));
                    self.ship_kernel(sup,&mut Some(HTMLParent::N(&mut msub)));
                    if msub.children.len() < 5 { node!(self,mrow,None,"",&mut Some(HTMLParent::N(&mut msub))) }
                })
            }
            Simple(MSkip(m)) => {
                node!(self,mspace,m.sourceref,"mskip",node_top,a => {
                    a.attr("width".into(),numtostr((m.skip.base as f32 / 1179648.0).round() as i32,"em").into())
                })
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
            Box(_) => node!(self,mtext,None,"box",node_top,mt => {
                node!(self,HTML_NS:span,None,"box",&mut Some(HTMLParent::N(&mut mt)),span => {
                    self.ship_h(w,&mut Some(HTMLParent::N(&mut span)))
                })
            }),
            Simple(HAlign(_)) => node!(self,mtext,None,"",node_top,mt => {
                node!(self,HTML_NS:span,None,"",&mut Some(HTMLParent::N(&mut mt)),span => {
                    self.ship_top(w,&mut Some(HTMLParent::N(&mut span)))
                })
            }),
            Simple(HKern(m)) => {
                node!(self,mspace,m.sourceref,"mskip",node_top,a => {
                    a.attr("width".into(),dimtohtml(m.dim))
                })
            }
            Grouped(PDFLink(lnk)) => annotate!(self,mrow,lnk.sourceref,node_top,node => {
                node.attr("href".into(),lnk.action.as_link().into());
                for c in lnk.children{ self.ship_m(c,&mut Some(HTMLParent::A(&mut node))) }
            }),
            Grouped(ColorChange(cc)) => {
                annotate!(self,mrow,cc.sourceref,node_top,a => {
                    let mut color : HTMLStr = crate::stomach::groups::ColorChange::as_html(cc.color).into();
                    let hashcolor : HTMLStr = "#".into();
                    a.style("color".into(),hashcolor + &color);
                    let _oldcolor = std::mem::take(&mut self.state.currcolor);
                    self.state.currcolor = Some(color);
                    for c in cc.children { self.ship_m(c,&mut Some(HTMLParent::A(&mut a))) }
                    self.state.currcolor = _oldcolor;
                })
            }
            Simple(HSkip(h)) => {
                node!(self,mspace,h.sourceref,"hskip",node_top,a => {
                    a.attr("width".into(),dimtohtml(h.skip.base))
                })
            }
            Simple(Left(l)) => for c in l.bx { self.ship_m(c.as_whatsit(),node_top)},
            Simple(Middle(l)) => for c in l.bx { self.ship_m(c.as_whatsit(),node_top)},
            Simple(Right(l)) => for c in l.bx { self.ship_m(c.as_whatsit(),node_top)},
            Above(o) => match o.delimiters {
                None => node!(self,mfrac,o.sourceref,"over",node_top,over => {
                    match o.thickness {
                        Some(i) => over.attr("linethickness".into(),dimtohtml(i)),
                        _ => ()
                    }
                    annotate!(self,mrow,None,&mut Some(HTMLParent::N(&mut over)),a => {
                        for c in o.top { self.ship_m(c,&mut Some(HTMLParent::A(&mut a))) }
                    });
                    annotate!(self,mrow,None,&mut Some(HTMLParent::N(&mut over)),a => {
                        for c in o.bottom { self.ship_m(c,&mut Some(HTMLParent::A(&mut a))) }
                    })
                }),
                Some(d) => annotate!(self,mrow,None,node_top,mrow => {
                    let (a,b) = *d;
                    self.ship_m(a,&mut Some(HTMLParent::A(&mut mrow)));
                    node!(self,mfrac,o.sourceref,"over",&mut Some(HTMLParent::A(&mut mrow)),over => {
                        match o.thickness {
                            Some(i) => over.attr("linethickness".into(),dimtohtml(i)),
                            _ => ()
                        }
                        annotate!(self,mrow,None,&mut Some(HTMLParent::N(&mut over)),a => {
                            for c in o.top { self.ship_m(c,&mut Some(HTMLParent::A(&mut a))) }
                        });
                        annotate!(self,mrow,None,&mut Some(HTMLParent::N(&mut over)),a => {
                            for c in o.bottom { self.ship_m(c,&mut Some(HTMLParent::A(&mut a))) }
                        })
                    });
                    self.ship_m(b,&mut Some(HTMLParent::A(&mut mrow)))
                })
            }
            Simple(Leaders(ld)) => {
                self.ship_m(ld.bx.clone().as_whatsit(),node_top);
                self.ship_m(ld.bx.clone().as_whatsit(),node_top);
                self.ship_m(ld.bx.as_whatsit(),node_top);
            }
            Simple(HFil(_)|HFill(_)) => {
                node!(self,mspace,None,"hfil",node_top,a => {
                    a.attr("width".into(),dimtohtml(655360))
                })
            }
            Simple(Penalty(_)) => (),
            _ => literal!(self,node_top,"<!-- TODO -->" )
        }
    }
}

// -------------------------------------------------------------------------------------------------
#[derive(PartialEq,Clone)]
struct MiMoInfo {
    family:Option<HTMLStr>,
    weight:Option<HTMLStr>,
    style:Option<HTMLStr>,
    variant:Option<HTMLStr>,
    at:i32
}
impl MiMoInfo {
    fn new(font:&Arc<Font>) -> MiMoInfo {
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
struct HTMLNode {
    pub name:HTMLStr,
    pub namespace:&'static str,
    pub children:Vec<HTMLChild>,
    pub classes:Vec<HTMLStr>,
    attributes:HashMap<HTMLStr,HTMLStr>,
    styles:HashMap<HTMLStr,HTMLStr>,
    mimoinfo: Option<MiMoInfo>,
    pub sourceref:Option<SourceFileReference>
}
impl HTMLNode {
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
                ret += s.as_string();
                ret += "\""
            }
            _ => ()
        }
        ret += ">";
        ret += body;
        ret + "</" + &self.name + ">"
    }
}

struct HTMLAnnotation {
    pub name:HTMLStr,
    pub namespace:&'static str,
    pub children:Vec<HTMLChild>,
    pub classes:Vec<HTMLStr>,
    attributes:HashMap<HTMLStr,HTMLStr>,
    styles:HashMap<HTMLStr,HTMLStr>,
    pub sourceref:Option<SourceFileReference>
}
impl HTMLAnnotation {
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

enum HTMLParent<'a> {
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

enum HTMLChild {
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
enum HTMLStr {
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