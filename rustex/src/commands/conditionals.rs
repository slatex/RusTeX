use crate::interpreter::Interpreter;
use crate::ontology::Token;

#[derive(Clone)]
pub(in crate) struct Condition {
    pub cond:Option<bool>,
    pub unless:bool
}
fn expand<'a>(cs: Token, int: &'a mut Interpreter) -> &'a mut Condition {
    let cond = Condition {
        cond:None,
        unless:false
    };
    int.state.conditions.push(cond);
    int.state.conditions.last_mut().unwrap()
}
fn dotrue(int: &mut Interpreter,cond:&mut Condition,allow_unless:bool) {
    if cond.unless && allow_unless {
        dofalse(int,cond,false)
    } else {
        cond.cond = Some(true)
    }
}
fn dofalse(int: &mut Interpreter,cond:&mut Condition,allow_unless:bool) {
    if cond.unless && allow_unless {
        dotrue(int,cond,false)
    } else {
        todo!()
    }
}