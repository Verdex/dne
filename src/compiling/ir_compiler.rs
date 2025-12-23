
use std::rc::Rc;
use std::collections::HashMap;

use crate::parsing::ir_parser::{Lit, Expr, Type, Stmt, Top, Proc as PProc, Global};
use crate::eval::data::*;

pub fn compile(ir : &[Top]) -> Result<Vec<Proc>, ()> {
    
    let (procs, globals, proc_map) = {
        let mut ps = vec![];
        let mut gs = vec![];
        for t in ir {
            match t {
                Top::Global(x) => { gs.push(x); },
                Top::Proc(x) => { ps.push(x); },
            }
        }
        let pm = HashMap::from_iter(ps.iter().enumerate().map(|(v, k)| (Rc::clone(&k.name), v)));
        (ps, gs, pm)
    };

    // TODO handle globals (drop them all in an init proc?)

    let ret = procs.into_iter().map(|x| compile_proc(x, &proc_map)).collect::<Result<Vec<_>, ()>>();
    
    ret
}

fn compile_proc(proc : &PProc, proc_map : &HashMap<Rc<str>, usize>) -> Result<Proc, ()> {
    todo!()
}
