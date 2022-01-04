use std::borrow::BorrowMut;
use std::collections::HashMap;
use std::fmt::{Debug, Display, Formatter};
use std::hash::{Hash, Hasher};
use std::ops::{Add, AddAssign};
use std::str::from_utf8;
use std::sync::Arc;
use image::EncodableLayout;
use itertools::Itertools;
use crate::fonts::Font;
use crate::Interpreter;
use crate::interpreter::dimensions::numtostr;
use crate::stomach::colon::{Colon, ColonBase};
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
    current_node:Option<HTMLChild>,
    current_namespace:&'static str,
    top:Vec<HTMLChild>,
    currsize:i32,
    currcolor:Option<HTMLStr>
}
impl HTMLState {
    pub fn new() -> HTMLState { HTMLState {
        annotations_todo:vec!(),
        current_node:None,
        current_namespace:HTML_NS,
        top:vec!(),
        currsize:0,currcolor:None
    }}
}
macro_rules! node {
    ($sel:ident,$node:ident,$name:ident) => ({
        //node!($sel,$ns:$node,$name,() =>)
        let mut _node_newnode = HTMLNode::new($sel.state.current_namespace,stringify!($node).into());
        _node_newnode.classes.push(stringify!($name).into());
        match &mut $sel.state.current_node {
            Some(HTMLChild::Node(e)) => {
                e.children.push(HTMLChild::Node(_node_newnode))
            }
            Some(HTMLChild::Annot(e)) => {
                e.children.push(HTMLChild::Node(_node_newnode))
            }
            _ => {
                $sel.state.top.push(HTMLChild::Node(_node_newnode))
            }
        }
    });
    ($sel:ident,$ns:tt:$node:ident,$name:ident) => ({
        let mut _node_newnode = HTMLNode::new($ns,stringify!($node).into());
        _node_newnode.classes.push(stringify!($name).into());
        match &mut $sel.state.current_node {
            Some(HTMLChild::Node(e)) => {
                e.children.push(HTMLChild::Node(_node_newnode))
            }
            Some(HTMLChild::Annot(e)) => {
                e.children.push(HTMLChild::Node(_node_newnode))
            }
            _ => {
                $sel.state.top.push(HTMLChild::Node(_node_newnode))
            }
        }
    });
    ($sel:ident,$node:ident,$name:ident,$nodename:ident => $e:expr) => (
        {
            let _node_oldtop = $sel.state.current_node.replace(HTMLChild::Node(HTMLNode::new($sel.state.current_namespace,stringify!($node).into())));
            let mut $nodename = match $sel.state.current_node.as_mut().unwrap() {
                HTMLChild::Node(n) => {
                    n.classes.push(stringify!($name).into());
                    n
                }
                _ => unreachable!()
            };
            $e;
            let _node_ret = match $sel.state.current_node.take() {
                Some(HTMLChild::Node(n)) => n,
                _ => unreachable!()
            };
            $sel.state.current_node = _node_oldtop;
            match &mut $sel.state.current_node {
                Some(HTMLChild::Node(e)) => {
                    e.children.push(HTMLChild::Node(_node_ret))
                }
                Some(HTMLChild::Annot(e)) => {
                    e.children.push(HTMLChild::Node(_node_ret))
                }
                _ => {
                    $sel.state.top.push(HTMLChild::Node(_node_ret))
                }
            }
        }
    );
    ($sel:ident,$ns:tt:$node:ident,$name:ident,$nodename:ident => $e:expr) => (
        {
            let _node_oldns = $sel.state.current_namespace;
            let _node_oldtop = $sel.state.current_node.replace(HTMLChild::Node(HTMLNode::new($ns,stringify!($node).into())));
            let mut $nodename = match $sel.state.current_node.as_mut().unwrap() {
                HTMLChild::Node(n) => {
                    n.classes.push(stringify!($name).into());
                    n
                }
                _ => unreachable!()
            };
            $sel.state.current_namespace = $ns;
            $e;
            let _node_ret = match $sel.state.current_node.take() {
                Some(HTMLChild::Node(n)) => n,
                _ => unreachable!()
            };
            $sel.state.current_node = _node_oldtop;
            $sel.state.current_namespace = _node_oldns;
            match &mut $sel.state.current_node {
                Some(HTMLChild::Node(e)) => {
                    e.children.push(HTMLChild::Node(_node_ret))
                }
                Some(HTMLChild::Annot(e)) => {
                    e.children.push(HTMLChild::Node(_node_ret))
                }
                _ => {
                    $sel.state.top.push(HTMLChild::Node(_node_ret))
                }
            }
        }
    );
}

