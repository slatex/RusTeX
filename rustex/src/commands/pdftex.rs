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

pub static PDFMINORVERSION : RegisterReference = RegisterReference {
    name: "pdfminorversion",
    index:-36
};

pub static PDFOBJCOMPRESSLEVEL : RegisterReference = RegisterReference {
    name: "pdfobjcompresslevel",
    index:-37
};

pub static PDFCOMPRESSLEVEL : RegisterReference = RegisterReference {
    name: "pdfcompresslevel",
    index:-38
};

pub static PDFDECIMALDIGITS : RegisterReference = RegisterReference {
    name: "pdfdecimaldigits",
    index:-39
};

pub static PDFPKRESOLUTION : RegisterReference = RegisterReference {
    name: "pdfpkresolution",
    index:-40
};

pub static PDFHORIGIN : DimenReference = DimenReference {
    name: "pdfhorigin",
    index:-19
};

pub static PDFVORIGIN : DimenReference = DimenReference {
    name: "pdfvorigin",
    index:-20
};

pub fn pdftex_commands() -> Vec<TeXCommand<'static>> {vec![
    TeXCommand::Register(&PDFOUTPUT),
    TeXCommand::Dimen(&PDFPAGEHEIGHT),
    TeXCommand::Dimen(&PDFPAGEWIDTH),
    TeXCommand::Register(&PDFMINORVERSION),
    TeXCommand::Register(&PDFOBJCOMPRESSLEVEL),
    TeXCommand::Register(&PDFCOMPRESSLEVEL),
    TeXCommand::Register(&PDFDECIMALDIGITS),
    TeXCommand::Register(&PDFPKRESOLUTION),
    TeXCommand::Dimen(&PDFHORIGIN),
    TeXCommand::Dimen(&PDFVORIGIN),
]}