use chrono::{Datelike, Timelike, TimeZone};
use crate::commands::{AssignableValue, PrimitiveExecutable, Conditional, DimenReference, RegisterReference, NumericCommand, PrimitiveTeXCommand, TokReference, SimpleWhatsit, ProvidesWhatsit, TokenList, PrimitiveAssignment};
use crate::interpreter::{string_to_tokens, TeXMode, tokenize};
use crate::{Interpreter, pdf_to_img, Token, VERSION_INFO};
use crate::{log,TeXErr};
use crate::catcodes::CategoryCode;
use crate::interpreter::dimensions::{dimtostr, Numeric};
use crate::commands::conditionals::{dotrue,dofalse};
use crate::commands::primitives::read_font;
use crate::stomach::groups::{ColorChange, ColorEnd, LinkEnd, PDFLink, PDFMatrixSave, PDFRestore};
use crate::stomach::simple::{PDFDest, PDFImageRule, PDFInfo, PDFLiteral, PDFMatrix, PDFXForm, PDFXImage, SimpleWI};
use crate::stomach::whatsits::{ActionSpec, Whatsit, WhatsitTrait};
use crate::utils::{TeXError, TeXStr};

fn read_attrspec(int:&mut Interpreter) -> Result<Option<TeXStr>,TeXError> {
    int.expand_until(true)?;
    let ret = match int.read_keyword(vec!("attr"))? {
        Some(_) => {
            int.skip_ws();
            let tks = int.read_balanced_argument(true,false,false,false)?;
            Some(int.tokens_to_string(&tks).into())
        },
        None => None
    };
    Ok(ret)
}


fn read_rule_spec(int:&mut Interpreter )-> Result<PDFImageRule,TeXError> {
    let mut ret = "".to_string();
    let mut width : Option<i32> = None;
    let mut height : Option<i32> = None;
    let mut depth : Option<i32> = None;
    loop {
        int.expand_until(true)?;
        match int.read_keyword(vec!("width", "height", "depth"))? {
            Some(s) if s == "width" => {
                let dim = int.read_dimension()?;
                if ret == "" { ret = "width ".to_string() + &dimtostr(dim)} else {
                    ret = ret + " width " + &dimtostr(dim)
                }
                width = Some(dim);
            }
            Some(s) if s == "height" => {
                let dim = int.read_dimension()?;
                if ret == "" { ret = "height ".to_string() + &dimtostr(dim)} else {
                    ret = ret + " height " + &dimtostr(dim)
                }
                height = Some(dim);
            }
            Some(s) if s == "depth" => {
                let dim = int.read_dimension()?;
                if ret == "" { ret = "depth ".to_string() + &dimtostr(dim)} else {
                    ret = ret + " depth " + &dimtostr(dim)
                }
                depth = Some(dim);
            }
            _ => break
        }
    }
    Ok(PDFImageRule {
        string:ret.into(),width,height,depth
    })
}
fn read_resource_spec(int:&mut Interpreter) -> Result<Option<TeXStr>,TeXError> {
    int.expand_until(true)?;
    let ret = match int.read_keyword(vec!("resources"))? {
        Some(_) => {
            int.skip_ws();
            let tks = int.read_balanced_argument(true,false,false,false)?;
            Some(int.tokens_to_string(&tks).into())
        },
        None => None
    };
    Ok(ret)
}

fn read_action_spec(int:&mut Interpreter) -> Result<ActionSpec,TeXError> {
    match int.read_keyword(vec!("user","goto","thread"))? {
        Some(s) if s == "user" => {
            let tks = int.read_balanced_argument(true,false,false,true)?;
            let retstr = int.tokens_to_string(&tks);
            Ok(ActionSpec::User(retstr.into()))
        }
        Some(s) if s == "goto" => {
            match int.read_keyword(vec!("num","file","name","page"))? {
                Some(s) if s == "num" => {
                    let num = int.read_number()?;
                    Ok(ActionSpec::GotoNum(num))
                }
                Some(s) if s == "file" => {
                    let f = int.read_string()?;
                    match int.read_keyword(vec!("name","page"))? {
                        Some(s) if s == "name" => {
                            let name = int.read_string()?;
                            let target = int.read_keyword(vec!("newwindow", "nonewindow"))?;
                            Ok(ActionSpec::File(f.as_str().into(),name.as_str().into(),target.map(|x| x.as_str().into())))
                        }
                        Some(_) => {
                            let num = int.read_number()?;
                            let target = int.read_keyword(vec!("newwindow", "nonewindow"))?;
                            Ok(ActionSpec::FilePage(f.as_str().into(),num,target.map(|x| x.as_str().into())))
                        }
                        _ => TeXErr!("Expected \"name\" or \"page\" in action spec")
                    }
                }
                Some(s) if s == "name" => {
                    let name = int.read_string()?;
                    Ok(ActionSpec::Name(name.as_str().into()))
                }
                Some(_) => {
                    let num = int.read_number()?;
                    let _ = int.read_argument()?;
                    Ok(ActionSpec::Page(num))
                }
                _ => {
                    let ret = int.read_argument()?;
                    TeXErr!("Here: {}",TokenList(&ret))
                }
            }
        }
        Some(_) => {
            match int.read_keyword(vec!("num","file"))? {
                Some(s) if s == "num" => {
                    let num = int.read_number()?;
                    Ok(ActionSpec::GotoNum(num))
                }
                Some(_) => {
                    let f = int.read_string()?;
                    match int.read_keyword(vec!("name","page"))? {
                        Some(s) if s == "name" => {
                            let name = int.read_string()?;
                            let target = int.read_keyword(vec!("newwindow", "nonewindow"))?;
                            Ok(ActionSpec::File(f.as_str().into(),name.as_str().into(),target.map(|x| x.as_str().into())))
                        }
                        Some(_) => {
                            let num = int.read_number()?;
                            let target = int.read_keyword(vec!("newwindow", "nonewindow"))?;
                            Ok(ActionSpec::FilePage(f.as_str().into(),num,target.map(|x| x.as_str().into())))
                        }
                        _ => TeXErr!("Expected \"name\" or \"page\" in action spec")
                    }
                }
                _ => TeXErr!("Expected \"num\" or \"file\" after \"thread\" in action spec")
            }
        }
        _ => TeXErr!("Expected \"user\", \"goto\" or \"thread\" in action spec")
    }
}

