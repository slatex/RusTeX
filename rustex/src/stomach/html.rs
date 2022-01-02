use std::borrow::BorrowMut;
use std::collections::HashMap;
use std::fmt::{Debug, Display, Formatter};
use std::hash::{Hash, Hasher};
use std::sync::Arc;
use image::EncodableLayout;
use crate::fonts::Font;
use crate::Interpreter;
use crate::interpreter::dimensions::numtostr;
use crate::stomach::colon::{Colon, ColonBase};
use crate::stomach::Whatsit;
use crate::stomach::whatsits::WhatsitTrait;
use crate::utils::TeXStr;

static HTML_NS : &str = "http://www.w3.org/1999/xhtml";
static MATHML_NS : &str = "http://www.w3.org/1998/Math/MathML";
static SVG_NS : &str = "http://www.w3.org/2000/svg";
static RUSTEX_NS : &str = "http://kwarc.info/ns/RusTeX";

pub struct HTMLState {
    annotations_todo:Vec<(HTMLStr,HTMLStr)>,
    current_node:Option<HTMLNode>,
    current_namespace:&'static str,
    top:Vec<HTMLChild>
}
impl HTMLState {
    pub fn new() -> HTMLState { HTMLState {
        annotations_todo:vec!(),
        current_node:None,
        current_namespace:HTML_NS,
        top:vec!()
    }}
}
macro_rules! node {
    ($sel:ident,$node:ident,$name:ident) => ({
        //node!($sel,$ns:$node,$name,() =>)
        let mut _node_newnode = HTMLNode::new($sel.state.current_namespace,stringify!($node).into());
        _node_newnode.classes.push(stringify!($name).into());
        match &mut $sel.state.current_node {
            Some(e) => {
                e.children.push(HTMLChild::Node(_node_newnode))
            }
            None => {
                $sel.state.top.push(HTMLChild::Node(_node_newnode))
            }
        }
    });
    ($sel:ident,$ns:tt:$node:ident,$name:ident) => ({
        let mut _node_newnode = HTMLNode::new($ns,stringify!($node).into());
        _node_newnode.classes.push(stringify!($name).into());
        match &mut $sel.state.current_node {
            Some(e) => {
                e.children.push(HTMLChild::Node(_node_newnode))
            }
            None => {
                $sel.state.top.push(HTMLChild::Node(_node_newnode))
            }
        }
    });
    ($sel:ident,$node:ident,$name:ident,$nodename:ident => $e:expr) => (
        {
            let _node_oldtop = $sel.state.current_node.replace(HTMLNode::new($sel.state.current_namespace,stringify!($node).into()));
            $sel.state.current_node.as_mut().unwrap().classes.push(stringify!($name).into());
            let mut $nodename = $sel.state.current_node.as_mut().unwrap();
            $e;
            let _node_ret = $sel.state.current_node.take().unwrap();
            $sel.state.current_node = _node_oldtop;
            match &mut $sel.state.current_node {
                Some(e) => {
                    e.children.push(HTMLChild::Node(_node_ret))
                }
                None => {
                    $sel.state.top.push(HTMLChild::Node(_node_ret))
                }
            }
        }
    );
    ($sel:ident,$ns:tt:$node:ident,$name:ident,$nodename:ident => $e:expr) => (
        {
            let _node_oldns = $sel.state.current_namespace;
            let _node_oldtop = $sel.state.current_node.replace(HTMLNode::new($ns,stringify!($node).into()));
            $sel.state.current_node.as_mut().unwrap().classes.push(stringify!($name).into());
            $sel.state.current_namespace = $ns;
            let mut $nodename = $sel.state.current_node.as_mut().unwrap();
            $e;
            let _node_ret = $sel.state.current_node.take().unwrap();
            $sel.state.current_node = _node_oldtop;
            $sel.state.current_namespace = _node_oldns;
            match &mut $sel.state.current_node {
                Some(e) => {
                    e.children.push(HTMLChild::Node(_node_ret))
                }
                None => {
                    $sel.state.top.push(HTMLChild::Node(_node_ret))
                }
            }
        }
    );
}

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
    fn close(self) -> String {
        if self.doheader {
            self.ret + "\n    </div>\n  </body>\n</html>"
        } else { self.ret }
    }
    fn ship_whatsit(&mut self, w:Whatsit) { for w in self.normalize_whatsit(w) { self.ship_top(w) } }
    fn initialize(&mut self, basefont: Arc<Font>, basecolor: TeXStr, int: &Interpreter) {
        if self.doheader {
            let base = self.base_mut();
            base.basefont = Some(basefont);
            base.basecolor = Some(basecolor);
            // TODO moar
            self.ret += "<html><body><div>" // TODO
        }
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
                    node.style("margin-bottom".into(),numtostr(vsk.skip.base,"px").into());
                })
            },
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
                        Some(sk) if sk.base != 0 => node.style("margin-left".into(),numtostr(sk.base,"px").into()),
                        _ => ()
                    }
                    match p.rightskip {
                        Some(sk) if sk.base != 0 => node.style("margin-right".into(),numtostr(sk.base,"px").into()),
                        _ => ()
                    }
                    node.style("width".into(),numtostr(p.width(),"px").into());
                    node.style("min-width".into(),numtostr(p.width(),"px").into());
                    for c in p.children { self.ship_w(c) }
                });
            }
            _ => self.ret += &w.as_xml_internal("  ".to_string())
        }
    }
    fn ship_w(&mut self,w:Whatsit) {
        use Whatsit::*;
        use crate::stomach::simple::SimpleWI::*;
        use crate::stomach::boxes::TeXBox::*;
        use crate::stomach::groups::WIGroup::*;
        match w {
            Grouped(FontChange(fc)) => {
                todo!()
            }
            _ => self.ret += &w.as_xml_internal("  ".to_string())
        }
    }
}

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

}
impl Display for HTMLNode {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        todo!()
    }
}

enum HTMLChild {
    Str(HTMLStr),
    Node(HTMLNode)
}

enum HTMLStr {
    Imm(&'static str),
    Mut(String)
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