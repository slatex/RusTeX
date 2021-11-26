
struct FInfoEntry {
    char: u16,
    width_index: u8,
    height_index: u8,
    depth_index: u8,
    char_ic_index: u8,
    tag_field: u8,
    remainder: u8
}
impl FInfoEntry {
    pub fn ligature(&self) -> Option<u8> {
        match self.tag_field {
            1 => Some(self.remainder),
            _ => None
        }
    }
    pub fn larger(&self) -> Option<u8> {
        match self.tag_field {
            2 => Some(self.remainder),
            _ => None
        }
    }
    pub fn ext(&self) -> Option<u8> {
        match self.tag_field {
            3 => Some(self.remainder),
            _ => None
        }
    }
}

pub struct FontFile {
    pub hyphenchar:u16,
    pub skewchar:u16,
    pub dimen:HashMap<u16,f32>,
    pub size : i32,
    pub typestr : TeXStr,
    pub widths:HashMap<u16,f32>,
    pub heights: HashMap<u16,f32>,
    pub depths: HashMap<u16,f32>,
    pub ics:HashMap<u16,f32>,
    pub lps:HashMap<u16,u8>,
    pub rps: HashMap<u16,u8>,
    pub ligs:HashMap<(u16,u16),u16>,
    pub name:TeXStr
}
struct FontState {
    pub ret : Vec<u8>,
    pub i : usize
}
impl FontState {
    pub fn pop(&mut self) -> (u8,u8,u8,u8) {
        self.i += 1;
        (self.ret.pop().unwrap(),self.ret.pop().unwrap(),self.ret.pop().unwrap(),self.ret.pop().unwrap())
    }
}
use std::fs;
use std::borrow::BorrowMut;
impl FontFile {
    pub fn new(pb : PathBuf) -> FontFile {
        let name : TeXStr = pb.file_stem().unwrap().to_str().unwrap().into();
        let mut state = FontState {
            ret:fs::read(pb).unwrap(),
            i:0
        };
        state.ret.reverse();

        let mut hyphenchar : u16 = 45;
        let mut skewchar : u16 = 255;
        let mut dimen: HashMap<u16,f32> = HashMap::new();
        let mut size: i32 = 65536;
        let mut typestr: TeXStr = TeXStr::new(&[]);
        let mut widths:HashMap<u16,f32> = HashMap::new();
        let mut heights: HashMap<u16,f32> = HashMap::new();
        let mut depths: HashMap<u16,f32> = HashMap::new();
        let mut ics:HashMap<u16,f32> = HashMap::new();
        let mut lps:HashMap<u16,u8> = HashMap::new();
        let mut rps: HashMap<u16,u8> = HashMap::new();
        let mut ligs:HashMap<(u16,u16),u16> = HashMap::new();

        fn read_int(s : &mut FontState) -> (u16,u16) {
            let (a,b,c,d) = s.pop();
            let i1 = ((a as u16) << 8) | (b as u16);
            let i2 = ((c as u16) << 8) | (d as u16);
            (i1,i2)
        }
        fn read_float(s : &mut FontState) -> f32 {
            let (a,b,c,d) = s.pop();
            let int = ((a as i32) << 24) | ((b as i32) << 16) |
                ((c as i32) << 8) | (d as i32);
            let f = ((int & 0x7fffffff) as f32) / ((1 << 20) as f32);
            if int < 0 {-f} else {f}
        }
        fn skip(s : &mut FontState, len:u8) {
            for _ in 0..len {
                s.pop();
            }
        }
        fn read_fifo(s : &mut FontState,char:u16) -> FInfoEntry {
            let (a,b,c,d) = s.pop();
            let width_index = 0x000000FF & a;
            let (height_index,depth_index) = {
                let byte = 0x000000FF & b;
                let second = byte % 16;
                let first = (byte - second) / 16;
                (first,second)
            };
            let (char_ic_index,tag_field) = {
                let full = 0x000000FF & c;
                let second = full % 4;
                let first = (full - second) / 4;
                (first,second)
            };
            let remainder = 0x000000FF & d;

            FInfoEntry {
                char,width_index,height_index,depth_index,char_ic_index,tag_field,remainder
            }
        }

        let (lf,lh) = read_int(state.borrow_mut());
        let (bc,ec) = read_int(state.borrow_mut());
        let (nw,nh) = read_int(state.borrow_mut());
        let (nd,ni) = read_int(state.borrow_mut());
        let (nl,nk) = read_int(state.borrow_mut());
        let (ne,np) = read_int(state.borrow_mut());
        assert_eq!(lf,6+lh+(ec-bc+1)+nw+nh+nd+ni+nk+nl+ne+np);
        skip(state.borrow_mut(),1);

        size = ((size as f32) * read_float(state.borrow_mut())).round() as i32;

        if lh >= 12 {
            let mut sv : Vec<u8> = vec!();
            let (ln,b,c,d) = state.pop();
            sv.push(b);
            sv.push(c);
            sv.push(d);
            //let ln = (0x000000FF & first) as usize;
            for _ in 0..9 {
                let (a,b,c,d) = state.pop();
                sv.push(a);
                sv.push(b);
                sv.push(c);
                sv.push(d);
            }
            typestr = TeXStr::new(sv.get(0..(ln as usize)).unwrap());
        }
        {
            let i = state.i;
            skip(state.borrow_mut(), ((lh as u8) + 6) - (i as u8));
        }

        let finfo_table : Vec<FInfoEntry> = (bc..(ec+1)).map(|i| read_fifo(state.borrow_mut(),i)).collect();
        assert_eq!(state.i as u16,lh + 6 + (ec-bc+1));

        let widthls : Vec<f32> = (0..nw).map(|_| read_float(state.borrow_mut())).collect();
        let heightls: Vec<f32> = (0..nh).map(|_| read_float(state.borrow_mut())).collect();
        let depthls: Vec<f32> = (0..nd).map(|_| read_float(state.borrow_mut())).collect();
        let italicls: Vec<f32> = (0..ni).map(|_| read_float(state.borrow_mut())).collect();

        let mut ligatures : Vec<(bool,u16,bool,u16)> = vec!();
        for _ in 0..nl {
            let (a,b,c,d) = state.pop();
            let stop = a >= 128;
            let tag = c >= 128;
            ligatures.push((stop,b as u16,tag,d as u16))
        }
        skip(state.borrow_mut(),(nk + ne) as u8);
        {
            let i = state.i;
            assert_eq!(i as u16,lh + 6 + (ec-bc+1) + nw + nh + nd + ni + nl + nk + ne)
        }
        if np > 0 {
            dimen.insert(1,read_float(state.borrow_mut()));
        } else {
            dimen.insert(1,0.0);
        }
        for _ in 2..(np+1) {
            dimen.insert(1,read_float(state.borrow_mut()));
        }

        let factor = match dimen.get(&6) {
            Some(f) => *f as f32,
            None => 1.0
        };

        for t in finfo_table {
            match widthls.get(t.width_index as usize) {
                Some(0.0) | None => (),
                Some(f) => {widths.insert(t.char,factor * f);}
            }
            match heightls.get(t.height_index as usize) {
                Some(0.0) | None => (),
                Some(f) => {heights.insert(t.char,factor * f);}
            }
            match depthls.get(t.depth_index as usize) {
                Some(0.0) | None => (),
                Some(f) => {depths.insert(t.char,factor * f);}
            }
            match italicls.get(t.char_ic_index as usize) {
                Some(0.0) | None => (),
                Some(f) => {ics.insert(t.char,factor * f);}
            }
            match t.ligature() {
                Some(i) => match ligatures.get(i as usize) {
                    Some((_,nc ,false,rep)) => {ligs.insert((t.char,*nc),*rep);}
                    _ => ()
                }
                _ => ()
            }
        }
        assert_eq!(state.i as u16,lf);

        FontFile {
            hyphenchar,skewchar,dimen,size,typestr,widths,heights,depths,ics,lps,rps,ligs,name
        }
    }
}

use std::collections::HashMap;
use std::path::PathBuf;
use std::rc::Rc;
use crate::utils::TeXStr;

#[derive(Clone)]
pub struct Font {
    pub file:Rc<FontFile>,
    pub at:Option<i32>,
    pub dimen:HashMap<u16,i32>,
    pub hyphenchar:u16
}
impl Font {
    pub fn new(file:Rc<FontFile>,at:Option<i32>) -> Font {
        let hc = file.hyphenchar;
        Font {
            file,at,
            dimen:HashMap::new(),
            hyphenchar:hc
        }
    }
    pub fn set_dimen(&mut self,i : u16,vl : i32) {
        self.dimen.insert(i,vl);
    }
    pub fn get_dimen(&self,i:u16) -> i32 {
        match self.dimen.get(&i) {
            Some(r) => *r,
            None => match self.file.dimen.get(&i) {
                Some(f) => (f * (match self.at {
                    Some(a) => a as f32,
                    None => self.file.size as f32
                })).round() as i32,
                None => 0
            }
        }
    }
}