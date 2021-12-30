use crate::interpreter::dimensions::dimtostr;
use crate::references::SourceFileReference;
use crate::stomach::Whatsit;
use crate::stomach::whatsits::{HasWhatsitIter, HEIGHT_CORRECTION, WhatsitIter, WhatsitTrait, WIDTH_CORRECTION};

#[derive(Copy,Clone,PartialEq)]
pub enum BoxMode { H,V,M,DM,Void }

#[derive(Clone)]
pub enum TeXBox {
    Void,H(HBox),V(VBox)
}

impl TeXBox {
    fn pass_on<A>(&self, f: Box<dyn FnOnce(&dyn WhatsitTrait) -> A>, void: A) -> A {
        match self {
            TeXBox::H(hb) => f(hb),
            TeXBox::V(vb) => f(vb),
            TeXBox::Void => void
        }
    }

    fn iter(&self) -> WhatsitIter {
        match self {
            TeXBox::Void => WhatsitIter::default(),
            TeXBox::H(hb) => hb.children.iter_wi(),
            TeXBox::V(vb) => vb.children.iter_wi(),
        }
    }
    pub fn children(self) -> Vec<Whatsit> {
        match self {
            TeXBox::Void => vec!(),
            TeXBox::H(hb) => hb.children,
            TeXBox::V(vb) => vb.children
        }
    }
}

macro_rules! pass_on {
    ($s:tt,$d:expr,$e:ident$(,$tl:expr)*) => (match $s {
        TeXBox::Void => $d,
        TeXBox::H(hb) => HBox::$e(hb $(,$tl)*),
        TeXBox::V(vb) => VBox::$e(vb $(,$tl)*)
    })
}

impl WhatsitTrait for TeXBox {
    fn as_whatsit(self) -> Whatsit {
        Whatsit::Box(self)
    }
    fn width(&self) -> i32 { pass_on!(self,0,width) }
    fn height(&self) -> i32 { pass_on!(self,0,height) }
    fn depth(&self) -> i32 { pass_on!(self,0,depth) }
    fn as_xml_internal(&self, prefix: String) -> String {
        pass_on!(self,"".to_string(),as_xml_internal,prefix)
    }
    fn has_ink(&self) -> bool { pass_on!(self,false,has_ink) }
}

#[derive(Clone)]
pub struct HBox {
    pub children:Vec<Whatsit>,
    pub spread:i32,
    pub _width:Option<i32>,
    pub _height:Option<i32>,
    pub _depth:Option<i32>,
    pub rf : Option<SourceFileReference>
}

impl WhatsitTrait for HBox {
    fn as_whatsit(self) -> Whatsit {
        Whatsit::Box(TeXBox::H(self))
    }

    fn width(&self) -> i32 {
        match self._width {
            Some(i) => i,
            None => {
                let mut w = self.spread;
                for c in self.children.iter_wi() {
                    w += c.width() + WIDTH_CORRECTION
                }
                w
            }
        }
    }

    fn height(&self) -> i32 {
        match self._height {
            Some(i) => i,
            None => {
                let mut w = 0;
                for c in self.children.iter_wi() {
                    let ht = c.height();
                    if ht > w { w = ht }
                }
                w
            }
        }
    }

    fn depth(&self) -> i32 {
        match self._depth {
            Some(d) => d,
            None => {
                let mut d = 0;
                for c in self.children.iter_wi() {
                    let dp = c.depth();
                    if dp > d { d = dp }
                }
                d
            }
        }
    }

    fn as_xml_internal(&self, prefix: String) -> String {
        let mut ret = "\n".to_string() + &prefix + "<hbox";
        match self._width {
            Some(w) => {
                ret += " width=\"";
                ret += &dimtostr(w);
                ret += "\"";
            },
            None => ()
        }
        match self._height {
            Some(w) => {
                ret += " height=\"";
                ret += &dimtostr(w);
                ret += "\"";
            },
            None => ()
        }
        match self._depth {
            Some(w) => {
                ret += " depth=\"";
                ret += &dimtostr(w);
                ret += "\"";
            },
            None => ()
        }
        match self.spread {
            0 => (),
            w => {
                ret += " spread=\"";
                ret += &dimtostr(w);
                ret += "\"";
            },
        }
        ret += ">";
        for c in &self.children {
            ret += &c.as_xml_internal(prefix.clone() + "  ")
        }
        ret + "\n" + &prefix + "</hbox>"
    }

    fn has_ink(&self) -> bool {
        for c in &self.children { if c.has_ink() { return true } }
        false
    }
}

#[derive(Clone)]
pub struct VBox {
    pub children:Vec<Whatsit>,
    pub center:bool,
    pub spread:i32,
    pub _width:Option<i32>,
    pub _height:Option<i32>,
    pub _depth:Option<i32>,
    pub rf : Option<SourceFileReference>
}

impl WhatsitTrait for VBox {
    fn as_whatsit(self) -> Whatsit {
        Whatsit::Box(TeXBox::V(self))
    }

    fn width(&self) -> i32 {
        match self._width {
            Some(i) => i,
            None => {
                let mut w = 0;
                for c in self.children.iter_wi() {
                    let wd = c.width();
                    if wd > w { w = wd }
                }
                w
            }
        }
    }

    fn height(&self) -> i32 {
        let ht = match self._height {
            Some(i) => i,
            None => {
                let mut w = self.spread;
                for c in self.children.iter_wi() { w += c.height() + HEIGHT_CORRECTION }
                w
            }
        };
        if self.center { ht / 2 } else { ht }
    }

    fn depth(&self) -> i32 {
        let dp = match self._depth {
            Some(d) => d,
            None => {
                match self.children.iter_wi().last() {
                    None => 0,
                    Some(c) => c.depth()
                }
            }
        };
        if self.center { dp + self.height() } else { dp }
    }

    fn as_xml_internal(&self, prefix: String) -> String {
        let mut ret = "\n".to_string() + &prefix + "<vbox";
        match self._width {
            Some(w) => {
                ret += " width=\"";
                ret += &dimtostr(w);
                ret += "\"";
            },
            None => ()
        }
        match self._height {
            Some(w) => {
                ret += " height=\"";
                ret += &dimtostr(w);
                ret += "\"";
            },
            None => ()
        }
        match self._depth {
            Some(w) => {
                ret += " depth=\"";
                ret += &dimtostr(w);
                ret += "\"";
            },
            None => ()
        }
        match self.spread {
            0 => (),
            w => {
                ret += " spread=\"";
                ret += &dimtostr(w);
                ret += "\"";
            }
        }
        match self.center {
            true => ret += " center=\"true\"",
            _ => ()
        }
        ret += ">";
        for c in &self.children {
            ret += &c.as_xml_internal(prefix.clone() + "  ")
        }
        ret + "\n" + &prefix + "</vbox>"
    }

    fn has_ink(&self) -> bool {
        for c in &self.children { if c.has_ink() { return true } }
        false
    }
}