pub static PDFTEXVERSION : NumericCommand = NumericCommand {
    _getvalue: |_int| {
        Ok(Numeric::Int(VERSION_INFO.pdftexversion.to_string().parse().unwrap()))
    },
    name: "pdftexversion"
};

pub static PDFMAJORVERSION: NumericCommand = NumericCommand {
    name:"pdfmajorversion",
    _getvalue: |_int| {
        use std::str::from_utf8;
        Ok(Numeric::Int(from_utf8(&[*VERSION_INFO.pdftexversion.to_utf8().as_bytes().first().unwrap()]).unwrap().parse::<i32>().unwrap()))// texversion.to_string().parse().unwrap()))
    },
};

pub static PDFSHELLESCAPE: NumericCommand = NumericCommand {
    name:"pdfshellescape",
    _getvalue:|_int| {
        Ok(Numeric::Int(2))
    }
};

pub static PDFLASTXIMAGE: NumericCommand = NumericCommand {
    name: "pdflastximage",
    _getvalue:|int| {
        Ok(Numeric::Int(int.state.pdfximages.len() as i32 -1))
    }
};

pub static PDFTEXREVISION: PrimitiveExecutable = PrimitiveExecutable {
    name:"pdftexrevision",
    expandable:true,
    _apply:|rf,_int| {
        rf.2 = crate::interpreter::string_to_tokens(VERSION_INFO.pdftexrevision.clone());
        Ok(())
    }
};


pub static PDFSTRCMP: PrimitiveExecutable = PrimitiveExecutable {
    name:"pdfstrcmp",
    expandable:true,
    _apply:|rf,int| {
        let mut tks = int.read_balanced_argument(true,false,false,true)?;
        let first = int.tokens_to_string(&tks);
        tks = int.read_balanced_argument(true,false,false,true)?;
        let second = int.tokens_to_string(&tks);
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
        let str = int.tokens_to_string(&strtks);
        /*if str.to_string() == "hyphen.cfg" {
            unsafe { crate::LOG = true };
        }*/
        let file = int.get_file(&str.to_utf8())?;
        match &*file.string.read().unwrap() {
            None => (),
            Some(s) => rf.2 = crate::interpreter::tokenize(
                s.len().to_string().into(),&crate::catcodes::OTHER_SCHEME
            )
        };
        Ok(())
    }
};

pub static IFPDFABSNUM : Conditional = Conditional {
    name:"ifpdfabsnum",
    _apply: |int,cond,unless| {
        let i1 = int.read_number()?;
        let rel = int.read_keyword(vec!["<", "=", ">"])?;
        let i2 = int.read_number()?;
        let istrue = match rel {
            Some(ref s) if s == "<" => i1.abs() < i2.abs(),
            Some(ref s) if s == "=" => i1.abs() == i2.abs(),
            Some(ref s) if s == ">" => i1.abs() > i2.abs(),
            _ => TeXErr!("Expected '<','=' or '>' in \\ifpdfabsnum")
        };
        log!("\\ifpdfabsnum {}{}{}: {}",i1,rel.as_ref().unwrap(),i2,istrue);
        if istrue { dotrue(int, cond, unless) } else { dofalse(int, cond, unless) }
    },
};

pub static PDFMATCH: PrimitiveExecutable = PrimitiveExecutable {
    name:"pdfmatch",
    expandable:true,
    _apply:|rf,int| {
        use regex::Regex;
        let icase = match int.read_keyword(vec!("icase"))? {
            Some(_) => true, _ => false
        };
        let _ = match int.read_keyword(vec!("subcount"))? {
            Some(_) => int.read_number()?,
            _ => -1
        };
        let mut tks = int.read_balanced_argument(true,false,false,true)?;
        let mut pattern_string = int.tokens_to_string(&tks).to_string();
        tks = int.read_balanced_argument(true,false,false,false)?;
        let target = int.tokens_to_string(&tks).to_string();
        if icase {
            pattern_string = "(?i)".to_string() + &pattern_string
        }
        match Regex::new(pattern_string.as_ref()) {
            Ok(reg) => {
                let mut matches : Vec<(String,usize,usize,Vec<Option<(String,usize,usize)>>)> = vec!();
                for cap in reg.captures_iter(target.as_str()) {
                    matches.push((
                        cap.get(0).unwrap().as_str().to_string(),
                        cap.get(0).unwrap().start(),
                        cap.get(0).unwrap().end(),
                        cap.iter().skip(1).map(|x| x.map(|x| (x.as_str().to_string(),x.start(),x.end()))).collect()
                    ))
                }
                if matches.is_empty() {
                    int.state.pdfmatches = vec!();
                    rf.2 = tokenize("0".into(),int.state.catcodes.get_scheme());
                    Ok(())
                } else {
                    matches.reverse();
                    let mut rets : Vec<TeXStr> = vec!();
                    let (m,start,_,groups) = matches.pop().unwrap();
                    rets.push((start.to_string() + "->" + &m).as_str().into());
                    for group in groups {
                        match group {
                            None => rets.push("-1->".into()),
                            Some((st,s,_)) => rets.push((s.to_string() + "->" + &st).as_str().into())
                        }
                    }
                    int.state.pdfmatches = rets;
                    rf.2 = tokenize("1".into(),int.state.catcodes.get_scheme());
                    Ok(())
                }
            }
            Err(_) => {
                int.state.pdfmatches = vec!();
                rf.2 = tokenize("-1".into(),int.state.catcodes.get_scheme());
                Ok(())
            }
        }
    }
};

