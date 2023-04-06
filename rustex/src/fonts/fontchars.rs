use std::collections::hash_map::Entry;
use std::collections::HashMap;
use std::sync::{Arc, RwLock};
use crate::interpreter::params::InterpreterParams;
use crate::utils::TeXStr;

#[derive(PartialEq,Copy,Clone)]
pub enum FontTableParam {
    Text,Math,SansSerif,Italic,Bold,Script,Capital,Monospaced,Blackboard,Fraktur,CapitalLetters
}

pub struct FontTable {
    pub(crate) name:TeXStr,
    pub params:Vec<FontTableParam>,
    pub(crate) table:&'static HashMap<u8,&'static str>
}
impl FontTable {
    pub fn default(&self,u:u8) -> String {
        std::format!("?{}?{}?",self.name,u)
    }
    pub fn get_char(&self,u:u8,p:&dyn InterpreterParams) -> &'static str {
        match self.table.get(&u) {
            Some(c) => c,
            None => {
                p.write_other(&*std::format!("Unknown character {} in font {}",u,&self.name));
                "???"
            }
        }
    }
}

macro_rules! table {
    ($name:expr,$o:ident,$(($s:tt,$iname:ident,$($para:expr),*)),* ;$rest:expr) => (
        match &$name.to_string() {
            $(s if s==$s => Some($o.insert(Arc::new(FontTable {
            params:vec!($($para),*),
            table:&$iname,
            name:$s.into()
        })).clone())),*,
        _ => $rest
        }
    )
}
pub struct FontTableStore {
    map : RwLock<HashMap<TeXStr,Arc<FontTable>>>
}
impl FontTableStore {
    pub fn get(&self, name:TeXStr) -> Option<Arc<FontTable>> {
        match self.map.write().unwrap().entry(name.clone()) {
            Entry::Occupied(o) => Some(o.get().clone()),
            Entry::Vacant(o) => table!(name,o,
                ("cmr",STANDARD_TEXT_CM,FontTableParam::Text),
                ("rm-lmr",STANDARD_TEXT_CM,FontTableParam::Text),
                ("cmss",STANDARD_TEXT_CM,FontTableParam::Text,FontTableParam::SansSerif),
                ("cmcsc",STANDARD_TEXT_CM2,FontTableParam::Text,FontTableParam::Capital),
                ("cmtcsc",STANDARD_TEXT_CM2,FontTableParam::Text,FontTableParam::Capital,FontTableParam::Monospaced),
                ("cmssbx",STANDARD_TEXT_CM,FontTableParam::Text,FontTableParam::SansSerif,FontTableParam::Bold),
                ("rm-lmss",STANDARD_TEXT_CM,FontTableParam::Text,FontTableParam::SansSerif),
                ("cmtt",STANDARD_TEXT_CM2,FontTableParam::Text,FontTableParam::Monospaced),
                ("cmitt",STANDARD_TEXT_CM2,FontTableParam::Text,FontTableParam::Monospaced,FontTableParam::Italic),
                ("rm-lmtt",STANDARD_TEXT_CM,FontTableParam::Text,FontTableParam::Monospaced),
                ("cmti",STANDARD_TEXT_CM,FontTableParam::Italic),
                ("cmsl",STANDARD_TEXT_CM,FontTableParam::Italic),
                ("eufm",STANDARD_TEXT_CM,FontTableParam::Fraktur),
                ("cmbx",STANDARD_TEXT_CM,FontTableParam::Math,FontTableParam::Bold),
                ("cmbxti",STANDARD_TEXT_CM,FontTableParam::Math,FontTableParam::Bold,FontTableParam::Italic),
                ("rm-lmri",STANDARD_TEXT_CM,FontTableParam::Math,FontTableParam::Italic),
                ("rm-lmbx",STANDARD_TEXT_CM,FontTableParam::Math,FontTableParam::Bold),
                ("rm-lmcsc",STANDARD_TEXT_CM,FontTableParam::Math,FontTableParam::Capital),
                ("rm-lmssbx",STANDARD_TEXT_CM,FontTableParam::Math,FontTableParam::Bold,FontTableParam::SansSerif),
                ("ptmrt",STANDARD_TEXT_CM,FontTableParam::Text),
                ("ptmbt",STANDARD_TEXT_CM,FontTableParam::Text,FontTableParam::Bold),
                ("cmmi",STANDARD_MATH_CM,FontTableParam::Math,FontTableParam::Italic),
                ("cmmib",STANDARD_MATH_CM,FontTableParam::Math,FontTableParam::Italic,FontTableParam::Bold),
                ("lmmi",STANDARD_MATH_CM,FontTableParam::Math,FontTableParam::Italic),
                ("cmssi",STANDARD_MATH_CM,FontTableParam::Math,FontTableParam::Italic,FontTableParam::SansSerif),
                ("jkpbmi",STANDARD_MATH_CM,FontTableParam::Math,FontTableParam::Bold,FontTableParam::Italic),
                ("jkpttmnt",STANDARD_TEXT_CM2,FontTableParam::Math,FontTableParam::Bold,FontTableParam::Monospaced),
                ("mathkerncmssi",STANDARD_MATH_CM,FontTableParam::Math,FontTableParam::Italic,FontTableParam::SansSerif),
                // stix -------------------------------------------------------------------
                ("t-stixgeneral",STANDARD_TEXT_EC,FontTableParam::Text),
                ("t-stixtext",STANDARD_TEXT_EC,FontTableParam::Text),
                ("t-stixtext-italic",STANDARD_TEXT_EC,FontTableParam::Text,FontTableParam::Italic),
                ("t-stixtext-bold",STANDARD_TEXT_EC,FontTableParam::Text,FontTableParam::Bold),
                ("ts-stixgeneral",STIX_TS_GENERAL,FontTableParam::Text),
                ("ts-stixtext",TS1_STIXTEXT,FontTableParam::Text),
                ("t-stixgeneral-bold",STANDARD_TEXT_EC,FontTableParam::Text,FontTableParam::Bold),
                ("stix-mathrm",STIX_MATH_RM,FontTableParam::Math),
                ("stix-mathrm-bold",STIX_MATH_RM,FontTableParam::Math,FontTableParam::Bold),
                ("stix-mathit",STIX_MATH_IT,FontTableParam::Math,FontTableParam::Italic),
                ("stix-mathscr",STIX_MATH_SCR,FontTableParam::Math,FontTableParam::Script),
                ("stix-mathfrak",STIX_MATH_FRAK,FontTableParam::Math,FontTableParam::Fraktur),
                ("stix-mathbb",STIX_MATH_BB,FontTableParam::Math,FontTableParam::Blackboard),
                ("stix-mathbbit",STIX_MATH_BBIT,FontTableParam::Math,FontTableParam::Blackboard,FontTableParam::Italic),
                ("stix-mathcal",STIX_MATH_CAL,FontTableParam::Math,FontTableParam::Script),
                ("stix-mathsf",STIX_MATH_SF,FontTableParam::Math,FontTableParam::SansSerif),
                ("stix-mathsfit",STIX_MATH_SFIT,FontTableParam::Math,FontTableParam::SansSerif,FontTableParam::Italic),
                ("stix-mathtt",STIX_MATH_TT,FontTableParam::Math,FontTableParam::SansSerif,FontTableParam::Italic),
                ("stix-mathex",STIX_MATH_EX,FontTableParam::Math),
                // ec ---------------------------------------------------------------------
                ("ecrm",STANDARD_TEXT_EC,FontTableParam::Text),
                ("ec-lmr",STANDARD_TEXT_EC,FontTableParam::Text),
                ("ecbx",STANDARD_TEXT_EC,FontTableParam::Text,FontTableParam::Bold),
                ("ec-lmbx",STANDARD_TEXT_EC,FontTableParam::Text,FontTableParam::Bold),
                ("eccc",STANDARD_TEXT_EC,FontTableParam::Text,FontTableParam::Capital),
                ("ec-lmcsc",STANDARD_TEXT_EC,FontTableParam::Text,FontTableParam::Capital),
                ("ec-lmcsco",STANDARD_TEXT_EC,FontTableParam::Text,FontTableParam::Capital,FontTableParam::Italic),
                ("ecsi",STANDARD_TEXT_EC,FontTableParam::Text,FontTableParam::SansSerif,FontTableParam::Italic),
                ("ec-lmsso",STANDARD_TEXT_EC,FontTableParam::Text,FontTableParam::SansSerif,FontTableParam::Italic),
                ("ecss",STANDARD_TEXT_EC,FontTableParam::Text,FontTableParam::SansSerif),
                ("ec-lmss",STANDARD_TEXT_EC,FontTableParam::Text,FontTableParam::SansSerif),
                ("ecbi",STANDARD_TEXT_EC,FontTableParam::Text,FontTableParam::Bold,FontTableParam::Italic),
                ("ecbl",STANDARD_TEXT_EC,FontTableParam::Text,FontTableParam::Bold,FontTableParam::Italic),
                ("ec-lmbxi",STANDARD_TEXT_EC,FontTableParam::Text,FontTableParam::Bold,FontTableParam::Italic),
                ("ectt",STANDARD_TEXT_EC,FontTableParam::Text,FontTableParam::Monospaced),
                ("ec-lmtt",STANDARD_TEXT_EC,FontTableParam::Text,FontTableParam::Monospaced),
                ("ecit",STANDARD_TEXT_EC,FontTableParam::Text,FontTableParam::Italic),
                ("ecsl",STANDARD_TEXT_EC,FontTableParam::Text,FontTableParam::Italic),
                ("ec-lmri",STANDARD_TEXT_EC,FontTableParam::Text,FontTableParam::Italic),
                ("ecsl",STANDARD_TEXT_EC,FontTableParam::Text,FontTableParam::Italic),
                ("ecsx",STANDARD_TEXT_EC,FontTableParam::Text,FontTableParam::SansSerif,FontTableParam::Bold),
                ("ecso",STANDARD_TEXT_EC,FontTableParam::Text,FontTableParam::SansSerif,FontTableParam::Bold,FontTableParam::Italic),
                ("ecxc",STANDARD_TEXT_EC,FontTableParam::Text,FontTableParam::Bold,FontTableParam::Capital),
                ("ec-lmssbx",STANDARD_TEXT_EC,FontTableParam::Text,FontTableParam::SansSerif,FontTableParam::Bold),
                ("ecti",STANDARD_TEXT_EC,FontTableParam::Text,FontTableParam::Italic),
                ("ec-lmtti",STANDARD_TEXT_EC,FontTableParam::Text,FontTableParam::Italic),
                ("ec-lmtk",STANDARD_TEXT_EC,FontTableParam::Text,FontTableParam::Monospaced,FontTableParam::Bold),
                ("ec-lmtto",STANDARD_TEXT_EC,FontTableParam::Text,FontTableParam::Monospaced,FontTableParam::Italic),
                ("ec-lmro",STANDARD_TEXT_EC,FontTableParam::Text,FontTableParam::Italic),
                ("ec-lmtlc",STANDARD_TEXT_EC,FontTableParam::Text,FontTableParam::Monospaced),
                // -------------------------------------------------------------------------
                ("jkpmnt",STANDARD_TEXT_EC,FontTableParam::Text),
                ("jkpbnt",STANDARD_TEXT_EC,FontTableParam::Text,FontTableParam::Bold),
                ("jkpmi",STANDARD_MATH_CM,FontTableParam::Math,FontTableParam::Italic),
                ("jkpmitt",STANDARD_TEXT_EC,FontTableParam::Text,FontTableParam::Italic),
                ("jkpbitt",STANDARD_TEXT_EC,FontTableParam::Text,FontTableParam::Italic,FontTableParam::Bold),
                ("jkpmsct",STANDARD_TEXT_EC,FontTableParam::Text,FontTableParam::Capital),
                ("jkpex",CMEX,FontTableParam::Math),
                ("jkpbex",CMEX,FontTableParam::Math,FontTableParam::Bold),
                ("jkpexa",JKPEXA,FontTableParam::Math),
                ("jkpbexa",JKPEXA,FontTableParam::Math,FontTableParam::Bold),
                ("jkpmia",STANDARD_MATH_CM,FontTableParam::Math,FontTableParam::Fraktur),
                ("jkpbmia",STANDARD_MATH_CM,FontTableParam::Math,FontTableParam::Fraktur,FontTableParam::Bold),
                ("jkpsy",MATH_CMSY,FontTableParam::Math,FontTableParam::CapitalLetters,FontTableParam::Script),
                ("jkpbsy",MATH_CMSY,FontTableParam::Math,FontTableParam::CapitalLetters,FontTableParam::Script,FontTableParam::Bold),
                ("jkpsya",MSAM,FontTableParam::Math),
                ("jkpbsya",MSAM,FontTableParam::Math,FontTableParam::Bold),
                ("jkpsyb",MSBM,FontTableParam::Math,FontTableParam::CapitalLetters,FontTableParam::Blackboard),
                ("jkpbsyb",MSBM,FontTableParam::Math,FontTableParam::Bold),
                ("jkpsyc",JKPSYC,FontTableParam::Math),
                ("jkpbsyc",JKPSYC,FontTableParam::Math,FontTableParam::Bold),
                ("jkpsyd",CAPITALS,FontTableParam::Math,FontTableParam::CapitalLetters,FontTableParam::Script),
                ("jkpbsyd",CAPITALS,FontTableParam::Math,FontTableParam::CapitalLetters,FontTableParam::Script,FontTableParam::Bold),
                ("jkpmnc",TS1_LM,FontTableParam::Math),
                ("jkpbmnc",TS1_LM,FontTableParam::Math,FontTableParam::Bold),
                // math --------------------------------------------------------------------
                ("rsfs",CAPITALS,FontTableParam::Text,FontTableParam::CapitalLetters,FontTableParam::Script),
                ("eurm",EURM,FontTableParam::Math),
                ("cmsy",MATH_CMSY,FontTableParam::Math,FontTableParam::CapitalLetters,FontTableParam::Script),
                ("cmbsy",MATH_CMSY,FontTableParam::Math,FontTableParam::CapitalLetters,FontTableParam::Script,FontTableParam::Bold),
                ("lmsy",MATH_CMSY,FontTableParam::Math,FontTableParam::CapitalLetters,FontTableParam::Script),
                ("cmex",CMEX,FontTableParam::Math),
                ("lmex",CMEX,FontTableParam::Math),
                ("tcrm",MATH_TC,FontTableParam::Math),
                ("tctt",MATH_TC,FontTableParam::Math,FontTableParam::Monospaced),
                ("tcbx",MATH_TC,FontTableParam::Math,FontTableParam::Bold),
                ("tcss",MATH_TC,FontTableParam::Math,FontTableParam::SansSerif),
                ("tcsl",MATH_TC,FontTableParam::Math,FontTableParam::Italic),
                ("feymr",FEYMR,FontTableParam::Math),
                ("MnSymbolA",MNSYMBOL_A,FontTableParam::Math),
                ("MnSymbolB",MNSYMBOL_B,FontTableParam::Math),
                ("MnSymbolC",MNSYMBOL_C,FontTableParam::Math),
                ("MnSymbolD",MNSYMBOL_D,FontTableParam::Math),
                ("MnSymbolE",MNSYMBOL_E,FontTableParam::Math),
                ("MnSymbolF",MNSYMBOL_F,FontTableParam::Math),
                ("MnSymbolA-Bold",MNSYMBOL_A,FontTableParam::Math,FontTableParam::Bold),
                ("MnSymbolB-Bold",MNSYMBOL_B,FontTableParam::Math,FontTableParam::Bold),
                ("MnSymbolC-Bold",MNSYMBOL_C,FontTableParam::Math,FontTableParam::Bold),
                ("MnSymbolD-Bold",MNSYMBOL_D,FontTableParam::Math,FontTableParam::Bold),
                ("MnSymbolE-Bold",MNSYMBOL_E,FontTableParam::Math,FontTableParam::Bold),
                ("MnSymbolF-Bold",MNSYMBOL_F,FontTableParam::Math,FontTableParam::Bold),
                ("MnSymbolS",MNSYMBOL_S,FontTableParam::Math,FontTableParam::CapitalLetters,FontTableParam::Script),
                ("msam",MSAM,FontTableParam::Math),
                ("msbm",MSBM,FontTableParam::Math,FontTableParam::CapitalLetters,FontTableParam::Blackboard),
                ("stmary",STMARY,FontTableParam::Math),
                ("ts-lmr",TS1_LM,FontTableParam::Math),
                ("ts-lmss",TS1_LM,FontTableParam::Math,FontTableParam::SansSerif),
                ("ts-lmbx",TS1_LM,FontTableParam::Math,FontTableParam::Bold),
                ("ts-lmtt",TS1_LM,FontTableParam::Math,FontTableParam::Monospaced),
                ("pzdr",PZDR,FontTableParam::Math),
                ("psyr",PSYR,FontTableParam::Math),
                ("manfnt",MANFNT,FontTableParam::Math),
                ("line",LINE,FontTableParam::Math),
                ("linew",LINEW,FontTableParam::Math),
                ("lcircle",LCIRCLE,FontTableParam::Math),
                ("lcirclew",LCIRCLEW,FontTableParam::Math),
                ("bbm",BBM,FontTableParam::Math,FontTableParam::Blackboard),
                ("wasy",WASY,FontTableParam::Math),
                ("lasy",WASY,FontTableParam::Math),
                ("wasyb",WASY,FontTableParam::Math)
                ;{
                    //println!("Warning: No character table for font {}",name);
                    None
                }
            )
        }
    }
    pub(in crate::fonts::fontchars) fn new() -> FontTableStore { FontTableStore {map:RwLock::new(HashMap::new())}}
}

