use std::collections::HashMap;
use ahash::RandomState;
use crate::fonts::fontchars::FontTableParam;

static all : &'static str = "abcdefghijklmnopqrstuvwxyzABCDEFGHIJKLMNOPQRSTUVWXYZ0123456789,;.:-_#'+*~'`ß\\?=}])[({/&%$§\"!^°äöüÄÖÜ€@<>|ĂA̧ĆČĎĚȨĞĹL̛ŁŃŇŐŔŘŚŠŞŤŢŰŮŸŹŽŻİđăa̧ćčd̛ĕȩğĺl̛łńňőŕřśšşt̛ţűůÿźžż¡¿£ÀÁÂÃÅÆÇÈÉÊËÌÍÎÏÐÑÒÓÔÕŒØÙÚÛÝÞàáâãäåæçèéêëìíîïðñòóôõœøùúûýþΓ∆ΘΛΞΠΣΥΦΨΩαβγδϵζηθικλμνξπρστυɸχψωεϑϖϱςφ";
static script: &'static str = "𝒶𝒷𝒸𝒹ℯ𝒻ℊ𝒽𝒾𝒿𝓀𝓁𝓂𝓃ℴ𝓅𝓆𝓇𝓈𝓉𝓊𝓋𝓌𝓍𝓎𝓏𝒜ℬ𝒞𝒟ℰℱ𝒢ℋℐ𝒥𝒦ℒℳ𝒩𝒪𝒫𝒬ℛ𝒮𝒯𝒰𝒱𝒲𝒳𝒴𝒵0123456789,;.:-_#'+*~'`ß\\?=}])[({/&%$§\"!^°𝒶̈ℴ̈𝓊̈𝒜̈𝒪̈𝒰̈€@<>|𝒜̆𝒜̧𝒞́𝒞̌𝒟̌ℰ̌ℰ̧𝒢̆ℒ́ℒ̛Ł𝒩́𝒩̌𝒪̋ℛ́ℛ̌𝒮́𝒮̌𝒮̧𝒯̌𝒯̧𝒰̋𝒰̊𝒴̈𝒵́𝒵̌𝒵̇ℐ̇đ𝒶̆𝒶̧𝒸́𝒸̌𝒹̛ℯ̆ℯ̧ℊ̆𝓁́𝓁̛ł𝓃́𝓃̌ℴ̋𝓇́𝓇̌𝓈́𝓈̌𝓈̧𝓉̛𝓉̧𝓊̋𝓊̊𝓎̈𝓏́𝓏̌𝓏̇¡¿£𝒜̀𝒜́𝒜̂𝒜̃𝒜̊Æ𝒞̧ℰ̀ℰ́ℰ̂ℰ̈ℐ̀ℐ́ℐ̂ℐ̈Ð𝒩̃𝒪̀𝒪́𝒪̂𝒪̃ŒØ𝒰̀𝒰́𝒰̂𝒴́Þ𝒶̀𝒶́𝒶̂𝒶̃𝒶̈𝒶̊æ𝒸̧ℯ̀ℯ́ℯ̂ℯ̈𝒾̀𝒾́𝒾̂𝒾̈ð𝓃̃ℴ̀ℴ́ℴ̂ℴ̃œø𝓊̀𝓊́𝓊̂𝓎́þΓ∆ΘΛΞΠΣΥΦΨΩαβγδϵζηθικλμνξπρστυɸχψωεϑϖϱςφ";

fn str_to_map(s:&str,tg:&str,map:&mut HashMap<char,char/*,RandomState*/>) {
    let mut tgv = tg.chars();
    for c in s.chars() {
        map.insert(c,tgv.next().unwrap());
    }
}