pub static PDFLASTMATCH: PrimitiveExecutable = PrimitiveExecutable {
    name:"pdflastmatch",
    expandable:true,
    _apply:|rf,int| {
        let num = int.read_number()?;
        let str: TeXStr = match int.state.pdfmatches.get(num as usize) {
            Some(s) => s.clone(),
            _ => "-1->".into()
        };
        rf.2 = tokenize(str.into(),int.state.catcodes.get_scheme());
        Ok(())
    }
};

pub static PDFCOLORSTACK: PrimitiveExecutable = PrimitiveExecutable {
    name:"pdfcolorstack",
    expandable:false,
    _apply:|tk,int| {
        let num = int.read_number()?;
        let len = int.state.pdfcolorstacks.len();
        let prestring = int.read_keyword(vec!("push", "pop", "set", "current"))?;
        match prestring {
            Some(s) if s == "pop" => {
                match int.state.pdfcolorstacks.get_mut(len - 1 - num as usize) {
                    Some(s) => {s.pop();},
                    _ => TeXErr!("No color stack at index {}",num)
                }
                int.stomach_add(ColorEnd {
                    sourceref:int.update_reference(&tk.0)
                }.as_whatsit())?
            },
            Some(s) if s == "set" => {
                let tks = int.read_balanced_argument(true,false,false,false)?;
                let color: TeXStr = int.tokens_to_string(&tks).into();
                match int.state.pdfcolorstacks.get_mut(len - 1 - num as usize) {
                    Some(v) => {
                        v.pop();v.push(color.clone())
                    }
                    _ => TeXErr!("No color stack at index {}",num)
                }
                int.stomach_add(ColorEnd{
                    sourceref:int.update_reference(&tk.0)
                }.as_whatsit())?;
                int.stomach_add(ColorChange {
                    color,
                    children: vec![],
                    sourceref: int.update_reference(&tk.0)
                }.as_whatsit())?
            }
            Some(s) if s == "push" => {
                let tks = int.read_balanced_argument(true,false,false,false)?;
                let color : TeXStr = int.tokens_to_string(&tks).into();
                match int.state.pdfcolorstacks.get_mut(len - 1 - num as usize) {
                    Some(s) => s.push(color.clone()),
                    _ => TeXErr!("No color stack at index {}",num)
                }
                int.stomach_add(ColorChange {
                    color,
                    children: vec![],
                    sourceref: int.update_reference(&tk.0)
                }.as_whatsit())?
            }
            Some(s) if s == "current" => TeXErr!("TODO: current in \\pdfcolorstack"),
            _ => TeXErr!("Expected \"pop\", \"set\", \"push\" or \"current\" after \\pdfcolorstack")
        }
        Ok(())
    }
};

pub static PDFCOLORSTACKINIT: PrimitiveExecutable = PrimitiveExecutable {
    name:"pdfcolorstackinit",
    expandable:true,
    _apply:|tk,int| {
        let num = int.state.pdfcolorstacks.len();
        int.state.pdfcolorstacks.push(vec!());
        match int.read_keyword(vec!("page","direct"))? {
            Some(s) if s == "direct" => {
                let tks = int.read_balanced_argument(true,false,false,false)?;
                let str = int.tokens_to_string(&tks);
                int.state.pdfcolorstacks.last_mut().unwrap().push(str.into());
            }
            Some(_) => TeXErr!("TODO: page in \\pdfcolorstackinit"),
            None => TeXErr!("Expected \"page\" or \"direct\" after \\pdfcolorstackinit")
        }
        tk.2 = crate::interpreter::string_to_tokens(num.to_string().into());
        Ok(())
    }
};

pub static PDFOBJ: PrimitiveExecutable = PrimitiveExecutable {
    name:"pdfobj",
    expandable:false,
    _apply:|_,int| {
        match int.read_keyword(vec!("reserveobjnum","useobjnum","stream"))? {
            Some(s) if s == "reserveobjnum" => {
                let num = int.state.registers_prim.get(&(PDFLASTOBJ.index - 1));
                int.state.registers_prim.set(PDFLASTOBJ.index - 1,num+1,true);
                Ok(())
            }
            Some(s) if s == "useobjnum" => {
                let index = int.read_number()?;
                let tks = int.read_balanced_argument(true,false,false,false)?;
                let str = int.tokens_to_string(&tks);
                int.state.pdfobjs.insert(index as u16,str.into());
                Ok(())
            }
            Some(_) => TeXErr!("TODO: stream in \\pdfobj"),
            _ => TeXErr!("Expected \"reserveobjnum\",\"useobjnum\" or \"stream\" after \\pdfobj")
        }
    }
};

pub static PDFXFORM: PrimitiveExecutable = PrimitiveExecutable {
    name:"pdfxform",
    expandable:false,
    _apply:|tk,int| {
        let attr = read_attrspec(int)?;
        let resource = read_resource_spec(int)?;
        let ind = int.read_number()? as u16;
        let bx = int.state.boxes.take(ind);
        let lastform = int.state.registers_prim.get(&(PDFLASTXFORM.index - 1));
        int.state.registers_prim.set(PDFLASTXFORM.index - 1,lastform + 1,true);
        int.state.pdfxforms.push(PDFXForm {
            attr,
            resource,
            content: bx,
            sourceref: int.update_reference(&tk.0)
        });
        Ok(())
    }
};

