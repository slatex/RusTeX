use crate::commands::rustex_specials::HTMLLiteral;
use crate::commands::{PrimitiveTeXCommand, ProvidesWhatsit, SimpleWhatsit, TeXCommand};
use crate::stomach::whatsits::WhatsitTrait;
use crate::{TeXString, Token};
use crate::catcodes::CategoryCode;
use crate::interpreter::params::CommandListener;
use crate::utils::TeXStr;

pub static URL: SimpleWhatsit = SimpleWhatsit {
    name: "rustex@directHTML",
    modes: |_| { true },
    _get: |_, int| {
        let tks = int.read_balanced_argument(true,false,false,true)?;
        let mut str : TeXString = "<span style=\"font-family:monospace;\">".into();
        str += int.tokens_to_string(&tks);
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