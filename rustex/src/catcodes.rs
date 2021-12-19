#[derive(Copy,Clone,PartialEq)]
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
    pub fn toint(&self) -> u8 {
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
    pub (in crate) catcodes : [CategoryCode;256],// = HashMap::new();
    pub endlinechar: u8,
    pub newlinechar: u8,
    pub escapechar: u8
}
impl CategoryCodeScheme {
    pub fn get_code(&self,c : u8) -> CategoryCode {
        *self.catcodes.get(c as usize).unwrap()
    }
}

use CategoryCode::*;

lazy_static! {
    pub static ref OTHER_SCHEME : CategoryCodeScheme = {
        let mut catcodes = [CategoryCode::Other;256];
        catcodes[32] = CategoryCode::Space;
        CategoryCodeScheme {
            catcodes,
            newlinechar:10,
            endlinechar:13,
            escapechar:92
        }
    };
    pub static ref STARTING_SCHEME : CategoryCodeScheme = {
        let mut catcodes = [CategoryCode::Other;256];
        catcodes[92] = CategoryCode::Escape;
        catcodes[32] = CategoryCode::Space;
        catcodes[13] = CategoryCode::EOL;
        catcodes[37] = CategoryCode::Comment;
        for i in 65..91 { catcodes[i] = CategoryCode::Letter}
        for i in 97..123 { catcodes[i] = CategoryCode::Letter}

        CategoryCodeScheme {
            catcodes,
            newlinechar:10,
            endlinechar:13,
            escapechar:92
        }
    };
    pub static ref DEFAULT_SCHEME : CategoryCodeScheme = {
        let mut catcodes = [CategoryCode::Other;256];
        catcodes[123] = CategoryCode::BeginGroup;
        catcodes[125] = CategoryCode::EndGroup;
        catcodes[36] = CategoryCode::MathShift;
        catcodes[38] = CategoryCode::AlignmentTab;
        catcodes[35] = CategoryCode::Parameter;
        catcodes[94] = CategoryCode::Superscript;
        catcodes[95] = CategoryCode::Subscript;
        catcodes[126] = CategoryCode::Active;
        catcodes[92] = CategoryCode::Escape;
        catcodes[32] = CategoryCode::Space;
        catcodes[13] = CategoryCode::EOL;
        catcodes[37] = CategoryCode::Comment;
        for i in 65..91 { catcodes[i] = CategoryCode::Letter}
        for i in 97..123 { catcodes[i] = CategoryCode::Letter}

        CategoryCodeScheme {
            catcodes,
            newlinechar:10,
            endlinechar:13,
            escapechar:92
        }
    };
}