use crate::commands::{AssignableValue, PrimitiveExecutable, Conditional, DimenReference, RegisterReference, TeXCommand, IntCommand,PrimitiveTeXCommand};
use crate::interpreter::tokenize;
use crate::VERSION_INFO;
use crate::{log,TeXErr};
use crate::interpreter::dimensions::Numeric;

pub static PDFTEXVERSION : IntCommand = IntCommand {
    _getvalue: |_int| {
        Ok(Numeric::Int(VERSION_INFO.pdftexversion.to_string().parse().unwrap()))
    },
    name: "pdftexversion"
};

pub static PDFSTRCMP: PrimitiveExecutable = PrimitiveExecutable {
    name:"pdfstrcmp",
    expandable:true,
    _apply:|rf,int| {
        let first = int.tokens_to_string(int.read_balanced_argument(true,false,false,true)?);
        let second = int.tokens_to_string(int.read_balanced_argument(true,false,false,true)?);
        log!("\\pdfstrcmp: \"{}\" == \"{}\"?",first,second);
        if first == second {
            log!("true");
            rf.2 = tokenize("0".into(),&crate::catcodes::OTHER_SCHEME)
        } else if first.to_string() < second.to_string() {
            rf.2 = tokenize("-1".into(),&crate::catcodes::OTHER_SCHEME)
        } else {
            rf.2 = tokenize("1".into(),&crate::catcodes::OTHER_SCHEME)
        }
        Ok(())
    }
};

pub static PDFFILESIZE: PrimitiveExecutable = PrimitiveExecutable {
    name:"pdffilesize",
    expandable:true,
    _apply:|rf,int| {
        let strtks = int.read_balanced_argument(true,false,false,true)?;
        let str = int.tokens_to_string(strtks);
        let file = int.get_file(&str.to_utf8())?;
        match file.string {
            None => (),
            Some(s) => rf.2 = crate::interpreter::tokenize(
                s.len().to_string().into(),&crate::catcodes::OTHER_SCHEME
            )
        };
        Ok(())
    }
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
    expandable:true,
    _apply:|_tk,_int| {todo!()}
};

pub static PDFESCAPENAME: PrimitiveExecutable = PrimitiveExecutable {
    name:"pdfescapename",
    expandable:true,
    _apply:|_tk,_int| {todo!()}
};

pub static PDFANNOT: PrimitiveExecutable = PrimitiveExecutable {
    name:"pdfannot",
    expandable:true,
    _apply:|_tk,_int| {todo!()}
};

pub static PDFCATALOG: PrimitiveExecutable = PrimitiveExecutable {
    name:"pdfcatalog",
    expandable:true,
    _apply:|_tk,_int| {todo!()}
};

pub static PDFCOLORSTACKINIT: PrimitiveExecutable = PrimitiveExecutable {
    name:"pdfcolorstackinit",
    expandable:true,
    _apply:|_tk,_int| {todo!()}
};

pub static PDFCOLORSTACK: PrimitiveExecutable = PrimitiveExecutable {
    name:"pdfcolorstack",
    expandable:true,
    _apply:|_tk,_int| {todo!()}
};

pub static PDFDEST: PrimitiveExecutable = PrimitiveExecutable {
    name:"pdfdest",
    expandable:true,
    _apply:|_tk,_int| {todo!()}
};

pub static PDFESCAPEHEX: PrimitiveExecutable = PrimitiveExecutable {
    name:"pdfescapehex",
    expandable:true,
    _apply:|_tk,_int| {todo!()}
};

pub static PDFFILEDUMP: PrimitiveExecutable = PrimitiveExecutable {
    name:"pdffiledump",
    expandable:true,
    _apply:|_tk,_int| {todo!()}
};

pub static PDFFILEMODDATE: PrimitiveExecutable = PrimitiveExecutable {
    name:"pdffilemoddate",
    expandable:true,
    _apply:|_tk,_int| {todo!()}
};

pub static PDFFONTSIZE: PrimitiveExecutable = PrimitiveExecutable {
    name:"pdffontsize",
    expandable:true,
    _apply:|_tk,_int| {todo!()}
};

pub static PDFFONTEXPAND: PrimitiveExecutable = PrimitiveExecutable {
    name:"pdffontexpand",
    expandable:true,
    _apply:|_tk,_int| {todo!()}
};

pub static PDFGLYPHTOUNICODE: PrimitiveExecutable = PrimitiveExecutable {
    name:"pdfglyphtounicode",
    expandable:true,
    _apply:|_tk,_int| {todo!()}
};