pub static PDFREFXFORM: SimpleWhatsit = SimpleWhatsit {
    name:"pdfrefxform",
    modes: |_| {true},
    _get: |_,int| {
        let num = int.read_number()?;
        let len = int.state.pdfxforms.len();
        let form = match int.state.pdfxforms.get(len - num as usize) {
            Some(s) => s.clone(),
            None => TeXErr!("No pdfxform at index {}",num)
        };
        Ok(Whatsit::Simple(SimpleWI::PDFXForm(form)))
    }
};

pub static PDFLITERAL: SimpleWhatsit = SimpleWhatsit {
    name:"pdfliteral",
    modes: |_| {true},
    _get: |tk, int| {
        int.read_keyword(vec!("direct","page"))?;
        let tks = int.read_balanced_argument(true,false,false,false)?;
        let str : TeXStr = int.tokens_to_string(&tks).into();
        Ok(Whatsit::Simple(SimpleWI::PDFLiteral(PDFLiteral{
            literal:str,sourceref:int.update_reference(tk)
        })))
    }
};

pub static PDFDEST: SimpleWhatsit = SimpleWhatsit {
    name:"pdfdest",
    modes: |_| {true},
    _get:|tk,int| {
        let target = match int.read_keyword(vec!("num","name"))? {
            Some(s) if s == "num" => {
                "NUM_".to_string() +  &int.read_number()?.to_string()
            }
            Some(_) => int.read_string()?,
            None => TeXErr!("Expected \"num\" or \"name\" after \\pdfdest")
        };
        let dest = match int.read_keyword(vec!("xyz","XYZ","fitr","fitbh","fitbv","fitb","fith","fitv","fit"))? {
            Some(s) if s == "xyz" || s == "XYZ" => {
                "xyz".to_string() + &match int.read_keyword(vec!("zoom"))? {
                    None => "".to_string(),
                    Some(_) => {
                        " zoom ".to_string() + &int.read_number()?.to_string()
                    }
                }
            }
            Some(s) if s == "fitr" => {
                "fitr ".to_string() + read_rule_spec(int)?.string.to_string().as_str()
            }
            Some(s) => s,
            None => TeXErr!("Expected \"xyz\", \"XYZ\", \"fitr\", \"fitbh\", \"fitbv\", \"fitb\", \"fith\", \"fitv\" or \"fit\" in \\pdfdest")
        };
        Ok(Whatsit::Simple(SimpleWI::PDFDest(PDFDest {
            target:target.into(),dest:dest.into(),
            sourceref:int.update_reference(tk)
        })))
    }
};

pub static PDFSTARTLINK: SimpleWhatsit = SimpleWhatsit {
    name:"pdfstartlink",
    modes: |_| {true},
    _get:|tk,int| {
        let rule = read_rule_spec(int)?;
        let attr = match read_attrspec(int)?{
            Some(s) => s,
            None => "".into()
        };
        let action = read_action_spec(int)?;
        Ok(PDFLink {
            rule:rule.string.into(),
            attr,action,
            sourceref:int.update_reference(tk),
            children:vec!()
        }.as_whatsit())
    }
};

pub static PDFENDLINK: SimpleWhatsit = SimpleWhatsit {
    name:"pdfendlink",
    modes: |_| {true},
    _get:|tk,int| {
        Ok(LinkEnd {
            sourceref:int.update_reference(tk)
        }.as_whatsit())
    }
};

pub static PDFSETMATRIX: SimpleWhatsit = SimpleWhatsit {
    name:"pdfsetmatrix",
    modes: |_| {true},
    _get:|tk,int| {
        let tks = int.read_balanced_argument(true,false,false,true)?;
        let str = int.tokens_to_string(&tks);
        let nums : Vec<f32> = str.split(32).iter().map(|x| {
            match x.to_string().parse::<f32>() {
                Ok(f) => Ok(f),
                Err(_) => TeXErr!("Not a floating point number in \\pdfsetmatrix: {}",x)
            }
        }).collect::<Result<Vec<f32>,TeXError>>()?;
        assert_eq!(nums.len(),4);
        Ok(Whatsit::Simple(SimpleWI::PDFMatrix(PDFMatrix {
            scale: nums[0],
            rotate: nums[1],
            skewx: nums[2],
            skewy: nums[3],
            sourceref: int.update_reference(tk)
        })))
    }
};

pub static PDFSAVE: SimpleWhatsit = SimpleWhatsit {
    name:"pdfsave",
    modes: |_| {true},
    _get:|tk,int| {
        use crate::interpreter::TeXMode;
        Ok(PDFMatrixSave {
            is_vertical:match int.state.mode {
                TeXMode::Vertical | TeXMode::InternalVertical => true,
                _ => false
            },
            children:vec!(),
            sourceref:int.update_reference(tk)
        }.as_whatsit())
    }
};

pub static PDFRESTORE: SimpleWhatsit = SimpleWhatsit {
    name:"pdfrestore",
    modes: |_| {true},
    _get:|tk,int| {
        Ok(PDFRestore {
            sourceref:int.update_reference(tk)
        }.as_whatsit())
    }
};


pub static PDFREFXIMAGE: SimpleWhatsit = SimpleWhatsit {
    name:"pdfrefximage",
    modes: |m| {m == TeXMode::Horizontal || m == TeXMode::RestrictedHorizontal || m == TeXMode::Math || m == TeXMode::Displaymath},
    _get:|tk,int| {
        let num = int.read_number()?;
        let img = match int.state.pdfximages.get(num as usize) {
            Some(i) => i.clone(),
            None => TeXErr!(tk.clone() => "No image as index {}",num)
        };
        //unsafe {crate::LOG = true}
        Ok(Whatsit::Simple(SimpleWI::PDFXImage(img)))
    }
};