macro_rules! literal {
    ($sel:ident,$e:expr) => ({
        let _ret : HTMLStr = $e.into();
        match &mut $sel.state.current_node {
            Some(HTMLChild::Node(e)) => {
                e.children.push(HTMLChild::Str(_ret))
            }
            Some(HTMLChild::Annot(e)) => {
                e.children.push(HTMLChild::Str(_ret))
            }
            _ => {
                $sel.state.top.push(HTMLChild::Str(_ret))
            }
        }
    });
    ($sel:ident,>$e:tt<) => ({
        let _ret : HTMLStr = $e.into();
        match &mut $sel.state.current_node {
            Some(HTMLChild::Node(e)) => {
                e.children.push(HTMLChild::Str(_ret.html_escape()))
            }
            Some(HTMLChild::Annot(e)) => {
                e.children.push(HTMLChild::Str(_ret.html_escape()))
            }
            _ => {
                $sel.state.top.push(HTMLChild::Str(_ret.html_escape()))
            }
        }
    })
}

macro_rules! annotate {
    ($sel:ident,$node:ident,$nodename:ident => $e:expr) => (
        {
            let _node_oldtop = $sel.state.current_node.replace(HTMLChild::Annot(HTMLAnnotation::new($sel.state.current_namespace,stringify!($node).into())));
            let mut $nodename = match $sel.state.current_node.as_mut().unwrap() {
                HTMLChild::Annot(n) => n,
                _ => unreachable!()
            };
            $e;
            let _node_ret = match $sel.state.current_node.take() {
                Some(HTMLChild::Annot(n)) => n,
                _ => unreachable!()
            };
            $sel.state.current_node = _node_oldtop;
            match &mut $sel.state.current_node {
                Some(HTMLChild::Node(e)) => {
                    e.children.push(HTMLChild::Annot(_node_ret))
                }
                Some(HTMLChild::Annot(e)) => {
                    e.children.push(HTMLChild::Annot(_node_ret))
                }
                _ => {
                    $sel.state.top.push(HTMLChild::Annot(_node_ret))
                }
            }
        }
    );
    ($sel:ident,$ns:tt:$node:ident,$nodename:ident => $e:expr) => (
        {
            let _node_oldns = $sel.state.current_namespace;
            let _node_oldtop = $sel.state.current_node.replace(HTMLChild::Annot(HTMLAnnotation::new($ns,stringify!($node).into())));
            let mut $nodename = match $sel.state.current_node.as_mut().unwrap() {
                HTMLChild::Annot(n) => n,
                _ => unreachable!()
            };
            $sel.state.current_namespace = $ns;
            $e;
            let _node_ret = match $sel.state.current_node.take() {
                Some(HTMLChild::Annot(n)) => n,
                _ => unreachable!()
            };
            $sel.state.current_node = _node_oldtop;
            $sel.state.current_namespace = _node_oldns;
            match &mut $sel.state.current_node {
                Some(HTMLChild::Node(e)) => {
                    e.children.push(HTMLChild::Annot(_node_ret))
                }
                Some(HTMLChild::Annot(e)) => {
                    e.children.push(HTMLChild::Annot(_node_ret))
                }
                _ => {
                    $sel.state.top.push(HTMLChild::Annot(_node_ret))
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
    fn ship_whatsit(&mut self, w:Whatsit) { for w in self.normalize_whatsit(w) { self.ship_top(w) } }
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
            self.ret += "\n    <script type=\"text/javascript\" id=\"MathJax-script\" src=\"https://cdn.jsdelivr.net/npm/mathjax@3/es5/mml-chtml.js\"></script>";
            self.ret += "\n  </head>\n  <body width=\"";
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
            self.ret += &dimtohtml((lineheight.base as f32 / (fontsize as f32 / 65536.0)).round() as i32).to_string();
            self.ret += ";\">\n";

            let base = self.base_mut();
            base.basefont = Some(basefont);
            base.basecolor = Some(basecolor);
        }
    }
    fn close(self) -> String {
        if self.doheader {
            self.ret + "\n    </div>\n  </body>\n</html>"
        } else { self.ret }
    }
}
impl HTMLColon {
    pub fn new(doheader:bool) -> HTMLColon { HTMLColon {
        base:ColonBase::new(),
        ret:"".to_string(),
        state:HTMLState::new(),
        doheader
    }}
    fn ship_top(&mut self,w:Whatsit) {
        use Whatsit::*;
        use crate::stomach::simple::SimpleWI::*;
        use crate::stomach::boxes::TeXBox::*;
        use crate::interpreter::dimensions::SkipDim;
        use crate::stomach::groups::WIGroup::*;
        use crate::fonts::fontchars::FontTableParam;
        match w {
            Simple(VFil(_)) => node!(self,div,vfil),
            Simple(VFill(_)) => node!(self,div,vfill),
            Simple(PDFDest(d)) if d.dest.to_string() == "xyz" => {
                node!(self,a,pdfdest,node => {
                    node.attr("id".into(),d.target.clone().into());
                    node.attr("name".into(),d.target.into());
                })
            },
            Simple(VSkip(vsk)) => {
                node!(self,div,vskip,node => {
                    node.style("margin-bottom".into(),dimtohtml(vsk.skip.base));
                })
            },
            Simple(HRule(hr)) => {
                node!(self,div,hrule,n => {
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
            Grouped(FontChange(fc)) => {
                match &fc.font.file.chartable {
                    Some(ft) => {
                        annotate!(self,span,a => {
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
                            for c in fc.children { self.ship_top(c) }
                            self.state.currsize = _oldsize;
                        })
                    }
                    _ => {
                        for c in fc.children { self.ship_top(c) }
                    }
                }
            }
            Par(p) => {
                node!(self,div,paragraph,node => {
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
                    for c in p.children { self.ship_w(c) }
                });
            }
            Simple(HAlign(ha)) => {
                node!(self,table,halign) // TODO!
            }
            _ => literal!(self,"<!-- TODO -->" )//self.ret += &w.as_xml_internal("  ".to_string())
        }
        if self.state.current_node.is_none() {
            for n in std::mem::take(&mut self.state.top) {
                self.ret += &n.make_string("  ".into(),&HTML_NS).to_string()
            }
        }
    }
    fn ship_w(&mut self,w:Whatsit) {
        use Whatsit::*;
        use crate::stomach::simple::SimpleWI::*;
        use crate::stomach::boxes::TeXBox::*;
        use crate::stomach::groups::WIGroup::*;
        use crate::fonts::fontchars::FontTableParam;
        match w {
            Grouped(FontChange(fc)) => {
                match &fc.font.file.chartable {
                    Some(ft) => {
                        annotate!(self,span,a => {
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
                            for c in fc.children { self.ship_w(c) }
                            self.state.currsize = _oldsize;
                        })
                    }
                    _ => {
                        for c in fc.children { self.ship_w(c) }
                    }
                }
            }
            Char(pc) => literal!(self,>{
                match &pc.font.file.chartable {
                    Some(ct) => ct.get_char(pc.char).to_string(),
                    None => pc.as_xml_internal("".to_string())
                }
            }<),
            Space(_) => literal!(self,"&nbsp;"),
            Simple(VRule(vr)) => {
                node!(self,div,vrule,n => {
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
                node!(self,div,hskip,node => {
                    node.style("margin-left".into(),dimtohtml(vsk.skip.base));
                })
            },
            Simple(Indent(dim)) => {
                node!(self,div,indent,node => {
                    node.style("margin-left".into(),dimtohtml(dim.dim));
                })
            },
            Box(V(vb)) => {
                node!(self,div,vbox,node => {
                    match vb._height {
                        Some(h) => {
                            node.style("height".into(),dimtohtml(h));
                            node.style("min-height".into(),dimtohtml(h))
                        }
                        _ => ()
                    }
                    for c in vb.children { self.ship_top(c) }
                })
            }
            // TODO maybe? spread, center, vtop in general
            _ => literal!(self,"<!-- TODO -->" )
        }
    }
}

// -------------------------------------------------------------------------------------------------

struct HTMLNode {
    pub name:HTMLStr,
    pub namespace:&'static str,
    pub children:Vec<HTMLChild>,
    pub classes:Vec<HTMLStr>,
    attributes:HashMap<HTMLStr,HTMLStr>,
    styles:HashMap<HTMLStr,HTMLStr>
}
impl HTMLNode {
    pub fn new(namespace:&'static str,name:HTMLStr) -> HTMLNode { HTMLNode {
        name,namespace,children:vec!(),classes:vec!(),
        attributes:HashMap::new(),styles:HashMap::new()
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
        let mut ret : HTMLStr = "\n".into();
        let mut body : HTMLStr = "".into();
        for c in self.children.drain(..) {
            body += match c {
                HTMLChild::Node(mut n) => n.make_string(prefix.clone(),self.namespace),
                HTMLChild::Annot(mut a) => a.make_string(prefix.clone(),self.namespace),
                HTMLChild::Str(mut s) => s.clone(),
            }
        }
        ret += &prefix + "<" + &self.name;
        for (a,v) in &self.attributes {
            ret += " " + a + "=\"" + v + "\""
        }
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
        ret += ">";
        ret += body;
        ret + "\n" + &prefix + "</" + &self.name + ">"
    }
}

struct HTMLAnnotation {
    pub name:HTMLStr,
    pub namespace:&'static str,
    pub children:Vec<HTMLChild>,
    pub classes:Vec<HTMLStr>,
    attributes:HashMap<HTMLStr,HTMLStr>,
    styles:HashMap<HTMLStr,HTMLStr>
}
impl HTMLAnnotation {
    pub fn new(namespace:&'static str,name:HTMLStr) -> HTMLAnnotation { HTMLAnnotation {
        name,namespace,children:vec!(),classes:vec!(),
        attributes:HashMap::new(),styles:HashMap::new()
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
            styles: std::mem::take(&mut self.styles)
        }.make_string(prefix,namespace)
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

enum HTMLStr {
    Imm(&'static str),
    Mut(String)
}
impl HTMLStr {
    pub fn to_string(self) -> String { self.into() }
}
impl Clone for HTMLStr {
    fn clone(&self) -> Self {
        use HTMLStr::*;
        match self {
            Imm(s) => Imm(s),
            Mut(s) => Mut(s.clone())
        }
    }
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