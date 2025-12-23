
use std::rc::Rc;
use std::collections::HashMap;

use crate::parsing::ir_parser::{Lit, Expr, Type, Stmt, Top, Proc as PProc, Global};
use crate::eval::data::*;

type ProcMap<'a> = HashMap<Rc<str>, (&'a PProc, usize)>;
type LMap<'a> = HashMap<Rc<str>, (&'a Type, usize)>;
type LabelMap = HashMap<Rc<str>, usize>;

#[derive(Debug)]
pub enum CompileError {

}

pub fn compile(ir : &[Top]) -> Result<Vec<Proc>, CompileError> {
    
    let (procs, globals, proc_map) = {
        let mut ps = vec![];
        let mut gs = vec![];
        for t in ir {
            match t {
                Top::Global(x) => { gs.push(x); },
                Top::Proc(x) => { ps.push(x); },
            }
        }
        let pm = HashMap::from_iter(ps.iter().enumerate().map(|(v, k)| (Rc::clone(&k.name), (*k, v))));
        (ps, gs, pm)
    };

    // TODO handle globals (drop them all in an init proc?)

    let ret = procs.into_iter().map(|x| compile_proc(x, &proc_map)).collect::<Result<Vec<_>, _>>();
    
    ret
}

fn compile_proc(proc : &PProc, proc_map : &ProcMap) -> Result<Proc, CompileError> {

    let mut l_map : LMap = HashMap::from_iter(proc.params.iter().enumerate().map(|(i, (name, ttype))| (Rc::clone(name), (ttype, i))));
    let mut label_map : LabelMap = HashMap::new();


    todo!()
}

enum LOp {
    Op(Op),
    Label(Rc<str>),
    Branch { label: Rc<str>, var: Rc<str> },
    Jump(Rc<str>),
}

fn compile_stmt(stmt : &Stmt, proc_map : &ProcMap, l_map : &mut LMap) -> Result<Vec<LOp>, CompileError> {
    
    match stmt {
        Stmt::Jump(x) => Ok(vec![LOp::Jump(Rc::clone(x))]),
        Stmt::BranchEqual { label, var } => Ok(vec![LOp::Branch { label: Rc::clone(label), var: Rc::clone(var) }]),
        Stmt::Label(x) => Ok(vec![LOp::Label(Rc::clone(x))]),
        Stmt::Return(x) => {
            todo!()
        },
        _ => todo!(),
    }
    // TODO
    /*
    Set { var: Rc<str>, ttype : Type, val: Expr },
    Yield(Rc<str>),
    Break,
    SlotInsert { var: Rc<str>, input: Rc<str>, index: usize },
    SlotRemove { var: Rc<str>, index: usize },
    */
}