pub static PDFXIMAGE: PrimitiveExecutable = PrimitiveExecutable {
    name:"pdfximage",
    expandable:false,
    _apply:|tk,int| {
        //println!("Here! >>{}",int.preview());
        //unsafe {crate::LOG = true}
        let rule = read_rule_spec(int)?;
        let attr = read_attrspec(int)?;
        let pagespec = match int.read_keyword(vec!("page"))? {
            Some(_) => Some(int.read_number()?),
            None => None
        };
        let colorspace = match int.read_keyword(vec!("colorspace"))? {
            Some(_) => Some(int.read_number()?),
            None => None
        };
        let boxspec : Option<TeXStr> = match int.read_keyword(vec!("mediabox","cropbox","bleedbox","trimbox","artbox"))? {
            Some(s) => Some(s.as_str().into()),
            None => None
        };
        let tks = int.read_balanced_argument(true,false,false,true)?;
        let filename = int.tokens_to_string(&tks);
        let file = match int.kpsewhich(filename.to_string().as_str()) {
            Some((p,_)) if p.exists() => p,
            _ => TeXErr!("No image file by name {} found",filename)
        };
        let image = match match image::io::Reader::open(file.clone()) {
            Ok(x) => x,
            _ => TeXErr!("Error reading image {}",filename)
        }.with_guessed_format().unwrap().decode() {
            Ok(x) => Some(x),
            Err(e) => {
                match file.extension() {
                    Some(s) if s == "pdf" => pdf_to_img(file.to_str().unwrap()),
                    _ => TeXErr!("Error decoding image {} - {}",filename,e)
                }
            }
        };
        let mut _width = rule.width.clone();
        let mut _height = rule.height.clone();
        match (_width,_height,&image) {
            (Some(w),None,Some(img)) => {
                let ow = img.width() as f32;
                let oh = img.height() as f32;
                _height = Some(((oh as f32) / (ow / (w as f32))).round() as i32);
            }
            (None,Some(h),Some(img)) => {
                let ow = img.width() as f32;
                let oh = img.height() as f32;
                _width = Some(((ow as f32) / (oh / (h as f32))).round() as i32);
            }
            _ => {}
        }
        int.state.pdfximages.push(
            PDFXImage {
                rule,
                attr,
                pagespec,
                colorspace,
                boxspec,
                filename: file,
                image,
                sourceref: int.update_reference(&tk.0),
                _width,_height
            }
        );
        Ok(())
    }
};

pub static PDFFONTSIZE: PrimitiveExecutable = PrimitiveExecutable {
    name:"pdffontsize",
    expandable:true,
    _apply:|rf,int| {
        let font = crate::commands::primitives::read_font(int)?;
        let str = dimtostr(font.get_at());
        rf.2 = crate::interpreter::string_to_tokens(str.into());
        Ok(())
    }
};

pub static PDFFONTEXPAND: PrimitiveExecutable = PrimitiveExecutable {
    name:"pdffontexpand",
    expandable:false,
    _apply:|_,int| {
        crate::commands::primitives::read_font(int)?;
        // stretch             shrink           step
        int.read_number()?;int.read_number()?;int.read_number()?;
        int.read_keyword(vec!("autoexpand"))?;
        Ok(())
    }
};

pub static PDFOUTLINE: PrimitiveExecutable = PrimitiveExecutable {
    name:"pdfoutline",
    expandable:true,
    _apply:|_,int| {
        let _ = read_attrspec(int)?;
        let _ = read_action_spec(int)?;
        let _ = match int.read_keyword(vec!("count"))? {
            Some(_) => int.read_number()?,
            _ => 0
        };
        int.read_balanced_argument(true,false,false,true)?;
        Ok(())
    }
};

pub static PDFMDFIVESUM: PrimitiveExecutable = PrimitiveExecutable {
    name:"pdfmdfivesum",
    expandable:true,
    _apply:|rf,int| {
        let md = match int.read_keyword(vec!("file"))? {
            None => {
                let tks = int.read_balanced_argument(true,false,false,false)?;
                let str = int.tokens_to_string(&tks);
                md5::compute(str.to_utf8())
            },
            Some(_) => {
                let tks = int.read_balanced_argument(true,false,false,false)?;
                let str = int.tokens_to_string(&tks);
                match &*int.get_file(&str.to_utf8())?.string.read().unwrap() {
                    None => {
                        md5::compute("")
                    },
                    Some(s) => {
                        md5::compute(s.to_utf8())
                    }
                }
            }
        };
        let str = format!("{:X}", md);
        rf.2 = crate::interpreter::string_to_tokens(str.into());
        Ok(())
    }
};

pub static PDFCREATIONDATE: PrimitiveExecutable = PrimitiveExecutable {
    name:"pdfcreationdate",
    expandable:true,
    _apply:|rf,int| {
        let dt = int.jobinfo.time;
        let str = format!("D:{}{:02}{:02}{:02}{:02}{:02}{}'",
                          dt.year(),dt.month(),dt.day(),dt.hour(),dt.minute(),dt.second(),
                          dt.offset().to_string().replace(":","'"));
        rf.2 = crate::interpreter::string_to_tokens(str.into());
        Ok(())
    }
};

pub static PDFESCAPESTRING: PrimitiveExecutable = PrimitiveExecutable {
    name:"pdfescapestring",
    expandable:true,
    _apply:|rf,int| {
        rf.2 = int.read_balanced_argument(true,false,false,true)?;
        Ok(())
    }
};

pub static PDFCATALOG: PrimitiveExecutable = PrimitiveExecutable {
    name:"pdfcatalog",
    expandable:false,
    _apply:|_tk,int| {
        let _ = int.read_string()?;
        match int.read_keyword(vec!("openaction"))? {
            None => Ok(()),
            Some(_) => {
                read_action_spec(int)?;
                Ok(())
            }
        }
    }
};

pub static PDFGLYPHTOUNICODE: PrimitiveExecutable = PrimitiveExecutable {
    name:"pdfglyphtounicode",
    expandable:false,
    _apply:|_tk,int| {
        int.read_argument()?; int.read_argument()?;
        Ok(())
    }
};

