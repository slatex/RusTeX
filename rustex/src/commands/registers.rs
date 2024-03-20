use crate::commands::{DimenReference, MuSkipReference, RegisterReference, SkipReference, TokReference};

pub static PRETOLERANCE : RegisterReference = RegisterReference {
    name: "pretolerance",
    index:1
};

pub static TOLERANCE : RegisterReference = RegisterReference {
    name: "tolerance",
    index:2
};

pub static HBADNESS : RegisterReference = RegisterReference {
    name: "hbadness",
    index:3
};

pub static VBADNESS : RegisterReference = RegisterReference {
    name: "vbadness",
    index:4
};

pub static LINEPENALTY : RegisterReference = RegisterReference {
    name: "linepenalty",
    index:5
};

pub static HYPHENPENALTY : RegisterReference = RegisterReference {
    name: "hyphenpenalty",
    index:6
};

pub static EXHYPHENPENALTY : RegisterReference = RegisterReference {
    name: "exhyphenpenalty",
    index:7
};

pub static BINOPPENALTY : RegisterReference = RegisterReference {
    name: "binoppenalty",
    index:8
};

pub static RELPENALTY : RegisterReference = RegisterReference {
    name: "relpenalty",
    index:9
};

pub static CLUBPENALTY : RegisterReference = RegisterReference {
    name: "clubpenalty",
    index:10
};

pub static WIDOWPENALTY : RegisterReference = RegisterReference {
    name: "widowpenalty",
    index:11
};

pub static DISPLAYWIDOWPENALTY : RegisterReference = RegisterReference {
    name: "displaywidowpenalty",
    index:12
};

pub static BROKENPENALTY : RegisterReference = RegisterReference {
    name: "brokenpenalty",
    index:13
};

pub static PREDISPLAYPENALTY : RegisterReference = RegisterReference {
    name: "predisplaypenalty",
    index:14
};

pub static DOUBLEHYPHENDEMERITS : RegisterReference = RegisterReference {
    name: "doublehyphendemerits",
    index:15
};

pub static FINALHYPHENDEMERITS : RegisterReference = RegisterReference {
    name: "finalhyphendemerits",
    index:16
};

pub static ADJDEMERITS : RegisterReference = RegisterReference {
    name: "adjdemerits",
    index:17
};

pub static TRACINGLOSTCHARS : RegisterReference = RegisterReference {
    name: "tracinglostchars",
    index:18
};

pub static UCHYPH : RegisterReference = RegisterReference {
    name: "uchyph",
    index:19
};

pub static DEFAULTHYPHENCHAR : RegisterReference = RegisterReference {
    name: "defaulthyphenchar",
    index:20
};

pub static DEFAULTSKEWCHAR : RegisterReference = RegisterReference {
    name: "defaultskewchar",
    index:21
};

pub static DELIMITERFACTOR : RegisterReference = RegisterReference {
    name: "delimiterfactor",
    index:22
};

pub static SHOWBOXBREADTH : RegisterReference = RegisterReference {
    name: "showboxbreadth",
    index:23
};

pub static SHOWBOXDEPTH : RegisterReference = RegisterReference {
    name: "showboxdepth",
    index:24
};

pub static ERRORCONTEXTLINES : RegisterReference = RegisterReference {
    name: "errorcontextlines",
    index:25
};

pub static MAXDEADCYCLES : RegisterReference = RegisterReference {
    name: "maxdeadcycles",
    index:26
};

pub static TRACINGSTATS : RegisterReference = RegisterReference {
    name: "tracingstats",
    index:27
};

pub static LEFTHYPHENMIN : RegisterReference = RegisterReference {
    name: "lefthyphenmin",
    index:28
};

pub static RIGHTHYPHENMIN : RegisterReference = RegisterReference {
    name: "righthyphenmin",
    index:29
};

pub static SAVINGHYPHCODES : RegisterReference = RegisterReference {
    name: "savinghyphcodes",
    index:30
};

pub static FAM : RegisterReference = RegisterReference {
    name: "fam",
    index:31
};

pub static SPACEFACTOR : RegisterReference = RegisterReference {
    name: "spacefactor",
    index:32
};

pub static GLOBALDEFS : RegisterReference = RegisterReference {
    name: "globaldefs",
    index:33
};

pub static TRACINGNESTING : RegisterReference = RegisterReference {
    name: "tracingnesting",
    index:34
};

