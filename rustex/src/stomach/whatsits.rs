use std::cmp::{max, min, Ordering};
use crate::interpreter::Interpreter;
use crate::utils::{TeXError, TeXStr};
use std::rc::Rc;
use crate::commands::MathWhatsit;
use crate::fonts::Font;
use crate::interpreter::dimensions::{MuSkip, Skip};
use crate::references::SourceFileReference;
use crate::Token;

#[derive(Copy,Clone,PartialEq)]
pub enum BoxMode { H,V,M,DM,Void }

#[derive(Clone)]
pub struct HBox {
    pub children:Vec<Whatsit>,
    pub spread:i64,
    pub _width:Option<i64>,
    pub _height:Option<i64>,
    pub _depth:Option<i64>,
    pub rf : Option<SourceFileReference>
}

#[derive(Clone)]
pub struct VBox {
    pub children:Vec<Whatsit>,
    pub center:bool,
    pub spread:i64,
    pub _width:Option<i64>,
    pub _height:Option<i64>,
    pub _depth:Option<i64>,
    pub rf : Option<SourceFileReference>
}

#[derive(Clone)]
pub enum TeXBox {
    Void,H(HBox),V(VBox)
}

static WIDTH_CORRECTION : i64 = 0;
static HEIGHT_CORRECTION : i64 = 0;

trait HasWhatsitIter {
    fn iter_wi(&self) -> WhatsitIter;
}

impl HasWhatsitIter for Vec<Whatsit> {
    fn iter_wi(&self) -> WhatsitIter {
        WhatsitIter::new(self)
    }
}

struct WhatsitIter<'a> {
    children:&'a [Whatsit],
    parent:Option<Box<WhatsitIter<'a>>>
}

impl WhatsitIter<'_> {
    pub fn new(v:&Vec<Whatsit>) -> WhatsitIter {
        WhatsitIter {
            children:v.as_slice(),
            parent:None
        }
    }
}

impl <'a> Iterator for WhatsitIter<'a> {
    type Item = &'a Whatsit;
    fn next(&mut self) -> Option<Self::Item> {
        match self.children.get(0) {
            None => match self.parent.take() {
                Some(p) =>{
                    *self = *p;
                    self.next()
                }
                None => None
            }
            Some(Whatsit::Grouped(g)) => {
                self.children = &self.children[1..];
                *self = WhatsitIter {
                    children:g.children().as_slice(),
                    parent:Some(Box::new(std::mem::take(self)))
                };
                self.next()
            }
            Some(s) => {
                self.children = &self.children[1..];
                Some(s)
            }
        }
    }
}
impl<'a> Default for WhatsitIter<'a> {
    fn default() -> Self {
        WhatsitIter { children: &[], parent: None }
    }
}
/*
struct WhatsitIterMut<'a> {
    children:&'a mut Vec<Whatsit>,
    parent:Option<Box<WhatsitIterMut<'a>>>,
    index:usize
}

impl WhatsitIterMut<'_> {
    pub fn new(v:&mut Vec<Whatsit>) -> WhatsitIterMut {
        WhatsitIterMut {
            children:v,
            parent:None,index:0
        }
    }
    pub fn insert(&mut self,wi:Whatsit) {
        self.children.insert(self.index,wi);
        self.index += 1
    }
}

impl <'a> Iterator for WhatsitIterMut<'a> {
    type Item = (&'a Whatsit,&'a mut WhatsitIterMut<'a>);
    fn next(&mut self) -> Option<Self::Item> {
        match self.children.get(self.index) {
            None => match self.parent.take() {
                Some(p) => {
                    *self = *p;
                    self.next()
                }
                None => None
            }
            Some(w@Whatsit::Grouped(mut g)) => {
                self.index += 1;
                *self = WhatsitIterMut {
                    children:&mut g.children(),
                    parent:Some(Box::new(std::mem::take(self))),
                    index:0
                };
                Some((w,self))
            }
            _ => todo!()
        }
    }
}

impl<'a> Default for WhatsitIterMut<'a> {
    fn default() -> Self {
        WhatsitIterMut { children: &mut vec!(), parent: None, index:0 }
    }
}
*/

