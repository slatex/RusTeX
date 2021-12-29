use std::cmp::max;
use crate::Interpreter;
use crate::interpreter::dimensions::Skip;
use crate::stomach::{StomachGroup, Whatsit};
use crate::stomach::whatsits::HasWhatsitIter;
use crate::stomach::groups::WIGroupTrait;
use crate::stomach::simple::SimpleWI;

#[derive(Clone)]
pub struct Paragraph {
    pub parskip:i32,
    pub children:Vec<Whatsit>,
    leftskip:Option<Skip>,
    rightskip:Option<Skip>,
    hsize:Option<i32>,
    pub lineheight:Option<i32>,
    pub _width:i32,
    pub _height:i32,
    pub _depth:i32,
    lines : Option<Vec<(i32,i32)>>
}

impl Paragraph {
    pub fn as_xml_internal(&self,prefix: String) -> String {
        let mut ret = "\n".to_owned() + &prefix + "<paragraph>";
        for c in &self.children { ret += &c.as_xml_internal(prefix.clone() + "  ")}
        ret + "\n" + &prefix + "</paragraph>"
    }
    pub fn split(self,target:i32,int:&Interpreter) -> (Paragraph,Paragraph) {
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
                            unreachable!()
                        }
                    }
                    input.pop();
                }
                Some(sg) => {
                    let next = sg.get_mut().remove(0);
                    match next {
                        Whatsit::Simple(SimpleWI::Mark(_)) => {
                            todo!()
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
                                Whatsit::Char(_,_,_) => max(wi.height(),lineheight),
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
                    _ => unreachable!()
                }
            }
            Some(f) => {
                while match presplit.last() {
                    Some(StomachGroup::Top(_)) => false,
                    _ => true
                } {
                    let last = match presplit.pop().unwrap() {
                        StomachGroup::Other(wg) => wg,
                        _ => unreachable!()
                    };
                    presplit.last_mut().unwrap().push(Whatsit::Grouped(last))
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
                                    unreachable!()
                                }
                            }
                            input.pop();
                        }
                        Some(sg) => {
                            let next = sg.get_mut().remove(0);
                            match next {
                                Whatsit::Simple(SimpleWI::Mark(_)) => {
                                    todo!()
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
                    _ => unreachable!()
                };
                p2.children = sec;
                match presplit.pop() {
                    Some(StomachGroup::Top(v)) => p1.children = v,
                    _ => unreachable!()
                }
            }
        }
        p2.close(int,0,0,vec!());
        (p1,p2)
    }
    pub fn close(&mut self,int:&Interpreter,hangindent:i32,hangafter:usize,parshape:Vec<(i32,i32)>) {
        self.rightskip.get_or_insert(int.state_skip(-(crate::commands::primitives::LEFTSKIP.index as i32)));
        self.leftskip.get_or_insert(int.state_skip(-(crate::commands::primitives::LEFTSKIP.index as i32)));
        self.hsize.get_or_insert(int.state_dimension(-(crate::commands::primitives::HSIZE.index as i32)));
        self.lineheight.get_or_insert(int.state_skip(-(crate::commands::primitives::BASELINESKIP.index as i32)).base);
        self._width = self.hsize.unwrap() - (self.leftskip.unwrap().base  + self.rightskip.unwrap().base);

        self.lines.get_or_insert(if !parshape.is_empty() {
            let mut ilsr : Vec<(i32,i32)> = vec!();
            for (i,l) in parshape {
                ilsr.push((i,l - (self.leftskip.unwrap().base + self.rightskip.unwrap().base)))
            }
            ilsr
        } else if hangindent != 0 && hangafter != 0 {
            todo!()
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
    pub fn new(parskip:i32) -> Paragraph { Paragraph {
        parskip,children:vec!(),
        leftskip:None,rightskip:None,hsize:None,lineheight:None,
        _width:0,_height:0,_depth:0,lines:None
    }}
    pub fn width(&self) -> i32 { self._width }
    pub fn height(&self) -> i32 { self._height }
    pub fn depth(&self) -> i32 { self._depth }
}