lazy_static! {
    pub static ref FONT_TABLES : FontTableStore = FontTableStore::new();
    pub static ref STANDARD_TEXT_CM : HashMap<u8,&'static str> = HashMap::from([
        (0,"Γ"),(1,"∆"),(2,"Θ"),(3,"Λ"),(4,"Ξ"),(5,"Π"),(6,"Σ"),(7,"Υ"),(8,"Φ"),(9,"Ψ"),(10,"Ω"),
        (11,"ff"),(12,"fi"),(13,"fl"),(14,"ffi"),(15,"ffl"),(16,"ı"),(17,"ȷ"),(18,"`"),(19," ́"),
        (20,"ˇ"),(21," ̆"),(22," ̄"),(23," ̊"),(24," ̧"),(25,"ß"),(26,"æ"),(27,"œ"),(28,"ø"),(29,"Æ"),
        (30,"Œ"),(31,"Ø"),(32," "),(33,"!"),(34,"”"),(35,"#"),(36,"$"),(37,"%"),(38,"&"),(39,"’"),
        (40,"("),(41,")"),(42,"*"),(43,"+"),(44,","),(45,"-"),(46,"."),(47,"/"),(48,"0"),(49,"1"),
        (50,"2"),(51,"3"),(52,"4"),(53,"5"),(54,"6"),(55,"7"),(56,"8"),(57,"9"),(58,":"),(59,";"),
        (60,"¡"),(61,"="),(62,"¿"),(63,"?"),(64,"@"),(65,"A"),(66,"B"),(67,"C"),(68,"D"),(69,"E"),
        (70,"F"),(71,"G"),(72,"H"),(73,"I"),(74,"J"),(75,"K"),(76,"L"),(77,"M"),(78,"N"),(79,"O"),
        (80,"P"),(81,"Q"),(82,"R"),(83,"S"),(84,"T"),(85,"U"),(86,"V"),(87,"W"),(88,"X"),(89,"Y"),
        (90,"Z"),(91,"["),(92,"“"),(93,"]"),(94,"^"),(95," ̇"),(96,"‘"),(97,"a"),(98,"b"),(99,"c"),
        (100,"d"),(101,"e"),(102,"f"),(103,"g"),(104,"h"),(105,"i"),(106,"j"),(107,"k"),(108,"l"),
        (109,"m"),(110,"n"),(111,"o"),(112,"p"),(113,"q"),(114,"r"),(115,"s"),(116,"t"),(117,"u"),
        (118,"v"),(119,"w"),(120,"x"),(121,"y"),(122,"z"),(123,"–"),(124,"—"),(125," ̋"),(126,"~"), //  ̃
        (127," ̈")
    ]);
    pub static ref STANDARD_TEXT_CM2 : HashMap<u8,&'static str> = HashMap::from([
        (0,"Γ"),(1,"∆"),(2,"Θ"),(3,"Λ"),(4,"Ξ"),(5,"Π"),(6,"Σ"),(7,"Υ"),(8,"Φ"),(9,"Ψ"),(10,"Ω"),
        (11,"ff"),(12,"fi"),(13,"fl"),(14,"ffi"),(15,"ffl"),(16,"ı"),(17,"ȷ"),(18,"`"),(19," ́"),
        (20,"ˇ"),(21," ̆"),(22," ̄"),(23," ̊"),(24," ̧"),(25,"ß"),(26,"æ"),(27,"œ"),(28,"ø"),(29,"Æ"),
        (30,"Œ"),(31,"Ø"),(32," "),(33,"!"),(34,"\""),(35,"#"),(36,"$"),(37,"%"),(38,"&"),(39,"’"),
        (40,"("),(41,")"),(42,"*"),(43,"+"),(44,","),(45,"-"),(46,"."),(47,"/"),(48,"0"),(49,"1"),
        (50,"2"),(51,"3"),(52,"4"),(53,"5"),(54,"6"),(55,"7"),(56,"8"),(57,"9"),(58,":"),(59,";"),
        (60,"<"),(61,"="),(62,">"),(63,"?"),(64,"@"),(65,"A"),(66,"B"),(67,"C"),(68,"D"),(69,"E"),
        (70,"F"),(71,"G"),(72,"H"),(73,"I"),(74,"J"),(75,"K"),(76,"L"),(77,"M"),(78,"N"),(79,"O"),
        (80,"P"),(81,"Q"),(82,"R"),(83,"S"),(84,"T"),(85,"U"),(86,"V"),(87,"W"),(88,"X"),(89,"Y"),
        (90,"Z"),(91,"["),(92,"\\"),(93,"]"),(94,"^"),(95,"_"),(96,"‘"),(97,"a"),(98,"b"),(99,"c"),
        (100,"d"),(101,"e"),(102,"f"),(103,"g"),(104,"h"),(105,"i"),(106,"j"),(107,"k"),(108,"l"),
        (109,"m"),(110,"n"),(111,"o"),(112,"p"),(113,"q"),(114,"r"),(115,"s"),(116,"t"),(117,"u"),
        (118,"v"),(119,"w"),(120,"x"),(121,"y"),(122,"z"),(123,"{"),(124,"|"),(125,"}"),(126,"~"), //  ̃
        (127," ̈")
    ]);
    pub static ref STANDARD_TEXT_EC : HashMap<u8,&'static str> = HashMap::from([
        (0,"`"),(1," ́"),(2,"^"),(3," ̃"),(4," ̈"),(5," ̋"),(6," ̊"),(7,"ˇ"),(8," ̆"),(9," ̄"),(10," ̇"), //  ̃
        (11," ̧"),(12," ̨"),(13,","),(14,"<"),(15,">"),(16,"“"),(17,"”"),(18,"„"),(19,"«"),(20,"»"),
        (21,"—"),(22,"―"),(23,""),(24,"。"),(25,"ı"),(26,"ȷ"),(27,"ff"),(28,"fi"),(29,"fl"),(30,"ffi"),
        (31,"ffl"),(32,"␣"),(33,"!"),(34,"\""),(35,"#"),(36,"$"),(37,"%"),(38,"&"),(39,"’"),(40,"("),
        (41,")"),(42,"*"),(43,"+"),(44,","),(45,"-"),(46,"."),(47,"/"),(48,"0"),(49,"1"),
        (50,"2"),(51,"3"),(52,"4"),(53,"5"),(54,"6"),(55,"7"),(56,"8"),(57,"9"),(58,":"),(59,";"),
        (60,"<"),(61,"="),(62,">"),(63,"?"),(64,"@"),(65,"A"),(66,"B"),(67,"C"),(68,"D"),(69,"E"),
        (70,"F"),(71,"G"),(72,"H"),(73,"I"),(74,"J"),(75,"K"),(76,"L"),(77,"M"),(78,"N"),(79,"O"),
        (80,"P"),(81,"Q"),(82,"R"),(83,"S"),(84,"T"),(85,"U"),(86,"V"),(87,"W"),(88,"X"),(89,"Y"),
        (90,"Z"),(91,"["),(92,"\\"),(93,"]"),(94,"^"),(95,"_"),(96,"‘"),(97,"a"),(98,"b"),(99,"c"),
        (100,"d"),(101,"e"),(102,"f"),(103,"g"),(104,"h"),(105,"i"),(106,"j"),(107,"k"),(108,"l"),
        (109,"m"),(110,"n"),(111,"o"),(112,"p"),(113,"q"),(114,"r"),(115,"s"),(116,"t"),(117,"u"),
        (118,"v"),(119,"w"),(120,"x"),(121,"y"),(122,"z"),(123,"{"),(124,"|"),(125,"}"),
        (126,"~"),(127,"-"),(128,"Ă"),(129,"A̧"),(130,"Ć"),(131,"Č"),(132,"Ď"),(133,"Ě"),(134,"Ȩ"),
        (135,"Ğ"),(136,"Ĺ"),(137,"L̛"),(138,"Ł"),(139,"Ń"),(140,"Ň"),
        (142,"Ő"),(143,"Ŕ"),(144,"Ř"),(145,"Ś"),(146,"Š"),(147,"Ş"),(148,"Ť"),(149,"Ţ"),(150,"Ű"),
        (151,"Ů"),(152,"Ÿ"),(153,"Ź"),(154,"Ž"),(155,"Ż"),(156,"IJ"),(157,"İ"),(158,"đ"),(159,"§"),
        (160,"ă"),(161,"a̧"),(162,"ć"),(163,"č"),(164,"d̛"),(165,"ĕ"),(166,"ȩ"),(167,"ğ"),(168,"ĺ"),
        (169,"l̛"),(170,"ł"),(171,"ń"),(172,"ň"),
        (174,"ő"),(175,"ŕ"),(176,"ř"),(177,"ś"),(178,"š"),(179,"ş"),(180,"t̛"),(181,"ţ"),(182,"ű"),
        (183,"ů"),(184,"ÿ"),(185,"ź"),(186,"ž"),(187,"ż"),(188,"ij"),(189,"¡"),(190,"¿"),(191,"£"),
        (192,"À"),(193,"Á"),(194,"Â"),(195,"Ã"),(196,"Ä"),(197,"Å"),(198,"Æ"),(199,"Ç"),(200,"È"),
        (201,"É"),(202,"Ê"),(203,"Ë"),(204,"Ì"),(205,"Í"),(206,"Î"),(207,"Ï"),(208,"Ð"),(209,"Ñ"),
        (210,"Ò"),(211,"Ó"),(212,"Ô"),(213,"Õ"),(214,"Ö"),(215,"Œ"),(216,"Ø"),(217,"Ù"),(218,"Ú"),
        (219,"Û"),(220,"Ü"),(221,"Ý"),(222,"Þ"),(223,"SS"),(224,"à"),(225,"á"),(226,"â"),(227,"ã"),
        (228,"ä"),(229,"å"),(230,"æ"),(231,"ç"),(232,"è"),(233,"é"),(234,"ê"),(235,"ë"),(236,"ì"),
        (237,"í"),(238,"î"),(239,"ï"),(240,"ð"),(241,"ñ"),(242,"ò"),(243,"ó"),(244,"ô"),(245,"õ"),
        (246,"ö"),(247,"œ"),(248,"ø"),(249,"ù"),(250,"ú"),(251,"û"),(252,"ü"),(253,"ý"),(254,"þ"),
        (255,"ß")
    ]);
    pub static ref TS1_STIXTEXT : HashMap<u8,&'static str> = HashMap::from([
        (0,"`"),(1," ́"),(2,"^"),(3," ̃"),(4," ̈"),(5," ̋"),(6," ̊"),(7,"ˇ"),(8," ̆"),(9," ̄"),(10," ̇"),
        (11," ̧"),(12," ̨"),(13,","),
        (136,"•")
    ]);
    pub static ref EUFM : HashMap<u8,&'static str> = HashMap::from([
        (0,"b"),(1,"d"),(2,"f"),(3,"f"),(4,"g"),(5,"t"),(6,"t"),(7,"u"),
        (18,"`"),(19," ́"),
        (33,"!"),
        (38,"&"),(39,"'"),(40,"("),(41,")"),(42,"*"),(43,"+"),(44,","),(45,"-"),(46,"."),(47,"/"),
        (48,"0"),(49,"1"),(50,"2"),(51,"3"),(52,"4"),(53,"5"),(54,"6"),(55,"7"),(56,"8"),(57,"9"),
        (58,":"),(59,";"),
        (61,"="),
        (63,"?"),
        (65,"A"),(66,"B"),(67,"C"),(68,"D"),(69,"E"),(70,"F"),(71,"G"),(72,"H"),(73,"I"),(74,"J"),
        (75,"K"),(76,"L"),(77,"M"),(78,"N"),(79,"O"),(80,"P"),(81,"Q"),(82,"R"),(83,"S"),(84,"T"),
        (85,"U"),(86,"V"),(87,"W"),(88,"X"),(89,"Y"),(90,"Z"),(91,"["),
        (93,"]"),(94,"^"),
        (97,"a"),(98,"b"),(99,"c"),(100,"d"),(101,"e"),(102,"f"),(103,"g"),(104,"h"),(105,"i"),
        (106,"j"),(107,"k"),(108,"l"),(109,"m"),(110,"n"),(111,"o"),(112,"p"),(113,"q"),(114,"r"),
        (115,"s"),(116,"t"),(117,"u"),(118,"v"),(119,"w"),(120,"x"),(121,"y"),(122,"z"),
        (125," \""),
        (127,"1")

    ]);
    pub static ref STANDARD_MATH_CM : HashMap<u8,&'static str> = HashMap::from([
        (0,"Γ"),(1,"∆"),(2,"Θ"),(3,"Λ"),(4,"Ξ"),(5,"Π"),(6,"Σ"),(7,"Υ"),(8,"Φ"),(9,"Ψ"),(10,"Ω"),
        (11,"α"),(12,"β"),(13,"γ"),(14,"δ"),(15,"ϵ"),(16,"ζ"),(17,"η"),(18,"θ"),(19,"ι"),(20,"κ"),
        (21,"λ"),(22,"μ"),(23,"ν"),(24,"ξ"),(25,"π"),(26,"ρ"),(27,"σ"),(28,"τ"),(29,"υ"),(30,"ɸ"),
        (31,"χ"),(32,"ψ"),(33,"ω"),
        (34,"ε"),(35,"ϑ"),(36,"ϖ"),(37,"ϱ"),(38,"ς"),(39,"φ"),
        (40,"↼"),(41,"↽"),(42,"⇀"),(43,"⇁"),(44,"𝇋"),(45,"𝇌"),(46,"▹"),(47,"◃"),(48,"0"),(49,"1"),
        (50,"2"),(51,"3"),(52,"4"),(53,"5"),(54,"6"),(55,"7"),(56,"8"),(57,"9"),(58,"."),(59,","),
        (60,"<"),(61,"/"),(62,">"),(63,"*"),(64,"@"),(65,"A"),(66,"B"),(67,"C"),(68,"D"),(69,"E"),
        (70,"F"),(71,"G"),(72,"H"),(73,"I"),(74,"J"),(75,"K"),(76,"L"),(77,"M"),(78,"N"),(79,"O"),
        (80,"P"),(81,"Q"),(82,"R"),(83,"S"),(84,"T"),(85,"U"),(86,"V"),(87,"W"),(88,"X"),(89,"Y"),
        (90,"Z"),(91,"♭"),(92,"♮"),(93,"♯"),(94,"⌣"),
        (95,"⁀"),(96,"ℓ"),(97,"a"),(98,"b"),(99,"c"),(100,"d"),(101,"e"),(102,"f"),(103,"g"),
        (104,"h"),(105,"i"),(106,"j"),(107,"k"),(108,"l"),(109,"m"),(110,"n"),(111,"o"),(112,"p"),
        (113,"q"),(114,"r"),(115,"s"),(116,"t"),(117,"u"),(118,"v"),(119,"w"),(120,"x"),(121,"y"),
        (122,"z"),(123,"ı"),(124,"ȷ"),(125,"℘"),(126," ⃗"),(127,"⁀"),
        (191,""),(214,"")
    ]);
    pub static ref STIX_MATH_RM : HashMap<u8,&'static str> = HashMap::from([
        (0,"Γ"),(1,"∆"),(2,"Θ"),(3,"Λ"),(4,"Ξ"),(5,"Π"),(6,"Σ"),(7,"Υ"),(8,"Φ"),(9,"Ψ"),(10,"Ω"),
        (11,"α"),(12,"β"),(13,"γ"),(14,"δ"),(15,"ϵ"),(16,"ζ"),(17,"η"),(18,"θ"),(19,"ι"),(20,"κ"),
        (21,"λ"),(22,"μ"),(23,"ν"),(24,"ξ"),(25,"π"),(26,"ρ"),(27,"σ"),(28,"τ"),(29,"υ"),(30,"ɸ"),
        (31,"χ"),(32,"ψ"),(33,"ω"),
        (34,"ε"),(35,"ϑ"),(36,"ϖ"),(37,"ϱ"),(38,"ς"),(39,"φ"),
        (40,"∇"),(41,"∂"),(42,"—"),(43,"+"),(44,"±"),(45,"∓"),(46,"("),(47,")"),(48,"0"),(49,"1"),
        (50,"2"),(51,"3"),(52,"4"),(53,"5"),(54,"6"),(55,"7"),(56,"8"),(57,"9"),(58,":"),(59,";"),
        (60,"*"),(61,"="),(62,"$"),(63,"?"),(64,"!"),(65,"A"),(66,"B"),(67,"C"),(68,"D"),(69,"E"),
        (70,"F"),(71,"G"),(72,"H"),(73,"I"),(74,"J"),(75,"K"),(76,"L"),(77,"M"),(78,"N"),(79,"O"),
        (80,"P"),(81,"Q"),(82,"R"),(83,"S"),(84,"T"),(85,"U"),(86,"V"),(87,"W"),(88,"X"),(89,"Y"),
        (90,"Z"),(91,"["),(92,"\\"),(93,"]"),(94,"{"),
        (95,"/"),(96,"}"),(97,"a"),(98,"b"),(99,"c"),(100,"d"),(101,"e"),(102,"f"),(103,"g"),
        (104,"h"),(105,"i"),(106,"j"),(107,"k"),(108,"l"),(109,"m"),(110,"n"),(111,"o"),(112,"p"),
        (113,"q"),(114,"r"),(115,"s"),(116,"t"),(117,"u"),(118,"v"),(119,"w"),(120,"x"),(121,"y"),
        (122,"z"),(123,"ı"),(124,"ȷ"),(125,"#"),(126,"%"),(127,"'"),(128,"`"),(129,"`"),(130,"^"),
        (131," ̃"),(132," ̄"),(133," ̆"),(134," ̇"),(135," ̈"),
        (137," ̊"),
        (153,"&"),(154,"@"),(155,"¬"),(156,"·"),(157,"×"),(158,"≼"),(159,"÷"),
        (161,"̸"),
        (163,"†"),(164,"‡"),(165,"•"),(166,".."),(167,"..."),(168,"′"),(169,"″"),
        (170,"‴"),(171,"‵"),(172,"‶"),(173,"‷"),
        (175,"!!"),
        (177,"̸"),(178,"??"),
        (196,"⅋"),(197,"∀"),(199,"∃"),(200,"∄"),(201,"∅"),(202,"∆"),(203,"∈"),(204,"∉"),(205,"∊"),
        (206,"∋"),(207,"∌"),(208,"∍"),(209,"∎"),(210,"∔"),(211,"≽"),(212,"∖"),(213,"∘"),(214,"∙"),
        (215,"∝"),(216,"∞"),(217,"∟"),(218,"∠"),(219,"∡"),(220,"∢"),(221,"|"),
        (223,"‖"),
        (225,"∧"),(226,"∨"),(227,"∩"),(228,"∪"),
        (231,"Ø"),
        (237,"∼"),(238,"∽"),
        (243,"≂"),(244,"≃"),(245,"≄"),(246,"≅"),(247,"≆"),(248,"≇"),(249,"≈"),(250,"≉")
    ]);
    pub static ref STIX_MATH_IT : HashMap<u8,&'static str> = HashMap::from([
        (0,"Γ"),(1,"∆"),(2,"Θ"),(3,"Λ"),(4,"Ξ"),(5,"Π"),(6,"Σ"),(7,"Υ"),(8,"Φ"),(9,"Ψ"),(10,"Ω"),
        (11,"α"),(12,"β"),(13,"γ"),(14,"δ"),(15,"ϵ"),(16,"ζ"),(17,"η"),(18,"θ"),(19,"ι"),(20,"κ"),
        (21,"λ"),(22,"μ"),(23,"ν"),(24,"ξ"),(25,"π"),(26,"ρ"),(27,"σ"),(28,"τ"),(29,"υ"),(30,"ɸ"),
        (31,"χ"),(32,"ψ"),(33,"ω"),
        (34,"ε"),(35,"ϑ"),(36,"ϖ"),(37,"ϱ"),(38,"ς"),(39,"φ"),
        (40,"∇"),(41,"∂"),(42,"ℵ"),(43,"ℶ"),(44,"ℷ"),(45,"ℸ"),(46,"▹"),(47,"◃"),(48,"0"),(49,"1"),
        (50,"2"),(51,"3"),(52,"4"),(53,"5"),(54,"6"),(55,"7"),(56,"8"),(57,"9"),(58,"."),(59,","),
        (60,"<"),(61,"ℏ"),(62,">"),(63,"⋆"),
        (65,"A"),(66,"B"),(67,"C"),(68,"D"),(69,"E"),
        (70,"F"),(71,"G"),(72,"H"),(73,"I"),(74,"J"),(75,"K"),(76,"L"),(77,"M"),(78,"N"),(79,"O"),
        (80,"P"),(81,"Q"),(82,"R"),(83,"S"),(84,"T"),(85,"U"),(86,"V"),(87,"W"),(88,"X"),(89,"Y"),
        (90,"Z"),(91,"♭"),(92,"♮"),(93,"♯"),(94,"⌣"),
        (95,"⁀"),(96,"ℏ"),(97,"a"),(98,"b"),(99,"c"),(100,"d"),(101,"e"),(102,"f"),(103,"g"),
        (104,"h"),(105,"i"),(106,"j"),(107,"k"),(108,"l"),(109,"m"),(110,"n"),(111,"o"),(112,"p"),
        (113,"q"),(114,"r"),(115,"s"),(116,"t"),(117,"u"),(118,"v"),(119,"w"),(120,"x"),(121,"y"),
        (122,"z"),(123,"ı"),(124,"ȷ"),
        (126,"≪"),
        (128,"`̀"),(129," ́"),(130,"^"),
        (131," ̃"),(132," ̄"),(133," ̆"),(134," ̇"),(135," ̈"),(136," ̉"),(137," ̊"),
        (138," ̋"),(139," ̌"),
        (145," ⃖"),(146," ⃗"),
        (153,"‾"),(154," ̂"),
        (184,"≫"),
        (207,"⊂"),(208,"⊃"),(209,"⊄"),(210,"⊅"),(211,"⊆"),(212,"⊇"),(213,"⊈"),
        (214,"⊉"),(215,"⊊"),(216,"⊋"),(217,"⊌"),(218,"⊍"),(219,"⊎"),(220,"⊏"),
        (221,"⊐"),(222,"⊑"),(223,"⊒"),(224,"⊓"),(225,"⊔"),(226,"⊕"),(227,"⊖"),
        (228,"⊗"),(229,"⊘"),(230,"⊙"),(231,"⊚"),(232,"⊛"),(233,"⊜"),(234,"⊝"),
        (235,"⊞"),(236,"⊟"),(237,"⊠"),(238,"⊡"),(239,"⊢"),(240,"⊣"),(241,"⊤"),
        (242,"⊥"),(243,"⊦"),(244,"⊧"),(245,"⊨"),(246,"⊩"),(247,"⊪"),(248,"⊫"),
        (249,"⊬"),(250,"⊭"),(251,"⊮"),(252,"⊯"),(253,"⊰"),(254,"⊱"),(255,"⊴")
    ]);

    pub static ref STIX_MATH_SCR : HashMap<u8,&'static str> = HashMap::from([
        (0,"⊵"),(1,"⊶"),(2,"⊷"),(3,"⊸"),(4,"⊹"),(5,"⊺"),(6,"⊻"),(7,"⊼"),(8,"⊽"),
        (9,"⊾"),(10,"⊿"),
        (12,"·"),
        (52,"⋮"),(53,"⋯"),(54,"⋰"),(55,"⋱"),
        (65,"A"),(66,"B"),(67,"C"),(68,"D"),(69,"E"),
        (70,"F"),(71,"G"),(72,"H"),(73,"I"),(74,"J"),(75,"K"),(76,"L"),(77,"M"),(78,"N"),(79,"O"),
        (80,"P"),(81,"Q"),(82,"R"),(83,"S"),(84,"T"),(85,"U"),(86,"V"),(87,"W"),(88,"X"),(89,"Y"),
        (90,"Z"),
        (97,"a"),(98,"b"),(99,"c"),(100,"d"),(101,"e"),(102,"f"),(103,"g"),
        (104,"h"),(105,"i"),(106,"j"),(107,"k"),(108,"l"),(109,"m"),(110,"n"),(111,"o"),(112,"p"),
        (113,"q"),(114,"r"),(115,"s"),(116,"t"),(117,"u"),(118,"v"),(119,"w"),(120,"x"),(121,"y"),
        (122,"z"),(123,"ı"),(124,"ȷ"),
        (183,"■"),(184,"□"),(185,"▢"),(186,"▣"),(187,"▤"),(188,"▥"),(189,"▦"),(190,"▧"),
        (191,"▨"),(192,"▩"),(193,"▪"),(194,"▫"),(195,"▬"),(196,"▭"),(197,"▮"),(198,"▯"),
        (199,"▰"),(200,"▱"),(201,"▲"),(202,"△"),(203,"▴"),(204,"▵"),(205,"▶"),(206,"▷"),
        (207,"▸"),(208,"▹"),(209,"►"),(210,"▻"),(211,"▼"),(212,"▽"),(213,"▾"),(214,"▿"),
        (215,"◀"),(216,"◁"),(217,"◂"),(218,"◃"),(219,"◄"),(220,"◅"),(221,"◆"),(222,"◇"),
        (223,"◈"),(224,"◉"),(225,"◊"),(226,"○"),(227,"◌"),(228,"◍"),(229,"◎"),(230,"●"),
        (231,"◐"),(232,"◑"),(233,"◒"),(234,"◓"),(235,"◔"),(236,"◕"),(237,"◖"),(238,"◗"),
        (239,"◘"),(240,"◙"),(241,"◚"),(242,"◛"),(243,"◜"),(244,"◝"),(245,"◞"),(246,"◟"),
        (247,"◠"),(248,"◡"),(249,"◢"),(250,"◣"),(251,"◤"),(252,"◥"),(253,"◦"),(254,"◧"),
        (255,"◨")
    ]);
    pub static ref STIX_MATH_FRAK : HashMap<u8,&'static str> = HashMap::from([
        (2,"⟀"),(3,"⟁"),(4,"⟂"),(5,"⟃"),(6,"⟄"),(7,"⟅"),(8,"⟆"),(9,"⟇"),(10,"⟈"),
        (11,"⟉"),
        (65,"A"),(66,"B"),(67,"C"),(68,"D"),(69,"E"),
        (70,"F"),(71,"G"),(72,"H"),(73,"I"),(74,"J"),(75,"K"),(76,"L"),(77,"M"),(78,"N"),(79,"O"),
        (80,"P"),(81,"Q"),(82,"R"),(83,"S"),(84,"T"),(85,"U"),(86,"V"),(87,"W"),(88,"X"),(89,"Y"),
        (90,"Z"),
        (97,"a"),(98,"b"),(99,"c"),(100,"d"),(101,"e"),(102,"f"),(103,"g"),
        (104,"h"),(105,"i"),(106,"j"),(107,"k"),(108,"l"),(109,"m"),(110,"n"),(111,"o"),(112,"p"),
        (113,"q"),(114,"r"),(115,"s"),(116,"t"),(117,"u"),(118,"v"),(119,"w"),(120,"x"),(121,"y"),
        (122,"z"),(123,"ı"),(124,"ȷ")
    ]);
    pub static ref STIX_MATH_BB : HashMap<u8,&'static str> = HashMap::from([
        (65,"A"),(66,"B"),(67,"C"),(68,"D"),(69,"E"),
        (70,"F"),(71,"G"),(72,"H"),(73,"I"),(74,"J"),(75,"K"),(76,"L"),(77,"M"),(78,"N"),(79,"O"),
        (80,"P"),(81,"Q"),(82,"R"),(83,"S"),(84,"T"),(85,"U"),(86,"V"),(87,"W"),(88,"X"),(89,"Y"),
        (90,"Z"),
        (97,"a"),(98,"b"),(99,"c"),(100,"d"),(101,"e"),(102,"f"),(103,"g"),
        (104,"h"),(105,"i"),(106,"j"),(107,"k"),(108,"l"),(109,"m"),(110,"n"),(111,"o"),(112,"p"),
        (113,"q"),(114,"r"),(115,"s"),(116,"t"),(117,"u"),(118,"v"),(119,"w"),(120,"x"),(121,"y"),
        (122,"z"),(123,"ı"),(124,"ȷ")
    ]);
    pub static ref STIX_MATH_BBIT : HashMap<u8,&'static str> = HashMap::from([
        (65,"A"),(66,"B"),(67,"C"),(68,"D"),(69,"E"),
        (70,"F"),(71,"G"),(72,"H"),(73,"I"),(74,"J"),(75,"K"),(76,"L"),(77,"M"),(78,"N"),(79,"O"),
        (80,"P"),(81,"Q"),(82,"R"),(83,"S"),(84,"T"),(85,"U"),(86,"V"),(87,"W"),(88,"X"),(89,"Y"),
        (90,"Z"),
        (97,"a"),(98,"b"),(99,"c"),(100,"d"),(101,"e"),(102,"f"),(103,"g"),
        (104,"h"),(105,"i"),(106,"j"),(107,"k"),(108,"l"),(109,"m"),(110,"n"),(111,"o"),(112,"p"),
        (113,"q"),(114,"r"),(115,"s"),(116,"t"),(117,"u"),(118,"v"),(119,"w"),(120,"x"),(121,"y"),
        (122,"z"),(123,"ı"),(124,"ȷ")
    ]);
    pub static ref STIX_MATH_CAL : HashMap<u8,&'static str> = HashMap::from([
        (65,"A"),(66,"B"),(67,"C"),(68,"D"),(69,"E"),
        (70,"F"),(71,"G"),(72,"H"),(73,"I"),(74,"J"),(75,"K"),(76,"L"),(77,"M"),(78,"N"),(79,"O"),
        (80,"P"),(81,"Q"),(82,"R"),(83,"S"),(84,"T"),(85,"U"),(86,"V"),(87,"W"),(88,"X"),(89,"Y"),
        (90,"Z"),
        (97,"⊊"),(98,"⊋"),
        (102,"≤"),(103,"≥"),(104,"≦"),(105,"≧"),
        (142,"≝"),
        (144,"≟"),(145,"≠"),(146,"≡"),(147,"≢"),
        (148,"∫"),(149,"∬"),(150,"∭"),(151,"∮"),(152,"∯"),(153,"∰")
    ]);
    pub static ref STIX_MATH_SF : HashMap<u8,&'static str> = HashMap::from([
        (44,"-"),
        (65,"A"),(66,"B"),(67,"C"),(68,"D"),(69,"E"),
        (70,"F"),(71,"G"),(72,"H"),(73,"I"),(74,"J"),(75,"K"),(76,"L"),(77,"M"),(78,"N"),(79,"O"),
        (80,"P"),(81,"Q"),(82,"R"),(83,"S"),(84,"T"),(85,"U"),(86,"V"),(87,"W"),(88,"X"),(89,"Y"),
        (90,"Z"),
        (97,"a"),(98,"b"),(99,"c"),(100,"d"),(101,"e"),(102,"f"),(103,"g"),
        (104,"h"),(105,"i"),(106,"j"),(107,"k"),(108,"l"),(109,"m"),(110,"n"),(111,"o"),(112,"p"),
        (113,"q"),(114,"r"),(115,"s"),(116,"t"),(117,"u"),(118,"v"),(119,"w"),(120,"x"),(121,"y"),
        (122,"z"),(123,"ı"),(124,"ȷ"),(125,"←"),(126,"↑"),
        (153,"→"),(154,"↓"),(155,"↔"),(156,"↕"),(157,"↖"),
        (158,"↗"),(159,"↘"),(160,"↙"),(161,"↚"),(162,"↛"),(163,"↜"),(164,"↝"),
        (165,"↞"),(166,"↟"),(167,"↠"),(168,"↡"),(169,"↢"),(170,"↣"),(171,"↤"),
        (172,"↥"),(173,"↦"),(174,"↧"),(175,"↨"),(176,"↩"),(177,"↪"),(178,"↫"),
        (179,"↬"),(180,"↭"),(181,"↮"),(182,"↯"),
        (195,"↼"),(196,"↽"),(197,"↾"),(198,"↿"),(199,"⇀"),(200,"⇁"),(201,"⇂"),
        (202,"⇃"),(203,"⇄"),(204,"⇅"),(205,"⇆"),(206,"⇇"),(207,"⇈"),(208,"⇉"),
        (209,"⇊"),(210,"⇋"),(211,"⇌"),(212,"⇍"),(213,"⇎"),(214,"⇏"),(215,"⇐"),
        (216,"⇑"),(217,"⇒"),(218,"⇓"),(219,"⇔"),(220,"⇕"),(221,"⇖"),(222,"⇗"),
        (223,"⇘"),(224,"⇙"),(225,"⇚"),(226,"⇛"),(227,"⇜"),(228,"⇝"),(229,"⇞"),
        (230,"⇟"),(231,"⇠"),(232,"⇡"),(233,"⇢"),(234,"⇣")

    ]);
    pub static ref STIX_MATH_SFIT : HashMap<u8,&'static str> = HashMap::from([
        (65,"A"),(66,"B"),(67,"C"),(68,"D"),(69,"E"),
        (70,"F"),(71,"G"),(72,"H"),(73,"I"),(74,"J"),(75,"K"),(76,"L"),(77,"M"),(78,"N"),(79,"O"),
        (80,"P"),(81,"Q"),(82,"R"),(83,"S"),(84,"T"),(85,"U"),(86,"V"),(87,"W"),(88,"X"),(89,"Y"),
        (90,"Z"),
        (97,"a"),(98,"b"),(99,"c"),(100,"d"),(101,"e"),(102,"f"),(103,"g"),
        (104,"h"),(105,"i"),(106,"j"),(107,"k"),(108,"l"),(109,"m"),(110,"n"),(111,"o"),(112,"p"),
        (113,"q"),(114,"r"),(115,"s"),(116,"t"),(117,"u"),(118,"v"),(119,"w"),(120,"x"),(121,"y"),
        (122,"z"),(123,"ı"),(124,"ȷ")
    ]);
    pub static ref STIX_MATH_TT : HashMap<u8,&'static str> = HashMap::from([
        (48,"0"),(49,"1"),
        (50,"2"),(51,"3"),(52,"4"),(53,"5"),(54,"6"),(55,"7"),(56,"8"),(57,"9"),
        (65,"A"),(66,"B"),(67,"C"),(68,"D"),(69,"E"),
        (70,"F"),(71,"G"),(72,"H"),(73,"I"),(74,"J"),(75,"K"),(76,"L"),(77,"M"),(78,"N"),(79,"O"),
        (80,"P"),(81,"Q"),(82,"R"),(83,"S"),(84,"T"),(85,"U"),(86,"V"),(87,"W"),(88,"X"),(89,"Y"),
        (90,"Z"),
        (97,"a"),(98,"b"),(99,"c"),(100,"d"),(101,"e"),(102,"f"),(103,"g"),
        (104,"h"),(105,"i"),(106,"j"),(107,"k"),(108,"l"),(109,"m"),(110,"n"),(111,"o"),(112,"p"),
        (113,"q"),(114,"r"),(115,"s"),(116,"t"),(117,"u"),(118,"v"),(119,"w"),(120,"x"),(121,"y"),
        (122,"z"),(123,"ı"),(124,"ȷ")
    ]);
    pub static ref STIX_MATH_EX : HashMap<u8,&'static str> = HashMap::from([
        (177,"∏"),(178,"∐"),(179,"∑"),(180,"⋀"),(181,"⋁"),(182,"⋂"),(183,"⋃"),
        (199,"∏"),(200,"∐"),(201,"∑"),(202,"⋀"),(203,"⋁"),(204,"⋂"),(205,"⋃"),
        (226,"⌊"),(227,"⌋"),(228,"⌈"),(229,"⌉"),
        (234,"⟨"),(235,"⟩"),(240,"|"),(241,"‖"),
        (249,"√")
    ]);
    pub static ref STIX_TS_GENERAL : HashMap<u8,&'static str> = HashMap::from([
        (48,"0"),(49,"1"),
        (50,"2"),(51,"3"),(52,"4"),(53,"5"),(54,"6"),(55,"7"),(56,"8"),(57,"9"),
        (136,"●")
    ]);


    pub static ref MATH_CMSY : HashMap<u8,&'static str> = HashMap::from([
        (0,"−"),(1,"·"),(2,"×"),(3,"*"),
        (6,"±"),
        (8,"⊕"),(9,"⊖"),(10,"⊗"),(11,"⊘"),(12,"⊙"),
        (14,"◦"),(15,"•"),
        (17,"≡"),(18,"⊆"),(19,"⊇"),(20,"≤"),(21,"≥"),(22,"≼"),(23,"≽"),(24,"∼"),(25,"≈"),(26,"⊂"),
        (27,"⊃"),(28,"≪"),(29,"≫"),(30,"≺"),(31,"≻"),(32,"←"),(33,"→"),(34,"↑"),(35,"↓"),(36,"↔"),
        (37,"↗"),(38,"↘"),
        (39,"≃"),(40,"⇐"),(41,"⇒"),(42,"⇑"),(43,"⇓"),(44,"⇔"),(45,"↖"),(46,"↙"),(47,"∝"),(48,"\'"),
        (49,"∞"),(50,"∊"),(51,"∍"),(52,"△"),(53,"▽"),(54,"̸"),(55,"/"),(56,"∀"),(57,"∃"),(58,"¬"),
        (59,"∅"),
        (62,"⊤"),(63,"⊥"),
        (65,"A"),(66,"B"),(67,"C"),(68,"D"),(69,"E"),(70,"F"),(71,"G"),(72,"H"),(73,"I"),(74,"J"),
        (75,"K"),(76,"L"),(77,"M"),(78,"N"),(79,"O"),(80,"P"),(81,"Q"),(82,"R"),(83,"S"),(84,"T"),
        (85,"U"),(86,"V"),(87,"W"),(88,"X"),(89,"Y"),(90,"Z"),
        (91,"∪"),(92,"∩"),(93,"⊎"),(94,"∧"),(95,"∨"),(96,"⊢"),(97,"⊣"),(98,"⌊"),(99,"⌋"),(100,"⌈"),
        (101,"⌉"),(102,"{"),(103,"}"),(104,"〈"),(105,"〉"),(106,"|"),(107,"∥"),(108,"↕"),(109,"⇕"),
        (110,"\\"),(111,"≀"),(112,"√"),(113,"⨿"),(114,"∇"),(115,"∫"),(116,"⊔"),(117,"⊓"),(118,"⊑"),
        (119,"⊒"),(120,"§"),(121,"†"),(122,"‡"),
        (124,"♣"),(125,"♢"),(126,"♡"),(127,"♠")
    ]);

    pub static ref CMEX : HashMap<u8,&'static str> = HashMap::from([
        (8,"{"),(9,"}"),(10,"⟨"),(11,"⟩"),
        (76,"⊕"),(77,"⊕"),
        (80,"∑"),(81,"∏"),(82,"∫"),(83,"⋃"),(84,"⋂"),
        (86,"⋀"),(87,"⋁"),
        (96,"∐"),
        (98,"^"),
        (101,"~"),
        (122," "),(123," "),(124," "),(125," ")
    ]);

    pub static ref MNSYMBOL_A : HashMap<u8,&'static str> = HashMap::from([
        (0,"→"),(1,"↑"),(2,"←"),(3,"↓"),(4,"↗"),(5,"↖"),(6,"↙"),(7,"↘"),(8,"⇒"),(9,"⇑"),(10,"⇐"),
        (11,"⇓"),(12,"⇗"),(13,"⇖"),(14,"⇙"),(15,"⇘"),(16,"↔"),(17,"↕"),(18,"⤡"),(19,"⤢"),(20,"⇔"),
        (21,"⇕"),
        (24,"↠"),(25,"↟"),(26,"↞"),(27,"↡"),
        (32,"↣"),
        (34,"↢"),
        (40,"↦"),(41,"↥"),(42,"↤"),(43,"↧"),
        (48,"↪"),
        (53,"⤣"),
        (55,"⤥"),
        (58,"↩"),
        (60,"⤤"),
        (62,"⤦"),
        (64,"⇀"),(65,"↿"),(66,"↽"),(67,"⇂"),
        (72,"⇁"),(73,"↾"),(74,"↼"),(75,"⇃"),
        (80,"⥋"),
        (84,"⥊"),
        (88,"⇌"),
        (92,"⇋"),(93,"⥯"),
        (96,"⇢"),(97,"⇡"),(98,"⇠"),(99,"⇣"),
        (104,"⊸"),(105,"⫯"),(106,"⟜"),(107,"⫰"),
        (160,"↝"),
        (170,"↜"),
        (176,"↭"),
        (184,"↷"),
        (187,"⤸"),
        (194,"↶"),(195,"⤹"),
        (212,"＝"),(213,"∥"),
        (216,"⊢"),(217,"⊥"),(218,"⊣"),(219,"⊤"),
        (232,"⊩"),(233,"⍊"),
        (235,"⍑")
    ]);
    pub static ref MNSYMBOL_B : HashMap<u8,&'static str> = HashMap::from([]);
    pub static ref MNSYMBOL_C : HashMap<u8,&'static str> = HashMap::from([
        (0,"⋅"),
        (2,"∶"),
        (5,"⋯"),(6,"⋮"),(7,"⋰"),(8,"⋱"),
        (10,"∴"),
        (12,"∵"),
        (14,"∷"),
        (16,"−"),(17,"∣"),(18,"∕"),(19,"∖"),(20,"+"),(21,"×"),(22,"±"),(23,"∓"),
        (28,"÷"),
        (32,"¬"),(33,"⌐"),
        (44,"∧"),(45,"∨"),
        (56,"∪"),(57,"∩"),(58,"⋓"),(59,"⋒"),(60,"⊍"),(61,"⩀"),(62,"⊎"),
        (64,"⊔"),(65,"⊓"),(66,"⩏"),(67,"⩎"),
        (72,"▹"),(73,"▵"),(74,"◃"),(75,"▿"),(76,"▸"),(77,"▴"),(78,"◂"),(79,"▾"),
        (84,"▷"),(85,"△"),(86,"◁"),(87,"▽"),(88,"◦"),(89,"●"),(90,"◯"),(91,"◯"),(92,"⊖"),(93,"⦶"),
        (94,"⊘"),(95,"⦸"),(96,"⊕"),(97,"⊗"),(98,"⊙"),(99,"⊚"),
        (101,"⊛"),(102,"⍟"),(103,"∅"),
        (134,"⋆"),(135,"*"),
        (150,"⊥"),(151,"⊤"),
        (156,"′"),(157,"‵"),
        (166,"∀"),(167,"∃"),(168,"∄"),(169,"∇"),(170,"∞"),(171,"∫"),(172,"♭"),(173,"♮"),(174,"♯")
    ]);
    pub static ref MNSYMBOL_D : HashMap<u8,&'static str> = HashMap::from([
        (0,"="),(1,"≡"),(2,"∼"),(3,"∽"),(4,"≈"),
        (6,"≋"),
        (8,"≃"),(9,"⋍"),(10,"≂"),
        (12,"≅"),(13,"≌"),(14,"≊"),
        (16,"≏"),
        (18,"≎"),(19,"≐"),(20,"⩦"),(21,"≑"),(22,"≒"),(23,"≓"),(24,"⌣"),(25,"⌢"),
        (30,"≍"),
        (62,"∈"),
        (64,"<"),(65,">"),(66,"≤"),(67,"≥"),(68,"⩽"),(69,"⩾"),(70,"≦"),(71,"≧"),(72,"≶"),(73,"≷"),
        (74,"⋚"),(75,"⋛"),(76,"⪋"),(77,"⪌"),
        (96,"⊂"),(97,"⊃"),(98,"⊆"),(99,"⊇"),(100,"⫅"),(101,"⫆"),(102,"⋐"),(103,"⋑"),(104,"≺"),
        (105,"≻"),(106,"⪯"),(107,"⪰"),(108,"≼"),(109,"≽"),
        (120,"≠"),(121,"≢"),(122,"≁"),(123,"∽̸"),(124,"≉"),
        (126,"≋̸"),
        (128,"≄"),(129,"⋍̸"),(130,"≂̸"),
        (132,"≇"),(133,"≌̸"),(134,"≊̸"),
        (136,"≏̸"),
        (138,"≎̸"),(139,"≐̸"),(140,"⩦̸"),(141,"≑̸"),(142,"≒̸"),(143,"≓̸"),
        (144,"⌣̸"),(145,"⌢̸"),
        (150,"≭")
    ]);
    pub static ref MNSYMBOL_E : HashMap<u8,&'static str> = HashMap::from([
        (0,"["),
        (5,"]"),
        (66,"⟦"),(67,"⟦"),(68,"⟦"),(69,"⟦"),(70,"⟦"),(71,"⟧"),(72,"⟧"),(73,"⟧"),(74,"⟧"),(75,"⟧"),

        (83,"|"),
        (96,"⟨"),
        (101,"⟩"),
        (106,"⟬"),(107,"⟬"),(108,"⟬"),(109,"⟬"),(110,"⟬"),(111,"⟭"),(112,"⟭"),(113,"⟭"),(114,"⟭"),
        (115,"⟭"),(116,"⟪"),(117,"⟪"),(118,"⟪"),(119,"⟪"),(120,"⟪"),(121,"⟫"),(122,"⟫"),(123,"⟫"),
        (124,"⟫"),(125,"⟫"),(126,"/"),
        (131,"\\"),
        (136,"("),
        (141,")"),
        (152,"{"),
        (157,"}"),
        (179,"³"),(180,"´"),(181,"µ"),(182,"¶"),(183,"⏞"),(184,"⏟"),(185,"-"),(186,"√"),
        (209," ⃗"),(210,"̵"),(211," ̷"),(212," ̸")
    ]);
    pub static ref MNSYMBOL_F : HashMap<u8,&'static str> = HashMap::from([
        (40,"⊓"),
        (42,"⊔"),
        (52,"◯"),
        (54,"⊖"),
        (56,"⦶"),
        (58,"⊘"),
        (60,"⦸"),
        (62,"⊕"),
        (64,"⊗"),
        (66,"⊙"),
        (68,"⊚"),
        (72,"⊛"),
        (74,"⍟"),
        (76,"∏"),
        (78,"∐"),
        (80,"∑"),(81,"∑"),(82,"∫"),(83,"∫")
    ]);
    pub static ref MNSYMBOL_S : HashMap<u8,&'static str> = HashMap::from([
        (65,"A"),(66,"B"),(67,"C"),(68,"D"),(69,"E"),(70,"F"),(71,"G"),(72,"H"),(73,"I"),(74,"J"),
        (75,"K"),(76,"L"),(77,"M"),(78,"N"),(79,"O"),(80,"P"),(81,"Q"),(82,"R"),(83,"S"),(84,"T"),
        (85,"U"),(86,"V"),(87,"W"),(88,"X"),(89,"Y"),(90,"Z"),
    ]);
    pub static ref MSAM : HashMap<u8,&'static str> = HashMap::from([
        (3,"□"),(4,"■"),
        (6,"◇"),(7,"◆"),
        (13,"⊩"),
        (20,"⇈"),(21,"⇊"),(22,"↾"),(23,"⇂"),(24,"↿"),(25,"⇃"),
        (28,"⇆"),(29,"⇄"),
        (32,"⇝"),
        (66,"▷"),(67,"◁"),(68,"⊵"),(69,"⊴"),
        (72,"▼"),(73,"▶"),(74,"◀"),
        (78,"▲"),
        (88,"✓"),
        (93,"∡"),(94,"∢")
    ]);
    pub static ref FEYMR : HashMap<u8,&'static str> = HashMap::from([
        (0,"€"),(32," "),(101,"€")
    ]);
    pub static ref MSBM : HashMap<u8,&'static str> = HashMap::from([
        (63,"∅"),
        (65,"A"),(66,"B"),(67,"C"),(68,"D"),(69,"E"),(70,"F"),(71,"G"),(72,"H"),(73,"I"),(74,"J"),
        (75,"K"),(76,"L"),(77,"M"),(78,"N"),(79,"O"),(80,"P"),(81,"Q"),(82,"R"),(83,"S"),(84,"T"),
        (85,"U"),(86,"V"),(87,"W"),(88,"X"),(89,"Y"),(90,"Z"),(91,"^"),
        (97,"ວ"),
        (108,"⋖"),(109,"⋗"),(110,"⋉"),(111,"⋊"),
        (117,"≊"),
        (120,"↶"),(121,"↷"),
        (128,"A"),(129,"B"),(130,"C"),(131,"D"),(132,"E"),(133,"F"),(134,"G"),(135,"H"),(136,"I"),(137,"J"),
        (138,"K"),(139,"L"),(140,"M"),(141,"N"),(142,"O"),(143,"P"),(144,"Q"),(145,"R"),(146,"S"),(147,"T"),
        (148,"U"),(149,"V"),(150,"W"),(151,"X"),(152,"Y"),(153,"Z"),
    ]);
    pub static ref TS1_LM : HashMap<u8,&'static str> = HashMap::from([
        (42,"*"),
        (61,"-"),
        (132,"†"),(133,"‡"),
        (136,"•")
    ]);
    pub static ref WASY : HashMap<u8,&'static str> = HashMap::from([
        (1,"◁"),(2,"⊴"),(3,"▷"),(4,"⊵"),
        (25,"♀"),(26,"♂"),
        (35,"○"),
        (44,"🙂"),
        (47,"🙁"),
        (50,"□"),(51,"◇"),
        (59,"⤳"),(60,"⊏"),(61,"⊐")
    ]);

    pub static ref MATH_TC : HashMap<u8,&'static str> = HashMap::from([
        (36,"$"),
        (39,"\'"),
        (42,"*"),
        (44,","),(45,"="),(46,"."),(47,"/"),
        (61,"―"),
        (136,"•"),
        (169,"©"),
        (191,"€"),
        (214,"Ö")
    ]);

    pub static ref BBM : HashMap<u8,&'static str> = HashMap::from([
        (40,"⦅"),(41,"⦆"),
        (49,"1"),(50,"2"),
        (65,"A"),(66,"B"),(67,"C"),(68,"D"),(69,"E"),(70,"F"),(71,"G"),(72,"H"),(73,"I"),(74,"J"),
        (75,"K"),(76,"L"),(77,"M"),(78,"N"),(79,"O"),(80,"P"),(81,"Q"),(82,"R"),(83,"S"),(84,"T"),
        (85,"U"),(86,"V"),(87,"W"),(88,"X"),(89,"Y"),(90,"Z"),
        (91,"⟦"),(93,"⟧"),
        (97,"a"),(98,"b"),(99,"c"),(100,"d"),(101,"e"),(102,"f"),(103,"g"),(104,"h"),(105,"i"),
        (106,"j"),(107,"k"),(108,"l"),(109,"m"),(110,"n"),(111,"o"),(112,"p"),(113,"q"),(114,"r"),
        (115,"s"),(116,"t"),(117,"u"),(118,"v"),(119,"w"),(120,"x"),(121,"y"),(122,"z")
    ]);
    pub static ref EURM : HashMap<u8,&'static str> = HashMap::from([
        (0,"Γ"),(1,"∆"),(2,"Θ"),(3,"Λ"),(4,"Ξ"),(5,"Π"),(6,"Σ"),(7,"Υ"),(8,"Φ"),(9,"Ψ"),(10,"Ω"),
        (11,"α"),(12,"β"),(13,"γ"),(14,"δ"),(15,"ϵ"),(16,"ζ"),(17,"η"),(18,"θ"),(19,"ι"),(20,"κ"),
        (21,"λ"),(22,"μ"),(23,"ν"),(24,"ξ"),(25,"π"),(26,"ρ"),(27,"σ"),(28,"τ"),(29,"υ"),(30,"ɸ"),
        (31,"χ"),(32,"ψ"),(33,"ω"),
        (34,"ε"),(35,"ϑ"),(36,"ϖ"),//(37,"ϱ"),(38,"ς"),
        (39,"φ"),//(40,"↼"),(41,"↽"),(42,"⇀"),(43,"⇁"),(44,"𝇋"),(45,"𝇌"),(46,"▹"),(47,"◃"),
        (48,"0"),(49,"1"),
        (50,"2"),(51,"3"),(52,"4"),(53,"5"),(54,"6"),(55,"7"),(56,"8"),(57,"9"),(58,"."),(59,","),
        (60,"<"),(61,"/"),(62,">"),//(63,"*"),
        (64,"∂"),(65,"A"),(66,"B"),(67,"C"),(68,"D"),(69,"E"),
        (70,"F"),(71,"G"),(72,"H"),(73,"I"),(74,"J"),(75,"K"),(76,"L"),(77,"M"),(78,"N"),(79,"O"),
        (80,"P"),(81,"Q"),(82,"R"),(83,"S"),(84,"T"),(85,"U"),(86,"V"),(87,"W"),(88,"X"),(89,"Y"),
        (90,"Z"),//(91,"♭"),(92,"♮"),(93,"♯"),(94,"⌣"),(95,"⁀"),
        (96,"ℓ"),(97,"a"),(98,"b"),(99,"c"),(100,"d"),(101,"e"),(102,"f"),(103,"g"),
        (104,"h"),(105,"i"),(106,"j"),(107,"k"),(108,"l"),(109,"m"),(110,"n"),(111,"o"),(112,"p"),
        (113,"q"),(114,"r"),(115,"s"),(116,"t"),(117,"u"),(118,"v"),(119,"w"),(120,"x"),(121,"y"),
        (122,"z"),(123,"ı"),(124,"ȷ"),(125,"℘")
    ]);
    pub static ref CAPITALS : HashMap<u8,&'static str> = HashMap::from([
        (65,"A"),(66,"B"),(67,"C"),(68,"D"),(69,"E"),(70,"F"),(71,"G"),(72,"H"),(73,"I"),(74,"J"),
        (75,"K"),(76,"L"),(77,"M"),(78,"N"),(79,"O"),(80,"P"),(81,"Q"),(82,"R"),(83,"S"),(84,"T"),
        (85,"U"),(86,"V"),(87,"W"),(88,"X"),(89,"Y"),(90,"Z")
    ]);

    pub static ref JKPSYC : HashMap<u8,&'static str> = HashMap::from([
        (27,"≅"),
        (44,"≠")
    ]);
    pub static ref JKPEXA : HashMap<u8,&'static str> = HashMap::from([]);

    pub static ref LINE : HashMap<u8,&'static str> = HashMap::from([]);
    pub static ref LINEW : HashMap<u8,&'static str> = HashMap::from([]);
    pub static ref LCIRCLE : HashMap<u8,&'static str> = HashMap::from([]);
    pub static ref LCIRCLEW : HashMap<u8,&'static str> = HashMap::from([]);
    pub static ref STMARY : HashMap<u8,&'static str> = HashMap::from([
        (32,"☇")
    ]);
    pub static ref PZDR : HashMap<u8,&'static str> = HashMap::from([
        (72,"★"),(73,"☆"),(74,"✪")
    ]);
    pub static ref PSYR : HashMap<u8,&'static str> = HashMap::from([]);
    pub static ref MANFNT : HashMap<u8,&'static str> = HashMap::from([]);
}