pub static MAG : RegisterReference = RegisterReference {
    name: "mag",
    index:35
};

pub static LANGUAGE : RegisterReference = RegisterReference {
    name: "language",
    index:36
};

pub static INTERLINEPENALTY : RegisterReference = RegisterReference {
    name: "interlinepenalty",
    index:37
};

pub static FLOATINGPENALTY : RegisterReference = RegisterReference {
    name: "floatingpenalty",
    index:38
};

pub static LASTNODETYPE : RegisterReference = RegisterReference {
    name: "lastnodetype",
    index:39
};

pub static INSERTPENALTIES : RegisterReference = RegisterReference {
    name: "insertpenalties",
    index:40
};

pub static BADNESS : RegisterReference = RegisterReference {
    name: "badness",
    index:41
};

pub static DEADCYCLES : RegisterReference = RegisterReference {
    name: "deadcycles",
    index:42
};

pub static INTERLINEPENALTIES : RegisterReference = RegisterReference {
    name: "interlinepenalties",
    index:43
};

pub static CLUBPENALTIES : RegisterReference = RegisterReference {
    name: "clubpenalties",
    index:44
};

pub static WIDOWPENALTIES : RegisterReference = RegisterReference {
    name: "widowpenalties",
    index:45
};

pub static DISPLAYWIDOWPENALTIES : RegisterReference = RegisterReference {
    name: "displaywidowpenalties",
    index:46
};

pub static OUTPUTPENALTY : RegisterReference = RegisterReference {
    name: "outputpenalty",
    index:47
};

pub static SAVINGVDISCARDS : RegisterReference = RegisterReference {
    name: "savingvdiscards",
    index:48
};

pub static SYNCTEX : RegisterReference = RegisterReference {
    name: "synctex",
    index:50
};

pub static POSTDISPLAYPENALTY : RegisterReference = RegisterReference {
    name: "postdisplaypenalty",
    index:51
};

pub static TRACINGSCANTOKENS : RegisterReference = RegisterReference {
    name: "tracingscantokens",
    index:52
};

pub static TRACINGPAGES : RegisterReference = RegisterReference {
    name: "tracingpages",
    index:53
};

pub static TRACINGCOMMANDS : RegisterReference = RegisterReference {
    name: "tracingcommands",
    index:54
};

pub static TRACINGMACROS : RegisterReference = RegisterReference {
    name: "tracingmacros",
    index:55
};

pub static TRACINGONLINE : RegisterReference = RegisterReference {
    name: "tracingonline",
    index:56
};

pub static TRACINGOUTPUT : RegisterReference = RegisterReference {
    name: "tracingoutput",
    index:57
};

pub static TRACINGPARAGRAPHS : RegisterReference = RegisterReference {
    name: "tracingparagraphs",
    index:58
};

pub static TRACINGRESTORES : RegisterReference = RegisterReference {
    name: "tracingrestores",
    index:59
};

pub static TRACINGASSIGNS : RegisterReference = RegisterReference {
    name: "tracingassigns",
    index:60
};

pub static TRACINGGROUPS : RegisterReference = RegisterReference {
    name: "tracinggroups",
    index:61
};

pub static TRACINGIFS : RegisterReference = RegisterReference {
    name: "tracingifs",
    index:62
};

pub static PREVGRAF: RegisterReference = RegisterReference {
    name: "prevgraf",
    index:63
};

// PDF --------------------------------------------------------------------------

pub static PDFOUTPUT : RegisterReference = RegisterReference {
    name: "pdfoutput",
    index:64
};

pub static PDFMINORVERSION : RegisterReference = RegisterReference {
    name: "pdfminorversion",
    index:65
};

pub static PDFOBJCOMPRESSLEVEL : RegisterReference = RegisterReference {
    name: "pdfobjcompresslevel",
    index:66
};

pub static PDFCOMPRESSLEVEL : RegisterReference = RegisterReference {
    name: "pdfcompresslevel",
    index:67
};

pub static PDFDECIMALDIGITS : RegisterReference = RegisterReference {
    name: "pdfdecimaldigits",
    index:68
};

pub static PDFPKRESOLUTION : RegisterReference = RegisterReference {
    name: "pdfpkresolution",
    index:69
};

pub static PDFLASTOBJ : RegisterReference = RegisterReference {
    name: "pdflastobj",
    index:70
};

