use crate::commands::{AssignableValue, PrimitiveExecutable, Conditional, DimenReference, RegisterReference, NumericCommand, PrimitiveTeXCommand, TokReference, SimpleWhatsit, ProvidesWhatsit, TokenList};
use crate::interpreter::tokenize;
use crate::{Interpreter, VERSION_INFO};
use crate::{log,TeXErr};
use crate::interpreter::dimensions::{dimtostr, Numeric};
use crate::commands::conditionals::{dotrue,dofalse};
use crate::interpreter::state::StateChange;
use crate::stomach::groups::{ColorChange, ColorEnd, LinkEnd, PDFLink, PDFMatrixSave, PDFRestore};
use crate::stomach::simple::{PDFDest, PDFLiteral, PDFMatrix, PDFXForm, PDFXImage, SimpleWI};
use crate::stomach::whatsits::{ActionSpec, Whatsit, WhatsitTrait};
use crate::utils::{TeXError, TeXStr};

fn read_attrspec(int:&Interpreter) -> Result<Option<TeXStr>,TeXError> {
    int.expand_until(true)?;
    let ret = match int.read_keyword(vec!("attr"))? {
        Some(_) => {
            int.skip_ws();
            Some(int.tokens_to_string(&int.read_balanced_argument(true,false,false,false)?).into())
        },
        None => None
    };
    Ok(ret)
}

fn read_rule_spec(int:&Interpreter )-> Result<String,TeXError> {
    int.expand_until(true)?;
    match int.read_keyword(vec!("width", "height", "depth"))? {
        Some(s) => Ok((s + " " + &dimtostr(int.read_dimension()?) + " " + &read_rule_spec(int)?.to_string()).into()),
        None => Ok("".into())
    }
}
fn read_resource_spec(int:&Interpreter) -> Result<Option<TeXStr>,TeXError> {
    int.expand_until(true)?;
    let ret = match int.read_keyword(vec!("resources"))? {
        Some(_) => {
            int.skip_ws();
            Some(int.tokens_to_string(&int.read_balanced_argument(true,false,false,false)?).into())
        },
        None => None
    };
    Ok(ret)
}

fn read_action_spec(int:&Interpreter) -> Result<ActionSpec,TeXError> {
    match int.read_keyword(vec!("user","goto","thread"))? {
        Some(s) if s == "user" => {
            let retstr = int.tokens_to_string(&int.read_balanced_argument(true,false,false,true)?);
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
                        _ => TeXErr!((int,None),"Expected \"name\" or \"page\" in action spec")
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
                    TeXErr!((int,None),"Here: {}",TokenList(&ret))
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
                        _ => TeXErr!((int,None),"Expected \"name\" or \"page\" in action spec")
                    }
                }
                _ => TeXErr!((int,None),"Expected \"num\" or \"file\" after \"thread\" in action spec")
            }
        }
        _ => TeXErr!((int,None),"Expected \"user\", \"goto\" or \"thread\" in action spec")
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
        Ok(Numeric::Int(int.state.borrow().pdfximages.len() as i32 -1))
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
        let first = int.tokens_to_string(&int.read_balanced_argument(true,false,false,true)?);
        let second = int.tokens_to_string(&int.read_balanced_argument(true,false,false,true)?);
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
            _ => TeXErr!((int,None),"Expected '<','=' or '>' in \\ifpdfabsnum")
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
        let mut pattern_string = int.tokens_to_string(&int.read_argument()?).to_string();
        let target = int.tokens_to_string(&int.read_balanced_argument(true,false,false,false)?).to_string();
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
                    int.change_state(StateChange::Pdfmatches(vec!()));
                    rf.2 = tokenize("0".into(),&int.state_catcodes());
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
                    int.change_state(StateChange::Pdfmatches(rets));
                    rf.2 = tokenize("1".into(),&int.state_catcodes());
                    Ok(())
                }
            }
            Err(_) => {
                int.change_state(StateChange::Pdfmatches(vec!()));
                rf.2 = tokenize("-1".into(),&int.state_catcodes());
                Ok(())
            }
        }
    }
};

pub static PDFCOLORSTACK: PrimitiveExecutable = PrimitiveExecutable {
    name:"pdfcolorstack",
    expandable:false,
    _apply:|tk,int| {
        let num = int.read_number()?;
        let prestring = int.read_keyword(vec!("push", "pop", "set", "current"))?;
        match prestring {
            Some(s) if s == "pop" => {
                int.state_color_pop(num as usize);
                int.stomach.borrow_mut().add(int,ColorEnd {
                    sourceref:int.update_reference(&tk.0)
                }.as_whatsit())?
            },
            Some(s) if s == "set" => {
                let color: TeXStr = int.tokens_to_string(&int.read_balanced_argument(true,false,false,false)?).into();
                int.state_color_set(num as usize,color.clone());
                int.stomach.borrow_mut().add(int,ColorEnd{
                    sourceref:int.update_reference(&tk.0)
                }.as_whatsit())?;
                int.stomach.borrow_mut().add(int,ColorChange {
                    color,
                    children: vec![],
                    sourceref: int.update_reference(&tk.0)
                }.as_whatsit())?
            }
            Some(s) if s == "push" => {
                let color : TeXStr = int.tokens_to_string(&int.read_balanced_argument(true,false,false,false)?).into();
                int.state_color_push(num as usize,color.clone());
                int.stomach.borrow_mut().add(int,ColorChange {
                    color,
                    children: vec![],
                    sourceref: int.update_reference(&tk.0)
                }.as_whatsit())?
            }
            Some(s) if s == "current" => todo!(),
            _ => TeXErr!((int,None),"Expected \"pop\", \"set\", \"push\" or \"current\" after \\pdfcolorstack")
        }
        Ok(())
    }
};

