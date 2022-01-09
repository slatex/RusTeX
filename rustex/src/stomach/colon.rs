use std::sync::Arc;
use std::sync::mpsc::Receiver;
use std::thread::JoinHandle;
use crate::fonts::Font;
use crate::Interpreter;
use crate::interpreter::dimensions::Skip;
use crate::interpreter::state::State;
use crate::stomach::{StomachMessage, Whatsit};
use crate::stomach::boxes::{HBox, VBox};
use crate::stomach::groups::{WIGroup, WIGroupTrait};
use crate::stomach::groups::WIGroup::PDFMatrixSave;
use crate::stomach::math::{GroupedMath, MathGroup, MathKernel};
use crate::stomach::simple::AlignBlock;
use crate::stomach::whatsits::{Insert, WhatsitTrait};
use crate::utils::TeXStr;

#[derive(PartialEq,Clone)]
pub enum ColonMode { V,H,M,External(TeXStr) }

pub struct ColonBase {
    pub basefont:Option<Arc<Font>>,
    pub basecolor:Option<TeXStr>,
    pub receiver:Option<Receiver<StomachMessage>>
}
impl ColonBase {
    pub fn new() -> ColonBase {
        ColonBase {
            basefont:None,
            basecolor:None,
            receiver:None
        }
    }
}

pub trait Colon<A> : Send {
    fn base_mut(&mut self) -> &mut ColonBase;
    fn base(&self) -> &ColonBase;
    fn ship_whatsit(&mut self, wi:Whatsit);
    fn close(&mut self) -> A;

    fn normalize_whatsit(&self, wi:Whatsit) -> Vec<Whatsit> {
        let mut top : Vec<Whatsit> = vec!();
        wi.normalize(&ColonMode::V,&mut top,None);
        top
    }
    fn initialize(&mut self,basefont:Arc<Font>,basecolor:TeXStr,_:&Interpreter) {
        let base = self.base_mut();
        base.basefont = Some(basefont);
        base.basecolor = Some(basecolor)
    }
}