pub static PDFLASTXFORM : RegisterReference = RegisterReference {
    name: "pdflastxform",
    index:71
};

pub static PDFLASTANNOT : RegisterReference = RegisterReference {
    name: "pdflastannot",
    index:72
};

pub static PDFLASTLINK : RegisterReference = RegisterReference {
    name: "pdflastlink",
    index:73
};

pub static PDFSUPPRESSWARNINGDUPDEST : RegisterReference = RegisterReference {
    name: "pdfsuppresswarningdupdest",
    index:74
};

pub static PDFPROTRUDECHARS : RegisterReference = RegisterReference {
    name: "pdfprotrudechars",
    index:75
};

pub static PDFADJUSTSPACING : RegisterReference = RegisterReference {
    name: "pdfadjustspacing",
    index:76
};

pub static PDFDRAFTMODE : RegisterReference = RegisterReference {
    name: "pdfdraftmode",
    index:77
};

pub static PDFGENTOUNICODE : RegisterReference = RegisterReference {
    name: "pdfgentounicode",
    index:78
};

// ---------------------------------
pub static PREDISPLAYDIRECTION : RegisterReference = RegisterReference {
    name: "predisplaydirection",
    index:79
};


// Dimensions --------------------------------------------------------------------------------------


pub static HFUZZ : DimenReference = DimenReference {
    name: "hfuzz",
    index:1
};

pub static VFUZZ : DimenReference = DimenReference {
    name: "vfuzz",
    index:2
};

pub static OVERFULLRULE : DimenReference = DimenReference {
    name: "overfullrule",
    index:3
};

pub static MAXDEPTH : DimenReference = DimenReference {
    name: "maxdepth",
    index:4
};

pub static SPLITMAXDEPTH : DimenReference = DimenReference {
    name: "splitmaxdepth",
    index:5
};

pub static BOXMAXDEPTH : DimenReference = DimenReference {
    name: "boxmaxdepth",
    index:6
};

pub static DELIMITERSHORTFALL : DimenReference = DimenReference {
    name: "delimitershortfall",
    index:7
};

pub static NULLDELIMITERSPACE : DimenReference = DimenReference {
    name: "nulldelimiterspace",
    index:8
};

pub static SCRIPTSPACE : DimenReference = DimenReference {
    name: "scriptspace",
    index:9
};

pub static PARINDENT : DimenReference = DimenReference {
    name: "parindent",
    index:10
};

pub static VSIZE : DimenReference = DimenReference {
    name: "vsize",
    index:11
};

pub static HSIZE : DimenReference = DimenReference {
    name: "hsize",
    index:12
};

pub static LINESKIPLIMIT : DimenReference = DimenReference {
    name: "lineskiplimit",
    index:13
};

pub static MATHSURROUND : DimenReference = DimenReference {
    name: "mathsurround",
    index:14
};

pub static PAGETOTAL : DimenReference = DimenReference {
    name: "pagetotal",
    index:15
};

pub static PAGESTRETCH : DimenReference = DimenReference {
    name: "pagestretch",
    index:16
};

pub static PAGEFILSTRETCH : DimenReference = DimenReference {
    name: "pagefilstretch",
    index:17
};

pub static PAGEFILLSTRETCH : DimenReference = DimenReference {
    name: "pagefillstretch",
    index:18
};

pub static PAGEFILLLSTRETCH : DimenReference = DimenReference {
    name: "pagefilllstretch",
    index:19
};

pub static PAGESHRINK : DimenReference = DimenReference {
    name: "pageshrink",
    index:20
};

pub static PAGEDEPTH : DimenReference = DimenReference {
    name: "pagedepth",
    index:21
};

pub static EMERGENCYSTRETCH : DimenReference = DimenReference {
    name: "emergencystretch",
    index:22
};

pub static VOFFSET : DimenReference = DimenReference {
    name: "voffset",
    index:23
};

pub static HOFFSET : DimenReference = DimenReference {
    name: "hoffset",
    index:24
};

pub static DISPLAYWIDTH : DimenReference = DimenReference {
    name: "displaywidth",
    index:25
};

pub static PREDISPLAYSIZE : DimenReference = DimenReference {
    name: "predisplaysize",
    index:26
};

// PDF ------------------------------------------------------------------

pub static PDFLINKMARGIN : DimenReference = DimenReference {
    name: "pdflinkmargin",
    index:27
};

