use std::borrow::BorrowMut;
use std::collections::HashMap;
use std::fmt::{Display, Formatter};
use std::hash::{Hash, Hasher};
use std::ops::{Add, AddAssign};
use std::sync::Arc;
use itertools::Itertools;
use crate::fonts::{Font, NULL_FONT};
use crate::fonts::fontchars::FontTableParam;
use crate::Interpreter;
use crate::interpreter::dimensions::{numtostr, Skip};
use crate::references::SourceFileReference;
use crate::stomach::colon::{Colon, ColonBase, ColonMode};
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
    pub current_namespace:&'static str,
    pub top:Vec<HTMLChild>,
    pub currsize:i32,
    pub currcolor:Option<HTMLStr>
}
impl HTMLState {
    pub fn new() -> HTMLState { HTMLState {
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
    pub state:HTMLState,
    pub namespaces : HashMap<String,String>,
    pagewidth:i32,textwidth:i32,lineheight:Skip
}
unsafe impl Send for HTMLColon {}

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
    fn initialize(&mut self, basefont: Arc<Font>, basecolor: TeXStr, int: &Interpreter) {
        if self.doheader {
            self.state.currsize = basefont.at.unwrap_or(655360);
            self.state.currcolor = match &basecolor {
                s if s.to_string() == "000000" => None,
                s => Some(s.clone().into())
            };
            self.pagewidth = int.state.dimensions.get(&-(crate::commands::pdftex::PDFPAGEWIDTH.index as i32));
            self.textwidth = int.state.dimensions.get(&-(crate::commands::primitives::HSIZE.index as i32));
            self.lineheight = int.state.skips.get(&-(crate::commands::primitives::BASELINESKIP.index as i32));

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
            ret += "\n  </head>\n  <body style=\"width:";
            ret += &dimtohtml(self.pagewidth).to_string();
            ret += "\">\n    <div class=\"body\" style=\"font-size:";
            let fontsize = match &self.base.basefont.as_ref() {
                Some(f) => match f.at {
                    Some(i) => i,
                    None => 655360
                }
                None => 655360
            };
            ret += &dimtohtml(fontsize).to_string();
            ret += ";width:";
            ret += &dimtohtml(self.textwidth).to_string();
            ret += ";padding-left:";
            ret += &dimtohtml(((self.pagewidth - self.textwidth) as f32 / 2.0).round() as i32).to_string();
            ret += ";padding-right:";
            ret += &dimtohtml(((self.pagewidth - self.textwidth) as f32 / 2.0).round() as i32).to_string();
            ret += ";line-height:";
            ret += &(self.lineheight.base as f32 / fontsize as f32).to_string();
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
            textwidth: 0,
            lineheight: Skip {base:0, stretch: None, shrink: None }
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
    pub name:HTMLStr,
    pub namespace:&'static str,
    pub children:Vec<HTMLChild>,
    pub classes:Vec<HTMLStr>,
    attributes:HashMap<HTMLStr,HTMLStr>,
    styles:HashMap<HTMLStr,HTMLStr>,
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
        name,namespace,children:vec!(),classes:vec!(),
        attributes:HashMap::new(),styles:HashMap::new(),
        fontinfo:None,sourceref
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
                        self.children = std::mem::take(&mut a.children)
                    }
                }
                _ => ()
            }
        }

        let fi_o = self.fontinfo.take();
        let nfi = match &fi_o {
            None => fi,
            Some(ref mi) if self.namespace == MATHML_NS => {
                if mi.params.contains(&FontTableParam::SansSerif) &&
                    mi.params.contains(&FontTableParam::Bold) &&
                    mi.params.contains(&FontTableParam::Italic) {
                    self.attr("mathvariant".into(),"sans-serif-bold-italic".into())
                }
                else if mi.params.contains(&FontTableParam::SansSerif) &&
                    mi.params.contains(&FontTableParam::Italic) {
                    self.attr("mathvariant".into(),"sans-serif-italic".into())
                }
                else if mi.params.contains(&FontTableParam::SansSerif) &&
                    mi.params.contains(&FontTableParam::Bold) {
                    self.attr("mathvariant".into(),"bold-sans-serif".into())
                }
                else if mi.params.contains(&FontTableParam::Script) &&
                    mi.params.contains(&FontTableParam::Bold) {
                    self.attr("mathvariant".into(),"bold-script".into())
                }
                else if mi.params.contains(&FontTableParam::Fraktur) &&
                    mi.params.contains(&FontTableParam::Bold) {
                    self.attr("mathvariant".into(),"bold-fraktur".into())
                }
                else if mi.params.contains(&FontTableParam::Italic) &&
                    mi.params.contains(&FontTableParam::Bold) {
                    self.attr("mathvariant".into(),"bold-italic".into())
                }
                else if mi.params.contains(&FontTableParam::Bold) {
                    self.attr("mathvariant".into(),"bold".into())
                }
                else if mi.params.contains(&FontTableParam::Italic) {
                    self.attr("mathvariant".into(),"italic".into())
                }
                else if mi.params.contains(&FontTableParam::Blackboard) {
                    self.attr("mathvariant".into(),"double-struck".into())
                }
                else if mi.params.contains(&FontTableParam::Script) {
                    self.attr("mathvariant".into(),"script".into())
                }
                else if mi.params.contains(&FontTableParam::Fraktur) {
                    self.attr("mathvariant".into(),"fraktur".into())
                }
                else if mi.params.contains(&FontTableParam::SansSerif) {
                    self.attr("mathvariant".into(),"sans-serif".into())
                }
                else if mi.params.contains(&FontTableParam::Monospaced) {
                    self.attr("mathvariant".into(),"monospace".into())
                }
                else {
                    self.attr("mathvariant".into(),"normal".into())
                }
                let ratio = (mi.at as f32) / (fi.at as f32);
                if ratio != 1.0 {
                    self.attr("mathsize".into(), ((ratio * 100.0).round().to_string() + "%").into())
                }
                mi
            }
            Some(ref mi) => {
                self.style("font-family".into(),"STIXgeneral, Times, Symbol, cmr10, CMSY10, CMEX10, serif".into());
                self.style("font-style".into(),"normal".into());
                self.style("font-weight".into(),"normal".into());
                self.style("font-variant".into(),"normal".into());
                if mi.params.contains(&FontTableParam::Monospaced) {
                    self.style("font-family".into(),"monospace".into())
                }
                if mi.params.contains(&FontTableParam::Italic) {
                    self.style("font-style".into(),"italic".into())
                }
                if mi.params.contains(&FontTableParam::Bold) {
                    self.style("font-weight".into(),"bold".into())
                }
                if mi.params.contains(&FontTableParam::Blackboard) {
                    self.style("font-family".into(),"msbm".into())
                }
                if mi.params.contains(&FontTableParam::Capital) {
                    self.style("font-variant".into(),"small-caps".into())
                }
                if mi.params.contains(&FontTableParam::SansSerif) {
                    self.style("font-family".into(),"sans-serif".into())
                }
                if mi.params.contains(&FontTableParam::Script) {
                    self.style("font-family".into(),"URW Chancery L, cursive".into())
                }
                self.style("font-size".into(),((100.0 * (mi.at as f32) / (fi.at as f32)).round().to_string() + "%").into());
                mi
            }
        };

        let mut ret : HTMLStr = "".into();
        let mut body : HTMLStr = "".into();
        for c in self.children.drain(..) {
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
        HTMLNode {
            name: self.name.clone(),
            namespace:self.namespace,
            children: std::mem::take(&mut self.children),
            classes: std::mem::take(&mut self.classes),
            attributes: std::mem::take(&mut self.attributes),
            styles: std::mem::take(&mut self.styles),
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