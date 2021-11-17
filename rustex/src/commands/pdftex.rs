use crate::commands::{AssignableValue, PrimitiveExecutable, Conditional, DimenReference, RegisterReference, TeXCommand};

pub static PDFOUTPUT : RegisterReference = RegisterReference {
    name: "pdfoutput",
    index:35
};

pub static PDFPAGEHEIGHT : DimenReference = DimenReference {
    name: "pdfpageheight",
    index:17
};

pub static PDFPAGEWIDTH : DimenReference = DimenReference {
    name: "pdfpagewidth",
    index:17
};

pub static PDFMINORVERSION : RegisterReference = RegisterReference {
    name: "pdfminorversion",
    index:36
};

pub static PDFOBJCOMPRESSLEVEL : RegisterReference = RegisterReference {
    name: "pdfobjcompresslevel",
    index:37
};

pub static PDFCOMPRESSLEVEL : RegisterReference = RegisterReference {
    name: "pdfcompresslevel",
    index:38
};

pub static PDFDECIMALDIGITS : RegisterReference = RegisterReference {
    name: "pdfdecimaldigits",
    index:39
};

pub static PDFPKRESOLUTION : RegisterReference = RegisterReference {
    name: "pdfpkresolution",
    index:40
};

pub static PDFHORIGIN : DimenReference = DimenReference {
    name: "pdfhorigin",
    index:19
};

pub static PDFVORIGIN : DimenReference = DimenReference {
    name: "pdfvorigin",
    index:20
};

// TODO --------------------------------------------------------------------------------------------

pub static IFPDFABSNUM : Conditional = Conditional {
    name:"ifpdfabsnum",
    _apply: |_int,_cond,_unless| {
        todo!()
    }
};

pub static IFPDFABSDIM : Conditional = Conditional {
    name:"ifpdfabsdim",
    _apply: |_int,_cond,_unless| {
        todo!()
    }
};

pub static IFPDFPRIMITIVE : Conditional = Conditional {
    name:"ifpdfprimitive",
    _apply: |_int,_cond,_unless| {
        todo!()
    }
};

pub static PDFESCAPESTRING: PrimitiveExecutable = PrimitiveExecutable {
    name:"pdfescapestring",
    expandable:false,
    _apply:|_tk,_int| {todo!()}
};

pub static PDFESCAPENAME: PrimitiveExecutable = PrimitiveExecutable {
    name:"pdfescapename",
    expandable:false,
    _apply:|_tk,_int| {todo!()}
};

pub static PDFANNOT: PrimitiveExecutable = PrimitiveExecutable {
    name:"pdfannot",
    expandable:false,
    _apply:|_tk,_int| {todo!()}
};

pub static PDFCATALOG: PrimitiveExecutable = PrimitiveExecutable {
    name:"pdfcatalog",
    expandable:false,
    _apply:|_tk,_int| {todo!()}
};

pub static PDFCOLORSTACKINIT: PrimitiveExecutable = PrimitiveExecutable {
    name:"pdfcolorstackinit",
    expandable:false,
    _apply:|_tk,_int| {todo!()}
};

pub static PDFCOLORSTACK: PrimitiveExecutable = PrimitiveExecutable {
    name:"pdfcolorstack",
    expandable:false,
    _apply:|_tk,_int| {todo!()}
};

pub static PDFDEST: PrimitiveExecutable = PrimitiveExecutable {
    name:"pdfdest",
    expandable:false,
    _apply:|_tk,_int| {todo!()}
};

pub static PDFESCAPEHEX: PrimitiveExecutable = PrimitiveExecutable {
    name:"pdfescapehex",
    expandable:false,
    _apply:|_tk,_int| {todo!()}
};

pub static PDFFILEDUMP: PrimitiveExecutable = PrimitiveExecutable {
    name:"pdffiledump",
    expandable:false,
    _apply:|_tk,_int| {todo!()}
};

pub static PDFFILEMODDATE: PrimitiveExecutable = PrimitiveExecutable {
    name:"pdffilemoddate",
    expandable:false,
    _apply:|_tk,_int| {todo!()}
};

pub static PDFFILESIZE: PrimitiveExecutable = PrimitiveExecutable {
    name:"pdffilesize",
    expandable:false,
    _apply:|_tk,_int| {todo!()}
};