pub static PDFINFO: SimpleWhatsit = SimpleWhatsit {
    name:"pdfinfo",
    modes: |_| {true},
    _get: |tk, int| {
        let tks = int.read_balanced_argument(true,false,false,false)?;
        let str : TeXStr = int.tokens_to_string(&tks).into();
        Ok(PDFInfo{
            info:str,sourceref:int.update_reference(tk)
        }.as_whatsit())
    }
};

pub static PDFESCAPEHEX: PrimitiveExecutable = PrimitiveExecutable {
    name:"pdfescapehex",
    expandable:true,
    _apply:|tk,int| {
        let tks = int.read_balanced_argument(true,false,false,false)?;
        for t in tks {
            for t in string_to_tokens(format!("{:x?}",t.char).into()) {
                tk.2.push(t)
            }
        }
        Ok(())
    }
};

pub static PDFUNESCAPEHEX: PrimitiveExecutable = PrimitiveExecutable {
    name:"pdfunescapehex",
    expandable:true,
    _apply:|tk,int| {
        let tks = int.read_balanced_argument(true,false,false,false)?;
        let str = int.tokens_to_string(&tks).to_utf8();
        for i in (0..str.len()).step_by(2) {
            let u = match u8::from_str_radix(&str[i..i + 2], 16){
                Ok(u) => u,
                _ => TeXErr!(tk.0.clone() => "Hex value expected!")
            };
            tk.2.push(Token::new(u,CategoryCode::Other,None,None,true))
        }
        Ok(())
    }
};

// -------------------------------------------------------------------------------------------------



// TODO --------------------------------------------------------------------------------------------

pub static IFPDFABSDIM : Conditional = Conditional {
    name:"ifpdfabsdim",
    _apply: |_int,_cond,_unless| {
        TeXErr!("TODO: \\ifpdfabsdim")
    }
};

pub static IFPDFPRIMITIVE : Conditional = Conditional {
    name:"ifpdfprimitive",
    _apply: |_int,_cond,_unless| {
        TeXErr!("TODO: \\ifpdfprimitive")
    }
};

pub static PDFESCAPENAME: PrimitiveExecutable = PrimitiveExecutable {
    name:"pdfescapename",
    expandable:true,
    _apply:|_tk,_int| {TeXErr!("TODO: \\pdfescapename")}
};

pub static PDFANNOT: PrimitiveExecutable = PrimitiveExecutable {
    name:"pdfannot",
    expandable:true,
    _apply:|_,int| {
        match int.read_keyword(vec!("reserveobjnum","useobjnum"))? {
            Some(s) if s == "reserveobjnum" => {
                Ok(())
            }
            Some(s) if s == "useobjnum" => {
                int.read_number()?;
                read_rule_spec(int);
                int.read_balanced_argument(true,false,false,false)?;
                Ok(())
            }
            _ => {
                read_rule_spec(int);
                int.read_balanced_argument(true,false,false,false)?;
                Ok(())
            }
        }
    }
};

pub static PDFFILEDUMP: PrimitiveExecutable = PrimitiveExecutable {
    name:"pdffiledump",
    expandable:true,
    _apply:|_tk,_int| {TeXErr!("TODO: \\pdffiledump")}
};

pub static PDFFILEMODDATE: PrimitiveExecutable = PrimitiveExecutable {
    name:"pdffilemoddate",
    expandable:true,
    _apply:|_tk,_int| {TeXErr!("TODO: \\pdffilemoddate")}
};

pub static PDFPAGEATTR: PrimitiveExecutable = PrimitiveExecutable {
    name:"pdfpageattr",
    expandable:true,
    _apply:|_tk,_int| {TeXErr!("TODO: \\pdfpageattr")}
};

pub static PDFSAVEPOS: PrimitiveExecutable = PrimitiveExecutable {
    name:"pdfsavepos",
    expandable:true,
    _apply:|_tk,_int| {Ok(())}
};

pub static PDFLASTXPOS: NumericCommand = NumericCommand {
    name:"pdflastxpos",
    _getvalue: |int| {
        Ok(Numeric::Int(0))
    },
};

pub static PDFLASTYPOS: NumericCommand = NumericCommand {
    name:"pdflastypos",
    _getvalue: |int| {
        Ok(Numeric::Int(0))
    },
};

pub static PDFELAPSEDTIME: PrimitiveExecutable = PrimitiveExecutable {
    name:"pdfelapsedtime",
    expandable:true,
    _apply:|_tk,_int| {TeXErr!("TODO: \\pdfelapsedtime")}
};

pub static LETTERSPACEFONT: PrimitiveAssignment = PrimitiveAssignment {
    name:"letterspacefont",
    _assign: |rf,int,global| {
        let cmd = int.read_command_token()?;
        let font = read_font(int)?;
        let val = int.read_number()?;
        int.change_command(cmd.cmdname(),Some(PrimitiveTeXCommand::AV(AssignableValue::FontRef(font)).as_command()),global);
        Ok(())
    }
};