pub static PDFDESTMARGIN : DimenReference = DimenReference {
    name: "pdfdestmargin",
    index:28
};

pub static PDFPXDIMEN : DimenReference = DimenReference {
    name: "pdfpxdimen",
    index:29
};

pub static PDFPAGEHEIGHT : DimenReference = DimenReference {
    name: "pdfpageheight",
    index:30
};

pub static PDFPAGEWIDTH : DimenReference = DimenReference {
    name: "pdfpagewidth",
    index:31
};

pub static PDFHORIGIN : DimenReference = DimenReference {
    name: "pdfhorigin",
    index:32
};

pub static PDFVORIGIN : DimenReference = DimenReference {
    name: "pdfvorigin",
    index:33
};


pub static DISPLAYINDENT : DimenReference = DimenReference {
    name: "displayindent",
    index:34
};

// Skips -------------------------------------------------------------------------------------------


pub static PARSKIP : SkipReference = SkipReference {
    name: "parskip",
    index:1
};

pub static ABOVEDISPLAYSKIP : SkipReference = SkipReference {
    name: "abovedisplayskip",
    index:2
};

pub static ABOVEDISPLAYSHORTSKIP : SkipReference = SkipReference {
    name: "abovedisplayshortskip",
    index:3
};

pub static BELOWDISPLAYSKIP : SkipReference = SkipReference {
    name: "belowdisplayskip",
    index:4
};

pub static BELOWDISPLAYSHORTSKIP : SkipReference = SkipReference {
    name: "belowdisplayshortskip",
    index:5
};

pub static TOPSKIP : SkipReference = SkipReference {
    name: "topskip",
    index:6
};

pub static SPLITTOPSKIP : SkipReference = SkipReference {
    name: "splittopskip",
    index:7
};

pub static PARFILLSKIP : SkipReference = SkipReference {
    name: "parfillskip",
    index:8
};

pub static BASELINESKIP : SkipReference = SkipReference {
    name: "baselineskip",
    index:9
};

pub static LINESKIP : SkipReference = SkipReference {
    name: "lineskip",
    index:10
};

pub static PREVDEPTH : SkipReference = SkipReference {
    name: "prevdepth",
    index:11
};

pub static LEFTSKIP : SkipReference = SkipReference {
    name: "leftskip",
    index:12
};

pub static RIGHTSKIP : SkipReference = SkipReference {
    name: "rightskip",
    index:13
};

pub static TABSKIP : SkipReference = SkipReference {
    name: "tabskip",
    index:14
};

pub static SPACESKIP : SkipReference = SkipReference {
    name: "spaceskip",
    index:15
};

pub static XSPACESKIP : SkipReference = SkipReference {
    name: "xspaceskip",
    index:16
};

pub static BIGSKIPAMOUNT : SkipReference = SkipReference {
    name: "bigskipamount",
    index:17
};

// MUSKIPS ---------------------------------------------------------------

pub static THINMUSKIP : MuSkipReference = MuSkipReference {
    name: "thinmuskip",
    index:1
};

pub static MEDMUSKIP : MuSkipReference = MuSkipReference {
    name: "medmuskip",
    index:2
};

pub static THICKMUSKIP : MuSkipReference = MuSkipReference {
    name: "thickmuskip",
    index:3
};


// Tokens ------------------------------------------------------------------------------------------

pub static EVERYJOB : TokReference = TokReference {
    name:"everyjob",
    index:1
};

pub static EVERYPAR : TokReference = TokReference {
    name:"everypar",
    index:2
};

pub static EVERYMATH : TokReference = TokReference {
    name:"everymath",
    index:3
};

pub static EVERYDISPLAY : TokReference = TokReference {
    name:"everydisplay",
    index:4
};

pub static EVERYHBOX : TokReference = TokReference {
    name:"everyhbox",
    index:5
};

pub static EVERYVBOX : TokReference = TokReference {
    name:"everyvbox",
    index:6
};

pub static EVERYCR : TokReference = TokReference {
    name:"everycr",
    index:7
};

pub static ERRHELP : TokReference = TokReference {
    name:"errhelp",
    index:8
};

pub static OUTPUT : TokReference = TokReference {
    name:"output",
    index:9
};

pub static EVERYEOF : TokReference = TokReference {
    name:"everyeof",
    index:10
};

// PDF ------------

pub static PDFPAGERESOURCES: TokReference = TokReference {
    name:"pdfpageresources",
    index:11
};