pub static PDFFONTSIZE: PrimitiveExecutable = PrimitiveExecutable {
    name:"pdffontsize",
    expandable:false,
    _apply:|_tk,_int| {todo!()}
};

pub static PDFFONTEXPAND: PrimitiveExecutable = PrimitiveExecutable {
    name:"pdffontexpand",
    expandable:false,
    _apply:|_tk,_int| {todo!()}
};

pub static PDFGLYPHTOUNICODE: PrimitiveExecutable = PrimitiveExecutable {
    name:"pdfglyphtounicode",
    expandable:false,
    _apply:|_tk,_int| {todo!()}
};

pub static PDFUNESCAPEHEX: PrimitiveExecutable = PrimitiveExecutable {
    name:"pdfunescapehex",
    expandable:false,
    _apply:|_tk,_int| {todo!()}
};

pub static PDFINFO: PrimitiveExecutable = PrimitiveExecutable {
    name:"pdfinfo",
    expandable:false,
    _apply:|_tk,_int| {todo!()}
};

pub static PDFLITERAL: PrimitiveExecutable = PrimitiveExecutable {
    name:"pdfliteral",
    expandable:false,
    _apply:|_tk,_int| {todo!()}
};

pub static PDFMATCH: PrimitiveExecutable = PrimitiveExecutable {
    name:"pdfmatch",
    expandable:false,
    _apply:|_tk,_int| {todo!()}
};

pub static PDFLASTMATCH: PrimitiveExecutable = PrimitiveExecutable {
    name:"pdflastmatch",
    expandable:false,
    _apply:|_tk,_int| {todo!()}
};

pub static PDFOBJ: PrimitiveExecutable = PrimitiveExecutable {
    name:"pdfobj",
    expandable:false,
    _apply:|_tk,_int| {todo!()}
};

pub static PDFOUTLINE: PrimitiveExecutable = PrimitiveExecutable {
    name:"pdfoutline",
    expandable:false,
    _apply:|_tk,_int| {todo!()}
};

pub static PDFPAGEATTR: PrimitiveExecutable = PrimitiveExecutable {
    name:"pdfpageattr",
    expandable:false,
    _apply:|_tk,_int| {todo!()}
};

pub static PDFREFXFORM: PrimitiveExecutable = PrimitiveExecutable {
    name:"pdfrefxform",
    expandable:false,
    _apply:|_tk,_int| {todo!()}
};

pub static PDFRESTORE: PrimitiveExecutable = PrimitiveExecutable {
    name:"pdfrestore",
    expandable:false,
    _apply:|_tk,_int| {todo!()}
};

pub static PDFSAVE: PrimitiveExecutable = PrimitiveExecutable {
    name:"pdfsave",
    expandable:false,
    _apply:|_tk,_int| {todo!()}
};

pub static PDFSAVEPOS: PrimitiveExecutable = PrimitiveExecutable {
    name:"pdfsavepos",
    expandable:false,
    _apply:|_tk,_int| {todo!()}
};

pub static PDFLASTXPOS: PrimitiveExecutable = PrimitiveExecutable {
    name:"pdflastxpos",
    expandable:false,
    _apply:|_tk,_int| {todo!()}
};

pub static PDFLASTYPOS: PrimitiveExecutable = PrimitiveExecutable {
    name:"pdflastypos",
    expandable:false,
    _apply:|_tk,_int| {todo!()}
};

pub static PDFSETMATRIX: PrimitiveExecutable = PrimitiveExecutable {
    name:"pdfsetmatrix",
    expandable:false,
    _apply:|_tk,_int| {todo!()}
};

pub static PDFSHELLESCAPE: PrimitiveExecutable = PrimitiveExecutable {
    name:"pdfshellescape",
    expandable:false,
    _apply:|_tk,_int| {todo!()}
};

pub static PDFSTARTLINK: PrimitiveExecutable = PrimitiveExecutable {
    name:"pdfstartlink",
    expandable:false,
    _apply:|_tk,_int| {todo!()}
};

pub static PDFENDLINK: PrimitiveExecutable = PrimitiveExecutable {
    name:"pdfendlink",
    expandable:false,
    _apply:|_tk,_int| {todo!()}
};

