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

impl PartialEq for CategoryCode {
    fn eq(&self, other: &Self) -> bool {
        matches!(self,other)
    }
}

impl std::fmt::Display for CategoryCode {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        use CategoryCode::*;
        write!(f,"{}",match self {
            Escape => "Escape",
            BeginGroup => "BeginGroup",
            EndGroup => "EndGroup",
            MathShift => "MathShift",
            AlignmentTab => "AlignmentTab",
            EOL => "EOL",
            Parameter => "Parameter",
            Superscript => "Superscript",
            Subscript => "Subscript",
            Ignored => "Ignored",
            Space => "Space",
            Letter => "Letter",
            Other => "Other",
            Active => "Active",
            Comment => "Comment",
            Invalid => "Invalid"
        })
    }
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
use std::fmt::Formatter;

#[derive(Clone)]
pub struct CategoryCodeScheme {
    pub (in crate) catcodes : HashMap<u8,CategoryCode>,// = HashMap::new();
    pub endlinechar: u8,
    pub newlinechar: u8,
    pub escapechar: u8
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
        },
        newlinechar:10,
        endlinechar:13,
        escapechar:255
    };
    pub static ref STARTING_SCHEME : CategoryCodeScheme = CategoryCodeScheme {
        catcodes:{
            let mut map : HashMap<u8,CategoryCode> = HashMap::new();
            map.insert(92,Escape);
            map.insert(32,Space);
            map.insert(13,EOL);
            for i in 65..90 { map.insert(i,Letter); }
            for i in 97..122 { map.insert(i,Letter); }
            map.insert(37,Comment);
            map
        },
        newlinechar:10,
        endlinechar:13,
        escapechar:92
    };
    pub static ref DEFAULT_SCHEME : CategoryCodeScheme = CategoryCodeScheme {
        catcodes:{
            let mut map : HashMap<u8,CategoryCode> = HashMap::new();
            map.insert(92,Escape);
            map.insert(123,BeginGroup);
            map.insert(125,EndGroup);
            map.insert(36,MathShift);
            map.insert(38,AlignmentTab);
            map.insert(35,Parameter);
            map.insert(94,Superscript);
            map.insert(95,Subscript);
            map.insert(32,Space);
            map.insert(13,EOL);
            for i in 65..90 { map.insert(i,Letter); }
            for i in 97..122 { map.insert(i,Letter); }
            map.insert(126,Active);
            map.insert(37,Comment);
            map
        },
        newlinechar:10,
        endlinechar:13,
        escapechar:92
    };
}