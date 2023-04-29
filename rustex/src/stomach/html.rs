use std::borrow::BorrowMut;
use std::collections::HashMap;
use std::fmt::{Display, Formatter};
use std::hash::{Hash, Hasher};
use std::ops::{Add, AddAssign};
use std::sync::Arc;
use itertools::Itertools;
use crate::fonts::{ArcFont, Font, NULL_FONT};
use crate::fonts::fontchars::FontTableParam;
use crate::Interpreter;
use crate::interpreter::dimensions::{numtostr, Skip};
use crate::references::SourceFileReference;
use crate::stomach::colon::{Colon, ColonBase, ColonMode};
use crate::stomach::Whatsit;
use crate::stomach::whatsits::{lineheight, WhatsitTrait};
use crate::utils::TeXStr;

pub static HTMLSCALE : f32 = 1.5;

pub fn dimtohtml(num:i32) -> HTMLStr {
    numtostr((HTMLSCALE * (num as f32)).round() as i32,"px").into()
}
pub fn numtohtml(num:i32) -> HTMLStr {
    numtostr((HTMLSCALE * (num as f32)).round() as i32,"").into()
}

pub static HTML_NS : &str = "http://www.w3.org/1999/xhtml";
pub static MATHML_NS : &str = "http://www.w3.org/1998/Math/MathML";
pub static SVG_NS : &str = "http://www.w3.org/2000/svg";
pub static RUSTEX_NS : &str = "http://kwarc.info/ns/RusTeX";

pub struct HTMLState {
    pub current_namespace:&'static str,
    pub top:Vec<HTMLChild>,
    pub currsize:i32,pub fontsize:i32,
    //pub squaresize:bool,
    pub currcolor:Option<HTMLStr>,
    //kern:i32,
    pub line_height:i32,pub line_scale:f32
}
impl HTMLState {
    pub fn new() -> HTMLState { HTMLState {
        current_namespace:HTML_NS,
        top:vec!(),fontsize:0,//squaresize:false,
        currsize:0,currcolor:None,//kern:0,
        line_height:0,line_scale:1.0
    }}
    pub fn lineheight(&self) -> f32 {
        (self.line_height as f32) / (self.fontsize as f32)
    }
    /*pub fn add_kern(&mut self,v : i32) {
        self.kern = self.kern + v;
    }*/
}

#[macro_export]
macro_rules! setwidth {
    ($colon:ident,$wd:expr,$node:expr) => ({
        let _withwidth_pctg = (($wd as f32) / ($colon.state.currsize as f32));
         if _withwidth_pctg == 1.0 {
             $node.style("width".into(),"var(--document-width)".into());
             $node.style("min-width".into(),"var(--document-width)".into());
        } else {
            let _withwidth_str = "calc(".to_string() + &_withwidth_pctg.to_string() + " * var(--document-width))";
             $node.style("width".into(),_withwidth_str.clone().into());
             $node.style("min-width".into(),_withwidth_str.into());
        };
    })
}

#[macro_export]
macro_rules! withlinescale {
    ($colon:ident,$lineheight:expr,$node:expr,$e:expr) => ({
        if let Some(_lineheight) = $lineheight {
            let _newscale = if ($colon.state.fontsize == 0) {0.0} else {(_lineheight as f32) / ($colon.state.fontsize as f32)};
            if _newscale == $colon.state.line_scale { $e } else {
                let _oldlinescale = $colon.state.line_scale;
                let _oldlineheight = $colon.state.line_height;
                $colon.state.line_scale = _newscale;
                $colon.state.line_height = _lineheight;
                $node.style("line-height".into(),_newscale.to_string().into());
                $e;
                $colon.state.line_scale = _oldlinescale;
                $colon.state.line_height = _oldlineheight;
            }
        } else { $e }
    })
}