impl TeXBox {
    fn iter(&self) -> WhatsitIter {
        match self {
            TeXBox::Void => WhatsitIter::default(),
            TeXBox::H(hb) => hb.children.iter_wi(),
            TeXBox::V(vb) => vb.children.iter_wi(),
        }
    }
    pub fn has_ink(&self) -> bool {
        match self {
            TeXBox::Void => false,
            TeXBox::H(hb) => {
                for c in &hb.children { if c.has_ink() { return true } }
                false
            }
            TeXBox::V(vb) => {
                for c in &vb.children { if c.has_ink() { return true } }
                false
            }
        }
    }
    pub fn width(&self) -> i64 {
        match self {
            TeXBox::Void => 0,
            TeXBox::H(hb) => match hb._width {
                Some(i) => i,
                None => {
                    let mut w = hb.spread;
                    for c in self.iter() {
                        w += c.width() + WIDTH_CORRECTION
                    }
                    w
                }
            },
            TeXBox::V(vb) => match vb._width {
                Some(i) => i,
                None => {
                    let mut w = 0;
                    for c in self.iter() {
                        let wd = c.width();
                        if wd > w { w = wd }
                    }
                    w
                }
            },
        }
    }
    pub fn height(&self) -> i64 {
        match self {
            TeXBox::Void => 0,
            TeXBox::H(hb) =>  match hb._height {
                Some(i) => i,
                None => {
                    let mut w = 0;
                    for c in self.iter() {
                        let ht = c.height();
                        if ht > w { w = ht }
                    }
                    w
                }
            },
            TeXBox::V(vb) => {
                let ht = match vb._height {
                    Some(i) => i,
                    None => {
                        let mut w = vb.spread;
                        for c in self.iter() { w += c.height() + HEIGHT_CORRECTION }
                        w
                    }
                };
                if vb.center { ht / 2} else { ht }
            },
        }
    }
    pub fn depth(&self) -> i64 {
        match self {
            TeXBox::Void => 0,
            TeXBox::H(hb) => match hb._depth {
                Some(d) => d,
                None => {
                    let mut d = 0;
                    for c in self.iter() {
                        let dp = c.depth();
                        if dp > d { d = dp }
                    }
                    d
                }
            },
            TeXBox::V(vb) => {
                let dp = match vb._depth {
                    Some(d) => d,
                    None => {
                        match vb.children.last() {
                            None => 0,
                            Some(c) => c.depth()
                        }
                    }
                };
                if vb.center { dp + self.height() } else { dp }
            },
        }
    }
}

#[derive(Clone)]
pub struct MathGroup {
    pub kernel : MathKernel,
    pub superscript : Option<MathKernel>,
    pub subscript : Option<MathKernel>,
    pub limits:bool
}
impl MathGroup {
    pub fn new(kernel:MathKernel,display:bool) -> MathGroup {
        MathGroup {
            kernel,subscript:None,superscript:None,limits:display
        }
    }
    pub fn width(&self) -> i64 {
        self.kernel.width() + match &self.superscript {
            None => 0,
            Some(k) => k.width()
        } + match &self.subscript {
            None => 0,
            Some(k) => k.width()
        }
    }
    pub fn height(&self) -> i64 {
        self.kernel.height() + match &self.superscript {
            None => 0,
            Some(k) => k.height() / 2
        } + match &self.subscript {
            None => 0,
            Some(k) => k.height() / 2
        }
    }
    pub fn depth(&self) -> i64 {
        match &self.subscript {
            Some(s) => max(s.height() / 2,self.kernel.depth()),
            None => self.kernel.depth()
        }
    }
    pub fn has_ink(&self) -> bool {
        self.kernel.has_ink() || match &self.superscript {
            None => false,
            Some(s) => s.has_ink()
        } || match &self.subscript {
            None => false,
            Some(s) => s.has_ink()
        }
    }
}