/*
  val pdfcreationdate = new PrimitiveCommandProcessor("pdfcreationdate") {}
  val pdfendthread = new PrimitiveCommandProcessor("pdfendthread") {}
  val pdffontattr = new PrimitiveCommandProcessor("pdffontattr") {}
  val pdffontname = new PrimitiveCommandProcessor("pdffontname") {}
  val pdffontobjnum = new PrimitiveCommandProcessor("pdffontobjnum") {}
  val pdfgamma = new PrimitiveCommandProcessor("pdfgamma") {}
  val pdfimageapplygamma = new PrimitiveCommandProcessor("pdfimageapplygamma") {}
  val pdfimagegamma = new PrimitiveCommandProcessor("pdfimagegamma") {}
  val pdfimagehicolor = new PrimitiveCommandProcessor("pdfimagehicolor") {}
  val pdfimageresolution = new PrimitiveCommandProcessor("pdfimageresolution") {}
  val pdfincludechars = new PrimitiveCommandProcessor("pdfincludechars") {}
  val pdfinclusioncopyfonts = new PrimitiveCommandProcessor("pdfinclusioncopyfonts") {}
  val pdfinclusionerrorlevel = new PrimitiveCommandProcessor("pdfinclusionerrorlevel") {}
  val pdflastximagecolordepth = new PrimitiveCommandProcessor("pdflastximagecolordepth") {}
  val pdflastximagepages = new PrimitiveCommandProcessor("pdflastximagepages") {}
  val pdfnames = new PrimitiveCommandProcessor("pdfnames") {}
  val pdfpagesattr = new PrimitiveCommandProcessor("pdfpagesattr") {}
  val pdfpagebox = new PrimitiveCommandProcessor("pdfpagebox") {}
  val pdfpageref = new PrimitiveCommandProcessor("pdfpageref") {}
  val pdfrefobj = new PrimitiveCommandProcessor("pdfrefobj") {}
  val pdfretval = new PrimitiveCommandProcessor("pdfretval") {}
  val pdfstartthread = new PrimitiveCommandProcessor("pdfstartthread") {}
  val pdfsuppressptexinfo = new PrimitiveCommandProcessor("pdfsuppressptexinfo") {}
  val pdfthread = new PrimitiveCommandProcessor("pdfthread") {}
  val pdfthreadmargin = new PrimitiveCommandProcessor("pdfthreadmargin") {}
  val pdftrailer = new PrimitiveCommandProcessor("pdftrailer") {}
  val pdfuniqueresname = new PrimitiveCommandProcessor("pdfuniqueresname") {}
  val pdfxformname = new PrimitiveCommandProcessor("pdfxformname") {}
  val pdfximagebbox = new PrimitiveCommandProcessor("pdfximagebbox") {}
  val pdfcopyfont = new PrimitiveCommandProcessor("pdfcopyfont") {}
  val pdfeachlinedepth = new PrimitiveCommandProcessor("pdfeachlinedepth") {}
  val pdfeachlineheight = new PrimitiveCommandProcessor("pdfeachlineheight") {}
  val pdffirstlineheight = new PrimitiveCommandProcessor("pdffirstlineheight") {}
  val pdfignoreddimen = new PrimitiveCommandProcessor("pdfignoreddimen") {}
  val pdfinsertht = new PrimitiveCommandProcessor("pdfinsertht") {}
  val pdflastlinedepth = new PrimitiveCommandProcessor("pdflastlinedepth") {}
  val pdfmapfile = new PrimitiveCommandProcessor("pdfmapfile") {}
  val pdfmapline = new PrimitiveCommandProcessor("pdfmapline") {}
  val pdfnoligatures = new PrimitiveCommandProcessor("pdfnoligatures") {}
  val pdfnormaldeviate = new PrimitiveCommandProcessor("pdfnormaldeviate") {}
  val pdfpkmode = new PrimitiveCommandProcessor("pdfpkmode") {}
  val pdfprimitive = new PrimitiveCommandProcessor("pdfprimitive") {}
  val pdfrandomseed = new PrimitiveCommandProcessor("pdfrandomseed") {}
  val pdfresettimer = new PrimitiveCommandProcessor("pdfresettimer") {}
  val pdfsetrandomseed = new PrimitiveCommandProcessor("pdfsetrandomseed") {}
  val pdftracingfonts = new PrimitiveCommandProcessor("pdftracingfonts") {}
  val pdfuniformdeviate = new PrimitiveCommandProcessor("pdfuniformdeviate") {}
  val pdftexbanner = new PrimitiveCommandProcessor("pdftexbanner") {}
 */

// -------------------------------------------------------------------------------------------------

use crate::commands::registers::{PDFADJUSTSPACING, PDFCOMPRESSLEVEL, PDFDECIMALDIGITS, PDFDESTMARGIN, PDFDRAFTMODE, PDFGENTOUNICODE, PDFHORIGIN, PDFLASTANNOT, PDFLASTLINK, PDFLASTOBJ, PDFLASTXFORM, PDFLINKMARGIN, PDFMINORVERSION, PDFOBJCOMPRESSLEVEL, PDFOUTPUT, PDFPAGEHEIGHT, PDFPAGERESOURCES, PDFPAGEWIDTH, PDFPKRESOLUTION, PDFPROTRUDECHARS, PDFPXDIMEN, PDFSUPPRESSWARNINGDUPDEST, PDFVORIGIN};

