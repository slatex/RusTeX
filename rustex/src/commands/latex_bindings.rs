use std::collections::HashMap;
use std::sync::Arc;
use crate::commands::rustex_specials::{AnnotateBegin, HTMLLiteral};
use crate::commands::{ParamToken, PrimitiveExecutable, PrimitiveTeXCommand, ProvidesWhatsit, SimpleWhatsit, TeXCommand};
use crate::stomach::whatsits::WhatsitTrait;
use crate::{interpreter, TeXErr, TeXString, Token};
use crate::catcodes::CategoryCode;
use crate::commands::primitives::RELAX;
use crate::interpreter::params::CommandListener;
use crate::interpreter::{string_to_tokens, TeXMode};
use crate::interpreter::state::State;
use crate::stomach::boxes::TeXBox;
use crate::stomach::groups::WIGroup;
use crate::stomach::html::HTMLStr;
use crate::stomach::math::{CustomMathChar, GroupedMath, MathGroup, MathKernel, MathOp};
use crate::stomach::Whatsit;
use crate::stomach::Whatsit::Math;
use crate::utils::TeXStr;

pub static URL: SimpleWhatsit = SimpleWhatsit {
    name: "Url",
    modes: |_| { true },
    _get: |_, int| {
        let tks = int.read_balanced_argument(true,false,false,true)?;
        let mut str : TeXString = "<span class=\"contents monospaced\">".into();
        str += HTMLStr::from(int.tokens_to_string(&tks).to_string()).html_escape().to_string();
        str += "</span>";
        let endgroup = Token::new(92,CategoryCode::Escape,Some("endgroup".into()),None,true);
        int.requeue(endgroup);
        Ok(HTMLLiteral { str:str.into() }.as_whatsit())
    },
};