#[derive(Clone)]
pub enum MathKernel {
    Group(Vec<Whatsit>),
    MathChar(u32,u32,u32,Rc<Font>,Option<SourceFileReference>),
    MKern(MuSkip,Option<SourceFileReference>),
    Delimiter(Box<Whatsit>,Option<SourceFileReference>),
}
impl MathKernel {
    pub fn width(&self) -> i64 {
        use MathKernel::*;
        match self {
            Group(g) => {
                let mut ret = 0;
                for c in g { ret += c.width() }
                ret
            }
            MKern(s,_) => s.base,
            MathChar(_,_,u,f,_) => f.get_width(*u as u16),
            Delimiter(w,_) => w.width()
        }
    }
    pub fn height(&self) -> i64 {
        use MathKernel::*;
        match self {
            Group(g) => {
                let mut ret = 0;
                for c in g {
                    let w = c.height();
                    if w > ret { ret = w }
                }
                ret
            }
            MKern(_,_) => 0,
            MathChar(_,_,u,f,_) => f.get_height(*u as u16),
            Delimiter(w,_) => w.height()
        }
    }
    pub fn depth(&self) -> i64 {
        use MathKernel::*;
        match self {
            Group(g) => {
                let mut ret = 0;
                for c in g {
                    let w = c.depth();
                    if w > ret { ret = w }
                }
                ret
            }
            MKern(_,_) => 0,
            MathChar(_,_,u,f,_) => f.get_depth(*u as u16),
            Delimiter(w,_) => w.depth()
        }
    }
    pub fn has_ink(&self) -> bool {
        use MathKernel::*;
        match self {
            Group(v) => {
                for c in v { if c.has_ink() { return true } }
                false
            }
            MKern(_,_) => false,
            MathChar(_,_,_,_,_) => true,
            Delimiter(w,_) => w.has_ink()
        }
    }
}

#[derive(Clone)]
pub enum Whatsit {
    Exec(Rc<ExecutableWhatsit>),
    Box(TeXBox),
    Ext(Rc<dyn ExtWhatsit>),
    GroupOpen(WIGroup),
    GroupClose(WIGroup),
    Simple(SimpleWI),
    Char(u8,Rc<Font>,Option<SourceFileReference>),
    Math(MathGroup),
    Ls(Vec<Whatsit>),
    Grouped(WIGroup),
    Par(Paragraph)
}

impl Whatsit {
    pub fn has_ink(&self) -> bool {
        use Whatsit::*;
        match self {
            Exec(_) | GroupClose(_) => false,
            Box(b) => b.has_ink(),
            Ext(e) => e.has_ink(),
            GroupOpen(w) => w.has_ink(),
            Grouped(w) => w.has_ink(),
            Simple(s) => s.has_ink(),
            Char(_,_,_) | Par(_) => true,
            Math(m) => m.has_ink(),
            Ls(_) => unreachable!()
        }
    }
    pub fn width(&self) -> i64 {
        use Whatsit::*;
        match self {
            Exec(_) | GroupClose(_) => 0,
            Box(b) => b.width(),
            Ext(e) => e.width(),
            GroupOpen(w) => w.width(),
            Grouped(w) => w.width(),
            Simple(s) => s.width(),
            Char(u,f,_) => f.get_width(*u as u16),
            Math(m) => m.width(),
            Par(p) => p.width(),
            Ls(_) => unreachable!()
        }
    }
    pub fn height(&self) -> i64 {
        use Whatsit::*;
        match self {
            Exec(_) | GroupClose(_) => 0,
            Box(b) => b.height(),
            Ext(e) => e.height(),
            GroupOpen(w) => w.height(),
            Grouped(w) => w.height(),
            Simple(s) => s.height(),
            Char(u,f,_) => f.get_height(*u as u16),
            Math(m) => m.height(),
            Par(p) => p.height(),
            Ls(_) => unreachable!()
        }
    }
    pub fn depth(&self) -> i64 {
        use Whatsit::*;
        match self {
            Exec(_) | GroupClose(_) => 0,
            Box(b) => b.depth(),
            Ext(e) => e.depth(),
            GroupOpen(w) => w.depth(),
            Grouped(w) => w.depth(),
            Simple(s) => s.depth(),
            Char(u,f,_) => f.get_depth(*u as u16),
            Math(m) => m.depth(),
            Par(p) => p.depth(),
            Ls(_) => unreachable!()
        }
    }
}