pub fn convert(s: &str,modifiers: &Vec<FontTableParam>) -> String {
    s.chars().map(|c| convert_char(c,modifiers)).collect()
}
fn convert_char(c : char,modifiers:&Vec<FontTableParam>) -> char {
        macro_rules! get { ($($tl:ident),* => $map:expr) => {
            if $(modifiers.contains(&FontTableParam::$tl))&&* {
            if modifiers.contains(&FontTableParam::CapitalLetters) {
                if c.is_ascii_uppercase() {
                    match $map.get(&c) {
                        Some(nc) => return nc.clone(),
                        _ => ()
                    }
                }
            } else {
                match $map.get(&c) {
                    Some(nc) => return nc.clone(),
                    _ => ()
                }
            }
        }}
    }
    get!(Blackboard => BLACKBOARD);
    get!(Fraktur,Bold => BOLD_FRAKTUR);
    get!(Fraktur => FRAKTUR);
    get!(Script,Bold => BOLD_SCRIPT);
    get!(Script => SCRIPT);
    get!(Bold,Italic,SansSerif => BOLD_ITALIC_SANS);
    get!(Bold,Italic => BOLD_ITALIC);
    get!(Bold,SansSerif => BOLD_SANS);
    get!(Bold => BOLD);
    get!(Capital => CAPITAL);
    get!(Monospaced => MONOSPACED);
    get!(Italic,SansSerif => ITALIC_SANS);
    get!(Italic => ITALIC);
    get!(SansSerif => SANS);
    c
}

macro_rules! stringtomap {
    ($sel:ident = $src:expr => $trg:expr) => {
        lazy_static! {
            pub static ref $sel : HashMap<char,char> = {
                let mut map = HashMap::<char,char>::default();
                str_to_map($src,$trg,&mut map);
                map
            };
        }
    }
}
// https://yaytext.com/script/
// https://en.wikipedia.org/wiki/Mathematical_Alphanumeric_Symbols
/* done */ stringtomap!(SCRIPT = "abcdefghijklmnopqrstuvwxyzABCDEFGHIJKLMNOPQRSTUVWXYZ"
    => "𝒶𝒷𝒸𝒹ℯ𝒻ℊ𝒽𝒾𝒿𝓀𝓁𝓂𝓃ℴ𝓅𝓆𝓇𝓈𝓉𝓊𝓋𝓌𝓍𝓎𝓏𝒜ℬ𝒞𝒟ℰℱ𝒢ℋℐ𝒥𝒦ℒℳ𝒩𝒪𝒫𝒬ℛ𝒮𝒯𝒰𝒱𝒲𝒳𝒴𝒵");
/* done */ stringtomap!(BOLD = "abcdefghijklmnopqrstuvwxyzABCDEFGHIJKLMNOPQRSTUVWXYZ0123456789Γ∆ΘΛΞΠϴΣΥΦΨΩαβγδϵζηθικλμνξπρστυφχψω∂ϵϑϰϕϱϖ"
    => "𝐚𝐛𝐜𝐝𝐞𝐟𝐠𝐡𝐢𝐣𝐤𝐥𝐦𝐧𝐨𝐩𝐪𝐫𝐬𝐭𝐮𝐯𝐰𝐱𝐲𝐳𝐀𝐁𝐂𝐃𝐄𝐅𝐆𝐇𝐈𝐉𝐊𝐋𝐌𝐍𝐎𝐏𝐐𝐑𝐒𝐓𝐔𝐕𝐖𝐗𝐘𝐙𝟎𝟏𝟐𝟑𝟒𝟓𝟔𝟕𝟖𝟗𝚪𝚫𝚯𝚲𝚵𝚷𝚹𝚺𝚼𝚽𝚿𝛀𝛂𝛃𝛄𝛅𝛆𝛇𝛈𝛉𝛊𝛋𝛌𝛍𝛎𝛏𝛑𝛒𝛔𝛕𝛖𝛗𝛘𝛙𝛚𝛛𝛜𝛝𝛞𝛟𝛠𝛡");
stringtomap!(ITALIC = "abcdefghijklmnopqrstuvwxyzABCDEFGHIJKLMNOPQRSTUVWXYZΓ∆ΘΛΞΠϴΣΥΦΨΩαβγδϵζηθικλμνξπρστυφχψω∂ϵϑϰϕϱϖ"
    => "𝑎𝑏𝑐𝑑𝑒𝑓𝑔ℎ𝑖𝑗𝑘𝑙𝑚𝑛𝑜𝑝𝑞𝑟𝑠𝑡𝑢𝑣𝑤𝑥𝑦𝑧𝐴𝐵𝐶𝐷𝐸𝐹𝐺𝐻𝐼𝐽𝐾𝐿𝑀𝑁𝑂𝑃𝑄𝑅𝑆𝑇𝑈𝑉𝑊𝑋𝑌𝑍𝛤𝛥𝛩𝛬𝛯𝛱𝛳𝛴𝛶𝛷𝛹𝛺𝛼𝛽𝛾𝛿𝜀𝜁𝜂𝜃𝜄𝜅𝜆𝜇𝜈𝜉𝜋𝜌𝜎𝜏𝜐𝜑𝜒𝜓𝜔𝜕𝜖𝜗𝜘𝜙𝜚𝜛");