pub static PDFCOLORSTACKINIT: PrimitiveExecutable = PrimitiveExecutable {
    name:"pdfcolorstackinit",
    expandable:true,
    _apply:|tk,int| {
        let num = int.state_color_push_stack();
        match int.read_keyword(vec!("page","direct"))? {
            Some(s) if s == "direct" => {
                let tks = int.read_balanced_argument(true,false,false,false)?;
                let str = int.tokens_to_string(&tks);
                int.state_color_push(num,str.into());
            }
            Some(_) => todo!(),
            None => TeXErr!((int,None),"Expected \"page\" or \"direct\" after \\pdfcolorstackinit")
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
                let num = int.state_register(-(PDFLASTOBJ.index as i32));
                int.change_state(StateChange::Register(-(PDFLASTOBJ.index as i32),num+1,true));
                Ok(())
            }
            Some(s) if s == "useobjnum" => {
                let index = int.read_number()?;
                let str = int.tokens_to_string(&int.read_balanced_argument(true,false,false,false)?);
                int.state_set_pdfobj(index as u16,str.into());
                Ok(())
            }
            Some(_) => todo!(),
            _ => TeXErr!((int,None),"Expected \"reserveobjnum\",\"useobjnum\" or \"stream\" after \\pdfobj")
        }
    }
};