// -------------------------------------------------------------------------------------------------
/*
pub fn normalize_v(w : Whatsit, ret:&mut Vec<Whatsit>, scale:Option<f32>) {
    use Whatsit::*;
    use crate::stomach::simple::SimpleWI::*;
    use crate::stomach::boxes::TeXBox::*;
    match w {
        Simple(Penalty(_)) | Simple(Mark(_)) => (),
        Box(V(mut vb)) if vb._height.is_none() => {
            for c in vb.children { normalize_v(c, ret, scale) }
        }
        Box(V(vb)) => {
            let mut nch : Vec<Whatsit> = vec!();
            for c in vb.children { normalize_v(c, &mut nch, scale) }
            if !nch.is_empty() || vb._height != Some(0) {
                ret.push(VBox {
                    children: nch,
                    spread: vb.spread,
                    _width: vb._width,
                    _height: vb._height,
                    _depth: vb._depth,
                    rf: vb.rf,
                    tp:vb.tp
                }.as_whatsit())
            }
        }
        Simple(VSkip(sk)) if sk.skip.base == 0 => (),
        Simple(VKern(k)) if k.dim == 0 => (),
        Simple(VSkip(sk)) => {
            match ret.last_mut() {
                Some(Simple(VSkip(sk2))) => {
                    sk2.skip = sk.skip + sk2.skip;
                    if sk2.skip.base == 0 {
                        ret.pop();
                    }
                },
                _ => ret.push(Simple(VSkip(sk)))
            }
        }
        Grouped(mut wg) => {
            let mut ng = wg.new_from();
            if wg.opaque() {
                for c in wg.children_prim() { normalize_v(c, ng.children_mut(), scale) }
                ret.push(Grouped(ng));
                return ()
            }
            let mut in_ink = false;
            for c in wg.children_prim() {
                if c.has_ink() { in_ink = true }
                if in_ink { normalize_v(c, ng.children_mut(), scale) } else { normalize_v(c, ret, scale) }
            }
            let mut nret : Vec<Whatsit> = vec!();
            loop {
                match ng.children_mut().pop() {
                    None => break,
                    Some(w) if !w.has_ink() => nret.push(w),
                    Some(o) => {
                        ng.children_mut().push(o);
                        break
                    }
                }
            }
            if !ng.children().is_empty() {
                ret.push(Grouped(ng))
            }
            nret.reverse();
            ret.append(&mut nret);
        }
        Simple(HRule(v)) if v.width() == 0 && v.height() + v.depth() == 0 => (),
        Simple(HAlign(ha)) => {
            let mut nrows : Vec<AlignBlock> = vec!();
            for block in ha.rows {
                match block {
                    AlignBlock::Noalign(v) => {
                        let mut na : Vec<Whatsit> = vec!();
                        for w in v { normalize_v(w, &mut na, scale)}
                        if !na.is_empty() { nrows.push(AlignBlock::Noalign(na))}
                    }
                    AlignBlock::Block(vv) => {
                        let mut nb : Vec<(Vec<Whatsit>,Skip,usize)> = vec!();
                        for (v,sk,num) in vv {
                            let mut nv : Vec<Whatsit> = vec!();
                            for w in v { normalize_h(w,&mut nv,scale) }
                            nb.push((nv,sk,num))
                        }
                        nrows.push(AlignBlock::Block(nb))
                    }
                }
            }
            ret.push(Simple(HAlign(crate::stomach::simple::HAlign {
                skip:ha.skip,
                template:ha.template,
                rows:nrows,
                sourceref:ha.sourceref
            })))
        }
        Inserts(is) => {
            let mut iret : Vec<Vec<Whatsit>> = vec!();
            for v in is.0 {
                let mut iiret : Vec<Whatsit> = vec!();
                for w in v { normalize_v(w, &mut iiret, scale) }
                if !iiret.is_empty() {iret.push(iiret)}
            }
            if !iret.is_empty() { ret.push(Inserts(Insert(iret)))}
        }
        Simple(MoveRight(crate::stomach::simple::MoveRight { dim:0,content:bx, sourceref:_})) => normalize_v(Box(bx), ret, scale),
        Simple(MoveRight(crate::stomach::simple::MoveRight { dim,content:bx, sourceref})) => {
            let mut nch: Vec<Whatsit> = vec!();
            match bx {
                V(vb) => {
                    for c in vb.children { normalize_v(c, &mut nch, scale) }
                    if !nch.is_empty() || vb._width != Some(0) {
                        ret.push(Simple(MoveRight(crate::stomach::simple::MoveRight{
                            content: V(VBox {
                                children: nch,
                                spread: vb.spread,
                                _width: vb._width,
                                _height: vb._height,
                                _depth: vb._depth,
                                rf: vb.rf,
                                tp:vb.tp
                            }),
                            dim,sourceref
                        })))
                    }
                }
                H(hb) => {
                    for c in hb.children { normalize_h(c, &mut nch,scale) }
                    if !nch.is_empty() || hb._width != Some(0) {
                        ret.push(Simple(MoveRight(crate::stomach::simple::MoveRight{
                            content: H(HBox {
                                children: nch,
                                spread: hb.spread,
                                _width: hb._width,
                                _height: hb._height,
                                _depth: hb._depth,
                                rf: hb.rf
                            }),
                            dim,sourceref
                        })))
                    }
                }
                _ => ()
            }
        }
        Float(V(vb)) => {
            let mut nch: Vec<Whatsit> = vec!();
            for c in vb.children { normalize_v(c, &mut nch, scale) }
            if !nch.is_empty() || vb._width != Some(0) {
                ret.push(Float(V(VBox {
                    children: nch,
                    spread: vb.spread,
                    _width: vb._width,
                    _height: vb._height,
                    _depth: vb._depth,
                    rf: vb.rf,
                    tp: vb.tp
                })))
            }
        },
        Simple(VFil(_) | VFill(_) | PDFDest(_) | HRule(_) | Vss(_) | External(_) | VKern(_)) => ret.push(w),
        Simple(VKern(_)) => ret.push(w),
        _ => {
            ret.push(w)
        }
    }
}

pub fn normalize_h(w:Whatsit,ret:&mut Vec<Whatsit>,scale:Option<f32>) {
    use Whatsit::*;
    use crate::stomach::simple::SimpleWI::*;
    use crate::stomach::boxes::TeXBox::*;
    use crate::stomach::math::MathKernel::*;
    match w {
        Simple(Penalty(p)) if p.penalty > -10000 => (),
        Simple(HSkip(sk)) if sk.skip.base == 0 => (),
        Simple(HKern(k)) if k.dim == 0 => (),
        Simple(Mark(_)) => (),
        Simple(HSkip(sk)) => {
            match ret.last_mut() {
                Some(Simple(HSkip(sk2))) => {
                    sk2.skip = sk.skip + sk2.skip;
                    if sk2.skip.base == 0 {
                        ret.pop();
                    }
                },
                _ => ret.push(Simple(HSkip(sk)))
            }
        }
        Grouped(PDFMatrixSave(mut sg)) => match sg.children.iter().filter(|x| match x {
            Simple(PDFMatrix(_)) => true,
            _ => false
        }).next() {
            Some(Simple(PDFMatrix(g))) => {
                let mut ng = sg.new_from();
                let matrix = g.clone();
                let nch : Vec<Whatsit> = sg.children.drain(..).filter(|x| match x {
                    Simple(PDFMatrix(_)) => false,
                    _ => true
                }).collect();
                if matrix.scale == matrix.skewy && matrix.rotate == 0.0 && matrix.skewx == 0.0 {
                    for c in nch { normalize_h(c,ret,Some(matrix.scale)) }
                } else {
                    ng.children.push(matrix.as_whatsit());
                    for c in nch { normalize_h(c, &mut ng.children,scale) }
                    ret.push(Grouped(PDFMatrixSave(ng)))
                }
            }
            _ => for c in sg.children { normalize_h(c,ret,scale) }
        }
        Grouped(mut wg) => {
            let mut ng = wg.new_from();
            if wg.opaque() {
                for c in wg.children_prim() { normalize_h(c,ng.children_mut(),scale) }
                ret.push(Grouped(ng));
                return ()
            }
            let mut in_ink = false;
            for c in wg.children_prim() {
                if c.has_ink() { in_ink = true }
                if in_ink { normalize_h(c, ng.children_mut(),scale) } else { normalize_h(c, ret,scale) }
            }
            let mut nret : Vec<Whatsit> = vec!();
            loop {
                match ng.children_mut().pop() {
                    None => break,
                    Some(w) if !w.has_ink() => nret.push(w),
                    Some(o) => {
                        ng.children_mut().push(o);
                        break
                    }
                }
            }
            if !ng.children().is_empty() {
                ret.push(Grouped(ng))
            }
            nret.reverse();
            ret.append(&mut nret);
        }
        Simple(VRule(v)) if v.width() == 0 && v.height() + v.depth() == 0 => (),
        Math(MathGroup {kernel:Group(GroupedMath(mut g)),subscript:None,superscript:None,limits}) if g.len() == 1 && match g.last() {
            Some(Grouped(/*WIGroup::ColorChange(_) |*/ WIGroup::FontChange(_) /*| WIGroup::PDFLink(_)*/)) => true,
            _ => false
        } => {
            match g.pop() {
                Some(Grouped(mut gr)) => {
                    let mut ngr = gr.new_from();
                    ngr.push(Math(MathGroup { kernel: Group(GroupedMath(gr.children_prim())), subscript: None, superscript: None, limits }));
                    return normalize_h(Grouped(ngr), ret, scale)
                }
                _ => unreachable!()
            }
        }
        Math(MathGroup {kernel:Group(GroupedMath(mut g)),subscript:None,superscript:None,limits:_}) if g.len() == 1 && match g.last() {
            Some(Box(_) | Math(_) | Simple(HKern(_) | HAlign(_))) => true,
            | Some(Grouped(_)) => false,
            _ => false
        } => {
            match g.pop() {
                Some(w) => normalize_h(w,ret,scale),
                _ => unreachable!()
            }
        }
        Math(mut mg) =>  {
            let superscript = normalize_kernel(mg.superscript.take());
            let subscript = normalize_kernel(mg.superscript.take());
            let nbody = match normalize_kernel(Some(mg.kernel)) {
                None if subscript.is_none() && superscript.is_none() => return (),
                None => Group(GroupedMath(vec!())),
                Some(Group(GroupedMath(mut v))) if v.len()==1 && subscript.is_none() && superscript.is_none() => {
                    match v.pop() {
                        Some(Math(mg)) => return normalize_h(mg.as_whatsit(),ret,scale),
                        Some(Box(tb)) => return normalize_h(Box(tb),ret,scale),
                        _ => Group(GroupedMath(v))
                    }
                }
                Some(k) => k
            };
            ret.push(Math(MathGroup { kernel:nbody,subscript,superscript,limits:mg.limits }))
        }
        Box(V(vb)) => {
            let mut nch: Vec<Whatsit> = vec!();
            for c in vb.children { normalize_v(c, &mut nch, scale) }
            if !nch.is_empty() || vb._width != Some(0) {
                ret.push(VBox {
                    children: nch,
                    spread: vb.spread,
                    _width: vb._width,
                    _height: vb._height,
                    _depth: vb._depth,
                    rf: vb.rf,
                    tp:vb.tp
                }.as_whatsit())
            }
        }
        Simple(Raise(crate::stomach::simple::Raise { dim:0,content:bx, sourceref:_})) => normalize_h(Box(bx),ret,scale),
        Simple(Raise(crate::stomach::simple::Raise { dim,content:bx, sourceref})) => {
            let mut nch: Vec<Whatsit> = vec!();
            match bx {
                V(vb) => {
                    for c in vb.children { normalize_v(c, &mut nch, scale) }
                    if !nch.is_empty() || vb._width != Some(0) {
                        ret.push(Simple(Raise(crate::stomach::simple::Raise{
                            content: V(VBox {
                                children: nch,
                                spread: vb.spread,
                                _width: vb._width,
                                _height: vb._height,
                                _depth: vb._depth,
                                rf: vb.rf,
                                tp:vb.tp
                            }),
                            dim,sourceref
                        })))
                    }
                }
                H(hb) => {
                    for c in hb.children { normalize_h(c, &mut nch,scale) }
                    if !nch.is_empty() || hb._width != Some(0) {
                        ret.push(Simple(Raise(crate::stomach::simple::Raise{
                            content: H(HBox {
                                children: nch,
                                spread: hb.spread,
                                _width: hb._width,
                                _height: hb._height,
                                _depth: hb._depth,
                                rf: hb.rf
                            }),
                            dim,sourceref
                        })))
                    }
                }
                _ => ()
            }
        }
        Simple(PDFXImage(mut img)) if scale.is_some() => {
            img._width = Some(((img.width() as f32) * scale.unwrap()).round() as i32);
            img._height = Some(((img.height() as f32) * scale.unwrap()).round() as i32);
            ret.push(Simple(PDFXImage(img)))
        }
        Simple(HFil(_)) | Simple(PDFDest(_)) | Char(_) | Space(_) | Simple(VRule(_)) | Simple(Indent(_))
            | Simple(Hss(_)) | Simple(HKern(_)) | Simple(HFill(_)) | Simple(PDFMatrix(_))
            | Simple(PDFXImage(_)) => ret.push(w),
        Simple(Leaders(_)) => ret.push(w), // TODO maybe?
        Simple(External(_)) => ret.push(w), // TODO maybe?
        Simple(VSkip(_)) => (), // TODO investigate: this shouldn't happen
        Simple(HAlign(_)) => normalize_v(w, ret, scale), _ => {
            ret.push(w)
        }
    }
}