stringtomap!(SANS = "abcdefghijklmnopqrstuvwxyzABCDEFGHIJKLMNOPQRSTUVWXYZ0123456789"
    => "𝖺𝖻𝖼𝖽𝖾𝖿𝗀𝗁𝗂𝗃𝗄𝗅𝗆𝗇𝗈𝗉𝗊𝗋𝗌𝗍𝗎𝗏𝗐𝗑𝗒𝗓𝖠𝖡𝖢𝖣𝖤𝖥𝖦𝖧𝖨𝖩𝖪𝖫𝖬𝖭𝖮𝖯𝖰𝖱𝖲𝖳𝖴𝖵𝖶𝖷𝖸𝖹𝟢𝟣𝟤𝟥𝟦𝟧𝟨𝟩𝟪𝟫");
/* done */stringtomap!(CAPITAL = "abcdefghijklmnopqrstuvwxyzABCDEFGHIJKLMNOPQRSTUVWXYZ"
    => "ᴀʙᴄᴅᴇғɢʜɪᴊᴋʟᴍɴᴏᴘǫʀsᴛᴜᴠᴡxʏᴢ𝖠𝖡𝖢𝖣𝖤𝖥𝖦𝖧𝖨𝖩𝖪𝖫𝖬𝖭𝖮𝖯𝖰𝖱𝖲𝖳𝖴𝖵𝖶𝖷𝖸𝖹");
/* done */ stringtomap!(FRAKTUR = "abcdefghijklmnopqrstuvwxyzABCDEFGHIJKLMNOPQRSTUVWXYZ"
    => "𝔞𝔟𝔠𝔡𝔢𝔣𝔤𝔥𝔦𝔧𝔨𝔩𝔪𝔫𝔬𝔭𝔮𝔯𝔰𝔱𝔲𝔳𝔴𝔵𝔶𝔷𝔄𝔅ℭ𝔇𝔈𝔉𝔊ℌℑ𝔍𝔎𝔏𝔐𝔑𝔒𝔓𝔔ℜ𝔖𝔗𝔘𝔙𝔚𝔛𝔜ℨ");
/* done */ stringtomap!(BOLD_SCRIPT = "abcdefghijklmnopqrstuvwxyzABCDEFGHIJKLMNOPQRSTUVWXYZ"
    => "𝓪𝓫𝓬𝓭𝓮𝓯𝓰𝓱𝓲𝓳𝓴𝓵𝓶𝓷𝓸𝓹𝓺𝓻𝓼𝓽𝓾𝓿𝔀𝔁𝔂𝔃𝓐𝓑𝓒𝓓𝓔𝓕𝓖𝓗𝓘𝓙𝓚𝓛𝓜𝓝𝓞𝓟𝓠𝓡𝓢𝓣𝓤𝓥𝓦𝓧𝓨𝓩");
/* done */ stringtomap!(BOLD_ITALIC = "abcdefghijklmnopqrstuvwxyzABCDEFGHIJKLMNOPQRSTUVWXYZΓ∆ΘΛΞΠϴΣΥΦΨΩαβγδϵζηθικλμνξπρστυφχψω∂ϵϑϰϕϱϖ"
    => "𝒂𝒃𝒄𝒅𝒆𝒇𝒈𝒉𝒊𝒋𝒌𝒍𝒎𝒏𝒐𝒑𝒒𝒓𝒔𝒕𝒖𝒗𝒘𝒙𝒚𝒛𝑨𝑩𝑪𝑫𝑬𝑭𝑮𝑯𝑰𝑱𝑲𝑳𝑴𝑵𝑶𝑷𝑸𝑹𝑺𝑻𝑼𝑽𝑾𝑿𝒀𝒁𝜞𝜟𝜣𝜦𝜩𝜫𝜭𝜮𝜰𝜱𝜳𝜴𝜶𝜷𝜸𝜹𝜺𝜻𝜼𝜽𝜾𝜿𝝀𝝁𝝂𝝃𝝅𝝆𝝈𝝉𝝊𝝋𝝌𝝍𝝎𝝏𝝐𝝑𝝒𝝓𝝔𝝕");
