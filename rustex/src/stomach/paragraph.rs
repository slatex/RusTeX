use std::cmp::max;
use crate::{htmlliteral, htmlnode, htmlparent};
use crate::interpreter::dimensions::{Skip, SkipDim};
use crate::interpreter::state::State;
use crate::references::SourceFileReference;
use crate::stomach::{StomachGroup, Whatsit};
use crate::stomach::colon::ColonMode;
use crate::stomach::whatsits::{HasWhatsitIter, WhatsitTrait};
use crate::stomach::groups::WIGroupTrait;
use crate::stomach::html::{dimtohtml, HTMLChild, HTMLColon, HTMLNode, HTMLParent, HTMLStr};
use crate::stomach::simple::SimpleWI;

#[derive(Clone)]
pub struct Paragraph {
    pub parskip:i32,
    pub children:Vec<Whatsit>,
    pub leftskip:Option<Skip>,
    pub rightskip:Option<Skip>,
    hsize:Option<i32>,
    pub lineheight:Option<i32>,
    pub _width:i32,
    pub _height:i32,
    pub _depth:i32,
    lines : Option<Vec<(i32,i32)>>
}

impl WhatsitTrait for Paragraph {
    fn get_ref(&self) -> Option<SourceFileReference> {
        SourceFileReference::from_wi_list(&self.children)
    }
    fn as_xml_internal(&self,prefix: String) -> String {
        let mut ret = "\n".to_owned() + &prefix + "<paragraph>";
        for c in &self.children { ret += &c.as_xml_internal(prefix.clone() + "  ")}
        ret + "\n" + &prefix + "</paragraph>"
    }
    fn width(&self) -> i32 { self._width }
    fn height(&self) -> i32 { self._height }
    fn depth(&self) -> i32 { self._depth }
    fn has_ink(&self) -> bool { true }
    fn as_whatsit(self) -> Whatsit { Whatsit::Par(self) }
    fn normalize(self, _: &ColonMode, ret: &mut Vec<Whatsit>, scale: Option<f32>) {
        let (mut np, ch) = self.destroy();
        let mut hret: Vec<Whatsit> = vec!();
        for c in ch { c.normalize(&ColonMode::P, &mut hret, scale) }
        np.children = hret;
        ret.push(Whatsit::Par(np))
    }
    fn as_html(mut self, _: &ColonMode, colon: &mut HTMLColon, node_top: &mut Option<HTMLParent>) {
        htmlliteral!(colon,node_top,"\n");
        htmlnode!(colon,div,self.get_ref(),"paragraph",node_top,node => {
            match self.lines.as_ref().unwrap().last() {
                Some((a,b)) if *a != 0 => {
                    self.rightskip = Some(match self.rightskip {
                        Some(sk) => Skip { base: sk.base + self._width - b, stretch:sk.stretch, shrink:sk.shrink},
                        None => Skip { base: self._width - b, stretch:None, shrink:None}
                    });
                    self.leftskip = Some(match self.leftskip {
                        Some(sk) => Skip { base: sk.base + a, stretch:sk.stretch, shrink:sk.shrink},
                        None => Skip { base: *a, stretch:None, shrink:None}
                    });
                    self._width = *b;
                }
                _ => ()
            }
            if crate::INSERT_RUSTEX_ATTRS {
                node.attr("rustex:width".into(),dimtohtml(self.width()));
                node.attr("rustex:height".into(),dimtohtml(self.height()));
            }
            match self.leftskip {
                Some(sk) if match sk.stretch {
                    Some(SkipDim::Fil(_) | SkipDim::Fill(_) | SkipDim::Filll(_)) => true,
                    _ => false
                } => match self.rightskip {
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
                _ => match self.rightskip {
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
            match self.leftskip {
                Some(sk) if sk.base != 0 => node.style("margin-left".into(),dimtohtml(sk.base)),
                _ => ()
            }
            match self.rightskip {
                Some(sk) if sk.base != 0 => node.style("margin-right".into(),dimtohtml(sk.base)),
                _ => ()
            }
            if self.parskip != 0 {
                node.style("margin-top".into(),dimtohtml(self.parskip))
            }
            node.style("width".into(),dimtohtml(self.width()));
            node.style("min-width".into(),dimtohtml(self.width()));
            for c in self.children { c.as_html(&ColonMode::P,colon,htmlparent!(node)) }
        });
        htmlliteral!(colon,node_top,"\n");
    }
}

impl Paragraph {
    pub fn destroy(self) -> (Paragraph,Vec<Whatsit>) {
        let np = Paragraph {
            parskip:self.parskip,
            children:vec!(),
            leftskip:self.leftskip,
            rightskip:self.rightskip,
            hsize:self.hsize,
            lineheight:self.lineheight,
            _width:self._width,
            _height:self._height,
            _depth:self._depth,
            lines:self.lines
        };
        (np,self.children)
    }
    pub fn split(self,target:i32,state:&State) -> (Paragraph,Paragraph) {
        let mut presplit : Vec<StomachGroup> = vec!(StomachGroup::Top(vec!()));
        let mut currentwidth : i32 = 0;
        let mut currentheight : i32 = 0;
        let mut currentlineheight : i32 = 0;
        let mut currentdepth : i32 = 0;
        let mut currline : usize = 0;
        let lineheight = self.lineheight.unwrap();
        let mut input : Vec<StomachGroup> = vec!(StomachGroup::Top(self.children));
        let lines = self.lines.as_ref().unwrap();
        let mut hgoal = lines.first().unwrap().1;

        let first = loop {
            match input.last_mut() {
                None => break None,
                Some(StomachGroup::Top(sg)) if sg.is_empty() => {
                    input.pop();
                    break None
                },
                Some(sg) if sg.get().is_empty() => {
                    let pop = presplit.pop();
                    match pop {
                        Some(StomachGroup::Other(wg)) if wg.children().is_empty() => (),
                        Some(StomachGroup::Other(wg)) => presplit.last_mut().unwrap().push(Whatsit::Grouped(wg)),
                        _ => {
                            // TeXErr!("Should be unreachable!")
                        }
                    }
                    input.pop();
                }
                Some(sg) => {
                    let next = sg.get_mut().remove(0);
                    match next {
                        Whatsit::Simple(SimpleWI::Mark(_)) => {
                            //TeXErr!("TODO")
                        },
                        Whatsit::Grouped(wg) => {
                            presplit.push(StomachGroup::Other(wg.new_from()));
                            input.push(StomachGroup::Other(wg))
                        },
                        Whatsit::Simple(SimpleWI::Penalty(ref p)) if p.penalty <= -10000 => {
                            if currentheight + currentlineheight + lineheight > target {
                                break Some(next)
                            }
                            currentwidth = 0;
                            currentheight += currentlineheight;
                            currentlineheight = 0;
                            currentdepth = 0;
                            currline += 1;
                            hgoal = lines.get(currline).unwrap_or(lines.last().unwrap()).1;
                        }
                        wi => {
                            let width = wi.width();
                            if currentwidth + width > hgoal {
                                if currentheight + currentlineheight + lineheight > target {
                                    break Some(wi)
                                }
                                currentwidth = 0;
                                currentheight += currentlineheight;
                                currentlineheight = 0;
                                currentdepth = 0;
                                currline += 1;
                                hgoal = lines.get(currline).unwrap_or(lines.last().unwrap()).1;
                            }
                            currentlineheight = max(currentlineheight,match wi {
                                Whatsit::Char(_) => max(wi.height(),lineheight),
                                _ => wi.height()
                            });
                            currentdepth = max(currentdepth,wi.depth());
                            currentwidth += width;
                            presplit.last_mut().unwrap().push(wi)
                        }
                    }
                }
            }
        };
        let mut p1 = Paragraph::new(self.parskip);
        let mut p2 = Paragraph::new(self.parskip);
        p1.lineheight = Some(lineheight);
        p1.lines = Some(lines.clone());
        p1.leftskip = self.leftskip;
        p1.rightskip = self.rightskip;
        p1._depth = currentdepth;
        p1._width = self._width;
        p1._depth = currentdepth;
        p1._height = target;
        p1.hsize = self.hsize;
        p2.lineheight = Some(lineheight);
        p2.leftskip = self.leftskip;
        p2.rightskip = self.rightskip;
        p2._width = self._width;
        p2.hsize = self.hsize;
        match first {
            None => {
                assert!(input.is_empty());
                match presplit.pop() {
                    Some(StomachGroup::Top(v)) => p1.children = v,
                    _ => ()//TeXErr!("Should be unreachable!")
                }
            }
            Some(_) => {
                while match presplit.last() {
                    Some(StomachGroup::Top(_)) => false,
                    _ => true
                } {
                    match presplit.pop().unwrap() {
                        StomachGroup::Other(wg) =>
                            presplit.last_mut().unwrap().push(Whatsit::Grouped(wg)),
                        _ => () //TeXErr!("Should be unreachable!")
                    };
                }
                let mut second : Vec<StomachGroup> = vec!(StomachGroup::Top(vec!()));
                for g in &input[1..] {
                    second.push(g.new_from())
                }
                loop {
                    match input.last_mut() {
                        None => break,
                        Some(StomachGroup::Top(sg)) if sg.is_empty() => {
                            input.pop();
                            break
                        },
                        Some(sg) if sg.get().is_empty() => {
                            let pop = second.pop();
                            match pop {
                                Some(StomachGroup::Other(wg)) if wg.children().is_empty() => (),
                                Some(StomachGroup::Other(wg)) => second.last_mut().unwrap().push(Whatsit::Grouped(wg)),
                                _ => {
                                    ()//TeXErr!("Should be unreachable!")
                                }
                            }
                            input.pop();
                        }
                        Some(sg) => {
                            let next = sg.get_mut().remove(0);
                            match next {
                                Whatsit::Simple(SimpleWI::Mark(_)) => {
                                    ()//TeXErr!("TODO")
                                },
                                next => {
                                    second.last_mut().unwrap().push(next)
                                }
                            }
                        }
                    }
                }
                let sec = match second.pop() {
                    Some(StomachGroup::Top(v)) => v,
                    _ => unreachable!()//TeXErr!("Should be unreachable!")
                };
                p2.children = sec;
                match presplit.pop() {
                    Some(StomachGroup::Top(v)) => p1.children = v,
                    _ => ()//TeXErr!("Should be unreachable!")
                }
            }
        }
        p2.close(state,0,0,vec!());
        (p1,p2)
    }
    pub fn close(&mut self,state:&State,hangindent:i32,hangafter:usize,parshape:Vec<(i32,i32)>) {
        self.rightskip.get_or_insert(state.skips.get(&-(crate::commands::primitives::LEFTSKIP.index as i32)));
        self.leftskip.get_or_insert(state.skips.get(&-(crate::commands::primitives::LEFTSKIP.index as i32)));
        self.hsize.get_or_insert(state.dimensions.get(&-(crate::commands::primitives::HSIZE.index as i32)));
        self.lineheight.get_or_insert(state.skips.get(&-(crate::commands::primitives::BASELINESKIP.index as i32)).base);
        self._width = self.hsize.unwrap() - (self.leftskip.unwrap().base  + self.rightskip.unwrap().base);

        self.lines.get_or_insert(if !parshape.is_empty() {
            let mut ilsr : Vec<(i32,i32)> = vec!();
            for (i,l) in parshape {
                ilsr.push((i,l - (self.leftskip.unwrap().base + self.rightskip.unwrap().base)))
            }
            ilsr
        } else if hangindent != 0 && hangafter != 0 {
            vec!((0,self.hsize.unwrap() - (self.leftskip.unwrap().base + self.rightskip.unwrap().base)))
            //TeXErr!("TODO")
        } else {
            vec!((0,self.hsize.unwrap() - (self.leftskip.unwrap().base + self.rightskip.unwrap().base)))
        });
        let lines = self.lines.as_ref().unwrap();

        let mut currentwidth : i32 = 0;
        let mut currentheight : i32 = 0;
        let mut currentlineheight : i32 = 0;
        let mut currentdepth : i32 = 0;
        let mut currline : usize = 0;
        let mut hgoal = lines.first().unwrap().1;
        let lineheight = self.lineheight.unwrap();
        for wi in self.children.iter_wi() {
            match wi {
                Whatsit::Simple(SimpleWI::Penalty(p)) if p.penalty <= -10000 => {
                    currentwidth = 0;
                    currentheight += currentlineheight;
                    currentlineheight = 0;
                    currentdepth = 0;
                    currline += 1;
                    hgoal = lines.get(currline).unwrap_or(lines.last().unwrap()).1;
                }
                wi => {
                    let width = wi.width();
                    if currentwidth + width > hgoal {
                        currentwidth = 0;
                        currentheight += currentlineheight;
                        currentlineheight = 0;
                        currentdepth = 0;
                        currline += 1;
                        hgoal = lines.get(currline).unwrap_or(lines.last().unwrap()).1;
                    }
                    currentlineheight = max(currentlineheight,match wi {
                        Whatsit::Char(_) => max(wi.height(),lineheight),
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
    pub fn new(parskip:i32) -> Paragraph { Paragraph {
        parskip,children:vec!(),
        leftskip:None,rightskip:None,hsize:None,lineheight:None,
        _width:0,_height:0,_depth:0,lines:None
    }}
}