macro_rules! singleton {
    ($math:ident,$content:ident,$sourceref:ident,$single:ident => $ret:expr) => ({
        let mut nret : Vec<Whatsit> = vec!();
        normalize_m(*$content,&mut nret);
        let nw = match nret.len() {
            1 => {
                let $single = nret.pop().unwrap();
                $ret
            },
            _ => GroupedMath(nret).as_whatsit()
        };
        Some($math(crate::stomach::math::$math {content:std::boxed::Box::new(nw),$sourceref}))
    })
}

pub fn normalize_kernel(k : Option<MathKernel>) -> Option<MathKernel> {
    use Whatsit::*;
    use crate::stomach::simple::SimpleWI::*;
    use crate::stomach::boxes::TeXBox::*;
    use crate::stomach::math::MathKernel::*;
    match k {
        None => None,
        Some(Group(wis)) => {
            let mut nret : Vec<Whatsit> = vec!();
            for w in wis.0 { normalize_m(w,&mut nret) }
            if nret.is_empty() { return None }
            Some(Group(GroupedMath(nret)))
        }
        Some(Overline(crate::stomach::math::Overline {content, sourceref})) =>
            singleton!(Overline,content,sourceref,s => {
            s
        }),
        Some(Underline(crate::stomach::math::Underline {content, sourceref})) =>
            singleton!(Underline,content,sourceref,s => {
            s
        }),
        Some(MathInner(crate::stomach::math::MathInner {content, sourceref})) =>
            singleton!(MathInner,content,sourceref,s => {
            s
        }),
        Some(MathOp(crate::stomach::math::MathOp {content, sourceref})) =>
            singleton!(MathOp,content,sourceref,s => {
            s
        }),
        Some(MathOpen(crate::stomach::math::MathOpen {content, sourceref})) =>
            singleton!(MathOpen,content,sourceref,s => {
            s
        }),
        Some(MathClose(crate::stomach::math::MathClose {content, sourceref})) =>
            singleton!(MathOp,content,sourceref,s => {
            s
        }),
        Some(MathPunct(crate::stomach::math::MathPunct {content, sourceref})) =>
            singleton!(MathPunct,content,sourceref,s => {
            s
        }),
        Some(MathRel(crate::stomach::math::MathRel {content, sourceref})) =>
            singleton!(MathRel,content,sourceref,s => {
            s
        }),
        Some(MathBin(crate::stomach::math::MathBin {content, sourceref})) =>
            singleton!(MathBin,content,sourceref,s => {
            s
        }),
        Some(MathAccent(crate::stomach::math::MathAccent {content,accent,sourceref})) => {
            let mut nret : Vec<Whatsit> = vec!();
            normalize_m(*content,&mut nret);
            let nw = match nret.len() {
                1 => {
                    let s = nret.pop().unwrap();
                    s
                },
                _ => GroupedMath(nret).as_whatsit()
            };
            Some(MathAccent(crate::stomach::math::MathAccent {content:std::boxed::Box::new(nw),accent,sourceref}))
        }
        Some(MKern(k)) if k.sk.base == 0 => None,
        Some(o@(MathChar(_)|Delimiter(_)|MKern(_))) => Some(o),
        o => {
            o
        }
    }
}

