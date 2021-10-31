#[derive(Copy,Clone)]
pub enum CategoryCode {
    Escape,
    BeginGroup,
    EndGroup,
    MathShift,
    AlignmentTab,
    EOL,
    Parameter,
    Superscript,
    Subscript,
    Ignored,
    Space,
    Letter,
    Other,
    Active,
    Comment,
    Invalid
}

impl CategoryCode {
    pub fn fromint(int : i32) -> CategoryCode {
        use CategoryCode::*;
        match int {
            0 => Escape,
            1 => BeginGroup,
            2 => EndGroup,
            3 => MathShift,
            4 => AlignmentTab,
            5 => EOL,
            6 => Parameter,
            7 => Superscript,
            8 => Subscript,
            9 => Ignored,
            10 => Space,
            11 => Letter,
            12 => Other,
            13 => Active,
            14 => Comment,
            15 => Invalid,
            _ => unreachable!()
        }
    }
    pub fn toint(&self) -> i8 {
        use CategoryCode::*;
        match *self {
            Escape => 0,
            BeginGroup => 1,
            EndGroup => 2,
            MathShift => 3,
            AlignmentTab => 4,
            EOL => 5,
            Parameter => 6,
            Superscript => 7,
            Subscript => 8,
            Ignored => 9,
            Space => 10,
            Letter => 11,
            Other => 12,
            Active => 13,
            Comment => 14,
            Invalid => 15
        }
    }
}

use std::collections::HashMap;
#[derive(Clone)]
pub struct CategoryCodeScheme {
    catcodes : HashMap<u8,CategoryCode>// = HashMap::new();
}
impl CategoryCodeScheme {
    pub fn get_code(&self,c : u8) -> CategoryCode {
        match self.catcodes.get(&c) {
            None => CategoryCode::Other,
            Some(cc) => *cc
        }
    }
}

use CategoryCode::*;

lazy_static! {
    pub static ref OTHER_SCHEME : CategoryCodeScheme = CategoryCodeScheme {
        catcodes:{
            let mut map : HashMap<u8,CategoryCode> = HashMap::new();
            map.insert(32,Space);
            map
        }
    };
    pub static ref DEFAULT_SCHEME : CategoryCodeScheme = CategoryCodeScheme {
        catcodes:{
            let mut map : HashMap<u8,CategoryCode> = HashMap::new();
            map.insert(92,Escape);
            map.insert(123,BeginGroup);
            map.insert(125,EndGroup);
            map.insert(36,MathShift);
            map.insert(38,AlignmentTab);
            map.insert(32,Space);
            for i in 65..90 { map.insert(i,Letter); }
            for i in 97..122 { map.insert(i,Letter); }
            map.insert(126,Active);
            map.insert(37,Comment);
            map
        }
    };
}