pub static PDFXFORM: PrimitiveExecutable = PrimitiveExecutable {
    name:"pdfxform",
    expandable:false,
    _apply:|_tk,_int| {todo!()}
};

pub static PDFXIMAGE: PrimitiveExecutable = PrimitiveExecutable {
    name:"pdfximage",
    expandable:false,
    _apply:|_tk,_int| {todo!()}
};

pub static PDFREFXIMAGE: PrimitiveExecutable = PrimitiveExecutable {
    name:"pdfrefximage",
    expandable:false,
    _apply:|_tk,_int| {todo!()}
};

pub static PDFMDFIVESUM: PrimitiveExecutable = PrimitiveExecutable {
    name:"pdfmdfivesum",
    expandable:false,
    _apply:|_tk,_int| {todo!()}
};

pub static PDFSTRCMP: PrimitiveExecutable = PrimitiveExecutable {
    name:"pdfstrcmp",
    expandable:false,
    _apply:|_tk,_int| {todo!()}
};

pub static PDFTEXREVISION: PrimitiveExecutable = PrimitiveExecutable {
    name:"pdftexrevision",
    expandable:false,
    _apply:|_tk,_int| {todo!()}
};

pub static PDFTEXVERSION: PrimitiveExecutable = PrimitiveExecutable {
    name:"pdftexversion",
    expandable:false,
    _apply:|_tk,_int| {todo!()}
};

pub static PDFMAJORVERSION: PrimitiveExecutable = PrimitiveExecutable {
    name:"pdfmajorversion",
    expandable:false,
    _apply:|_tk,_int| {todo!()}
};

// -------------------------------------------------------------------------------------------------

pub fn pdftex_commands() -> Vec<TeXCommand> {vec![
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

    // TODO ----------------------------------------------------------------------------------------

    TeXCommand::Primitive(&PDFESCAPESTRING),
    TeXCommand::Primitive(&PDFESCAPENAME),
    TeXCommand::Primitive(&PDFANNOT),
    TeXCommand::Primitive(&PDFCATALOG),
    TeXCommand::Primitive(&PDFCOLORSTACKINIT),
    TeXCommand::Primitive(&PDFCOLORSTACK),
    TeXCommand::Primitive(&PDFDEST),
    TeXCommand::Primitive(&PDFESCAPEHEX),
    TeXCommand::Primitive(&PDFFILEDUMP),
    TeXCommand::Primitive(&PDFFILEMODDATE),
    TeXCommand::Primitive(&PDFFILESIZE),
    TeXCommand::Primitive(&PDFFONTSIZE),
    TeXCommand::Primitive(&PDFFONTEXPAND),
    TeXCommand::Primitive(&PDFGLYPHTOUNICODE),
    TeXCommand::Primitive(&PDFUNESCAPEHEX),
    TeXCommand::Primitive(&PDFINFO),
    TeXCommand::Primitive(&PDFLITERAL),
    TeXCommand::Primitive(&PDFMATCH),
    TeXCommand::Primitive(&PDFLASTMATCH),
    TeXCommand::Primitive(&PDFOBJ),
    TeXCommand::Primitive(&PDFOUTLINE),
    TeXCommand::Primitive(&PDFPAGEATTR),
    TeXCommand::Primitive(&PDFREFXFORM),
    TeXCommand::Primitive(&PDFRESTORE),
    TeXCommand::Primitive(&PDFSAVE),
    TeXCommand::Primitive(&PDFSAVEPOS),
    TeXCommand::Primitive(&PDFLASTXPOS),
    TeXCommand::Primitive(&PDFLASTYPOS),
    TeXCommand::Primitive(&PDFSETMATRIX),
    TeXCommand::Primitive(&PDFSHELLESCAPE),
    TeXCommand::Primitive(&PDFSTARTLINK),
    TeXCommand::Primitive(&PDFENDLINK),
    TeXCommand::Primitive(&PDFXFORM),
    TeXCommand::Primitive(&PDFXIMAGE),
    TeXCommand::Primitive(&PDFREFXIMAGE),
    TeXCommand::Primitive(&PDFMDFIVESUM),
    TeXCommand::Primitive(&PDFSTRCMP),
    TeXCommand::Primitive(&PDFTEXREVISION),
    TeXCommand::Primitive(&PDFTEXVERSION),
    TeXCommand::Primitive(&PDFMAJORVERSION),
]}