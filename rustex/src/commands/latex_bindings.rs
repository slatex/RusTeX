use std::collections::HashMap;
use crate::commands::rustex_specials::HTMLLiteral;
use crate::commands::{PrimitiveExecutable, PrimitiveTeXCommand, ProvidesWhatsit, SimpleWhatsit, TeXCommand};
use crate::stomach::whatsits::WhatsitTrait;
use crate::{TeXErr, TeXString, Token};
use crate::catcodes::CategoryCode;
use crate::commands::primitives::RELAX;
use crate::interpreter::params::CommandListener;
use crate::interpreter::TeXMode;
use crate::stomach::html::HTMLStr;
use crate::stomach::math::{CustomMathChar, GroupedMath, MathGroup, MathKernel};
use crate::stomach::Whatsit;
use crate::utils::TeXStr;

pub static URL: SimpleWhatsit = SimpleWhatsit {
    name: "Url",
    modes: |_| { true },
    _get: |_, int| {
        let tks = int.read_balanced_argument(true,false,false,true)?;
        let mut str : TeXString = "<span style=\"font-family:monospace;\">".into();
        str += HTMLStr::from(int.tokens_to_string(&tks).to_string()).html_escape().to_string();
        str += "</span>";
        let endgroup = Token::new(92,CategoryCode::Escape,Some("endgroup".into()),None,true);
        int.requeue(endgroup);
        Ok(HTMLLiteral { str:str.into() }.as_whatsit())
    },
};

pub struct UrlListener();
impl CommandListener for UrlListener {
    fn apply(&self, name: &TeXStr, _cmd: &Option<TeXCommand>, file: &TeXStr, _line: &String) -> Option<Option<TeXCommand>> {
        if name.to_string() == "Url" && file.to_string().ends_with("url.sty") {
            Some(Some(PrimitiveTeXCommand::Whatsit(ProvidesWhatsit::Simple(&URL)).as_command()))
        } else {
            None
        }
    }
}

pub static NOT: SimpleWhatsit = SimpleWhatsit {
    name: "not",
    modes: |m| {match m {
        TeXMode::Math | TeXMode::Displaymath => true,
        _ => false
    }},
    _get: |tk,int| {
        let pc = match int.read_math_whatsit(None)? {
            Some(Whatsit::Math(MathGroup{ superscript:None,subscript:None,limits:_,kernel:MathKernel::MathChar(mc) })) => (mc.font.clone(),match &mc.font.file.chartable {
                Some(ct) => ct.table.get(&(mc.position as u8)).map(|s| *s).unwrap_or(""),
                _ => ""
            }),
            Some(Whatsit::Math(MathGroup{ superscript:None,subscript:None,limits:_,kernel:MathKernel::Group(GroupedMath(v)) })) => {
                match &v[..] {
                    [Whatsit::Math(MathGroup{ superscript:None,subscript:None,limits:_,kernel:MathKernel::MathChar(mc) })] =>
                        (mc.font.clone(),match &mc.font.file.chartable {
                            Some(ct) => ct.table.get(&(mc.position as u8)).map(|s| *s).unwrap_or(""),
                            _ => ""
                        }),
                    _ => {
                        (int.state.currfont.get(&()),"")
                    }
                }
            },
            _ => (int.state.currfont.get(&()),"")
        };

        Ok(CustomMathChar {
            str: NOTS.get(pc.1).map(|x| x.to_string()).unwrap_or(pc.1.to_string() + "̸"),
            font:pc.0,
            sourceref:int.update_reference(tk)
        }.as_whatsit())
    }
};

lazy_static! {
    pub static ref NOTS : HashMap<&'static str,&'static str> = HashMap::from([
        ("∍","∌"),("∋","∌"),("∈","∉"),("∊","∉"),("∃","∄")
    ]);
}
pub struct NotListener();
impl CommandListener for NotListener {
    fn apply(&self, name: &TeXStr, cmd: &Option<TeXCommand>, file: &TeXStr, _line: &String) -> Option<Option<TeXCommand>> {
        match cmd {
            Some(tc) => match *tc.orig {
                PrimitiveTeXCommand::Primitive(e) if *e == RELAX => None,
                _ => {
                    if name.to_string() == "not" && file.to_string().ends_with("fontmath.ltx") {
                        Some(Some(PrimitiveTeXCommand::Whatsit(ProvidesWhatsit::Simple(&NOT)).as_command()))
                    } else {
                        None
                    }
                }
            }
            _ => None
        }
    }
}
pub static CANCEL: PrimitiveExecutable = PrimitiveExecutable {
    expandable:true,
    name:"c@ncel",
    _apply:|tk, int| {
        let arg = int.read_argument()?;
        for t in arg {
            tk.2.push(t)
        }
        tk.2.push(Token::new(0,CategoryCode::Escape,Some("not".into()),None,true));
        Ok(())
    }
};
pub struct CancelListener();
impl CommandListener for CancelListener {
    fn apply(&self, name: &TeXStr, cmd: &Option<TeXCommand>, file: &TeXStr, _line: &String) -> Option<Option<TeXCommand>> {
        match cmd {
            Some(tc) => match *tc.orig {
                PrimitiveTeXCommand::Primitive(e) if *e == RELAX => None,
                _ => {
                    if name.to_string() == "c@ncel" && file.to_string().ends_with("fontmath.ltx") {
                        Some(Some(PrimitiveTeXCommand::Whatsit(ProvidesWhatsit::Simple(&NOT)).as_command()))
                    } else {
                        None
                    }
                }
            }
            _ => None
        }
    }
}

pub fn all_listeners() -> Vec<Box<dyn CommandListener>> {
    vec!(
        Box::new(UrlListener()),
        Box::new(NotListener()),
        Box::new(CancelListener()),
    )
}

// TODO mapsto, sout, tableofcontents?, underbrace, overbrace, sqrt,