#[macro_export]
macro_rules! withwidth {
 ($colon:ident,$wd:expr,$node:expr,$inner:ident => $e:expr) => ({
     if $wd <= 0 {
         $node.style("width".into(),"0".into());
         $node.style("min-width".into(),"0".into());
         if ($wd < 0) {$node.style("margin-right".into(),dimtohtml($wd))}
         let mut $inner = &mut $node;
         $e
     } else {
         let _withwidth_currsize = $colon.state.currsize;
         let _withwidth_pctg = (($wd as f32) / (_withwidth_currsize as f32));
         if _withwidth_pctg == 1.0 {
             $node.style("width".into(),"var(--document-width)".into());
             $node.style("min-width".into(),"var(--document-width)".into());
             let mut $inner = &mut $node;
             $e
         } else {
             let _withwidth_str = "calc(".to_string() + &_withwidth_pctg.to_string() + " * var(--document-width))";
             $node.style("--temp-width".into(),_withwidth_str.into());
             $node.classes.push("rustex-withwidth".into());
            $colon.state.currsize = $wd;
             htmlnode!($colon,span,None,"rustex-contents",htmlparent!($node),$inner => {
                $e
             });
             $colon.state.currsize = _withwidth_currsize;
         }
     }
 });
}
#[macro_export]
macro_rules! htmlnode {
    ($sel:ident,$node:ident,$sref:expr,$name:tt,$node_parent:expr) => ({
        //$sel.do_kern($node_parent);
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
        //$sel.do_kern($node_parent);
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
            //$sel.do_kern($node_parent);
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
            //$sel.do_kern($node_parent);
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
        //$sel.do_kern($node_parent);
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
        //$sel.do_kern($node_parent);
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
    pub state:HTMLState,
    pub namespaces : HashMap<String,String>,
    pagewidth:i32,pub textwidth:i32
}
//unsafe impl Send for HTMLColon {}

impl Colon<String> for HTMLColon {
    fn base(&self) -> &ColonBase { &self.base }
    fn base_mut(&mut self) -> &mut ColonBase { &mut self.base }
    fn ship_whatsit(&mut self, w:Whatsit) {
        for w in self.normalize_whatsit(w) { w.as_html(&ColonMode::V,self,&mut None) }
        let fi = match self.base.basefont.as_ref() {
            None => NULL_FONT.try_with(|x| FontInfo::new(x)).unwrap(),
            Some(f) => FontInfo::new(f)
        };
        for n in std::mem::take(&mut self.state.top) {
            self.ret += &n.make_string("  ".into(),&HTML_NS,&fi).to_string()
        }
    } //}
    fn initialize(&mut self, basefont: ArcFont, basecolor: TeXStr, int: &Interpreter) {
        if self.doheader {
            self.state.currsize =  int.state.dimensions_prim.get(&(crate::commands::registers::HSIZE.index - 1));
            self.state.currcolor = match &basecolor {
                s if s.to_string() == "000000" => None,
                s => Some(s.clone().into())
            };
            self.state.fontsize = match &self.base.basefont.as_ref() {
                Some(f) => match f.at {
                    Some(i) => i,
                    None => 655360
                }
                None => 655360
            };
            self.pagewidth = int.state.dimensions_prim.get(&(crate::commands::registers::PDFPAGEWIDTH.index - 1));
            self.textwidth = int.state.dimensions_prim.get(&(crate::commands::registers::HSIZE.index - 1));
            self.state.line_height = lineheight(&int.state);//int.state.skips_prim.get(&(crate::commands::registers::BASELINESKIP.index - 1));
            self.state.line_scale = self.state.lineheight();

            let base = self.base_mut();
            base.basefont = Some(basefont);
            base.basecolor = Some(basecolor);
        }
    }
    fn close(&mut self) -> String {
        if self.doheader {
            self.header() + &std::mem::take(&mut self.ret) + "\n    </div>\n  </body>\n</html>"
        } else { std::mem::take(&mut self.ret) }
    }
}
impl HTMLColon {
    fn header(&self) -> String {
        let mut ret : String = "".to_string();
        if self.doheader {
            ret += "<!DOCTYPE html PUBLIC \"-//W3C//DTD XHTML 1.1 plus MathML 2.0//EN\" \"http://www.w3.org/Math/DTD/mathml2/xhtml-math11-f.dtd\">\n";
            ret += "<html xmlns=\"";
            ret += HTML_NS;
            ret += "\"";
            for (a,b) in &self.namespaces {
                ret += " xmlns:";
                ret += a;
                ret += "=\"";
                ret += b;
                ret += "\""
            }
            ret += ">\n  <head>\n    <style>\n";
            ret += CSS;
            ret += "\n    </style>";
            //self.ret += "\n    <script type=\"text/javascript\" id=\"MathJax-script\" src=\"https://cdn.jsdelivr.net/npm/mathjax@3/es5/mml-chtml.js\"></script>";
            ret += "\n  </head>\n  <body style=\"max-width:";
            ret += &dimtohtml(self.pagewidth).to_string();
            ret += "\">\n    <div class=\"rustex-body\" id=\"rustexbody\" style=\"font-size:";
            ret += &dimtohtml(self.state.fontsize).to_string();
            let textwidthstr = ((self.textwidth as f32) / (self.pagewidth as f32)).to_string();
            ret += ";max-width:";
            ret += &dimtohtml(self.pagewidth).to_string();
            ret += ";--document-width:calc(";
            ret += &textwidthstr;
            ret += " * min(100vw,";
            ret += &dimtohtml(self.pagewidth).to_string();
            ret += ")";
            //ret += ";--document-width:min(100vw,";
            //ret += &dimtohtml(self.textwidth).to_string();
            ret += ");padding-left:";
            let padding = ((self.pagewidth - self.textwidth) as f32 / (2.0 * (self.pagewidth as f32)) * 100.0).to_string() + "%";
            ret += padding.as_str();
            //ret += &dimtohtml(((self.pagewidth - self.textwidth) as f32 / 2.0).round() as i32).to_string();
            ret += ";padding-right:";
            ret += padding.as_str();
            //ret += &dimtohtml(((self.pagewidth - self.textwidth) as f32 / 2.0).round() as i32).to_string();
            ret += ";line-height:";
            ret += &(self.state.line_scale).to_string();
            if crate::INSERT_RUSTEX_ATTRS {
                ret += ";\"";
                ret += " rustex:font=\"";
                match self.base.basefont.as_ref() {
                    Some(f) => ret += f.file.name.to_string().as_str(),
                    _ => ()
                };
            }
            ret += "\">\n";
        }
        ret
    }
    pub fn new(doheader:bool) -> HTMLColon {
        let mut ret = HTMLColon {
            base:ColonBase::new(),
            ret:"".to_string(),
            state:HTMLState::new(),
            doheader,
            namespaces:HashMap::new(),
            pagewidth: 0,
            textwidth: 0
        };
        ret.namespaces.insert("xhtml".into(),HTML_NS.into());
        ret.namespaces.insert("mml".into(),MATHML_NS.into());
        ret.namespaces.insert("svg".into(),SVG_NS.into());
        ret.namespaces.insert("rustex".into(),RUSTEX_NS.into());
        ret
    }
}

// -------------------------------------------------------------------------------------------------
#[derive(PartialEq,Clone)]
pub struct FontInfo {
    params: Vec<FontTableParam>,
    at:i32
}
impl FontInfo {
    pub fn new(font:&Arc<Font>) -> FontInfo {
        let mut ret = FontInfo { params:vec!(),at:font.get_at()};
        match font.file.chartable {
            None => (),
            Some(ref tbl) => {
                ret.params = tbl.params.clone()
            }
        }
        ret
    }
}
pub struct HTMLNode {
    pub ht:i32,
    pub name:HTMLStr,
    pub namespace:&'static str,
    pub children:Vec<HTMLChild>,
    pub classes:Vec<HTMLStr>,
    attributes:HashMap<HTMLStr,HTMLStr>,
    styles:HashMap<HTMLStr,HTMLStr>,
    pub forcefont: bool,
    pub fontinfo: Option<FontInfo>,
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
        name,namespace,children:vec!(),classes:vec!(),ht:0,
        attributes:HashMap::new(),styles:HashMap::new(),forcefont:false,
        fontinfo:None,sourceref
    }}
    pub fn attr(&mut self,name:HTMLStr,value:HTMLStr) {
        self.attributes.borrow_mut().insert(name, value);
    }
    pub fn style(&mut self,name:HTMLStr,value:HTMLStr) {
        self.styles.borrow_mut().insert(name,value);
    }
    pub fn make_string(&mut self,prefix:HTMLStr,namespace:&str,fi:&FontInfo) -> HTMLStr {
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
                        if a.fontinfo.is_some() { self.fontinfo = std::mem::take(&mut a.fontinfo)}
                        for (k,v) in std::mem::take(&mut a.attributes) { self.attributes.insert(k,v); }
                        for (k,v) in std::mem::take(&mut a.styles) { self.styles.insert(k,v); }
                        self.children = std::mem::take(&mut a.children);
                        return self.make_string(prefix,namespace,fi)
                    }
                }
                Some(HTMLChild::Str(ref s)) if s.to_string() == " " => {
                    self.children = vec!(HTMLChild::Str("&#160;".into()))
                }
                _ => ()
            }
        }

        let fi_o = if self.forcefont {
            Some(std::mem::take(&mut self.fontinfo).unwrap_or(fi.clone()))
        } else {self.fontinfo.take()};
        let nfi = match &fi_o {
            None => fi,
            Some(ref mi) if self.namespace == MATHML_NS => {
                if !mi.params.contains(&FontTableParam::Italic) {
                    self.attr("mathvariant".into(),"normal".into())
                };
                let ratio = (mi.at as f32) / (fi.at as f32);
                if ratio != 1.0 {
                    self.style("font-size".into(), ((ratio * 100.0).round().to_string() + "%").into())
                }
                mi
            }
            Some(ref mi) => {
                self.classes.push("rustex-reset-font".into());
                if mi.params.contains(&FontTableParam::Monospaced) {
                    self.classes.push("rustex-monospaced".into())
                }
                if mi.params.contains(&FontTableParam::Italic) {
                    self.style("font-style".into(),"italic".into())
                }
                if mi.params.contains(&FontTableParam::Bold) {
                    self.style("font-weight".into(),"bold".into())
                }
                if mi.params.contains(&FontTableParam::Blackboard) {
                    self.classes.push("rustex-blackboard".into())
                    //self.style("font-family".into(),"msbm".into())
                }
                if mi.params.contains(&FontTableParam::Capital) {
                    self.style("font-variant".into(),"small-caps".into())
                }
                if mi.params.contains(&FontTableParam::SansSerif) {
                    self.classes.push("rustex-sans-serif".into())
                    //self.style("font-family".into(),"sans-serif".into())
                }
                if mi.params.contains(&FontTableParam::Script) {
                    self.classes.push("rustex-script-font".into())
                    //self.style("font-family".into(),"URW Chancery L, cursive".into())
                }
                self.style("font-size".into(),((100.0 * (mi.at as f32) / (fi.at as f32)).round().to_string() + "%").into());
                mi
            }
        };

        let mut ret : HTMLStr = "".into();
        let mut body : HTMLStr = "".into();
        for c in std::mem::take(&mut self.children).into_iter() {
            body += match c {
                HTMLChild::Node(mut n) => n.make_string(prefix.clone(),self.namespace,nfi),
                HTMLChild::Annot(mut a) => a.make_string(prefix.clone(),self.namespace,nfi),
                HTMLChild::Str(s) => s.clone(),
            }
        }
        ret += "<" + &self.name;
        for (a,v) in &self.attributes {
            ret += " " + a + "=\"" + v + "\""
        }
        self.classes = std::mem::take(&mut self.classes).into_iter().filter(|x| x.to_string() != "").collect();
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
    pub fontinfo: Option<FontInfo>,
    pub sourceref:Option<SourceFileReference>
}
impl HTMLAnnotation {
    pub fn as_parent(&mut self) -> HTMLParent {
        HTMLParent::A(self)
    }
    pub fn new(namespace:&'static str,name:HTMLStr,sourceref:Option<SourceFileReference>) -> HTMLAnnotation { HTMLAnnotation {
        name,namespace,children:vec!(),classes:vec!(),
        attributes:HashMap::new(),styles:HashMap::new(),sourceref,fontinfo:None
    }}
    pub fn attr(&mut self,name:HTMLStr,value:HTMLStr) {
        self.attributes.borrow_mut().insert(name,value);
    }
    pub fn style(&mut self,name:HTMLStr,value:HTMLStr) {
        self.styles.borrow_mut().insert(name,value);
    }
    pub fn make_string(&mut self,prefix:HTMLStr,namespace:&str,fi:&FontInfo) -> HTMLStr {
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
                        match n.fontinfo {
                            None => n.fontinfo = std::mem::take(&mut self.fontinfo),
                            _ => ()
                        }
                        for (k,v) in std::mem::take(&mut self.attributes) {
                            n.attributes.insert(k,v);
                        }
                        for (k,v) in std::mem::take(&mut self.styles) {
                            if n.styles.get(&k).is_none() { n.styles.insert(k,v); }
                        }
                        return n.make_string(prefix,namespace,fi)
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
                        match a.fontinfo {
                            None => a.fontinfo = std::mem::take(&mut self.fontinfo),
                            _ => ()
                        }
                        for (k,v) in std::mem::take(&mut self.attributes) {
                            a.attributes.insert(k,v);
                        }
                        for (k,v) in std::mem::take(&mut self.styles) {
                            if a.styles.get(&k).is_none() { a.styles.insert(k,v); }
                        }
                        return a.make_string(prefix,namespace,fi)
                    }
                }
                _ => ()
            }
        }
        if self.namespace != MATHML_NS {self.classes.push("rustex-contents".into())}
        HTMLNode {
            name: self.name.clone(),
            namespace:self.namespace,
            children: std::mem::take(&mut self.children),
            classes: std::mem::take(&mut self.classes),
            attributes: std::mem::take(&mut self.attributes),
            styles: std::mem::take(&mut self.styles),ht:0,
            forcefont:false,
            fontinfo: std::mem::take(&mut self.fontinfo),
            sourceref: std::mem::take(&mut self.sourceref)
        }.make_string(prefix,namespace,fi)
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
    pub fn make_string(self,prefix:HTMLStr,namespace:&str,fi:&FontInfo) -> HTMLStr {
        match self {
            HTMLChild::Str(s) => s,
            HTMLChild::Node(mut n) => n.make_string(prefix,namespace,fi),
            HTMLChild::Annot(mut a) => a.make_string(prefix,namespace,fi)
        }
    }
    pub fn stringify(&mut self,prefix:HTMLStr,namespace:&str,fi:&FontInfo) {
        match self {
            HTMLChild::Str(_) => (),
            HTMLChild::Node(n) => {
                *self = HTMLChild::Str(n.make_string(prefix,namespace,fi))
            }
            HTMLChild::Annot(n) => {
                *self = HTMLChild::Str(n.make_string(prefix,namespace,fi))
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
            '\'' => "&#39;".to_string(),
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
impl From<&String> for HTMLStr {
    fn from(s: &String) -> Self {
        HTMLStr::Mut(s.clone())
    }
}
impl From<TeXStr> for HTMLStr {
    fn from(s: TeXStr) -> Self {
        HTMLStr::Mut(s.to_utf8())
    }
}
impl From<&TeXStr> for HTMLStr {
    fn from(s: &TeXStr) -> Self {
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