pub fn pdftex_commands() -> Vec<PrimitiveTeXCommand> {vec![
    PrimitiveTeXCommand::Num(&PDFTEXVERSION),
    PrimitiveTeXCommand::Num(&PDFSHELLESCAPE),
    PrimitiveTeXCommand::Num(&PDFMAJORVERSION),
    PrimitiveTeXCommand::Num(&PDFLASTXIMAGE),

    PrimitiveTeXCommand::Cond(&IFPDFABSNUM),
    PrimitiveTeXCommand::Cond(&IFPDFABSDIM),
    PrimitiveTeXCommand::Cond(&IFPDFPRIMITIVE),

    PrimitiveTeXCommand::Primitive(&PDFOBJ),
    PrimitiveTeXCommand::Whatsit(ProvidesWhatsit::Simple(&PDFLITERAL)),
    PrimitiveTeXCommand::Whatsit(ProvidesWhatsit::Simple(&PDFDEST)),
    PrimitiveTeXCommand::Whatsit(ProvidesWhatsit::Simple(&PDFSTARTLINK)),
    PrimitiveTeXCommand::Whatsit(ProvidesWhatsit::Simple(&PDFENDLINK)),
    PrimitiveTeXCommand::Whatsit(ProvidesWhatsit::Simple(&PDFREFXFORM)),
    PrimitiveTeXCommand::Whatsit(ProvidesWhatsit::Simple(&PDFREFXIMAGE)),
    PrimitiveTeXCommand::Whatsit(ProvidesWhatsit::Simple(&PDFSETMATRIX)),
    PrimitiveTeXCommand::Whatsit(ProvidesWhatsit::Simple(&PDFSAVE)),
    PrimitiveTeXCommand::Whatsit(ProvidesWhatsit::Simple(&PDFRESTORE)),
    PrimitiveTeXCommand::Whatsit(ProvidesWhatsit::Simple(&PDFINFO)),

    PrimitiveTeXCommand::AV(AssignableValue::PrimReg(&PDFOUTPUT)),
    PrimitiveTeXCommand::AV(AssignableValue::PrimReg(&PDFMINORVERSION)),
    PrimitiveTeXCommand::AV(AssignableValue::PrimReg(&PDFOBJCOMPRESSLEVEL)),
    PrimitiveTeXCommand::AV(AssignableValue::PrimReg(&PDFCOMPRESSLEVEL)),
    PrimitiveTeXCommand::AV(AssignableValue::PrimReg(&PDFDECIMALDIGITS)),
    PrimitiveTeXCommand::AV(AssignableValue::PrimReg(&PDFPKRESOLUTION)),
    PrimitiveTeXCommand::AV(AssignableValue::PrimReg(&PDFLASTOBJ)),
    PrimitiveTeXCommand::AV(AssignableValue::PrimReg(&PDFLASTXFORM)),
    PrimitiveTeXCommand::AV(AssignableValue::PrimReg(&PDFLASTANNOT)),
    PrimitiveTeXCommand::AV(AssignableValue::PrimReg(&PDFLASTLINK)),
    PrimitiveTeXCommand::AV(AssignableValue::PrimReg(&PDFSUPPRESSWARNINGDUPDEST)),
    PrimitiveTeXCommand::AV(AssignableValue::PrimReg(&PDFPROTRUDECHARS)),
    PrimitiveTeXCommand::AV(AssignableValue::PrimReg(&PDFADJUSTSPACING)),
    PrimitiveTeXCommand::AV(AssignableValue::PrimReg(&PDFDRAFTMODE)),
    PrimitiveTeXCommand::AV(AssignableValue::PrimReg(&PDFGENTOUNICODE)),

    PrimitiveTeXCommand::AV(AssignableValue::PrimDim(&PDFLINKMARGIN)),
    PrimitiveTeXCommand::AV(AssignableValue::PrimDim(&PDFDESTMARGIN)),
    PrimitiveTeXCommand::AV(AssignableValue::PrimDim(&PDFPXDIMEN)),
    PrimitiveTeXCommand::AV(AssignableValue::PrimDim(&PDFPAGEHEIGHT)),
    PrimitiveTeXCommand::AV(AssignableValue::PrimDim(&PDFPAGEWIDTH)),
    PrimitiveTeXCommand::AV(AssignableValue::PrimDim(&PDFHORIGIN)),
    PrimitiveTeXCommand::AV(AssignableValue::PrimDim(&PDFVORIGIN)),

    PrimitiveTeXCommand::AV(AssignableValue::PrimToks(&PDFPAGERESOURCES)),

    // TODO ----------------------------------------------------------------------------------------

    PrimitiveTeXCommand::Primitive(&PDFESCAPESTRING),
    PrimitiveTeXCommand::Primitive(&PDFESCAPENAME),
    PrimitiveTeXCommand::Primitive(&PDFANNOT),
    PrimitiveTeXCommand::Primitive(&PDFCATALOG),
    PrimitiveTeXCommand::Primitive(&PDFCOLORSTACKINIT),
    PrimitiveTeXCommand::Primitive(&PDFCOLORSTACK),
    PrimitiveTeXCommand::Primitive(&PDFESCAPEHEX),
    PrimitiveTeXCommand::Primitive(&PDFFILEDUMP),
    PrimitiveTeXCommand::Primitive(&PDFFILEMODDATE),
    PrimitiveTeXCommand::Primitive(&PDFFILESIZE),
    PrimitiveTeXCommand::Primitive(&PDFFONTSIZE),
    PrimitiveTeXCommand::Primitive(&PDFFONTEXPAND),
    PrimitiveTeXCommand::Primitive(&PDFGLYPHTOUNICODE),
    PrimitiveTeXCommand::Primitive(&PDFUNESCAPEHEX),
    PrimitiveTeXCommand::Primitive(&PDFMATCH),
    PrimitiveTeXCommand::Primitive(&PDFLASTMATCH),
    PrimitiveTeXCommand::Primitive(&PDFOUTLINE),
    PrimitiveTeXCommand::Primitive(&PDFPAGEATTR),
    PrimitiveTeXCommand::Primitive(&PDFSAVEPOS),
    PrimitiveTeXCommand::Num(&PDFLASTXPOS),
    PrimitiveTeXCommand::Num(&PDFLASTYPOS),
    PrimitiveTeXCommand::Primitive(&PDFXFORM),
    PrimitiveTeXCommand::Primitive(&PDFXIMAGE),
    PrimitiveTeXCommand::Primitive(&PDFMDFIVESUM),
    PrimitiveTeXCommand::Primitive(&PDFCREATIONDATE),
    PrimitiveTeXCommand::Primitive(&PDFSTRCMP),
    PrimitiveTeXCommand::Primitive(&PDFTEXREVISION),
    PrimitiveTeXCommand::Primitive(&PDFELAPSEDTIME),
    PrimitiveTeXCommand::Ass(&LETTERSPACEFONT),
]}