#[derive(Clone)]
pub enum WIGroup {
    FontChange(Rc<Font>,Option<SourceFileReference>,bool,Vec<Whatsit>),
    ColorChange(TeXStr,Option<SourceFileReference>,Vec<Whatsit>),
    //       rule   attr  action
    PDFLink(TeXStr,TeXStr,ActionSpec,Option<SourceFileReference>,Vec<Whatsit>),
    LinkEnd(Option<SourceFileReference>),
    ColorEnd(Option<SourceFileReference>)
}
impl WIGroup {
    pub fn push(&mut self,wi:Whatsit) {
        use WIGroup::*;
        match self {
            FontChange(_,_,_,v) => v.push(wi),
            ColorChange(_,_,v) => v.push(wi),
            PDFLink(_,_,_,_,v) => v.push(wi),
            ColorEnd(_) | LinkEnd(_) => unreachable!(),
        }
    }
    pub fn priority(&self) -> i16 {
        use WIGroup::*;
        match self {
            FontChange(_,_,true,_) => 25,
            FontChange(_,_,_,_) => 2,
            ColorChange(_,_,_) | ColorEnd(_) => 50,
            PDFLink(_,_,_,_,_) | LinkEnd(_) => 60,
        }
    }
    pub fn has_ink(&self) -> bool {
        use WIGroup::*;
        match self {
            ColorEnd(_) | LinkEnd(_) => false,
            _ => {
                for x in self.children() { if x.has_ink() {return true} }
                false
            }
        }
    }
    pub fn children_d(self) -> Vec<Whatsit> {
        use WIGroup::*;
        match self {
            FontChange(_,_,_,v) => v,
            ColorChange(_,_,v) => v,
            PDFLink(_,_,_,_,v) => v,
            ColorEnd(_) | LinkEnd(_) => unreachable!()
        }
    }
    pub fn children(&self) -> &Vec<Whatsit> {
        use WIGroup::*;
        match self {
            FontChange(_,_,_,v) => v,
            ColorChange(_,_,v) => v,
            PDFLink(_,_,_,_,v) => v,
            ColorEnd(_) | LinkEnd(_) => unreachable!()
        }
    }
    pub fn new_from(&self) -> WIGroup {
        use WIGroup::*;
        match self {
            FontChange(f,r,b,_) => FontChange(f.clone(),r.clone(),*b,vec!()),
            ColorChange(c,r,_) => ColorChange(c.clone(),r.clone(),vec!()),
            PDFLink(a,b,c,d,_) => PDFLink(a.clone(),b.clone(),c.clone(),d.clone(),vec!()),
            ColorEnd(_) | LinkEnd(_) => unreachable!()
        }
    }
    pub fn width(&self) -> i64 {
        use WIGroup::*;
        let c = match self {
            FontChange(_,_,_,c) => c,
            ColorChange(_,_,c) => c,
            PDFLink(_,_,_,_,v) => v,
            ColorEnd(_) | LinkEnd(_) => return 0
        };
        let mut ret : i64 = 0;
        for x in c {
            ret += x.width() + WIDTH_CORRECTION
        }
        ret
    }
    pub fn height(&self) -> i64 { todo!( )}
    pub fn depth(&self) -> i64 { todo!( )}
    pub fn closesWithGroup(&self) -> bool {
        match self {
            WIGroup::FontChange(_,_,b,_) => !*b,
            _ => false
        }
    }
}

#[derive(Clone)]
pub enum ActionSpec {
    User(TeXStr),
    GotoNum(i64),
    //    file   name    window
    File(TeXStr,TeXStr,Option<TeXStr>),
    FilePage(TeXStr,i64,Option<TeXStr>),
    Name(TeXStr),
    Page(i64)
}

#[derive(Clone)]
pub enum AlignBlock {
    Noalign(Vec<Whatsit>),
    Block(Vec<(Vec<Whatsit>,Skip)>)
}