pub static PDFXFORM: PrimitiveExecutable = PrimitiveExecutable {
    name:"pdfxform",
    expandable:false,
    _apply:|tk,int| {
        let attr = read_attrspec(int)?;
        let resource = read_resource_spec(int)?;
        let ind = int.read_number()?;
        let bx = int.state_get_box(ind as i32);
        let lastform = int.state_register(-(PDFLASTXFORM.index as i32));
        int.change_state(StateChange::Register(-(PDFLASTXFORM.index as i32),lastform + 1,true));
        int.state_set_pdfxform(PDFXForm {
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
        Ok(Whatsit::Simple(SimpleWI::PDFXForm(int.state_get_pdfxform(num as usize)?)))
    }
};

pub static PDFLITERAL: SimpleWhatsit = SimpleWhatsit {
    name:"pdfliteral",
    modes: |_| {true},
    _get: |tk, int| {
        int.read_keyword(vec!("direct","page"))?;
        let str : TeXStr = int.tokens_to_string(&int.read_balanced_argument(true,false,false,false)?).into();
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
            None => TeXErr!((int,None),"Expected \"num\" or \"name\" after \\pdfdest")
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
                "fitr ".to_string() + &read_rule_spec(int)?
            }
            Some(s) => s,
            None => TeXErr!((int,None),"Expected \"xyz\", \"XYZ\", \"fitr\", \"fitbh\", \"fitbv\", \"fitb\", \"fith\", \"fitv\" or \"fit\" in \\pdfdest")
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
            rule:rule.into(),
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
                Err(_) => TeXErr!((int,None),"Not a floating point number in \\pdfsetmatrix: {}",x)
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
            is_vertical:match int.get_mode() {
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
    modes: |_| {true},
    _get:|tk,int| {
        let num = int.read_number()?;
        let img = match int.state.borrow().pdfximages.get(num as usize) {
            Some(i) => i.clone(),
            None => TeXErr!((int,Some(tk.clone())),"No image as index {}",num)
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
        let filename = int.tokens_to_string(&int.read_balanced_argument(true,false,false,true)?);
        let file = match int.kpsewhich(filename.to_string().as_str()) {
            Some((p,_)) if p.exists() => p,
            _ => TeXErr!((int,None),"No image file by name {} found",filename)
        };
        let image = match match image::io::Reader::open(file.clone()) {
            Ok(x) => x,
            _ => TeXErr!((int,None),"Error reading image {}",filename)
        }.decode() {
            Ok(x) => Some(x),
            _ => {
                match file.extension() {
                    Some(s) if s == "pdf" => None,
                    _ => TeXErr!((int,None),"Error decoding image {}",filename)
                }
            }
        };
        int.state.borrow_mut().pdfximages.push(
            PDFXImage {
                rule:rule.into(),
                attr,
                pagespec,
                colorspace,
                boxspec,
                filename: file,
                image,
                sourceref: int.update_reference(&tk.0),
                _width:None,_height:None
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
        match int.read_keyword(vec!("file"))? {
            None => todo!(),
            Some(_) => ()
        }
        let tks = int.read_balanced_argument(true,false,false,false)?;
        let str = int.tokens_to_string(&tks);
        //let file = int.get_file(&str.to_utf8())?;
        //let file = int.get_file(&str.to_string())?;
        match &*int.get_file(&str.to_utf8())?.string.read().unwrap() {
            None => {
                let md = md5::compute("");
                let str = format!("{:X}", md);
                rf.2 = crate::interpreter::string_to_tokens(str.into());
                Ok(())
            },
            Some(s) => {
                let md = md5::compute(s.to_utf8());
                let str = format!("{:X}", md);
                rf.2 = crate::interpreter::string_to_tokens(str.into());
                Ok(())
            }
        }
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

// -------------------------------------------------------------------------------------------------

pub static PDFOUTPUT : RegisterReference = RegisterReference {
    name: "pdfoutput",
    index:35
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

// -----------------

pub static PDFLASTOBJ : RegisterReference = RegisterReference {
    name: "pdflastobj",
    index:43
};

pub static PDFLASTXFORM : RegisterReference = RegisterReference {
    name: "pdflastxform",
    index:44
};

// ----

pub static PDFLASTANNOT : RegisterReference = RegisterReference {
    name: "pdflastannot",
    index:48
};

pub static PDFLASTLINK : RegisterReference = RegisterReference {
    name: "pdflastlink",
    index:49
};

pub static PDFSUPPRESSWARNINGDUPDEST : RegisterReference = RegisterReference {
    name: "pdfsuppresswarningdupdest",
    index:50
};

pub static PDFPROTRUDECHARS : RegisterReference = RegisterReference {
    name: "pdfprotrudechars",
    index:51
};

pub static PDFADJUSTSPACING : RegisterReference = RegisterReference {
    name: "pdfadjustspacing",
    index:52
};

// ------------------

// -------------


pub static PDFDRAFTMODE : RegisterReference = RegisterReference {
    name: "pdfdraftmode",
    index:73
};

pub static PDFGENTOUNICODE : RegisterReference = RegisterReference {
    name: "pdfgentounicode",
    index:74
};

// -------------------------------------------------------------------------------------------------

pub static PDFLINKMARGIN : DimenReference = DimenReference {
    name: "pdflinkmargin",
    index:23
};

pub static PDFDESTMARGIN : DimenReference = DimenReference {
    name: "pdfdestmargin",
    index:24
};

// ------------

pub static PDFPXDIMEN : DimenReference = DimenReference {
    name: "pdfpxdimen",
    index:68
};


pub static PDFPAGEHEIGHT : DimenReference = DimenReference {
    name: "pdfpageheight",
    index:17
};

pub static PDFPAGEWIDTH : DimenReference = DimenReference {
    name: "pdfpagewidth",
    index:18
};

pub static PDFHORIGIN : DimenReference = DimenReference {
    name: "pdfhorigin",
    index:19
};

pub static PDFVORIGIN : DimenReference = DimenReference {
    name: "pdfvorigin",
    index:20
};

// -------------------------------------------------------------------------------------------------

pub static PDFPAGERESOURCES: TokReference = TokReference {
    name:"pdfpageresources",
    index:15
};


// TODO --------------------------------------------------------------------------------------------

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

pub static PDFLASTMATCH: PrimitiveExecutable = PrimitiveExecutable {
    name:"pdflastmatch",
    expandable:true,
    _apply:|_tk,_int| {todo!()}
};

pub static PDFPAGEATTR: PrimitiveExecutable = PrimitiveExecutable {
    name:"pdfpageattr",
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

pub static PDFELAPSEDTIME: PrimitiveExecutable = PrimitiveExecutable {
    name:"pdfelapsedtime",
    expandable:true,
    _apply:|_tk,_int| {todo!()}
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
    PrimitiveTeXCommand::Primitive(&PDFINFO),
    PrimitiveTeXCommand::Primitive(&PDFMATCH),
    PrimitiveTeXCommand::Primitive(&PDFLASTMATCH),
    PrimitiveTeXCommand::Primitive(&PDFOUTLINE),
    PrimitiveTeXCommand::Primitive(&PDFPAGEATTR),
    PrimitiveTeXCommand::Primitive(&PDFSAVEPOS),
    PrimitiveTeXCommand::Primitive(&PDFLASTXPOS),
    PrimitiveTeXCommand::Primitive(&PDFLASTYPOS),
    PrimitiveTeXCommand::Primitive(&PDFXFORM),
    PrimitiveTeXCommand::Primitive(&PDFXIMAGE),
    PrimitiveTeXCommand::Primitive(&PDFMDFIVESUM),
    PrimitiveTeXCommand::Primitive(&PDFSTRCMP),
    PrimitiveTeXCommand::Primitive(&PDFTEXREVISION),
    PrimitiveTeXCommand::Primitive(&PDFELAPSEDTIME),
]}