/* done */ stringtomap!(BOLD_SANS = "abcdefghijklmnopqrstuvwxyzABCDEFGHIJKLMNOPQRSTUVWXYZ0123456789Γ∆ΘΛΞΠϴΣΥΦΨΩαβγδϵζηθικλμνξπρστυφχψω∂ϵϑϰϕϱϖ"
    => "𝗮𝗯𝗰𝗱𝗲𝗳𝗴𝗵𝗶𝗷𝗸𝗹𝗺𝗻𝗼𝗽𝗾𝗿𝘀𝘁𝘂𝘃𝘄𝘅𝘆𝘇𝗔𝗕𝗖𝗗𝗘𝗙𝗚𝗛𝗜𝗝𝗞𝗟𝗠𝗡𝗢𝗣𝗤𝗥𝗦𝗧𝗨𝗩𝗪𝗫𝗬𝗭𝟬𝟭𝟮𝟯𝟰𝟱𝟲𝟳𝟴𝟵𝝘𝝙𝝝𝝠𝝣𝝥𝝧𝝨𝝪𝝫𝝭𝝮𝝰𝝱𝝲𝝳𝝴𝝵𝝶𝝷𝝸𝝹𝝺𝝻𝝼𝝽𝝿𝞀𝞂𝞃𝞄𝞅𝞆𝞇𝞈𝞉𝞊𝞋𝞌𝞍𝞎𝞏");
/* done */ stringtomap!(BOLD_ITALIC_SANS = "abcdefghijklmnopqrstuvwxyzABCDEFGHIJKLMNOPQRSTUVWXYZΓ∆ΘΛΞΠϴΣΥΦΨΩαβγδϵζηθικλμνξπρστυφχψω∂ϵϑϰϕϱϖ"
    => "𝙖𝙗𝙘𝙙𝙚𝙛𝙜𝙝𝙞𝙟𝙠𝙡𝙢𝙣𝙤𝙥𝙦𝙧𝙨𝙩𝙪𝙫𝙬𝙭𝙮𝙯𝘼𝘽𝘾𝘿𝙀𝙁𝙂𝙃𝙄𝙅𝙆𝙇𝙈𝙉𝙊𝙋𝙌𝙍𝙎𝙏𝙐𝙑𝙒𝙓𝙔𝙕𝞒𝞓𝞗𝞚𝞝𝞟𝞡𝞢𝞤𝞥𝞧𝞨𝞪𝞫𝞬𝞭𝞮𝞯𝞰𝞱𝞲𝞳𝞴𝞵𝞶𝞷𝞹𝞺𝞼𝞽𝞾𝞿𝟀𝟁𝟂𝟃𝟄𝟅𝟆𝟇𝟈𝟉");
stringtomap!(ITALIC_SANS = "abcdefghijklmnopqrstuvwxyzABCDEFGHIJKLMNOPQRSTUVWXYZ"
    => "𝘢𝘣𝘤𝘥𝘦𝘧𝘨𝘩𝘪𝘫𝘬𝘭𝘮𝘯𝘰𝘱𝘲𝘳𝘴𝘵𝘶𝘷𝘸𝘹𝘺𝘻𝘈𝘉𝘊𝘋𝘌𝘍𝘎𝘏𝘐𝘑𝘒𝘓𝘔𝘕𝘖𝘗𝘘𝘙𝘚𝘛𝘜𝘝𝘞𝘟𝘠𝘡");
/* done */ stringtomap!(BOLD_FRAKTUR = "abcdefghijklmnopqrstuvwxyzABCDEFGHIJKLMNOPQRSTUVWXYZ"
    => "𝖆𝖇𝖈𝖉𝖊𝖋𝖌𝖍𝖎𝖏𝖐𝖑𝖒𝖓𝖔𝖕𝖖𝖗𝖘𝖙𝖚𝖛𝖜𝖝𝖞𝖟𝕬𝕭𝕮𝕯𝕰𝕱𝕲𝕳𝕴𝕵𝕶𝕷𝕸𝕹𝕺𝕻𝕼𝕽𝕾𝕿𝖀𝖁𝖂𝖃𝖄𝖅");
