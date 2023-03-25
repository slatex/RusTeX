use crate::interpreter::dimensions::dimtostr;
use crate::{htmlliteral, htmlnode, htmlparent, withwidth};
use crate::references::SourceFileReference;
use crate::stomach::colon::ColonMode;
use crate::stomach::html::{dimtohtml, HTML_NS, HTMLChild, HTMLColon, HTMLNode, HTMLParent, HTMLStr};
use crate::stomach::math::{GroupedMath, MathKernel};
use crate::stomach::simple::{Left, Right};
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
            TeXBox::M(GroupedMath(v)) => v,
            TeXBox::DM(GroupedMath(v)) => v,
            TeXBox::LeftRight(l,GroupedMath(mut v),r) => {
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
    fn get_par_width(&self) -> Option<i32> { pass_on!(self,None,get_par_width) }
    fn get_par_widths(&self) -> Vec<i32> { pass_on!(self,vec!(),get_par_widths) }
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

impl HBox {
    pub fn new_trivial(v:Vec<Whatsit>) -> Self {HBox {children:v,spread:0,_depth:None,_height:None,_width:None,rf:None}}
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

    fn normalize(self, mode: &ColonMode, ret: &mut Vec<Whatsit>, scale: Option<f32>) {
        match mode {
            ColonMode::V | ColonMode::External(_) | ColonMode::P => {
                let mut nch : Vec<Whatsit> = vec!();
                for c in self.children { c.normalize(&ColonMode::H,&mut nch,scale) }
                if nch.is_empty() && (self._width.is_none() || self._width == Some(0)) { return () }
                /*else if nch.len() == 1 {
                    match nch.pop() {
                        Some(o) => {
                            nch.push(o)
                        }
                        _ => TeXErr!("Should be unreachable!")
                    }
                }*/
                ret.push(HBox {
                    children: nch,
                    spread: self.spread,
                    _width: self._width,
                    _height: self._height,
                    _depth: self._depth,
                    rf: self.rf
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
                        rf: self.rf
                    }.as_whatsit())
               // }
            }
            ColonMode::M => {
                let mut nch : Vec<Whatsit> = vec!();
                for c in self.children { c.normalize(&ColonMode::H,&mut nch,None) }
                if nch.is_empty() && (self._width.is_none() || self._width == Some(0)) { return () }
                else if nch.len() == 1 {
                    match nch.pop().unwrap() {
                        o@(Whatsit::Char(_)|Whatsit::Grouped(_)) => nch.push(o),
                        o => {
                            nch.push(o)
                        }
                    }
                }
                ret.push(HBox {
                    children: nch,
                    spread: self.spread,
                    _width: self._width,
                    _height: self._height,
                    _depth: self._depth,
                    rf: self.rf
                }.as_whatsit())
            }
        }
    }

    fn as_html(self, mode: &ColonMode, colon: &mut HTMLColon, node_top: &mut Option<HTMLParent>) {
        match mode {
            ColonMode::H | ColonMode::V | ColonMode::P => {
                htmlnode!(colon,div,None,"hboxcontainer",node_top,cont => {
                    if crate::INSERT_RUSTEX_ATTRS {
                        cont.attr("rustex:width".into(),dimtohtml(self.width()));
                        cont.attr("rustex:height".into(),dimtohtml(self.height()));
                    }
                    match self._height {
                        Some(v) => {
                            cont.style("height".into(),dimtohtml(v));
                            cont.style("min-height".into(),dimtohtml(v))
                        }
                        _ => ()
                    }
                    let width = self._width.or(if self.width() == 0 {Some(0)} else {self.get_par_width()});
                    match width {
                        Some(w) => withwidth!(colon,w,cont,inner,{
                            htmlnode!(colon,div,self.get_ref(),"hbox",htmlparent!(inner),node => {
                                //node.style("height".into(),"100%".into());
                                //node.style("min-height".into(),"100%".into());
                                node.style("width".into(),"100%".into());
                                node.style("max-width".into(),"100%".into());
                                HBox::ch_as_html(self.children,colon,&mut node);
                            })
                        }),
                        None => htmlnode!(colon,div,self.get_ref(),"hbox",htmlparent!(cont),node => {
                            //node.style("height".into(),"100%".into());
                            //node.style("min-height".into(),"100%".into());
                            HBox::ch_as_html(self.children,colon,&mut node);
                        })
                    }
                })
                /*
                htmlnode!(colon,div,self.get_ref(),"hbox",htmlparent!(cont),node => {
                    if crate::INSERT_RUSTEX_ATTRS {
                        node.attr("rustex:width".into(),dimtohtml(self.width()));
                        node.attr("rustex:height".into(),dimtohtml(self.height()));
                    }
                    match self._height {
                        Some(v) => {
                            node.style("height".into(),"100%".into());
                            node.style("min-height".into(),"100%".into())
                        }
                        _ => ()
                    }

                    match self._width {
                        Some(h) => {
                            withwidth!(colon,h,node,inner,{
                                HBox::ch_as_html(self.children,colon,&mut inner);
                            })
                        }
                        None if self.width() == 0 => {
                            withwidth!(colon,0,node,inner,{
                                HBox::ch_as_html(self.children,colon,&mut inner);
                            })
                        }
                        _ => {
                            match self.get_par_width() {
                                None => HBox::ch_as_html(self.children,colon,&mut node),
                                Some(w) =>
                                    withwidth!(colon,w,node,inner,{
                                        HBox::ch_as_html(self.children,colon,&mut inner);
                                    })
                            }
                        }
                    }
                })})
                 */
            }
            ColonMode::M => htmlnode!(colon,mtext,self.get_ref(),"",node_top,mt => {
                let currsize = colon.state.currsize;
                colon.state.currsize = self.width();
                mt.style("width".into(),dimtohtml(self.width()));
                htmlnode!(colon,HTML_NS:span,None,"",htmlparent!(mt),span => {
                    span.forcefont = true;
                    htmlliteral!(colon,htmlparent!(span),"\n");
                    self.as_html(&ColonMode::H,colon,htmlparent!(span));
                    htmlliteral!(colon,htmlparent!(span),"\n");
                });
                colon.state.currsize = currsize;
            }),
            _ => for c in self.children { c.as_html(mode,colon,node_top) }
        }
    }
    fn get_ref(&self) -> Option<SourceFileReference> {
        SourceFileReference::from_wi_list(&self.children).or(self.rf.clone())
    }
    fn get_par_width(&self) -> Option<i32> {
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
}
impl HBox {
    fn ch_as_html(children:Vec<Whatsit>, colon: &mut HTMLColon, node: &mut HTMLNode) {
        let mut inspace = false;
        for c in children {
            /* match c {
               Whatsit::Space(_) if !inspace => {
                    inspace = true;
                    htmlliteral!(colon,htmlparent!(node),"&nbsp;")
                }
                Whatsit::Space(_) => {}
                Whatsit::Char(ref pc) => {
                    match pc.font.file.chartable.as_ref().map(|ct| ct.table.get(&pc.char)) {
                        Some(Some(s)) if *s == " " && !inspace => {
                            inspace = true;
                            htmlliteral!(colon,htmlparent!(node),"&nbsp;")
                        }
                        Some(Some(s)) if *s == " " && inspace => {}
                        _ => {
                            inspace = false;
                            c.as_html(&ColonMode::H,colon,htmlparent!(node))
                        }
                    }
                }
                _ => {
                    inspace = false; */
                    c.as_html(&ColonMode::H,colon,htmlparent!(node))
                //}
            //}
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
    pub rf : Option<SourceFileReference>
}

impl WhatsitTrait for VBox {
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
    }
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
        let ht = match self._height {
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
            ColonMode::H | ColonMode::External(_) | ColonMode::P => {
                let mut nch : Vec<Whatsit> = vec!();
                for c in self.children { c.normalize(&ColonMode::V,&mut nch,scale) }
                if nch.is_empty() && (self._height.is_none() || self._height == Some(0)) { return () }
                ret.push(VBox {
                    children: nch,
                    spread: self.spread,
                    _width: self._width,
                    _height: self._height,
                    _depth: self._depth,
                    rf: self.rf,
                    tp:self.tp
                }.as_whatsit())
            }
            ColonMode::V => {
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
                        rf: self.rf,
                        tp:self.tp
                    }.as_whatsit())
                }
            }
            ColonMode::M => {
                use crate::stomach::simple::SimpleWI;
                let mut nch : Vec<Whatsit> = vec!();
                for c in self.children { c.normalize(&ColonMode::V,&mut nch,None) }
                if nch.is_empty() && (self._height.is_none() || self._height == Some(0)) { return () }
                else if nch.len() == 1 {
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
                }
                ret.push(VBox {
                    children: nch,
                    spread: self.spread,
                    _width: self._width,
                    _height: self._height,
                    _depth: self._depth,
                    rf: self.rf,
                    tp:self.tp
                }.as_whatsit())
            }
        }
    }

    fn as_html(self, mode: &ColonMode, colon: &mut HTMLColon, node_top: &mut Option<HTMLParent>) {
        match mode {
            ColonMode::V | ColonMode::H | ColonMode::P if self.tp == VBoxType::DMCenter => {
                htmlnode!(colon,div,None,"displayvbox",node_top,div => {
                    htmlnode!(colon,div,self.get_ref(),"vbox",htmlparent!(div),node => {
                        if crate::INSERT_RUSTEX_ATTRS {
                            node.attr("rustex:width".into(),dimtohtml(self.width()));
                            node.attr("rustex:height".into(),dimtohtml(self.height()));
                        }
                        match self._height {
                            Some(v) => {
                                node.style("height".into(),dimtohtml(v));
                                node.style("min-height".into(),dimtohtml(v))
                            }
                            _ => ()
                        }
                        match self._width {
                            Some(h) => {
                                withwidth!(colon,h,node,inner,{
                                    for c in self.children {
                                        htmlliteral!(colon,htmlparent!(inner),"\n");
                                        c.as_html(&ColonMode::V,colon,htmlparent!(inner));
                                        htmlliteral!(colon,htmlparent!(inner),"\n");
                                    }
                                })
                            }
                            _ => {
                                match self.get_par_width() {
                                    None => for c in self.children {
                                            htmlliteral!(colon,htmlparent!(node),"\n");
                                            c.as_html(&ColonMode::V,colon,htmlparent!(node));
                                            htmlliteral!(colon,htmlparent!(node),"\n");
                                        }
                                    Some(i) => withwidth!(colon,i,node,inner,{
                                        for c in self.children {
                                            htmlliteral!(colon,htmlparent!(inner),"\n");
                                            c.as_html(&ColonMode::V,colon,htmlparent!(inner));
                                            htmlliteral!(colon,htmlparent!(inner),"\n");
                                        }
                                    })
                                }
                            }
                        }
                    });
                });
            }
            ColonMode::V | ColonMode::H | ColonMode::P => {
                htmlnode!(colon,div,None,"vboxcontainer",node_top,cont => {
                    if crate::INSERT_RUSTEX_ATTRS {
                        cont.attr("rustex:width".into(),dimtohtml(self.width()));
                        cont.attr("rustex:height".into(),dimtohtml(self.height()));
                    }
                    match self._height {
                        Some(v) => {
                            cont.style("height".into(),dimtohtml(v));
                            cont.style("min-height".into(),dimtohtml(v));
                        }
                        _ => ()
                    }
                    let width = self._width.or(if self.width() == 0 {Some(0)} else {self.get_par_width()});
                    match width {
                        Some(w) => withwidth!(colon,w,cont,inner,{
                            htmlnode!(colon,div,self.get_ref(),"vbox",htmlparent!(inner),node => {
                                node.style("height".into(),"100%".into());
                                node.style("min-height".into(),"100%".into());
                                node.style("width".into(),"100%".into());
                                node.style("max-width".into(),"100%".into());
                                match self.tp {
                                    VBoxType::V => node.style("vertical-align".into(),"bottom".into()),
                                    VBoxType::Center | VBoxType::DMCenter =>
                                        node.style("vertical-align".into(),"middle".into()),
                                    VBoxType::Top(_) => node.style("vertical-align".into(),"top".into())
                                }
                                for c in self.children {
                                    c.as_html(&ColonMode::V,colon,htmlparent!(node));
                                }
                            })
                        }),
                        None => htmlnode!(colon,div,self.get_ref(),"vbox",htmlparent!(cont),node => {
                            node.style("height".into(),"100%".into());
                            node.style("min-height".into(),"100%".into());
                            match self.tp {
                                VBoxType::V => node.style("vertical-align".into(),"bottom".into()),
                                VBoxType::Center | VBoxType::DMCenter =>
                                    node.style("vertical-align".into(),"middle".into()),
                                VBoxType::Top(_) => node.style("vertical-align".into(),"top".into())
                            }
                            for c in self.children {
                                c.as_html(&ColonMode::V,colon,htmlparent!(node));
                            }
                        })
                    }
                })
            }/*htmlnode!(colon,div,None,"vboxcontainer",node_top,container => {
                htmlnode!(colon,div,self.get_ref(),"vbox",htmlparent!(container),node => {
                if crate::INSERT_RUSTEX_ATTRS {
                    node.attr("rustex:width".into(),dimtohtml(self.width()));
                    node.attr("rustex:height".into(),dimtohtml(self.height()));
                }
                match self.tp {
                    VBoxType::V => node.style("vertical-align".into(),"bottom".into()),
                    VBoxType::Center | VBoxType::DMCenter =>
                        node.style("vertical-align".into(),"middle".into()),
                    VBoxType::Top(_) => node.style("vertical-align".into(),"top".into())
                }
                match self._height {
                    Some(v) => {
                        node.style("height".into(),dimtohtml(v));
                        node.style("min-height".into(),dimtohtml(v))
                    }
                    _ => ()
                }
                match self._width {
                    Some(h) => {
                        withwidth!(colon,h,node,inner,{
                            for c in self.children {
                                htmlliteral!(colon,htmlparent!(inner),"\n");
                                c.as_html(&ColonMode::V,colon,htmlparent!(inner));
                                htmlliteral!(colon,htmlparent!(inner),"\n");
                            }
                        })
                    }
                    _ => {
                        match self.get_par_width() {
                            None => for c in self.children {
                                    htmlliteral!(colon,htmlparent!(node),"\n");
                                    c.as_html(&ColonMode::V,colon,htmlparent!(node));
                                    htmlliteral!(colon,htmlparent!(node),"\n");
                                }
                            Some(i) => withwidth!(colon,i,node,inner,{
                                for c in self.children {
                                    htmlliteral!(colon,htmlparent!(inner),"\n");
                                    c.as_html(&ColonMode::V,colon,htmlparent!(inner));
                                    htmlliteral!(colon,htmlparent!(inner),"\n");
                                }
                            })
                        }
                    }
                }
            })})*/,
            ColonMode::M => htmlnode!(colon,mtext,self.get_ref(),"",node_top,mt => {
                let currsize = colon.state.currsize;
                colon.state.currsize = self.width();
                mt.style("width".into(),dimtohtml(self.width()));
                htmlnode!(colon,HTML_NS:span,None,"",htmlparent!(mt),span => {
                    span.forcefont = true;
                    htmlliteral!(colon,htmlparent!(span),"\n");
                    self.as_html(&ColonMode::H,colon,htmlparent!(span));
                    htmlliteral!(colon,htmlparent!(span),"\n");
                });
                colon.state.currsize = currsize;
            }),
            _ => for c in self.children { c.as_html(mode, colon, node_top) }
        }
    }
}