pub static PDFUNESCAPEHEX: PrimitiveExecutable = PrimitiveExecutable {
    name:"pdfunescapehex",
    expandable:true,
    _apply:|_tk,_int| {todo!()}
};

pub static PDFINFO: PrimitiveExecutable = PrimitiveExecutable {
    name:"pdfinfo",
    expandable:true,
    _apply:|_tk,_int| {todo!()}
};

pub static PDFLITERAL: PrimitiveExecutable = PrimitiveExecutable {
    name:"pdfliteral",
    expandable:true,
    _apply:|_tk,_int| {todo!()}
};

pub static PDFMATCH: PrimitiveExecutable = PrimitiveExecutable {
    name:"pdfmatch",
    expandable:true,
    _apply:|_tk,_int| {todo!()}
};

pub static PDFLASTMATCH: PrimitiveExecutable = PrimitiveExecutable {
    name:"pdflastmatch",
    expandable:true,
    _apply:|_tk,_int| {todo!()}
};

pub static PDFOBJ: PrimitiveExecutable = PrimitiveExecutable {
    name:"pdfobj",
    expandable:true,
    _apply:|_tk,_int| {todo!()}
};

pub static PDFOUTLINE: PrimitiveExecutable = PrimitiveExecutable {
    name:"pdfoutline",
    expandable:true,
    _apply:|_tk,_int| {todo!()}
};

pub static PDFPAGEATTR: PrimitiveExecutable = PrimitiveExecutable {
    name:"pdfpageattr",
    expandable:true,
    _apply:|_tk,_int| {todo!()}
};

pub static PDFREFXFORM: PrimitiveExecutable = PrimitiveExecutable {
    name:"pdfrefxform",
    expandable:true,
    _apply:|_tk,_int| {todo!()}
};

pub static PDFRESTORE: PrimitiveExecutable = PrimitiveExecutable {
    name:"pdfrestore",
    expandable:true,
    _apply:|_tk,_int| {todo!()}
};

pub static PDFSAVE: PrimitiveExecutable = PrimitiveExecutable {
    name:"pdfsave",
    expandable:true,
    _apply:|_tk,_int| {todo!()}
};

pub static PDFSAVEPOS: PrimitiveExecutable = PrimitiveExecutable {
    name:"pdfsavepos",
    expandable:true,
    _apply:|_tk,_int| {todo!()}
};

pub static PDFLASTXPOS: PrimitiveExecutable = PrimitiveExecutable {
    name:"pdflastxpos",
    expandable:true,
    _apply:|_tk,_int| {todo!()}
};

pub static PDFLASTYPOS: PrimitiveExecutable = PrimitiveExecutable {
    name:"pdflastypos",
    expandable:true,
    _apply:|_tk,_int| {todo!()}
};

pub static PDFSETMATRIX: PrimitiveExecutable = PrimitiveExecutable {
    name:"pdfsetmatrix",
    expandable:true,
    _apply:|_tk,_int| {todo!()}
};

pub static PDFSHELLESCAPE: PrimitiveExecutable = PrimitiveExecutable {
    name:"pdfshellescape",
    expandable:true,
    _apply:|_tk,_int| {todo!()}
};

pub static PDFSTARTLINK: PrimitiveExecutable = PrimitiveExecutable {
    name:"pdfstartlink",
    expandable:true,
    _apply:|_tk,_int| {todo!()}
};

pub static PDFENDLINK: PrimitiveExecutable = PrimitiveExecutable {
    name:"pdfendlink",
    expandable:true,
    _apply:|_tk,_int| {todo!()}
};

pub static PDFXFORM: PrimitiveExecutable = PrimitiveExecutable {
    name:"pdfxform",
    expandable:true,
    _apply:|_tk,_int| {todo!()}
};

pub static PDFXIMAGE: PrimitiveExecutable = PrimitiveExecutable {
    name:"pdfximage",
    expandable:true,
    _apply:|_tk,_int| {todo!()}
};

pub static PDFREFXIMAGE: PrimitiveExecutable = PrimitiveExecutable {
    name:"pdfrefximage",
    expandable:true,
    _apply:|_tk,_int| {todo!()}
};

pub static PDFMDFIVESUM: PrimitiveExecutable = PrimitiveExecutable {
    name:"pdfmdfivesum",
    expandable:true,
    _apply:|_tk,_int| {todo!()}
};

pub static PDFTEXREVISION: PrimitiveExecutable = PrimitiveExecutable {
    name:"pdftexrevision",
    expandable:true,
    _apply:|_tk,_int| {todo!()}
};

pub static PDFMAJORVERSION: PrimitiveExecutable = PrimitiveExecutable {
    name:"pdfmajorversion",
    expandable:true,
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