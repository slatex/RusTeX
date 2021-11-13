use crate::commands::{AssignableValue, Conditional, DimenReference, RegisterReference, TeXCommand};

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

pub static IFPDFABSNUM : Conditional = Conditional {
    name:"ifpdfabsnum",
    _apply: |int,cond,unless| {
        todo!()
    }
};

pub static IFPDFABSDIM : Conditional = Conditional {
    name:"ifpdfabsdim",
    _apply: |int,cond,unless| {
        todo!()
    }
};

pub static IFPDFPRIMITIVE : Conditional = Conditional {
    name:"ifpdfprimitive",
    _apply: |int,cond,unless| {
        todo!()
    }
};

pub fn pdftex_commands() -> Vec<TeXCommand<'static>> {vec![
    TeXCommand::AV(AssignableValue::PrimReg(&PDFOUTPUT)),
    TeXCommand::AV(AssignableValue::PrimDim(&PDFPAGEHEIGHT)),
    TeXCommand::AV(AssignableValue::PrimDim(&PDFPAGEWIDTH)),
    TeXCommand::AV(AssignableValue::PrimReg(&PDFMINORVERSION)),
    TeXCommand::AV(AssignableValue::PrimReg(&PDFOBJCOMPRESSLEVEL)),
    TeXCommand::AV(AssignableValue::PrimReg(&PDFCOMPRESSLEVEL)),
    TeXCommand::AV(AssignableValue::PrimReg(&PDFDECIMALDIGITS)),
    TeXCommand::AV(AssignableValue::PrimReg(&PDFPKRESOLUTION)),
    TeXCommand::AV(AssignableValue::PrimDim(&PDFHORIGIN)),
    TeXCommand::AV(AssignableValue::PrimDim(&PDFVORIGIN)),
    TeXCommand::Cond(&IFPDFABSNUM),
    TeXCommand::Cond(&IFPDFABSDIM),
    TeXCommand::Cond(&IFPDFPRIMITIVE),
]}