/* done */ stringtomap!(MONOSPACED = "abcdefghijklmnopqrstuvwxyzABCDEFGHIJKLMNOPQRSTUVWXYZ0123456789"
    => "𝚊𝚋𝚌𝚍𝚎𝚏𝚐𝚑𝚒𝚓𝚔𝚕𝚖𝚗𝚘𝚙𝚚𝚛𝚜𝚝𝚞𝚟𝚠𝚡𝚢𝚣𝙰𝙱𝙲𝙳𝙴𝙵𝙶𝙷𝙸𝙹𝙺𝙻𝙼𝙽𝙾𝙿𝚀𝚁𝚂𝚃𝚄𝚅𝚆𝚇𝚈𝚉0𝟷𝟸𝟹𝟺𝟻𝟼𝟽𝟾𝟿");
/* done */ stringtomap!(BLACKBOARD = "abcdefghijklmnopqrstuvwxyzABCDEFGHIJKLMNOPQRSTUVWXYZ0123456789"
    => "𝕒𝕓𝕔𝕕𝕖𝕗𝕘𝕙𝕚𝕛𝕜𝕝𝕞𝕟𝕠𝕡𝕢𝕣𝕤𝕥𝕦𝕧𝕨𝕩𝕪𝕫𝔸𝔹ℂ𝔻𝔼𝔽𝔾ℍ𝕀𝕁𝕂𝕃𝕄ℕ𝕆ℙℚℝ𝕊𝕋𝕌𝕍𝕎𝕏𝕐ℤ𝟘𝟙𝟚𝟛𝟜𝟝𝟞𝟟𝟠𝟡");
// roman abcdefghijklmnopqrstuvwxyzABCDEFGHIJKLMNOPQRSTUVWXYZ0123456789,;.:-_#'+*~'`ß\?=}])[({/&%$§"!^°äöüÄÖÜ€@<>|ĂA̧ĆČĎĚȨĞĹL̛ŁŃŇŐŔŘŚŠŞŤŢŰŮŸŹŽŻİđăa̧ćčd̛ĕȩğĺl̛łńňőŕřśšşt̛ţűůÿźžż¡¿£ÀÁÂÃÅÆÇÈÉÊËÌÍÎÏÐÑÒÓÔÕŒØÙÚÛÝÞàáâãäåæçèéêëìíîïðñòóôõœøùúûýþΓ∆ΘΛΞΠΣΥΦΨΩαβγδϵζηθικλμνξπρστυɸχψωεϑϖϱςφ
// Italic 𝛢 𝛣 𝛤 𝛥 𝛦 𝛧 𝛨 𝛩 𝛪 𝛫 𝛬 𝛭 𝛮 𝛯 𝛰 𝛱 𝛲 𝛴 𝛵 𝛶 𝛷 𝛸 𝛹 𝛺 𝛼 𝛽 𝛾 𝛿 𝜀 𝜁 𝜂 𝜃 𝜄 𝜅 𝜆 𝜇 𝜈 𝜉 𝜊 𝜋 𝜌 𝜎 𝜏 𝜐 𝜑 𝜒 𝜓 𝜔
// Sans-Serif Bold 𝝖 𝝗 𝝘 𝝙 𝝚 𝝛 𝝜 𝝝 𝝞 𝝟 𝝠 𝝡 𝝢 𝝣 𝝤 𝝥 𝝦 𝝨 𝝩 𝝪 𝝫 𝝬 𝝭 𝝮 𝝰 𝝱 𝝲 𝝳 𝝴 𝝵 𝝶 𝝷 𝝸 𝝹 𝝺 𝝻 𝝼 𝝽 𝝾 𝝿 𝞀 𝞂 𝞃 𝞄 𝞅 𝞆 𝞇 𝞈
// Sans-Serif Bold Italic 𝞐 𝞑 𝞒 𝞓 𝞔 𝞕 𝞖 𝞗 𝞘 𝞙 𝞚 𝞛 𝞜 𝞝 𝞞 𝞟 𝞠 𝞢 𝞣 𝞤 𝞥 𝞦 𝞧 𝞨 𝞪 𝞫 𝞬 𝞭 𝞮 𝞯 𝞰 𝞱 𝞲 𝞳 𝞴 𝞵 𝞶 𝞷 𝞸 𝞹 𝞺 𝞼 𝞽 𝞾 𝞿 𝟀 𝟁 𝟂
