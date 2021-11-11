use crate::commands::{DimenReference, RegisterReference, TeXCommand};

pub static PDFOUTPUT : RegisterReference = RegisterReference {
    name: "pdfoutput",
    index:-35
};

pub static PDFPAGEHEIGHT : DimenReference = DimenReference {
    name: "pdfpageheight",
    index:-17
};

pub static PDFPAGEWIDTH : DimenReference = DimenReference {
    name: "pdfpagewidth",
    index:-17
};

pub fn pdftex_commands() -> Vec<TeXCommand<'static>> {vec![
    TeXCommand::Register(&PDFOUTPUT),
    TeXCommand::Dimen(&PDFPAGEHEIGHT),
    TeXCommand::Dimen(&PDFPAGEWIDTH),
]}