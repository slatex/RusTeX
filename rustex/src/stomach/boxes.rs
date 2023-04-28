use std::cmp::max;
use crate::interpreter::dimensions::dimtostr;
use crate::{htmlliteral, htmlnode, htmlparent, withlinescale, withwidth};
use crate::references::SourceFileReference;
use crate::stomach::colon::ColonMode;
use crate::stomach::html::{dimtohtml, HTML_NS, HTMLChild, HTMLColon, HTMLNode, HTMLParent, HTMLStr};
use crate::stomach::math::{GroupedMath, MathKernel};
use crate::stomach::simple::{Left, Right, SimpleWI};
use crate::stomach::Whatsit;
use crate::stomach::whatsits::{HasWhatsitIter, HEIGHT_CORRECTION, WhatsitTrait, WIDTH_CORRECTION};

#[derive(Copy,Clone,PartialEq)]
pub enum VBoxType { V, Center, Top(i32), DMCenter }

#[derive(Copy,Clone,PartialEq)]
pub enum BoxMode { H,V,M,DM,LeftRight,Void }

#[derive(Clone)]
pub enum TeXBox {
    Void,H(HBox),V(VBox),M(GroupedMath),DM(GroupedMath),LeftRight(Option<MathKernel>,GroupedMath,Option<MathKernel>)
}
impl Default for TeXBox {
    fn default() -> Self {TeXBox::Void}
}
impl PartialEq for TeXBox {
    fn eq(&self, other: &Self) -> bool {
        match (self,other) {
            (TeXBox::Void,TeXBox::Void) => true,
            _ => false
        }
    }
}

impl TeXBox {
    pub fn children(self) -> Vec<Whatsit> {
        match self {
            TeXBox::Void => vec!(),
            TeXBox::H(hb) => hb.children,
            TeXBox::V(vb) => vb.children,
            TeXBox::M(GroupedMath(v,_)) => v,
            TeXBox::DM(GroupedMath(v,_)) => v,
            TeXBox::LeftRight(l,GroupedMath(mut v,_),r) => {
                for le in l {
                    v.insert(0,le.as_whatsit());
                }
                for ri in r {
                    v.push(ri.as_whatsit());
                }
                v
            },
        }
    }
}