pub fn normalize_m(w:Whatsit,ret:&mut Vec<Whatsit>) {
    use Whatsit::*;
    use crate::stomach::simple::SimpleWI::*;
    use crate::stomach::boxes::TeXBox::*;
    use crate::stomach::math::MathKernel::*;
    match w {
        Simple(MSkip(sk)) if sk.skip.base == 0 => (),
        Box(V(vb)) => {
            let mut nch : Vec<Whatsit> = vec!();
            for c in vb.children { normalize_v(c, &mut nch, None) }
            if !nch.is_empty() || vb._width != Some(0) {
                ret.push(VBox {
                    children: nch,
                    spread: vb.spread,
                    _width: vb._width,
                    _height: vb._height,
                    _depth: vb._depth,
                    rf: vb.rf,
                    tp:vb.tp
                }.as_whatsit())
            }
        }
        Math(mut mg) =>  {
            let superscript = normalize_kernel(mg.superscript.take());
            let subscript = normalize_kernel(mg.superscript.take());
            let nbody = match normalize_kernel(Some(mg.kernel)) {
                None if subscript.is_none() && superscript.is_none() => return (),
                None => Group(GroupedMath(vec!())),
                Some(Group(GroupedMath(mut v))) if v.len()==1 && subscript.is_none() && superscript.is_none() => {
                    match v.pop() {
                        Some(Math(mg)) => return normalize_m(Math(mg),ret),
                        _ => Group(GroupedMath(v))
                    }
                }
                Some(k) => k
            };
            ret.push(Math(MathGroup { kernel:nbody,subscript,superscript,limits:mg.limits }))
        }
        Grouped(mut wg) => {
            let mut ng = wg.new_from();
            if wg.opaque() {
                for c in wg.children_prim() { normalize_m(c,ng.children_mut()) }
                ret.push(Grouped(ng));
                return ()
            }
            let mut in_ink = false;
            for c in wg.children_prim() {
                if c.has_ink() { in_ink = true }
                if in_ink { normalize_m(c, ng.children_mut()) } else { normalize_m(c, ret) }
            }
            let mut nret : Vec<Whatsit> = vec!();
            loop {
                match ng.children_mut().pop() {
                    None => break,
                    Some(w) if !w.has_ink() => nret.push(w),
                    Some(o) => {
                        ng.children_mut().push(o);
                        break
                    }
                }
            }
            if !ng.children().is_empty() {
                ret.push(Grouped(ng))
            }
            nret.reverse();
            ret.append(&mut nret);
        }
        Simple(MSkip(_)) | Simple(Left(_)) | Simple(Middle(_)) | Simple(Right(_)) | Simple(HSkip(_)) | Simple(HKern(_)) => ret.push(w),
        _ => {
            ret.push(w)
        }
    }
}

 */

