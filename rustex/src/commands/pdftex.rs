use crate::commands::{AssignableValue, DimenReference, RegisterReference, TeXCommand};

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
    TeXCommand::AV(AssignableValue::Register(&PDFOUTPUT)),
    TeXCommand::AV(AssignableValue::Dimen(&PDFPAGEHEIGHT)),
    TeXCommand::AV(AssignableValue::Dimen(&PDFPAGEWIDTH)),
    TeXCommand::AV(AssignableValue::Register(&PDFMINORVERSION)),
    TeXCommand::AV(AssignableValue::Register(&PDFOBJCOMPRESSLEVEL)),
    TeXCommand::AV(AssignableValue::Register(&PDFCOMPRESSLEVEL)),
    TeXCommand::AV(AssignableValue::Register(&PDFDECIMALDIGITS)),
    TeXCommand::AV(AssignableValue::Register(&PDFPKRESOLUTION)),
    TeXCommand::AV(AssignableValue::Dimen(&PDFHORIGIN)),
    TeXCommand::AV(AssignableValue::Dimen(&PDFVORIGIN)),
]}