#[derive(Clone)]
pub enum SimpleWI {
    //                                  height       width      depth
    VRule(Option<SourceFileReference>,Option<i64>,Option<i64>,Option<i64>),
    HRule(Option<SourceFileReference>,Option<i64>,Option<i64>,Option<i64>),
    VFil(Option<SourceFileReference>),
    VFill(Option<SourceFileReference>),
    VSkip(Skip,Option<SourceFileReference>),
    HSkip(Skip,Option<SourceFileReference>),
    MSkip(MuSkip,Option<SourceFileReference>),
    HFil(Option<SourceFileReference>),
    HFill(Option<SourceFileReference>),
    Penalty(i64),
    PdfLiteral(TeXStr,Option<SourceFileReference>),
    //          attr            resource
    Pdfxform(Option<TeXStr>,Option<TeXStr>,TeXBox,Option<SourceFileReference>),
    Raise(i64,TeXBox,Option<SourceFileReference>),
    VKern(i64,Option<SourceFileReference>),
    HKern(i64,Option<SourceFileReference>),
    PdfDest(TeXStr,TeXStr,Option<SourceFileReference>),
    Halign(Skip,Vec<(Vec<Token>,Vec<Token>,Skip)>,Vec<AlignBlock>,Option<SourceFileReference>),
    Valign(Skip,Vec<(Vec<Token>,Vec<Token>,Skip)>,Vec<AlignBlock>,Option<SourceFileReference>),
    Hss(Option<SourceFileReference>),
    Vss(Option<SourceFileReference>),
    Indent(i64,Option<SourceFileReference>),
    Mark(Vec<Token>,Option<SourceFileReference>),
    Leaders(Box<Whatsit>,Option<SourceFileReference>)
}
impl SimpleWI {
    pub fn has_ink(&self) -> bool {
        use SimpleWI::*;
        match self {
            VRule(_,_,_,_) | HRule(_,_,_,_)=> true,
            VFil(_) | VFill(_) | VSkip(_,_) | HSkip(_,_) | HFil(_) | HFill(_) | Penalty(_) |
            PdfLiteral(_,_) | Pdfxform(_,_,_,_) | VKern(_,_) | HKern(_,_) | PdfDest(_,_,_)
            | Hss(_) | Vss(_) | Indent(_,_) | MSkip(_,_) | Mark(_,_) => false,
            Raise(_,bx,_) => bx.has_ink(),
            Leaders(w,_) => w.has_ink(),
            Halign(_,_,ab,_) => {
                for v in ab {
                    match v {
                        AlignBlock::Noalign(v) => for c in v { if c.has_ink() {return true} }
                        AlignBlock::Block(v) => for (iv,_) in v { for c in iv { if c.has_ink() {return true} } }
                    }
                }
                false
            }
            Valign(_,_,ab,_) => {
                for v in ab {
                    match v {
                        AlignBlock::Noalign(v) => for c in v { if c.has_ink() {return true} }
                        AlignBlock::Block(v) => for (iv,_) in v { for c in iv { if c.has_ink() {return true} } }
                    }
                }
                false
            }
        }
    }
    pub fn width(&self) -> i64 {
        use SimpleWI::*;
        match self {
            VKern(_,_) | Penalty(_) | VSkip(_,_) | HFill(_) | HFil(_) | VFil(_) | VFill(_)
                | Hss(_) | Vss(_) | PdfDest(_,_,_) | Mark(_,_) => 0,
            HKern(i,_) => *i,
            VRule(_,_,w,_) => w.unwrap_or(26214),
            HSkip(sk,_) => sk.base,
            MSkip(sk,_) => sk.base,
            Indent(i,_) => *i,
            Halign(sk,_,bxs,_) => {
                let mut width:i64 = 0;
                for b in bxs {
                    match b {
                        AlignBlock::Noalign(v) => {
                            let mut max = 0;
                            for c in v.iter_wi() {
                                let w = c.width();
                                if w > max {max = w}
                            }
                            if max > width { width = max }
                        }
                        AlignBlock::Block(ls) => {
                            let mut w:i64 = 0;
                            for (v,s) in ls {
                                w += s.base;
                                for c in v.iter_wi() { w += c.width() }
                            }
                            if w > width { width = w }
                        }
                    }
                }
                width + sk.base
            }
            Valign(_,_,bxs,_) => {
                let mut width:i64 = 0;
                for b in bxs {
                    match b {
                        AlignBlock::Noalign(v) => {
                            for c in v.iter_wi() {
                                width += c.width();
                            }
                        }
                        AlignBlock::Block(ls) => {
                            let mut wd:i64 = 0;
                            for (v,s) in ls {
                                for c in v.iter_wi() {
                                    let w = c.width();
                                    if w > wd { wd = w }
                                }
                            }
                            width += wd
                        }
                    }
                }
                width
            }
            Raise(_,b,_) => b.width(),
            Leaders(b,_) => b.width(), // TODO maybe
            _ => {
                todo!()
            }
        }
    }
    pub fn height(&self) -> i64 {
        use SimpleWI::*;
        match self {
            HKern(_,_) | Penalty(_) | HSkip(_,_) | HFill(_) | HFil(_) | VFil(_) | VFill(_)
                | Hss(_) | Vss(_) | Indent(_,_) | MSkip(_,_) | PdfDest(_,_,_) | Mark(_,_)
                 => 0,
            VRule(_,h,_,_) => h.unwrap_or(0),
            VKern(i,_) => *i,
            Leaders(b,_) => b.height(),
            VSkip(sk,_) => sk.base,
            Halign(_,_,bxs,_) => {
                let mut height:i64 = 0;
                for b in bxs {
                    match b {
                        AlignBlock::Noalign(v) => {
                            for c in v.iter_wi() {
                                height += c.height();
                            }
                        }
                        AlignBlock::Block(ls) => {
                            let mut ht:i64 = 0;
                            for (v,s) in ls {
                                for c in v.iter_wi() {
                                    let h = c.height();
                                    if h > ht { ht = h }
                                }
                            }
                            height += ht
                        }
                    }
                }
                height
            }
            Valign(sk,_,bxs,_) => {
                let mut height:i64 = 0;
                for b in bxs {
                    match b {
                        AlignBlock::Noalign(v) => {
                            let mut max = 0;
                            for c in v.iter_wi() {
                                let w = c.height();
                                if w > max {max = w}
                            }
                            if max > height { height = max }
                        }
                        AlignBlock::Block(ls) => {
                            let mut w:i64 = 0;
                            for (v,s) in ls {
                                w += s.base;
                                for c in v.iter_wi() { w += c.height()}
                            }
                            if w > height { height = w }
                        }
                    }
                }
                height + sk.base
            }
            Raise(r,b,_) => b.height() + r,
            _ => {
                todo!()
            }
        }
    }
    pub fn depth(&self) -> i64 {
        use SimpleWI::*;
        match self {
            HKern(_,_) | VKern(_,_) | Penalty(_) | HSkip(_,_) | VSkip(_,_)
                | HFill(_) | HFil(_) | VFil(_) | VFill(_) | Halign(_,_,_,_) | Valign(_,_,_,_)
                | Hss(_) | Vss(_) | Indent(_,_) | MSkip(_,_) | PdfDest(_,_,_) | Mark(_,_) => 0,
            VRule(_,_,_,d) => d.unwrap_or(0),
            Raise(r,b,_) => max(b.depth() - r,0),
            Leaders(b,_) => b.depth(),
            _ => todo!()
        }
    }
}