// -------------------------------------------------------------------------------------------------

pub struct NoColon {
    pub base:ColonBase
}
impl NoColon {
    pub fn new() -> NoColon {
        NoColon { base: ColonBase::new()}
    }
}
unsafe impl Send for NoColon {}

impl Colon<()> for NoColon {
    fn base(&self) -> &ColonBase { &self.base }
    fn base_mut(&mut self) -> &mut ColonBase { &mut self.base }
    fn ship_whatsit(&mut self, _:Whatsit) {}
    fn close(&mut self) -> () {}
}

// -------------------------------------------------------------------------------------------------

pub struct XMLColon {
    pub base:ColonBase,
    ret : String
}
impl XMLColon {
    pub fn new() -> XMLColon { XMLColon {
        base: ColonBase::new(),
        ret:"<doc>\n".to_string()
    }}
}
unsafe impl Send for XMLColon {}
impl Colon<String> for XMLColon {
    fn base(&self) -> &ColonBase { &self.base }
    fn base_mut(&mut self) -> &mut ColonBase { &mut self.base }
    fn ship_whatsit(&mut self, w:Whatsit) { self.ret += &w.as_xml_internal("  ".to_string()) }
    fn close(&mut self) -> String { std::mem::take(&mut self.ret) + "\n</doc>"}
}