pub struct UrlListener();
impl CommandListener for UrlListener {
    fn apply(&self, name: &TeXStr, _cmd: &Option<TeXCommand>, file: &TeXStr, _line: &String,_:&mut State) -> Option<Option<TeXCommand>> {
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
        let pc = match int.read_math_whatsit()? {
            Some(Whatsit::Math(MathGroup{ superscript:None,subscript:None,limits:_,kernel:MathKernel::MathChar(mc) })) => (mc.font.clone(),match &mc.font.file.chartable {
                Some(ct) => ct.table.get(&(mc.position as u8)).map(|s| *s).unwrap_or(""),
                _ => ""
            }),
            Some(Whatsit::Math(MathGroup{ superscript:None,subscript:None,limits:_,kernel:MathKernel::Group(GroupedMath(v,_)) })) => {
                match &v[..] {
                    [Whatsit::Math(MathGroup{ superscript:None,subscript:None,limits:_,kernel:MathKernel::MathChar(mc) })] =>
                        (mc.font.clone(),match &mc.font.file.chartable {
                            Some(ct) => ct.table.get(&(mc.position as u8)).map(|s| *s).unwrap_or(""),
                            _ => ""
                        }),
                    _ => {
                        (int.state.currfont.get(),"")
                    }
                }
            },
            _ => (int.state.currfont.get(),"")
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
        ("∍","∌"),("∋","∌"),("∈","∉"),("∊","∉"),("∃","∄"),("=","≠")
    ]);
}
pub struct NotListener();
impl CommandListener for NotListener {
    fn apply(&self, name: &TeXStr, cmd: &Option<TeXCommand>, file: &TeXStr, _line: &String,_:&mut State) -> Option<Option<TeXCommand>> {
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
    fn apply(&self, name: &TeXStr, cmd: &Option<TeXCommand>, file: &TeXStr, _line: &String,_:&mut State) -> Option<Option<TeXCommand>> {
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


pub static MAPSTO: SimpleWhatsit = SimpleWhatsit {
    name: "mapsto ",
    modes: |m| {match m {
        TeXMode::Math | TeXMode::Displaymath => true,
        _ => false
    }},
    _get: |tk,int| {
        Ok(CustomMathChar {
            str: "↦".to_string(),
            font:int.state.currfont.get(),
            sourceref:int.update_reference(tk)
        }.as_whatsit())
    }
};

pub struct MapstoListener();
impl CommandListener for MapstoListener {
    fn apply(&self, name: &TeXStr, cmd: &Option<TeXCommand>, file: &TeXStr, _line: &String,_:&mut State) -> Option<Option<TeXCommand>> {
        match cmd {
            Some(tc) => match *tc.orig {
                PrimitiveTeXCommand::Primitive(e) if *e == RELAX => None,
                _ => {
                    if name.to_string() == "mapsto " && file.to_string().ends_with("fontmath.ltx") {
                        Some(Some(PrimitiveTeXCommand::Whatsit(ProvidesWhatsit::Simple(&MAPSTO)).as_command()))
                    } else if name.to_string() == "mapsto" && file.to_string().ends_with("amsmath.sty") {
                        Some(Some(PrimitiveTeXCommand::Whatsit(ProvidesWhatsit::Simple(&MAPSTO)).as_command()))
                    } else {
                        None
                    }
                }
            }
            _ => None
        }
    }
}


pub static UNDERBRACE: SimpleWhatsit = SimpleWhatsit {
    name: "underbrace",
    modes: |m| {match m {
        TeXMode::Math | TeXMode::Displaymath => true,
        _ => false
    }},
    _get: |tk,int| {
        let first = match int.read_math_whatsit()? {
            None => GroupedMath(vec!(),false).as_whatsit(),
            Some(s) => s
        };
        let kernel = MathKernel::MathOp(MathOp {
            sourceref:None,
            content:Box::new(
                MathGroup {
                    kernel:MathKernel::MathOp(MathOp {
                        content:Box::new(first),
                        sourceref: None,
                    }),
                    superscript: None,
                    subscript: Some(MathKernel::CustomMath(CustomMathChar {
                        str: "⏟".to_string(),
                        font:int.state.currfont.get(),
                        sourceref: None,
                    })),
                    limits: true,
                }.as_whatsit()
            )
        });
        Ok(MathGroup {
            kernel,superscript:None,subscript:None,limits:true
        }.as_whatsit())
    }
};

pub struct UnderbraceListener();
impl CommandListener for UnderbraceListener {
    fn apply(&self, name: &TeXStr, cmd: &Option<TeXCommand>, file: &TeXStr, _line: &String,_:&mut State) -> Option<Option<TeXCommand>> {
        match cmd {
            Some(tc) => match *tc.orig {
                PrimitiveTeXCommand::Primitive(e) if *e == RELAX => None,
                _ => {
                    if name.to_string() == "underbrace" {
                        Some(Some(PrimitiveTeXCommand::Whatsit(ProvidesWhatsit::Simple(&UNDERBRACE)).as_command()))
                    } else {
                        None
                    }
                }
            }
            _ => None
        }
    }
}

pub static OVERBRACE: SimpleWhatsit = SimpleWhatsit {
    name: "overbrace",
    modes: |m| {match m {
        TeXMode::Math | TeXMode::Displaymath => true,
        _ => false
    }},
    _get: |tk,int| {
        let first = match int.read_math_whatsit()? {
            None => GroupedMath(vec!(),false).as_whatsit(),
            Some(s) => s
        };
        let kernel = MathKernel::MathOp(MathOp {
            sourceref:None,
            content:Box::new(
                MathGroup {
                    kernel:MathKernel::MathOp(MathOp {
                        content:Box::new(first),
                        sourceref: None,
                    }),
                    subscript: None,
                    superscript: Some(MathKernel::CustomMath(CustomMathChar {
                        str: "⏞".to_string(),
                        font:int.state.currfont.get(),
                        sourceref: None,
                    })),
                    limits: true,
                }.as_whatsit()
            )
        });
        Ok(MathGroup {
            kernel,superscript:None,subscript:None,limits:true
        }.as_whatsit())
    }
};

pub struct OverbraceListener();
impl CommandListener for OverbraceListener {
    fn apply(&self, name: &TeXStr, cmd: &Option<TeXCommand>, file: &TeXStr, _line: &String,_:&mut State) -> Option<Option<TeXCommand>> {
        match cmd {
            Some(tc) => match *tc.orig {
                PrimitiveTeXCommand::Primitive(e) if *e == RELAX => None,
                _ => {
                    if name.to_string() == "overbrace" {
                        Some(Some(PrimitiveTeXCommand::Whatsit(ProvidesWhatsit::Simple(&OVERBRACE)).as_command()))
                    } else {
                        None
                    }
                }
            }
            _ => None
        }
    }
}

pub struct MarginParListener();
impl CommandListener for MarginParListener {
    fn apply(&self, name: &TeXStr, cmd: &Option<TeXCommand>, file: &TeXStr, line: &String,state:&mut State) -> Option<Option<TeXCommand>> {
        match (name,file) {
            (n,f) if n.to_string() == "marginpar" && f.to_string().ends_with("latex.ltx") => {
                let tksP = vec!(
                    Token::cs("@ifnextchar"),
                    Token::with_cat(91,CategoryCode::Other),
                    Token::cs("rustex!marginpar!bracket"),Token::cs("rustex!marginpar!nobracket")
                );
                let marginpar = self.def_cmd(tksP,true,false,vec!());

                let tksBr = vec!(Token::cs("rustex!marginpar!nobracket"));
                let sigBr = vec!(
                    ParamToken::Token(Token::with_cat(91,CategoryCode::Other)),
                    ParamToken::Param(1),
                    ParamToken::Token(Token::with_cat(93,CategoryCode::Other)),
                );
                let bracket = self.def_cmd(tksBr,false,false,sigBr);
                state.commands.set("rustex!marginpar!bracket".into(),Some(bracket),true);

                let sigNoBr = vec!(ParamToken::Param(1));
                let tksNoBr = vec!(
                    /* \par\begingroup\setbox1\hbox{#1}\ht1=0pt\setbox1\hbox{\kern\dimexpr-\wd1-8pt\relax\box1}\ht1=0pt\relax\box1\endgroup */
                    Token::cs("par"),
                    Token::cs("begingroup"),
                    Token::cs("setbox"),
                    Token::with_cat(49,CategoryCode::Other), // 1
                    Token::cs("hbox"),
                    Token::with_cat(123,CategoryCode::BeginGroup), // {
                    Token::with_cat(35,CategoryCode::Parameter), // #
                    Token::with_cat(49,CategoryCode::Other), // 1
                    Token::with_cat(125,CategoryCode::EndGroup), // }
                    Token::cs("ht"),
                    Token::with_cat(49,CategoryCode::Other), // 1
                    Token::with_cat(61,CategoryCode::Other), // =
                    Token::with_cat(48,CategoryCode::Other), // 0
                    Token::with_cat(112,CategoryCode::Letter), // p
                    Token::with_cat(116,CategoryCode::Letter), // t
                    Token::cs("setbox"),
                    Token::with_cat(49,CategoryCode::Other), // 1
                    Token::cs("hbox"),
                    Token::with_cat(123,CategoryCode::BeginGroup), // {
                    Token::cs("kern"),
                    Token::cs("dimexpr"),
                    Token::with_cat(45,CategoryCode::Other), // -
                    Token::cs("wd"),
                    Token::with_cat(49,CategoryCode::Other), // 1
                    Token::with_cat(45,CategoryCode::Other), // -
                    Token::with_cat(56,CategoryCode::Other), // 8
                    Token::with_cat(112,CategoryCode::Letter), // p
                    Token::with_cat(116,CategoryCode::Letter), // t
                    Token::cs("box"),
                    Token::with_cat(49,CategoryCode::Other), // 1
                    Token::with_cat(125,CategoryCode::EndGroup), // }
                    Token::cs("ht"),
                    Token::with_cat(49,CategoryCode::Other), // 1
                    Token::with_cat(61,CategoryCode::Other), // =
                    Token::with_cat(48,CategoryCode::Other), // 0
                    Token::with_cat(112,CategoryCode::Letter), // p
                    Token::with_cat(116,CategoryCode::Letter), // t
                    Token::cs("box"),
                    Token::with_cat(49,CategoryCode::Other), // 1
                    Token::cs("endgroup")
                );
                let nobracket = self.def_cmd(tksNoBr,false,true,sigNoBr);
                state.commands.set("rustex!marginpar!nobracket".into(),Some(nobracket),true);
                Some(Some(marginpar))
            }
            _ => None
        }
    }
}

pub static WRAPFIG: SimpleWhatsit = SimpleWhatsit {
    name: "WF@wraphand",
    modes: (|m| m == TeXMode::Horizontal),
    _get: |tk,int| {
        use crate::commands::rustex_specials::Sized;
        let place = match int.state.commands.get(&"WF@place".into()) {
            Some(tc) => {
                match &*tc.orig {
                    PrimitiveTeXCommand::Def(dm) =>
                        interpreter::tokens_to_string(&dm.ret,int.state.catcodes.get_scheme()).0.last().map(|c| c.clone()).unwrap_or(114),
                    _ => 114
                }
            }
            _ => 114 // 'r'
        };
        let boxnum = match int.state.commands.get(&"WF@box".into()) {
            Some(tc) => {
                match &*tc.orig {
                    PrimitiveTeXCommand::Char(tk) =>
                        tk.char,
                    _ => 0
                }
            }
            _ => 0 // 'r'
        };
        match int.state.boxes.take(boxnum as u16) {
            TeXBox::V(vb) => {
                let mut ret = AnnotateBegin { sourceref: int.update_reference(tk), attrs: HashMap::new(), styles: HashMap::new(), classes: vec!(), block: false, sized: Sized::None };
                match place {
                    114 => ret.styles.insert("float".into(),"right".into()),
                    _ => ret.styles.insert("float".into(),"left".into())
                };
                ret.styles.insert("margin".into(),"10px".into());
                ret.styles.insert("display".into(),"inline-block".into());
                Ok(Whatsit::Grouped(WIGroup::External(Arc::new(ret), vec!(vb.as_whatsit()))))
            }
            _ => Ok(Whatsit::Box(TeXBox::Void))
        }
    }
};

pub struct WrapfigListener();
impl CommandListener for WrapfigListener {
    fn apply(&self, name: &TeXStr, cmd: &Option<TeXCommand>, file: &TeXStr, line: &String, state: &mut State) -> Option<Option<TeXCommand>> {
        match (name, file) {
            (n, f) if n.to_string() == "WFclear" && f.to_string().ends_with("wrapfig.sty") => {
                let empty = self.def_cmd(vec!(),false,false,vec!());
                state.commands.set("WF@putfigmaybe".into(),Some(empty.clone()),true);
                state.commands.set("WF@floathand".into(),Some(empty.clone()),true);
                state.commands.set("WF@wraphand".into(),Some(PrimitiveTeXCommand::Whatsit(ProvidesWhatsit::Simple(&WRAPFIG)).as_command()),true);
                Some(Some(empty))
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
        Box::new(MapstoListener()),
        Box::new(UnderbraceListener()),
        Box::new(OverbraceListener()),
        Box::new(MarginParListener()),
        Box::new(WrapfigListener())
    )
}

// TODO sout, tableofcontents?, sqrt,