pub struct ExecutableWhatsit {
    pub _apply : Box<dyn FnOnce(&Interpreter) -> Result<(),TeXError>>
}

pub trait ExtWhatsit {
    fn name(&self) -> TeXStr;
    fn reference(&self) -> Option<SourceFileReference>;
    fn children(&self) -> Vec<Whatsit>;
    fn isGroup(&self) -> bool;
    fn height(&self) -> i64;
    fn width(&self) -> i64;
    fn depth(&self) -> i64;
    fn has_ink(&self) -> bool;
}

#[derive(Clone)]
pub struct Paragraph {
    pub parskip:i64,
    pub children:Vec<Whatsit>,
    leftskip:Option<i64>,
    rightskip:Option<i64>,
    hsize:Option<i64>,
    lineheight:Option<i64>,
    pub _width:i64,
    pub _height:i64,
    pub _depth:i64
    /*
    if (leftskip == null) leftskip = parser.state.getSkip(PrimitiveRegisters.leftskip.index).getOrElse(Skip(Point(0),None,None))
    if (rightskip == null) rightskip = parser.state.getSkip(PrimitiveRegisters.rightskip.index).getOrElse(Skip(Point(0),None,None))
    if (hsize == null) hsize = parser.state.getDimen(PrimitiveRegisters.hsize.index).getOrElse(Point(0))
    if (hgoal == null) hgoal = hsize + leftskip.base.negate + rightskip.base.negate
    if (lineheight == null) lineheight = parser.state.getSkip(PrimitiveRegisters.baselineskip.index).map(_.base).getOrElse(Point(Point.toSp(13.0)))
     */
}

