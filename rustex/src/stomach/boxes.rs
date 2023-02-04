use crate::interpreter::dimensions::dimtostr;
use crate::{htmlliteral, htmlnode, htmlparent, withwidth};
use crate::references::SourceFileReference;
use crate::stomach::colon::ColonMode;
use crate::stomach::html::{dimtohtml, HTML_NS, HTMLChild, HTMLColon, HTMLNode, HTMLParent, HTMLStr};
use crate::stomach::Whatsit;
use crate::stomach::whatsits::{HasWhatsitIter, HEIGHT_CORRECTION, WhatsitTrait, WIDTH_CORRECTION};

#[derive(Copy,Clone,PartialEq)]
pub enum VBoxType { V, Center, Top(i32) }

#[derive(Copy,Clone,PartialEq)]
pub enum BoxMode { H,V,M,DM,Void }

#[derive(Clone)]
pub enum TeXBox {
    Void,H(HBox),V(VBox)
}

impl TeXBox {
    /*fn pass_on<A>(&self, f: Box<dyn FnOnce(&dyn WhatsitTrait) -> A>, void: A) -> A {
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
    }*/
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
    fn get_ref(&self) -> Option<SourceFileReference> { pass_on!(self,None,get_ref) }
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
    fn get_ref(&self) -> Option<SourceFileReference> {
        SourceFileReference::from_wi_list(&self.children).or(self.rf.clone())
    }
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
                if self._width.is_none() && self._height.is_none() && self._depth.is_none() {
                    for c in self.children { c.normalize(&ColonMode::H,ret,scale) }
                } else {
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
                }
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
                htmlnode!(colon,div,self.get_ref(),"hbox",node_top,node => {
                    if crate::INSERT_RUSTEX_ATTRS {
                        node.attr("rustex:width".into(),dimtohtml(self.width()));
                        node.attr("rustex:height".into(),dimtohtml(self.height()));
                    }
                    //htmlliteral!(colon,node_top,"\n");
                    /*match self._width {
                        Some(h) => {
                            node.style("width".into(),dimtohtml(h));
                            node.style("min-width".into(),dimtohtml(h))
                        }
                        _ => ()
                    }*/
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
                                HBox::ch_as_html(self.children,colon,&mut inner);
                            })
                        }
                        None if self.width() == 0 => {
                            withwidth!(colon,0,node,inner,{
                                HBox::ch_as_html(self.children,colon,&mut inner);
                            })
                        }
                        _ => {
                            //let currsquare = colon.state.squaresize;
                            //colon.state.squaresize = true;
                            HBox::ch_as_html(self.children,colon,&mut node);
                            //colon.state.squaresize = currsquare;
                        }
                    }

                    //htmlliteral!(colon,node_top,"\n");
                })
            }
            ColonMode::M => htmlnode!(colon,mtext,self.get_ref(),"",node_top,mt => {
                htmlnode!(colon,HTML_NS:span,None,"",htmlparent!(mt),span => {
                    htmlliteral!(colon,htmlparent!(span),"\n");
                    self.as_html(&ColonMode::H,colon,htmlparent!(span));
                    htmlliteral!(colon,htmlparent!(span),"\n");
                })
            }),
            _ => for c in self.children { c.as_html(mode,colon,node_top) }
        }
    }
}
impl HBox {
    fn ch_as_html(children:Vec<Whatsit>, colon: &mut HTMLColon, node: &mut HTMLNode) {
        let mut inspace = false;
        for c in children {
            match c {
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
                    inspace = false;
                    c.as_html(&ColonMode::H,colon,htmlparent!(node))
                }
            }
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
            VBoxType::Center => ht / 2,
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
            VBoxType::Center => dp + self.height(),
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
                /*else if nch.len() == 1 {
                    match nch.pop() {
                        Some(o) => {
                            nch.push(o)
                        }
                        _ => TeXErr!("Should be unreachable!")
                    }
                }*/
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
            ColonMode::V | ColonMode::H | ColonMode::P => htmlnode!(colon,div,self.get_ref(),"vbox",node_top,node => {
                if crate::INSERT_RUSTEX_ATTRS {
                    node.attr("rustex:width".into(),dimtohtml(self.width()));
                    node.attr("rustex:height".into(),dimtohtml(self.height()));
                }
                match self.tp {
                    VBoxType::V => node.style("vertical-align".into(),"baseline".into()),
                    VBoxType::Center => node.style("vertical-align".into(),"middle".into()),
                    VBoxType::Top(_) => node.style("vertical-align".into(),"top".into())
                }
                match self._height {
                    Some(v) => {
                        node.style("height".into(),dimtohtml(v));
                        node.style("min-height".into(),dimtohtml(v))
                    }
                    _ => ()
                }
                /* match self._width {
                    Some(v) => {
                        node.style("width".into(),dimtohtml(v));
                        node.style("min-width".into(),dimtohtml(v))
                    }
                    _ => ()
                } */
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

                        //let currsquare = colon.state.squaresize;
                        //colon.state.squaresize = true;
                        for c in self.children {
                            htmlliteral!(colon,htmlparent!(node),"\n");
                            c.as_html(&ColonMode::V,colon,htmlparent!(node));
                            htmlliteral!(colon,htmlparent!(node),"\n");
                        }
                        //colon.state.squaresize = currsquare;
                    }
                }
            }),
            ColonMode::M => htmlnode!(colon,mtext,self.get_ref(),"",node_top,mt => {
                htmlnode!(colon,HTML_NS:span,None,"",htmlparent!(mt),span => {
                    htmlliteral!(colon,htmlparent!(span),"\n");
                    self.as_html(&ColonMode::H,colon,htmlparent!(span));
                    htmlliteral!(colon,htmlparent!(span),"\n");
                })
            }),
            _ => for c in self.children { c.as_html(mode,colon,node_top) }
        }
    }
}