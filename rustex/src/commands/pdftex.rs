use crate::commands::{AssignableValue, PrimitiveExecutable, Conditional, DimenReference, RegisterReference, TeXCommand, IntCommand,PrimitiveTeXCommand};
use crate::VERSION_INFO;

pub static PDFTEXVERSION : IntCommand = IntCommand {
    _getvalue: |_int| {
        Ok(VERSION_INFO.pdftexversion.to_string().parse().unwrap())
    },
    name: "pdftexversion"
};


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

pub static PDFMAJORVERSION: PrimitiveExecutable = PrimitiveExecutable {
    name:"pdfmajorversion",
    expandable:false,
    _apply:|_tk,_int| {todo!()}
};

// -------------------------------------------------------------------------------------------------

pub fn pdftex_commands() -> Vec<PrimitiveTeXCommand> {vec![
    PrimitiveTeXCommand::Int(&PDFTEXVERSION),

    PrimitiveTeXCommand::Cond(&IFPDFABSNUM),
    PrimitiveTeXCommand::Cond(&IFPDFABSDIM),
    PrimitiveTeXCommand::Cond(&IFPDFPRIMITIVE),

    PrimitiveTeXCommand::AV(AssignableValue::PrimReg(&PDFOUTPUT)),
    PrimitiveTeXCommand::AV(AssignableValue::PrimDim(&PDFPAGEHEIGHT)),
    PrimitiveTeXCommand::AV(AssignableValue::PrimDim(&PDFPAGEWIDTH)),
    PrimitiveTeXCommand::AV(AssignableValue::PrimReg(&PDFMINORVERSION)),
    PrimitiveTeXCommand::AV(AssignableValue::PrimReg(&PDFOBJCOMPRESSLEVEL)),
    PrimitiveTeXCommand::AV(AssignableValue::PrimReg(&PDFCOMPRESSLEVEL)),
    PrimitiveTeXCommand::AV(AssignableValue::PrimReg(&PDFDECIMALDIGITS)),
    PrimitiveTeXCommand::AV(AssignableValue::PrimReg(&PDFPKRESOLUTION)),
    PrimitiveTeXCommand::AV(AssignableValue::PrimDim(&PDFHORIGIN)),
    PrimitiveTeXCommand::AV(AssignableValue::PrimDim(&PDFVORIGIN)),

    // TODO ----------------------------------------------------------------------------------------

    PrimitiveTeXCommand::Primitive(&PDFESCAPESTRING),
    PrimitiveTeXCommand::Primitive(&PDFESCAPENAME),
    PrimitiveTeXCommand::Primitive(&PDFANNOT),
    PrimitiveTeXCommand::Primitive(&PDFCATALOG),
    PrimitiveTeXCommand::Primitive(&PDFCOLORSTACKINIT),
    PrimitiveTeXCommand::Primitive(&PDFCOLORSTACK),
    PrimitiveTeXCommand::Primitive(&PDFDEST),
    PrimitiveTeXCommand::Primitive(&PDFESCAPEHEX),
    PrimitiveTeXCommand::Primitive(&PDFFILEDUMP),
    PrimitiveTeXCommand::Primitive(&PDFFILEMODDATE),
    PrimitiveTeXCommand::Primitive(&PDFFILESIZE),
    PrimitiveTeXCommand::Primitive(&PDFFONTSIZE),
    PrimitiveTeXCommand::Primitive(&PDFFONTEXPAND),
    PrimitiveTeXCommand::Primitive(&PDFGLYPHTOUNICODE),
    PrimitiveTeXCommand::Primitive(&PDFUNESCAPEHEX),
    PrimitiveTeXCommand::Primitive(&PDFINFO),
    PrimitiveTeXCommand::Primitive(&PDFLITERAL),
    PrimitiveTeXCommand::Primitive(&PDFMATCH),
    PrimitiveTeXCommand::Primitive(&PDFLASTMATCH),
    PrimitiveTeXCommand::Primitive(&PDFOBJ),
    PrimitiveTeXCommand::Primitive(&PDFOUTLINE),
    PrimitiveTeXCommand::Primitive(&PDFPAGEATTR),
    PrimitiveTeXCommand::Primitive(&PDFREFXFORM),
    PrimitiveTeXCommand::Primitive(&PDFRESTORE),
    PrimitiveTeXCommand::Primitive(&PDFSAVE),
    PrimitiveTeXCommand::Primitive(&PDFSAVEPOS),
    PrimitiveTeXCommand::Primitive(&PDFLASTXPOS),
    PrimitiveTeXCommand::Primitive(&PDFLASTYPOS),
    PrimitiveTeXCommand::Primitive(&PDFSETMATRIX),
    PrimitiveTeXCommand::Primitive(&PDFSHELLESCAPE),
    PrimitiveTeXCommand::Primitive(&PDFSTARTLINK),
    PrimitiveTeXCommand::Primitive(&PDFENDLINK),
    PrimitiveTeXCommand::Primitive(&PDFXFORM),
    PrimitiveTeXCommand::Primitive(&PDFXIMAGE),
    PrimitiveTeXCommand::Primitive(&PDFREFXIMAGE),
    PrimitiveTeXCommand::Primitive(&PDFMDFIVESUM),
    PrimitiveTeXCommand::Primitive(&PDFSTRCMP),
    PrimitiveTeXCommand::Primitive(&PDFTEXREVISION),
    PrimitiveTeXCommand::Primitive(&PDFMAJORVERSION),
]}