impl Paragraph {
    pub fn close(&mut self,int:&Interpreter,hangindent:i64,hangafter:usize,parshape:Vec<(i64,i64)>) {
        self.rightskip.get_or_insert(int.state_skip(-(crate::commands::primitives::LEFTSKIP.index as i32)).base);
        self.leftskip.get_or_insert(int.state_skip(-(crate::commands::primitives::LEFTSKIP.index as i32)).base);
        self.hsize.get_or_insert(int.state_dimension(-(crate::commands::primitives::HSIZE.index as i32)));
        self.lineheight.get_or_insert(int.state_skip(-(crate::commands::primitives::BASELINESKIP.index as i32)).base);
        self._width = self.hsize.unwrap() - (self.leftskip.unwrap()  + self.rightskip.unwrap());

        let ils = if !parshape.is_empty() {
            let mut ilsr : Vec<(i64,i64)> = vec!();
            for (i,l) in parshape {
                ilsr.push((i,l - (self.leftskip.unwrap() + self.rightskip.unwrap())))
            }
            ilsr
        } else if hangindent != 0 && hangafter != 0 {
            todo!()
        } else {
            vec!((0,self.hsize.unwrap() - (self.leftskip.unwrap() + self.rightskip.unwrap())))
        };

        let mut currentwidth : i64 = 0;
        let mut currentheight : i64 = 0;
        let mut currentlineheight : i64 = 0;
        let mut currentdepth : i64 = 0;
        let mut currline : usize = 0;
        let mut hgoal = ils.first().unwrap().1;
        let lineheight = self.lineheight.unwrap();
        for wi in self.children.iter_wi() {
            match wi {
                Whatsit::Simple(SimpleWI::Penalty(i)) if *i <= -10000 => {
                    currentwidth = 0;
                    currentheight += currentlineheight;
                    currentlineheight = 0;
                    currentdepth = 0;
                    currline += 1;
                    hgoal = ils.get(currline).unwrap_or(ils.last().unwrap()).1;
                }
                wi => {
                    let width = wi.width();
                    if currentwidth + width > hgoal {
                        currentwidth = 0;
                        currentheight += currentlineheight;
                        currentlineheight = 0;
                        currentdepth = 0;
                        currline += 1;
                        hgoal = ils.get(currline).unwrap_or(ils.last().unwrap()).1;
                    }
                    currentlineheight = max(currentlineheight,match wi {
                        Whatsit::Char(_,_,_) => max(wi.height(),lineheight),
                        _ => wi.height()
                    });
                    currentdepth = max(currentdepth,wi.depth());
                    currentwidth += width
                }
            }
        }
        self._height = currentheight + currentlineheight;
        self._depth = currentdepth;
    }
    pub fn new(parskip:i64) -> Paragraph { Paragraph {
        parskip,children:vec!(),
        leftskip:None,rightskip:None,hsize:None,lineheight:None,
        _width:0,_height:0,_depth:0
    }}
    pub fn width(&self) -> i64 { self._width }
    pub fn height(&self) -> i64 { self._height }
    pub fn depth(&self) -> i64 { self._depth }
}