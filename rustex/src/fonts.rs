pub mod fontchars;

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
    pub ligs:HashMap<(u8,u8),u8>,
    pub name:TeXStr,
    pub chartable:Option<Arc<FontTable>>
}
impl PartialEq for FontFile {
    fn eq(&self, other: &Self) -> bool {
        self.name == other.name
    }
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
use std::cell::RefCell;

impl FontFile {
    pub fn new(pb : PathBuf) -> FontFile {
        let name : TeXStr = pb.file_stem().unwrap().to_str().unwrap().into();
        let mut state = FontState {
            ret:fs::read(pb).unwrap(),
            i:0
        };
        state.ret.reverse();

        let tablename : String = name.to_string().chars().map(|x| if !x.is_ascii_digit() {Some(x)} else {None}).flatten().collect();

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
        let mut ligs:HashMap<(u8,u8),u8> = HashMap::new();

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

        size = round_f((size as f32) * read_float(state.borrow_mut())) ;

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
        for i in 2..(np+1) {
            dimen.insert(i,read_float(state.borrow_mut()));
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
                    Some((_,nc ,false,rep)) => {ligs.insert((t.char as u8,*nc as u8),*rep as u8);}
                    _ => ()
                }
                _ => ()
            }
        }
        assert_eq!(state.i as u16,lf);
        let chartable = FONT_TABLES.get(tablename.into());
        /*match chartable {
            None => {
                println!("Here! {}",name)
            }
            _ => ()
        }*/

        FontFile {
            hyphenchar,skewchar,dimen,size,typestr,widths,heights,depths,ics,lps,rps,ligs,name,
            chartable
        }
    }
}

use std::collections::HashMap;
use std::path::PathBuf;
use std::rc::Rc;
use std::sync::{Arc, Mutex, RwLock};
use crate::commands::pgfsvg::get_dimen;
use crate::fonts::fontchars::{FONT_TABLES, FontTable};
use crate::interpreter::dimensions::round_f;
use crate::utils::TeXStr;

pub struct FontInner {
    pub dimen:HashMap<u16,i32>,
    pub hyphenchar:u16,
    pub skewchar:u16,
    pub lps:HashMap<u16,u8>,
    pub rps:HashMap<u16,u8>,
}
impl PartialEq for FontInner {
    fn eq(&self, other: &Self) -> bool {
        self.dimen == other.dimen
    }
}

pub struct Font {
    pub file:Arc<FontFile>,
    pub at:Option<i32>,
    pub inner: RwLock<FontInner>,
    pub name:TeXStr
}
impl PartialEq for Font {
    fn eq(&self, other: &Self) -> bool {
        *self.file == *other.file && self.name == other.name && self.at == other.at && *self.inner.read().unwrap() == *other.inner.read().unwrap()
    }
}

impl Font {
    pub fn get_at(&self) -> i32 {
        match self.at {
            Some(a) => a,
            None => self.file.size as i32
        }
    }
    pub fn new(file:Arc<FontFile>,at:Option<i32>,name:TeXStr) -> Arc<Font> {
        let hc = file.hyphenchar;
        let sc = file.skewchar;
        Arc::new(Font {
            file,at,name,
            inner:RwLock::new(FontInner {
                dimen:HashMap::new(),
                hyphenchar:hc,
                skewchar:sc,
                lps:HashMap::new(),
                rps:HashMap::new()
            })
        })
    }
    pub fn set_dimen(&self,i : u16,vl : i32) {
        self.inner.write().unwrap().dimen.insert(i,vl);
    }
    pub fn get_dimen(&self,i:u16) -> i32 {
        match self.inner.read().unwrap().dimen.get(&i) {
            Some(r) => *r,
            None => match self.file.dimen.get(&i) {
                Some(f) => round_f((*f as f32) * (match self.at {
                    Some(a) => a as f32,
                    None => self.file.size as f32
                })),
                None => 0
            }
        }
    }
    pub fn get_width(&self,i:u16) -> i32 {
        match self.file.widths.get(&i) {
            None => 0,
            Some(f) => round_f((self.get_at() as f32) * (*f as f32))
        }
    }
    pub fn get_height(&self,i:u16) -> i32 {
        match self.file.heights.get(&i) {
            None => 0,
            Some(f) => round_f((self.get_at() as f32) * (*f as f32))
        }
    }
    pub fn get_depth(&self,i:u16) -> i32 {
        match self.file.depths.get(&i) {
            None => 0,
            Some(f) => round_f((self.get_at() as f32) * (*f as f32))
        }
    }
    pub fn get_ic(&self,i:u16) -> i32 {
        match self.file.ics.get(&i) {
            None => 0,
            Some(f) => round_f((self.get_at() as f32) * (*f as f32))
        }
    }
    pub fn get_lp(&self,i:u16) -> i32 {
        match self.inner.read().unwrap().lps.get(&i) {
            None => match self.file.lps.get(&i) {
                None => 0,
                Some(u) => *u as i32
            },
            Some(u) => *u as i32
        }
    }
    pub fn get_rp(&self,i:u16) -> i32 {
        match self.inner.read().unwrap().rps.get(&i) {
            None => match self.file.rps.get(&i) {
                None => 0,
                Some(u) => *u as i32
            },
            Some(u) => *u as i32
        }
    }
    pub fn set_lp(&self,i:u16,v:u8) {
        self.inner.write().unwrap().lps.insert(i,v);
    }
    pub fn set_rp(&self,i:u16,v:u8) {
        self.inner.write().unwrap().rps.insert(i,v);
    }
}

thread_local! {
    pub static Nullfontfile : Arc<FontFile> = Arc::new(FontFile {
        hyphenchar : 45,
        skewchar : 255,
        dimen : HashMap::new(),
        size : 65536,
        typestr : TeXStr::new(&[]),
        widths : HashMap::new(),
        heights : HashMap::new(),
        depths : HashMap::new(),
        ics : HashMap::new(),
        lps : HashMap::new(),
        rps : HashMap::new(),
        ligs : HashMap::new(),
        name : TeXStr::new("Nullfont".as_bytes()),
        chartable:None
    });
    pub static Nullfont : std::sync::Arc<Font> = std::sync::Arc::new(Font {
            file:Nullfontfile.try_with(|x| x.clone()).unwrap(),at:Some(0),
            inner:RwLock::new(FontInner {
                dimen:HashMap::new(),
                hyphenchar:45,
                skewchar:255,lps:HashMap::new(),rps:HashMap::new()
            }),name:"nullfont".into()
    });
}