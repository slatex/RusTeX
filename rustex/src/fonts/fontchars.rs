use std::collections::hash_map::Entry;
use std::collections::HashMap;
use std::sync::{Arc, RwLock};
use crate::utils::TeXStr;

#[derive(PartialEq,Copy,Clone)]
pub enum FontTableParam {
    Text,Math,SansSerif,Italic,Bold,Script,Capital,Monospaced,Blackboard,Fraktur,CapitalLetters
}

pub struct FontTable {
    name:TeXStr,
    pub params:Vec<FontTableParam>,
    table:&'static HashMap<u8,&'static str>
}
impl FontTable {
    pub fn as_unicode(&self,_:u8) -> &str {
        todo!()
    }
    pub fn get_char(&self,u:u8) -> &'static str {
        match self.table.get(&u) {
            Some(c) => c,
            None => {
                println!("Unknown character {} in font {}",u,self.name);
                "???"
            }
        }
    }
}

/*macro_rules! table {
    ($s:tt,$o:ident,$name:ident,$($para:expr),*) => (
        Some($o.insert(Arc::new(FontTable {
            params:vec!($($para),*),
            table:&$name,
            name:$s.into()
        })).clone())
    )
}*/
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
                ("rm-lmss",STANDARD_TEXT_CM,FontTableParam::Text,FontTableParam::SansSerif),
                ("cmtt",STANDARD_TEXT_CM,FontTableParam::Text,FontTableParam::Monospaced),
                ("rm-lmtt",STANDARD_TEXT_CM,FontTableParam::Text,FontTableParam::Monospaced),
                ("cmti",STANDARD_TEXT_CM,FontTableParam::Italic),
                ("eufm",STANDARD_TEXT_CM,FontTableParam::Fraktur),
                ("cmbx",STANDARD_TEXT_CM,FontTableParam::Math,FontTableParam::Bold),
                ("rm-lmri",STANDARD_TEXT_CM,FontTableParam::Math,FontTableParam::Italic),
                ("cmmi",STANDARD_MATH_CM,FontTableParam::Math,FontTableParam::Italic),
                ("lmmi",STANDARD_MATH_CM,FontTableParam::Math,FontTableParam::Italic),
                ("cmssi",STANDARD_MATH_CM,FontTableParam::Math,FontTableParam::Italic,FontTableParam::SansSerif),
                ("mathkerncmssi",STANDARD_MATH_CM,FontTableParam::Math,FontTableParam::Italic,FontTableParam::SansSerif),
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
                ("ec-lmbxi",STANDARD_TEXT_EC,FontTableParam::Text,FontTableParam::Bold,FontTableParam::Italic),
                ("ectt",STANDARD_TEXT_EC,FontTableParam::Text,FontTableParam::Monospaced),
                ("ec-lmtt",STANDARD_TEXT_EC,FontTableParam::Text,FontTableParam::Monospaced),
                ("ecit",STANDARD_TEXT_EC,FontTableParam::Text,FontTableParam::Italic),
                ("ecsl",STANDARD_TEXT_EC,FontTableParam::Text,FontTableParam::Italic),
                ("ec-lmri",STANDARD_TEXT_EC,FontTableParam::Text,FontTableParam::Italic),
                ("ecsl",STANDARD_TEXT_EC,FontTableParam::Text,FontTableParam::Italic),
                ("ecsx",STANDARD_TEXT_EC,FontTableParam::Text,FontTableParam::SansSerif,FontTableParam::Bold),
                ("ec-lmssbx",STANDARD_TEXT_EC,FontTableParam::Text,FontTableParam::SansSerif,FontTableParam::Bold),
                ("ecti",STANDARD_TEXT_EC,FontTableParam::Text,FontTableParam::Italic),
                ("ec-lmtti",STANDARD_TEXT_EC,FontTableParam::Text,FontTableParam::Italic),
                ("ec-lmtk",STANDARD_TEXT_EC,FontTableParam::Text,FontTableParam::Monospaced,FontTableParam::Bold),
                ("ec-lmtto",STANDARD_TEXT_EC,FontTableParam::Text,FontTableParam::Monospaced,FontTableParam::Italic),
                ("ec-lmro",STANDARD_TEXT_EC,FontTableParam::Text,FontTableParam::Italic),
                ("ec-lmtlc",STANDARD_TEXT_EC,FontTableParam::Text,FontTableParam::Monospaced),
                // math --------------------------------------------------------------------
                ("cmsy",MATH_CMSY,FontTableParam::Math,FontTableParam::CapitalLetters,FontTableParam::Script),
                ("lmsy",MATH_CMSY,FontTableParam::Math,FontTableParam::CapitalLetters,FontTableParam::Script),
                ("cmex",CMEX,FontTableParam::Math),
                ("cmbx",CMEX,FontTableParam::Math,FontTableParam::Bold),
                ("lmex",CMEX,FontTableParam::Math),
                ("tcrm",MATH_TC,FontTableParam::Math),
                ("tcss",MATH_TC,FontTableParam::Math,FontTableParam::SansSerif),
                ("tcsl",MATH_TC,FontTableParam::Math,FontTableParam::Italic),
                ("MnSymbolA",MNSYMBOL_A,FontTableParam::Math),
                ("MnSymbolB",MNSYMBOL_B,FontTableParam::Math),
                ("MnSymbolC",MNSYMBOL_C,FontTableParam::Math),
                ("MnSymbolD",MNSYMBOL_D,FontTableParam::Math),
                ("MnSymbolE",MNSYMBOL_E,FontTableParam::Math),
                ("MnSymbolF",MNSYMBOL_F,FontTableParam::Math),
                ("MnSymbolS",MNSYMBOL_S,FontTableParam::Math,FontTableParam::CapitalLetters,FontTableParam::Script),
                ("msam",MSAM,FontTableParam::Math),
                ("msbm",MSBM,FontTableParam::Math,FontTableParam::CapitalLetters,FontTableParam::Blackboard),
                ("stmary",STMARY,FontTableParam::Math),
                ("ts-lmr",TS1_LM,FontTableParam::Math),
                ("ts-lmss",TS1_LM,FontTableParam::Math,FontTableParam::SansSerif),
                ("ts-lmbx",TS1_LM,FontTableParam::Math,FontTableParam::Bold),
                ("pzdr",PZDR,FontTableParam::Math),
                ("psyr",PSYR,FontTableParam::Math),
                ("manfnt",MANFNT,FontTableParam::Math),
                ("line",LINE,FontTableParam::Math),
                ("linew",LINEW,FontTableParam::Math),
                ("lcircle",LCIRCLE,FontTableParam::Math),
                ("lcirclew",LCIRCLEW,FontTableParam::Math),
                ("bbm",BBM,FontTableParam::Math,FontTableParam::Blackboard),
                ("wasy",WASY,FontTableParam::Math)
                ;{
                    println!("Warning: No character table for font {}",name);
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
        (118,"v"),(119,"w"),(120,"x"),(121,"y"),(122,"z"),(123,"–"),(124,"—"),(125," ̋"),(126," ̃"),
        (127," ̈")
    ]);
    pub static ref STANDARD_TEXT_EC : HashMap<u8,&'static str> = HashMap::from([
        (0,"`"),(1," ́"),(2,"^"),(3," ̃"),(4," ̈"),(5," ̋"),(6," ̊"),(7,"ˇ"),(8," ̆"),(9," ̄"),(10," ̇"),
        (11," ̧"),(12," ̨"),(13,","),(14,"<"),(15,">"),(16,"“"),(17,"”"),(18,"„"),(19,"«"),(20,"»"),
        (21,"-"),(22,"―"),(23,""),(24,"。"),(25,"ı"),(26,"ȷ"),(27,"ff"),(28,"fi"),(29,"fl"),(30,"ffi"),
        (31,"ffl"),(32,"␣"),(33,"!"),(34,"\""),(35,"#"),(36,"$"),(37,"%"),(38,"&"),(39,"’"),(40,"("),
        (41,")"),(42,"*"),(43,"+"),(44,","),(45,"-"),(46,"."),(47,"/"),(48,"0"),(49,"1"),
        (50,"2"),(51,"3"),(52,"4"),(53,"5"),(54,"6"),(55,"7"),(56,"8"),(57,"9"),(58,":"),(59,";"),
        (60,"¡"),(61,"="),(62,"¿"),(63,"?"),(64,"@"),(65,"A"),(66,"B"),(67,"C"),(68,"D"),(69,"E"),
        (70,"F"),(71,"G"),(72,"H"),(73,"I"),(74,"J"),(75,"K"),(76,"L"),(77,"M"),(78,"N"),(79,"O"),
        (80,"P"),(81,"Q"),(82,"R"),(83,"S"),(84,"T"),(85,"U"),(86,"V"),(87,"W"),(88,"X"),(89,"Y"),
        (90,"Z"),(91,"["),(92,"\\"),(93,"]"),(94,"^"),(95,"_"),(96,"‘"),(97,"a"),(98,"b"),(99,"c"),
        (100,"d"),(101,"e"),(102,"f"),(103,"g"),(104,"h"),(105,"i"),(106,"j"),(107,"k"),(108,"l"),
        (109,"m"),(110,"n"),(111,"o"),(112,"p"),(113,"q"),(114,"r"),(115,"s"),(116,"t"),(117,"u"),
        (118,"v"),(119,"w"),(120,"x"),(121,"y"),(122,"z"),(123,"{"),(124,"|"),(125,"}"),(124,"|"),
        (126," ̃"),(127,"-"),(128,"Ă"),(129,"A̧"),(130,"Ć"),(131,"Č"),(132,"Ď"),(133,"Ě"),(134,"Ȩ"),
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
        (60,"<"),(61,"/"),(62,">"),(63,"*"),(64,"𝜕"),(65,"A"),(66,"B"),(67,"C"),(68,"D"),(69,"E"),
        (70,"F"),(71,"G"),(72,"H"),(73,"I"),(74,"J"),(75,"K"),(76,"L"),(77,"M"),(78,"N"),(79,"O"),
        (80,"P"),(81,"Q"),(82,"R"),(83,"S"),(84,"T"),(85,"U"),(86,"V"),(87,"W"),(88,"X"),(89,"Y"),
        (90,"Z"),(91,"♭"),(92,"♮"),(93,"♯"),
        (95,"⁀"),(96,"ℓ"),(97,"a"),(98,"b"),(99,"c"),(100,"d"),(101,"e"),(102,"f"),(103,"g"),
        (104,"h"),(105,"i"),(106,"j"),(107,"k"),(108,"l"),(109,"m"),(110,"n"),(111,"o"),(112,"p"),
        (113,"q"),(114,"r"),(115,"s"),(116,"t"),(117,"u"),(118,"v"),(119,"w"),(120,"x"),(121,"y"),
        (122,"z"),(123,"ı"),(124,"ȷ"),(125,"℘"),(126," ⃗"),(127,"⁀"),
        (191,""),(214,"")
    ]);

    pub static ref MATH_CMSY : HashMap<u8,&'static str> = HashMap::from([
        (0,"−"),(1,"·"),(2,"×"),(3,"*"),
        (6,"±"),
        (10,"⊗"),
        (14,"◦"),(15,"•"),
        (17,"≡"),(18,"⊆"),(19,"⊇"),(20,"≤"),(21,"≥"),(22,"≼"),(23,"≽"),(24,"∼"),(25,"≈"),(26,"⊂"),
        (27,"⊃"),(28,"≪"),(29,"≫"),(30,"≺"),(31,"≻"),(32,"←"),(33,"→"),(34,"↑"),(35,"↓"),(36,"↔"),
        (39,"≃"),(40,"⇐"),(41,"⇒"),(42,"⇑"),(43,"⇓"),(44,"⇔"),(45,"⭦"),(46,"⭩"),(47,"∝"),(48,"\'"),
        (49,"∞"),(50,"∊"),(51,"∍"),(52,"△"),(53,"▽"),(54,"̸"),(55,"/"),(56,"∀"),(57,"∃"),(58,"¬"),
        (59,"∅"),
        (62,"⊤"),(63,"⊥"),
        (65,"A"),(66,"B"),(67,"C"),(68,"D"),(69,"E"),(70,"F"),(71,"G"),(72,"H"),(73,"I"),(74,"J"),
        (75,"K"),(76,"L"),(77,"M"),(78,"N"),(79,"O"),(80,"P"),(81,"Q"),(82,"R"),(83,"S"),(84,"T"),
        (85,"U"),(86,"V"),(87,"W"),(88,"X"),(89,"Y"),(90,"Z"),
        (91,"∪"),(92,"∩"),(93,"⊎"),(94,"∧"),(95,"∨"),(96,"⊢"),(97,"⊣"),(98,"⌊"),(99,"⌋"),(100,"⌈"),
        (101,"⌉"),(102,"{"),(103,"}"),(104,"〈"),(105,"〉"),(106,"|"),(107,"∥"),(108,"↕"),(109,"⇕"),
        (110,"\\"),(111,"≀"),(112,"√"),(113,"⨿"),(114,"∇"),(115,"∫"),(116,"⊔"),(117,"⊓"),(118,"⊑"),
        (119,"⊒"),(120,"§"),(121,"†"),(122,"‡")
    ]);

    pub static ref CMEX : HashMap<u8,&'static str> = HashMap::from([
        (80,"∑"),(81,"∏"),(82,"∫"),(83,"⋃"),(84,"⋂"),
        (86,"⋀"),(87,"⋁"),
        (98,"^"),
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
        (96,"〈"),
        (101,"〉"),
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
        (32,"⇝"),
        (72,"▼"),(73,"▶"),(74,"◀"),
        (78,"▲")
    ]);
    pub static ref MSBM : HashMap<u8,&'static str> = HashMap::from([
        (65,"A"),(66,"B"),(67,"C"),(68,"D"),(69,"E"),(70,"F"),(71,"G"),(72,"H"),(73,"I"),(74,"J"),
        (75,"K"),(76,"L"),(77,"M"),(78,"N"),(79,"O"),(80,"P"),(81,"Q"),(82,"R"),(83,"S"),(84,"T"),
        (85,"U"),(86,"V"),(87,"W"),(88,"X"),(89,"Y"),(90,"Z"),(91,"^"),
        (97,"ວ"),
        (108,"⋖"),(109,"⋗"),(110,"⋉"),(111,"⋊"),
        (117,"≊"),
        (120,"↶"),(121,"↷")
    ]);
    pub static ref TS1_LM : HashMap<u8,&'static str> = HashMap::from([
        (42,"*"),
        (61,"-"),
        (132,"†"),(133,"‡"),
        (136,"•")
    ]);
    pub static ref WASY : HashMap<u8,&'static str> = HashMap::from([
        (3,"▷"),
        (25,"♀"),(26,"♂"),
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


    pub static ref LINE : HashMap<u8,&'static str> = HashMap::from([]);
    pub static ref LINEW : HashMap<u8,&'static str> = HashMap::from([]);
    pub static ref LCIRCLE : HashMap<u8,&'static str> = HashMap::from([]);
    pub static ref LCIRCLEW : HashMap<u8,&'static str> = HashMap::from([]);
    pub static ref STMARY : HashMap<u8,&'static str> = HashMap::from([]);
    pub static ref PZDR : HashMap<u8,&'static str> = HashMap::from([]);
    pub static ref PSYR : HashMap<u8,&'static str> = HashMap::from([]);
    pub static ref MANFNT : HashMap<u8,&'static str> = HashMap::from([]);

    // ---------------------------------------------------------------------------------------------

    pub static ref SCRIPT : HashMap<char,char> = HashMap::from([
        ('A','𝓐'),('B','𝓑'),('C','𝓒'),('D','𝓓'),('E','𝓔'),('F','𝓕'),('G','𝓖'),('H','𝓗'),('I','𝓘'),
        ('J','𝓙'),('K','𝓚'),('L','𝓛'),('M','𝓜'),('N','𝓝'),('O','𝓞'),('P','𝓟'),('Q','𝓠'),('R','𝓡'),
        ('S','𝓢'),('T','𝓣'),('U','𝓤'),('V','𝓥'),('W','𝓦'),('X','𝓧'),('Y','𝓨'),('Z','𝓩')
    ]);
    pub static ref BOLD : HashMap<char,char> = HashMap::from([
        ('Γ','𝚪'),('∆','𝚫'),('Θ','𝚯'),('Λ','𝚲'),('Ξ','𝚵'),('Π','𝚷'),('Σ','𝚺'),('Υ','𝚼'),('Φ','𝚽'),
        ('Ψ','𝚿'),('Ω','𝛀'),
        ('α','𝛂'),('β','𝛃'),('γ','𝛄'),('δ','𝛅'),('ε','𝛆'),('ζ','𝛇'),('η','𝛈'),('θ','𝛉'),('ι','𝛊'),
        ('κ','𝛋'),('λ','𝛌'),('μ','𝛍'),('ν','𝛎'),('ξ','𝛏'),('ο','𝛐'),('π','𝛑'),('ρ','𝛒'),('σ','𝛔'),
        ('τ','𝛕'),('υ','𝛖'),('φ','𝛗'),('χ','𝛘'),('ψ','𝛙'),('ω','𝛚'),

        ('0','𝟎'),('1','𝟏'),('2','𝟐'),('3','𝟑'),('4','𝟒'),('5','𝟓'),('6','𝟔'),('7','𝟕'),('8','𝟖'),
        ('9','𝟗'),
        ('A','𝐀'),('B','𝐁'),('C','𝐂'),('D','𝐃'),('E','𝐄'),('F','𝐅'),('G','𝐆'),('H','𝐇'),('I','𝐈'),
        ('J','𝐉'),('K','𝐊'),('L','𝐋'),('M','𝐌'),('N','𝐍'),('O','𝐎'),('P','𝐏'),('Q','𝐐'),('R','𝐑'),
        ('S','𝐒'),('T','𝐓'),('U','𝐔'),('V','𝐕'),('W','𝐖'),('X','𝐗'),('Y','𝐘'),('Z','𝐙'),
        ('a','𝐚'),('b','𝐛'),('c','𝐜'),('d','𝐝'),('e','𝐞'),('f','𝐟'),('g','𝐠'),('h','𝐡'),('i','𝐢'),
        ('j','𝐣'),('k','𝐤'),('l','𝐥'),('m','𝐦'),('n','𝐧'),('o','𝐨'),('p','𝐩'),('q','𝐪'),('r','𝐫'),
        ('s','𝐬'),('t','𝐭'),('u','𝐮'),('v','𝐯'),('w','𝐰'),('x','𝐱'),('y','𝐲'),('z','𝐳')
    ]);
    pub static ref BOLD_ITALIC : HashMap<char,char> = HashMap::from([
        ('Γ','𝜞'),('∆','𝜟'),('Θ','𝜣'),('Λ','𝜦'),('Ξ','𝜩'),('Π','𝜫'),('Σ','𝜮'),('Υ','𝜰'),('Φ','𝜱'),
        ('Ψ','𝜳'),('Ω','𝜴'),
        ('α','𝜶'),('β','𝜷'),('γ','𝜸'),('δ','𝜹'),('ε','𝜺'),('ζ','𝜻'),('η','𝜼'),('θ','𝜽'),('ι','𝜾'),
        ('κ','𝜿'),('λ','𝝀'),('μ','𝝁'),('ν','𝝂'),('ξ','𝝃'),('ο','𝝄'),('π','𝝅'),('ρ','𝝆'),('σ','𝝈'),
        ('τ','𝝉'),('υ','𝝊'),('φ','𝝋'),('χ','𝝌'),('ψ','𝝍'),('ω','𝝎'),

        ('0','𝟎'),('1','𝟏'),('2','𝟐'),('3','𝟑'),('4','𝟒'),('5','𝟓'),('6','𝟔'),('7','𝟕'),('8','𝟖'),
        ('9','𝟗'),
        ('A','𝑨'),('B','𝑩'),('C','𝑪'),('D','𝑫'),('E','𝑬'),('F','𝑭'),('G','𝑮'),('H','𝑯'),('I','𝑰'),
        ('J','𝑱'),('K','𝑲'),('L','𝑳'),('M','𝑴'),('N','𝑵'),('O','𝑶'),('P','𝑷'),('Q','𝑸'),('R','𝑹'),
        ('S','𝑺'),('T','𝑻'),('U','𝑼'),('V','𝑽'),('W','𝑾'),('X','𝑿'),('Y','𝒀'),('Z','𝒁'),
        ('a','𝒂'),('b','𝒃'),('c','𝒄'),('d','𝒅'),('e','𝒆'),('f','𝒇'),('g','𝒈'),('h','𝒉'),('i','𝒊'),
        ('j','𝒋'),('k','𝒌'),('l','𝒍'),('m','𝒎'),('n','𝒏'),('o','𝒐'),('p','𝒑'),('q','𝒒'),('r','𝒓'),
        ('s','𝒔'),('t','𝒕'),('u','𝒖'),('v','𝒗'),('w','𝒘'),('x','𝒙'),('y','𝒚'),('z','𝒛')
    ]);
    pub static ref CAPITAL : HashMap<char,char> = HashMap::from([
        ('A','𝖠'),('B','𝖡'),('C','𝖢'),('D','𝖣'),('E','𝖤'),('F','𝖥'),('G','𝖦'),('H','𝖧'),('I','𝖨'),
        ('J','𝖩'),('K','𝖪'),('L','𝖫'),('M','𝖬'),('N','𝖭'),('O','𝖮'),('P','𝖯'),('Q','𝖰'),('R','𝖱'),
        ('S','𝖲'),('T','𝖳'),('U','𝖴'),('V','𝖵'),('W','𝖶'),('X','𝖷'),('Y','𝖸'),('Z','𝖹'),
        ('a','ᴀ'),('b','ʙ'),('c','ᴄ'),('d','ᴅ'),('e','ᴇ'),('f','ꜰ'),('g','ɢ'),('h','ʜ'),('i','ɪ'),
        ('j','ᴊ'),('k','ᴋ'),('l','ʟ'),('m','ᴍ'),('n','ɴ'),('o','ᴏ'),('p','ᴘ'),('q','Q'),('r','ʀ'),
        ('s','ꜱ'),('t','ᴛ'),('u','ᴜ'),('v','ᴠ'),('w','ᴡ'),('x','𝗑'),('y','ʏ'),('z','ᴢ')
    ]);
    pub static ref FRAKTUR : HashMap<char,char> = HashMap::from([
        ('A','𝕬'),('B','𝕭'),('C','𝕮'),('D','𝕯'),('E','𝕰'),('F','𝕱'),('G','𝕲'),('H','𝕳'),('I','𝕴'),
        ('J','𝕵'),('K','𝕶'),('L','𝕷'),('M','𝕸'),('N','𝕹'),('O','𝕺'),('P','𝕻'),('Q','𝕼'),('R','𝕽'),
        ('S','𝕾'),('T','𝕿'),('U','𝖀'),('V','𝖁'),('W','𝖂'),('X','𝖃'),('Y','𝖄'),('Z','𝖅'),
        ('a','𝖆'),('b','𝖇'),('c','𝖈'),('d','𝖉'),('e','𝖊'),('f','𝖋'),('g','𝖌'),('h','𝖍'),('i','𝖎'),
        ('j','𝖏'),('k','𝖐'),('l','𝖑'),('m','𝖒'),('n','𝖓'),('o','𝖔'),('p','𝖕'),('q','𝖖'),('r','𝖗'),
        ('s','𝖘'),('t','𝖙'),('u','𝖚'),('v','𝖛'),('w','𝖜'),('x','𝖝'),('y','𝖞'),('z','𝖟')
    ]);

    // Italic 𝛢 𝛣 𝛤 𝛥 𝛦 𝛧 𝛨 𝛩 𝛪 𝛫 𝛬 𝛭 𝛮 𝛯 𝛰 𝛱 𝛲 𝛴 𝛵 𝛶 𝛷 𝛸 𝛹 𝛺 𝛼 𝛽 𝛾 𝛿 𝜀 𝜁 𝜂 𝜃 𝜄 𝜅 𝜆 𝜇 𝜈 𝜉 𝜊 𝜋 𝜌 𝜎 𝜏 𝜐 𝜑 𝜒 𝜓 𝜔
    // Sans-Serif Bold 𝝖 𝝗 𝝘 𝝙 𝝚 𝝛 𝝜 𝝝 𝝞 𝝟 𝝠 𝝡 𝝢 𝝣 𝝤 𝝥 𝝦 𝝨 𝝩 𝝪 𝝫 𝝬 𝝭 𝝮 𝝰 𝝱 𝝲 𝝳 𝝴 𝝵 𝝶 𝝷 𝝸 𝝹 𝝺 𝝻 𝝼 𝝽 𝝾 𝝿 𝞀 𝞂 𝞃 𝞄 𝞅 𝞆 𝞇 𝞈
    // Sans-Serif Bold Italic 𝞐 𝞑 𝞒 𝞓 𝞔 𝞕 𝞖 𝞗 𝞘 𝞙 𝞚 𝞛 𝞜 𝞝 𝞞 𝞟 𝞠 𝞢 𝞣 𝞤 𝞥 𝞦 𝞧 𝞨 𝞪 𝞫 𝞬 𝞭 𝞮 𝞯 𝞰 𝞱 𝞲 𝞳 𝞴 𝞵 𝞶 𝞷 𝞸 𝞹 𝞺 𝞼 𝞽 𝞾 𝞿 𝟀 𝟁 𝟂
}