macro_rules! pass_on {
    ($s:tt,$d:expr,$e:ident$(,$tl:expr)*) => (match $s {
        TeXBox::Void => $d,
        TeXBox::H(hb) => HBox::$e(hb $(,$tl)*),
        TeXBox::V(vb) => VBox::$e(vb $(,$tl)*),
        TeXBox::M(m) => GroupedMath::$e(m $(,$tl)*),
        TeXBox::DM(m) => GroupedMath::$e(m $(,$tl)*),
        TeXBox::LeftRight(_,_,_) =>
            unreachable!(),
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
    fn normalize(self, mode: &ColonMode, ret: &mut Vec<Whatsit>, scale: Option<f32>) {
        pass_on!(self,(),normalize,mode,ret,scale)
    }
    fn as_html(self, mode: &ColonMode, colon: &mut HTMLColon, node_top: &mut Option<HTMLParent>) {
        pass_on!(self,(),as_html,mode,colon,node_top)
    }
    fn get_ref(&self) -> Option<SourceFileReference> { pass_on!(self,None,get_ref) }
    //fn get_par_width(&self) -> Option<i32> { pass_on!(self,None,get_par_width) }
    //fn get_par_widths(&self) -> Vec<i32> { pass_on!(self,vec!(),get_par_widths) }
}

#[derive(Clone)]
pub struct HBox {
    pub children:Vec<Whatsit>,
    pub spread:i32,
    pub _width:Option<i32>,
    pub _height:Option<i32>,
    pub _depth:Option<i32>,
    pub _to:Option<i32>,
    pub rf : Option<SourceFileReference>,
    pub lineheight:Option<i32>
}

impl HBox {
    pub fn new_trivial(v:Vec<Whatsit>) -> Self {HBox {children:v,spread:0,_depth:None,_height:None,_width:None,_to:None,rf:None,lineheight:None}}
}

impl WhatsitTrait for HBox {
    fn as_whatsit(self) -> Whatsit {
        Whatsit::Box(TeXBox::H(self))
    }
    fn width(&self) -> i32 {
        self._width.unwrap_or(self._to.unwrap_or(self.spread + self.inner_width()))
    }
    fn height(&self) -> i32 {
        match self._height {
            Some(i) => i,
            None => {
                let mut w = 0;
                for c in self.children.iter_wi() {
                    let ht = c.height();// + c.depth();
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

    fn normalize(self, mode: &ColonMode, ret: &mut Vec<Whatsit>, scale: Option<f32>) {
        match mode {
            ColonMode::V | ColonMode::External(_) | ColonMode::P => {
                let mut nch : Vec<Whatsit> = vec!();
                for c in self.children { c.normalize(&ColonMode::H,&mut nch,scale) }
                if nch.is_empty() && (self._width.is_none() || self._width == Some(0)) { return () }
                ret.push(HBox {
                    children: nch,
                    spread: self.spread,
                    _width: self._width,
                    _height: self._height,
                    _depth: self._depth,
                    _to:self._to,
                    rf: self.rf,
                    lineheight:self.lineheight
                }.as_whatsit())
            }
            ColonMode::H => {
               /* if self._width.is_none() && self._height.is_none() && self._depth.is_none() {
                    for c in self.children { c.normalize(&ColonMode::H,ret,scale) }
                } else {*/
                    let mut nch : Vec<Whatsit> = vec!();
                    for c in self.children { c.normalize(&ColonMode::H,&mut nch,scale) }
                    if nch.is_empty() && (self._width.is_none() || self._width == Some(0)) {return () }
                    ret.push(HBox {
                        children: nch,
                        spread: self.spread,
                        _width: self._width,
                        _height: self._height,
                        _depth: self._depth,
                        _to:self._to,
                        rf: self.rf,
                        lineheight:self.lineheight
                    }.as_whatsit())
               // }
            }
            ColonMode::M => {
                let mut nch : Vec<Whatsit> = vec!();
                for c in self.children { c.normalize(&ColonMode::H,&mut nch,None) }
                if nch.is_empty() && (self._width.is_none() && self._height.is_none() && self._depth.is_none()/* || self._width == Some(0)*/) { return () }
                ret.push(HBox {
                    children: nch,
                    spread: self.spread,
                    _width: self._width,
                    _height: self._height,
                    _depth: self._depth,
                    _to:self._to,
                    rf: self.rf,
                    lineheight:self.lineheight
                }.as_whatsit())
            }
        }
    }

    fn as_html(mut self, mode: &ColonMode, colon: &mut HTMLColon, node_top: &mut Option<HTMLParent>) {
        match mode {
            ColonMode::H | ColonMode::V | ColonMode::P => {
                //let parwidth = self.get_par_width();
                let width = self._width;//.or(if self.width() <= 0 {Some(0)} else {parwidth});
                match (self._height,self._depth,width,self._to) {
                    (None,None,None,None) => {
                        self.html_inner(colon,node_top,false);
                    },
                    _ => htmlnode!(colon,div,None,"rustex-hbox-container",node_top,cont => {
                        withlinescale!(colon,self.lineheight,cont,{
                        if let Some(ht) = self._height {
                            if ht < 0 {
                                if let Some(dp) = self._depth {
                                    let nd = dp + ht;
                                    self._depth = None;
                                    cont.style("margin-bottom".into(),dimtohtml(nd))
                                } else {
                                    cont.style("margin-bottom".into(),dimtohtml(ht))
                                }
                                cont.style("height".into(),"0".into());
                            } else {
                                cont.style("height".into(),dimtohtml(ht));
                            }
                        }
                        if let Some(dp) = self._depth {
                            cont.style("margin-bottom".into(),dimtohtml(dp))
                        }
                        if self._to.is_some() && self._to.unwrap() <= 0 && self._width.is_none() {
                            let wd = if let Some(wd) = self._to {wd} else {unreachable!()};
                            self._to = None;
                            withwidth!(colon,wd,cont,ncont => {
                                self.html_inner(colon,htmlparent!(ncont),true)
                            })
                        } else if let Some(wd) = width {
                            let iwd = self.inner_width();
                            withwidth!(colon,wd,cont,ncont => {
                                self.html_inner(colon,htmlparent!(ncont),wd < iwd || wd <= 0)
                            })
                        } else {
                            //let oldwidth = colon.state.currsize;
                            //colon.state.currsize = 2 * self.width(); // hack for wd=0
                            self.html_inner(colon,htmlparent!(cont),false);
                            //colon.state.currsize = oldwidth;
                        }
                        });
                    })
                }
            }
            ColonMode::M if self.children.is_empty() => {
                htmlnode!(colon,mi,None,"rustex-dummy",node_top);
                htmlnode!(colon,mspace,self.get_ref(),"rustex-phantom",node_top,mt => {
                    mt.attr("stretchy".into(),"false".into());
                    mt.attr("width".into(),dimtohtml(self.width()));
                    mt.attr("height".into(),dimtohtml(self.height()));
                    mt.attr("depth".into(),dimtohtml(self.depth()));
                });
            }
            ColonMode::M => htmlnode!(colon,mtext,self.get_ref(),"",node_top,mt => {
                withlinescale!(colon,self.lineheight,mt,{
                //let oldwd = colon.state.currsize;
                let mut wd = self.width();
                if wd == 0 {wd = 2048};
                //colon.state.currsize = wd;
                //mt.style("width".into(),dimtohtml(wd));
                htmlnode!(colon,HTML_NS:span,None,"rustex-math-escape",htmlparent!(mt),span => {
                    span.forcefont = true;
                    htmlliteral!(colon,htmlparent!(span),"\n");
                    self.as_html(&ColonMode::H,colon,htmlparent!(span));
                    htmlliteral!(colon,htmlparent!(span),"\n");
                });
                //colon.state.currsize = oldwd;
                    })
            }),
            _ => for c in self.children { c.as_html(mode,colon,node_top) }
        }
    }
    fn get_ref(&self) -> Option<SourceFileReference> {
        SourceFileReference::from_wi_list(&self.children).or(self.rf.clone())
    }
    /*fn get_par_width(&self) -> Option<i32> {
        self._width.or({
            let mut ret : Option<i32> = None;
            for c in &self.children {
                for w in c.get_par_widths() {
                    match ret {
                        None => ret = Some(w),
                        Some(ow) => ret = Some(w + ow)
                    }
                }
            }
            ret
        })
    }
    fn get_par_widths(&self) -> Vec<i32> { self.get_par_width().map(|i| vec!(i)).unwrap_or(vec!()) }

     */
}

enum FilLevel {
    None,Fil,Fill
}
impl FilLevel {
    fn add(&mut self,other:FilLevel) {
        use FilLevel::*;
        match (&self,other) {
            (None,o) => *self = o,
            (_,None) => (),
            (Fil,Fill) => *self = Fill,
            _ => (),
        }
    }
}

impl HBox {
    fn inner_width(&self) -> i32 {
        let mut w = 0;
        for c in self.children.iter_wi() {
            w += c.width() + WIDTH_CORRECTION
        }
        w
    }
    fn html_inner(mut self, colon: &mut HTMLColon, node_top: &mut Option<HTMLParent>,incontainer:bool) {
        let mut clss : Vec<HTMLStr> = vec!();
        let mut styles : Vec<(HTMLStr,HTMLStr)> = vec!();
        macro_rules! ch_as_html {
            ($node:expr) => {
                let mut startfil = FilLevel::None;
                let mut endfil = FilLevel::None;
                let mut repush:Vec<Whatsit> = vec!();
                while let Some(head) = self.children.first() {
                    match head {
                        Whatsit::Simple(SimpleWI::HFil(_) | SimpleWI::Hss(_)) => {
                            startfil.add(FilLevel::Fil);
                            self.children.remove(0);
                        }
                        Whatsit::Simple(SimpleWI::HFill(_)) => {
                            startfil.add(FilLevel::Fill);
                            self.children.remove(0);
                        }
                        o if !o.has_ink() => {
                            repush.push(self.children.remove(0))
                        }
                        _ => break
                    }
                }
                for c in repush.into_iter().rev() {self.children.insert(0,c)}
                let mut repush:Vec<Whatsit> = vec!();
                while let Some(head) = self.children.last() {
                    match head {
                        Whatsit::Simple(SimpleWI::HFil(_) | SimpleWI::Hss(_)) => {
                            endfil.add(FilLevel::Fil);
                            self.children.pop();
                        }
                        Whatsit::Simple(SimpleWI::HFill(_)) => {
                            endfil.add(FilLevel::Fill);
                            self.children.pop();
                        }
                        o if !o.has_ink() => {
                            repush.push(self.children.pop().unwrap())
                        }
                        _ => break
                    }
                }
                for c in repush.into_iter().rev() {self.children.push(c)}
                match (startfil,endfil) {
                    (FilLevel::None | FilLevel::Fil,FilLevel::Fill)|(FilLevel::None,FilLevel::Fil) =>{
                        styles.push(("justify-content".into(),"start".into()));
                        clss.push("rustex-hbox-no-space".into());
                    }
                    (FilLevel::Fil,FilLevel::Fil)|(FilLevel::Fill,FilLevel::Fill) =>{
                        styles.push(("justify-content".into(),"center".into()));
                        clss.push("rustex-hbox-no-space".into());
                    }
                    (FilLevel::Fil|FilLevel::Fill,FilLevel::None)|(FilLevel::Fill,FilLevel::Fil) =>{
                        styles.push(("justify-content".into(),"end".into()));
                        clss.push("rustex-hbox-no-space".into());
                    }
                    _ => ()
                }
                for c in self.children {
                    c.as_html(&ColonMode::H,colon,htmlparent!($node))
                }
            }
        }
        match (self._to,self.spread) {
            (None,0) => htmlnode!(colon,div,self.get_ref(),"rustex-hbox",node_top,node => {
                if crate::INSERT_RUSTEX_ATTRS {
                    node.attr("rustex:width".into(),dimtohtml(self.width()));
                    node.attr("rustex:height".into(),dimtohtml(self.height()));
                }
                if (incontainer) {
                    node.style("width".into(),"100%".into());
                }
                ch_as_html!(node);
                for c in clss { node.classes.push(c)}
                for (a,b) in styles {node.style(a,b)}
            }),
            (Some(to),_) => htmlnode!(colon,div,self.get_ref(),"rustex-hbox",node_top,node => {
                withwidth!(colon,to,node,nnode => {
                    if crate::INSERT_RUSTEX_ATTRS {
                        nnode.attr("rustex:width".into(),dimtohtml(self.width()));
                        nnode.attr("rustex:height".into(),dimtohtml(self.height()));
                        nnode.attr("rustex:to".into(),dimtohtml(to));
                    }
                    ch_as_html!(nnode);
                });
                for c in clss { node.classes.push(c)}
                for (a,b) in styles {node.style(a,b)}
            }),
            (_,spread) => htmlnode!(colon,div,self.get_ref(),"rustex-hbox",node_top,node => {
                withwidth!(colon,self.width(),node,nnode => {
                    if crate::INSERT_RUSTEX_ATTRS {
                        nnode.attr("rustex:width".into(),dimtohtml(self.width()));
                        nnode.attr("rustex:height".into(),dimtohtml(self.height()));
                        nnode.attr("rustex:spread".into(),dimtohtml(spread));
                    }
                    ch_as_html!(nnode);
                });
                for c in clss { node.classes.push(c)}
                for (a,b) in styles {node.style(a,b)}
            }),
        }
    }
}

#[derive(Clone)]
pub struct VBox {
    pub children:Vec<Whatsit>,
    pub tp:VBoxType,
    pub spread:i32,
    pub _width:Option<i32>,
    pub _height:Option<i32>,
    pub _depth:Option<i32>,
    pub _to:Option<i32>,
    pub rf : Option<SourceFileReference>
}

impl WhatsitTrait for VBox {
    /*
    fn get_par_width(&self) -> Option<i32> {
        self._width.or({
            let mut ret : Option<i32> = None;
            for c in &self.children {
                for w in c.get_par_widths() {
                    match ret {
                        Some(ow) if ow < w => ret = Some(w),
                        None => ret = Some(w),
                        _ => ()
                    }
                }
            }
            ret
        })
    }
    fn get_par_widths(&self) -> Vec<i32> {
        self.get_par_width().map(|i| vec!(i)).unwrap_or(vec!())
    }*/
    fn get_ref(&self) -> Option<SourceFileReference> {
        SourceFileReference::from_wi_list(&self.children).or(self.rf.clone())
    }
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
        if let VBoxType::Top(r) = self.tp {return r}
        let ht = match self._height.or(self._to) {
            Some(i) => i,
            None => {
                let mut w = self.spread;
                for c in self.children.iter_wi() { w += c.height() + HEIGHT_CORRECTION }
                w
            }
        };
        match self.tp {
            VBoxType::V => ht,
            VBoxType::Center | VBoxType::DMCenter => ht / 2,
            VBoxType::Top(i) => i
        }
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
        match self.tp {
            VBoxType::V => dp,
            VBoxType::Center | VBoxType::DMCenter => dp + self.height(),
            VBoxType::Top(i) =>  {
                let ht = match self._height {
                    Some(i) => i,
                    None => {
                        let mut w = self.spread;
                        for c in self.children.iter_wi() { w += c.height() + HEIGHT_CORRECTION }
                        w
                    }
                };
                dp + (ht - i)
            }
        }
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
        match self.tp {
            VBoxType::Center => ret += " center=\"true\"",
            VBoxType::Top(i) => ret += &(" top=\"".to_string() + &i.to_string() + "\""),
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

    fn normalize(self, mode: &ColonMode, ret: &mut Vec<Whatsit>, scale: Option<f32>) {
        match mode {
            ColonMode::H | ColonMode::External(_) | ColonMode::P | ColonMode::V => {
                let mut nch : Vec<Whatsit> = vec!();
                for c in self.children { c.normalize(&ColonMode::V,&mut nch,scale) }
                if nch.is_empty() && (self._height.is_none() || self._height == Some(0)) { return () }
                ret.push(VBox {
                    children: nch,
                    spread: self.spread,
                    _width: self._width,
                    _height: self._height,
                    _depth: self._depth,
                    _to:self._to,
                    rf: self.rf,
                    tp:self.tp
                }.as_whatsit())
            }
            /*ColonMode::V => {
                if self._width.is_none() && self._height.is_none() && self._depth.is_none() {
                    for c in self.children { c.normalize(&ColonMode::V,ret,scale) }
                } else {
                    let mut nch : Vec<Whatsit> = vec!();
                    for c in self.children { c.normalize(&ColonMode::V,&mut nch,scale) }
                    if nch.is_empty() && (self._height.is_none() || self._height == Some(0)) {return () }
                    ret.push(VBox {
                        children: nch,
                        spread: self.spread,
                        _width: self._width,
                        _height: self._height,
                        _depth: self._depth,
                        _to:self._to,
                        rf: self.rf,
                        tp:self.tp
                    }.as_whatsit())
                }
            }*/
            ColonMode::M => {
                use crate::stomach::simple::SimpleWI;
                let mut nch : Vec<Whatsit> = vec!();
                for c in self.children { c.normalize(&ColonMode::V,&mut nch,None) }
                if nch.is_empty() && (self._height.is_none() || self._height == Some(0)) { return () }
                /*else if nch.len() == 1 {
                    match nch.pop() {
                        Some(o@Whatsit::Simple(SimpleWI::HAlign(_))) => {
                            ret.push(o);
                            return
                        }
                        Some(o) => {
                            nch.push(o)
                        }
                        _ => unreachable!()
                    }
                }*/
                ret.push(VBox {
                    children: nch,
                    spread: self.spread,
                    _width: self._width,
                    _height: self._height,
                    _depth: self._depth,
                    _to:self._to,
                    rf: self.rf,
                    tp:self.tp
                }.as_whatsit())
            }
        }
    }

    fn as_html(mut self, mode: &ColonMode, colon: &mut HTMLColon, node_top: &mut Option<HTMLParent>) {
        match mode {
            ColonMode::V | ColonMode::H | ColonMode::P if self.tp == VBoxType::DMCenter => {
                htmlnode!(colon,div,None,"rustex-display-vcenter",node_top,div => {
                    htmlnode!(colon,div,self.get_ref(),"rustex-vcenter",htmlparent!(div),node => {
                        if let Some(dp) = self._depth {
                            div.style("margin-bottom".into(),dimtohtml(dp))
                        }
                        if crate::INSERT_RUSTEX_ATTRS {
                            node.attr("rustex:width".into(),dimtohtml(self.width()));
                            node.attr("rustex:height".into(),dimtohtml(self.height()));
                        }
                        match self._height {
                            Some(ht) => {
                                if ht < 0 {
                                    node.style("margin-bottom".into(),dimtohtml(ht));
                                    node.style("height".into(),"0".into());
                                } else {
                                    node.style("height".into(),dimtohtml(ht));
                                }
                            }
                            _ => ()
                        }
                        match self._width {
                            Some(h) => {
                                withwidth!(colon,h,node,inner => {
                                    VBox::ch_as_html(self.children,colon,&mut inner);
                                })
                            }
                            _ => VBox::ch_as_html(self.children,colon,&mut node),
                            /*
                            _ => {
                                match self.get_par_width() {
                                    None => VBox::ch_as_html(self.children,colon,&mut node),
                                    Some(i) => withwidth!(colon,i,node,inner => {
                                        VBox::ch_as_html(self.children,colon,&mut node);
                                    })
                                }
                            }

                             */
                        }
                    });
                });
            }
            ColonMode::V | ColonMode::H | ColonMode::P if matches!(self.tp,VBoxType::Top(_)) => {
                let width = self._width.or(if self.width() <= 0 {Some(0)} else {None});//self.get_par_width()});
                match width {
                    None => htmlnode!(colon,div,None,"rustex-vtop-container",node_top,cont => {
                        self.html_t_inner(colon,htmlparent!(cont),false);
                    }),
                    Some(wd) => htmlnode!(colon,div,None,"rustex-vtop-container",node_top,cont => {
                        withwidth!(colon,wd,cont,ncont => {self.html_t_inner(colon,htmlparent!(ncont),true)})
                    })
                }
            }
            ColonMode::V | ColonMode::H | ColonMode::P if matches!(self.tp,VBoxType::V) => {
                let width = self._width.or(if self.width() <= 0 {Some(0)} else {None});//self.get_par_width()});
                match width {
                    /*(None,None) => self.html_inner(colon,node_top,false),*/
                    None => htmlnode!(colon,div,None,"rustex-vbox-container",node_top,cont => {
                        if let Some(d) = self._depth {cont.style("margin-bottom".into(),dimtohtml(d))}
                        self.html_v_inner(colon,htmlparent!(cont),false);
                    }),
                    Some(wd) => htmlnode!(colon,div,None,"rustex-vbox-container",node_top,cont => {
                        //if let Some(ht) = self._height {cont.style("height".into(),dimtohtml(ht))}
                        if let Some(d) = self._depth {cont.style("margin-bottom".into(),dimtohtml(d))}
                        withwidth!(colon,wd,cont,ncont => {self.html_v_inner(colon,htmlparent!(ncont),true)})
                    })
                }
            }
            ColonMode::V | ColonMode::H | ColonMode::P => {
                let (innercls,cls) = if self.tp == VBoxType::Center {
                    ("rustex-vcenter-container","rustex-vcenter")
                } else {
                    ("rustex-vbox-container","rustex-vbox")
                };
                //htmlnode!(colon,div,None,outercls,node_top,conta => {
                    htmlnode!(colon,div,None,innercls,node_top,cont => {
                    if crate::INSERT_RUSTEX_ATTRS {
                        cont.attr("rustex:width".into(),dimtohtml(self.width()));
                        cont.attr("rustex:height".into(),dimtohtml(self.height()));
                    }
                        if let Some(ht) = self._height {
                            if ht < 0 {
                                if let Some(d) = self._depth {
                                    let nd = d + ht;
                                    self._depth = None;
                                    cont.style("margin-bottom".into(),dimtohtml(nd));
                                } else {
                                    cont.style("margin-bottom".into(),dimtohtml(ht));
                                }
                                cont.style("height".into(),"0".into());
                            } else {
                                cont.style("height".into(),dimtohtml(ht));
                            }
                        }
                        if let Some(d) = self._depth {
                            cont.style("margin-bottom".into(),dimtohtml(d))
                        }
                        let width = self._width.or(if self.width() == 0 {Some(0)} else {None});//self.get_par_width()});
                        match width {
                            Some(w) => withwidth!(colon,w,cont,inner => {
                                htmlnode!(colon,div,None,cls,htmlparent!(inner),node => {
                                    node.style("width".into(),"100%".into());
                                    node.style("max-width".into(),"100%".into());
                                    VBox::ch_as_html(self.children,colon,&mut node);
                                });
                            }),
                            None => htmlnode!(colon,div,None,cls,htmlparent!(cont),node => {
                                VBox::ch_as_html(self.children,colon,&mut node);
                            })
                        }
                    });
                //});
            }
            ColonMode::M if self.children.is_empty() => {
                htmlnode!(colon,mi,None,"rustex-dummy",node_top);
                htmlnode!(colon,mspace,self.get_ref(),"rustex-phantom",node_top,mt => {
                    mt.attr("stretchy".into(),"false".into());
                    mt.attr("width".into(),dimtohtml(self.width()));
                    mt.attr("height".into(),dimtohtml(self.height()));
                    mt.attr("depth".into(),dimtohtml(self.depth()));
                });
            }
            ColonMode::M => htmlnode!(colon,mtext,self.get_ref(),"",node_top,mt => {
                //let oldwd = colon.state.currsize;
                let mut wd = self.width();
                if wd == 0 {wd = 2048};
                //colon.state.currsize = wd;
                //mt.style("width".into(),dimtohtml(wd));
                htmlnode!(colon,HTML_NS:span,None,"rustex-math-escape",htmlparent!(mt),span => {
                    span.forcefont = true;
                    htmlliteral!(colon,htmlparent!(span),"\n");
                    self.as_html(&ColonMode::H,colon,htmlparent!(span));
                    htmlliteral!(colon,htmlparent!(span),"\n");
                });
                //colon.state.currsize = oldwd;
            }),
            _ => for c in self.children { c.as_html(mode, colon, node_top) }
        }
    }
}

impl VBox {
    fn html_t_inner(self, colon: &mut HTMLColon, node_top: &mut Option<HTMLParent>, withwidth:bool) {
        match (self._height,self._depth) {
            (None,None) => self.html_t_inner_i(colon, node_top, withwidth),
            _ => htmlnode!(colon,div,None,"rustex-vtop-height-container",node_top,inner => {
                let VBoxType::Top(lineht) = self.tp else { unreachable!()};
                if let Some(ht) = self._height {
                    inner.style("margin-top".into(),dimtohtml(ht - lineht));
                    inner.style("bottom".into(),dimtohtml(ht - lineht));
                }
                if let Some(dp) = self._depth {
                    inner.style("height".into(),dimtohtml(dp + lineht));
                }
                if withwidth {inner.style("width".into(),"100%".into())}
                self.html_t_inner_i(colon,htmlparent!(inner),withwidth);
            })
        }
    }
    fn html_t_inner_i(self, colon: &mut HTMLColon, node_top: &mut Option<HTMLParent>, withwidth:bool) {
        htmlnode!(colon,div,self.get_ref(),"rustex-vtop",node_top,node => {
            if withwidth {node.style("width".into(),"100%".into())}
            if crate::INSERT_RUSTEX_ATTRS {
                node.attr("rustex:width".into(),dimtohtml(self.width()));
                node.attr("rustex:height".into(),dimtohtml(self.height()));
                node.attr("rustex:depth".into(),dimtohtml(self.depth()));
            }
            if let Some(ht) = self._to {
                if crate::INSERT_RUSTEX_ATTRS {
                    node.attr("rustex:to".into(),dimtohtml(ht));
                }
                if ht < 0 {
                    node.style("margin-bottom".into(),dimtohtml(ht));
                    node.style("height".into(),"0".into());
                } else {
                    node.style("height".into(),dimtohtml(ht));
                }
                //node.style("min-height".into(),dimtohtml(h));
            }
            //if let Some(d) = self._depth {node.style("margin-bottom".into(),dimtohtml(d))}
            VBox::ch_as_html(self.children,colon,&mut node);
        })
    }
    fn html_v_inner(self, colon: &mut HTMLColon, node_top: &mut Option<HTMLParent>, withwidth:bool) {
        if let Some(ht) = self._height {
            htmlnode!(colon,div,None,"rustex-vbox-height-container",node_top,inner => {
                if ht < 0 {
                    inner.style("margin-bottom".into(),dimtohtml(ht));
                    inner.style("height".into(),"0".into());
                } else {
                    inner.style("height".into(),dimtohtml(ht));
                }
                if withwidth {inner.style("width".into(),"100%".into())}
                self.html_v_inner_i(colon,htmlparent!(inner),withwidth);
            })
        } else { self.html_v_inner_i(colon, node_top, withwidth)}
    }
    fn html_v_inner_i(self, colon: &mut HTMLColon, node_top: &mut Option<HTMLParent>, withwidth:bool) {
        htmlnode!(colon,div,self.get_ref(),"rustex-vbox",node_top,node => {
            if withwidth {node.style("width".into(),"100%".into())}
            if crate::INSERT_RUSTEX_ATTRS {
                node.attr("rustex:width".into(),dimtohtml(self.width()));
                node.attr("rustex:height".into(),dimtohtml(self.height()));
                node.attr("rustex:depth".into(),dimtohtml(self.depth()));
            }
            if let Some(ht) = self._to {
                if crate::INSERT_RUSTEX_ATTRS {
                    node.attr("rustex:to".into(),dimtohtml(ht));
                }
                if ht < 0 {
                    node.style("margin-bottom".into(),dimtohtml(ht));
                    node.style("height".into(),"0".into());
                } else {
                    node.style("height".into(),dimtohtml(ht));
                }
                //node.style("min-height".into(),dimtohtml(h));
            }
            //if let Some(d) = self._depth {node.style("margin-bottom".into(),dimtohtml(d))}
            VBox::ch_as_html(self.children,colon,&mut node);
        })
    }
    fn html_depth(&self) -> i32 {
        let ht = match self._height {
            Some(i) => i,
            None => {
                let mut w = self.spread;
                for c in self.children.iter_wi() { w += c.height() + HEIGHT_CORRECTION }
                w
            }
        };
        self._depth.unwrap() - ht
    }
    fn ch_as_html(mut children:Vec<Whatsit>, colon: &mut HTMLColon, node: &mut HTMLNode) {
        let mut startfil = FilLevel::None;
        let mut endfil = FilLevel::None;
        let mut repush:Vec<Whatsit> = vec!();
        while let Some(head) = children.first() {
            match head {
                Whatsit::Simple(SimpleWI::VFil(_) | SimpleWI::Vss(_)) => {
                    startfil.add(FilLevel::Fil);
                    children.remove(0);
                }
                Whatsit::Simple(SimpleWI::VFill(_)) => {
                    startfil.add(FilLevel::Fill);
                    children.remove(0);
                }
                o if !o.has_ink() => {
                    repush.push(children.remove(0))
                }
                _ => break
            }
        }
        for c in repush.into_iter().rev() {children.insert(0,c)}
        let mut repush:Vec<Whatsit> = vec!();
        while let Some(head) = children.last() {
            match head {
                Whatsit::Simple(SimpleWI::VFil(_) | SimpleWI::Vss(_)) => {
                    endfil.add(FilLevel::Fil);
                    children.pop();
                }
                Whatsit::Simple(SimpleWI::VFill(_)) => {
                    endfil.add(FilLevel::Fill);
                    children.pop();
                }
                o if !o.has_ink() => {
                    repush.push(children.remove(0))
                }
                _ => break
            }
        }
        for c in repush.into_iter() {children.push(c)}
        match (startfil,endfil) {
            (FilLevel::None | FilLevel::Fil,FilLevel::Fill)|(FilLevel::None,FilLevel::Fil) =>
                node.style("justify-content".into(),"start".into()),
            (FilLevel::Fil,FilLevel::Fil)|(FilLevel::Fill,FilLevel::Fill) =>
                node.style("justify-content".into(),"center".into()),
            (FilLevel::Fil|FilLevel::Fill,FilLevel::None)|(FilLevel::Fill,FilLevel::Fil) =>
                node.style("justify-content".into(),"end".into()),
            _ => ()
        }
        for c in children {
            c.as_html(&ColonMode::V,colon,htmlparent!(node))
        }
    }
}