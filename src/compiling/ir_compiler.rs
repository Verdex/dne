
use std::rc::Rc;
use std::collections::HashMap;

use crate::parsing::ir_parser::{Lit, Expr, Type, Stmt, Top, Proc as PProc, Global};
use crate::eval::data::*;

type ProcMap<'a> = HashMap<Rc<str>, (&'a PProc, usize)>;
type LMap<'a> = HashMap<Rc<str>, (&'a Type, usize)>;
type LabelMap = HashMap<Rc<str>, usize>;

#[derive(Debug)]
pub enum CompileError {
    AccessMissingLocal { proc: Rc<str>, local: Rc<str> },
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

    let mut stmts = vec![];
    for stmt in &proc.body {
        stmts.push(compile_stmt(proc, stmt, proc_map, &mut l_map)?);
    }

    todo!()
}

enum LOp {
    Op(Op),
    Label(Rc<str>),
    Branch { label: Rc<str>, var: Rc<str> },
    Jump(Rc<str>),
}

fn compile_stmt(proc: &PProc, stmt : &Stmt, proc_map : &ProcMap, l_map : &mut LMap) -> Result<Vec<LOp>, CompileError> {
    
    fn s(x : Op) -> Result<Vec<LOp>, CompileError> { Ok(vec![LOp::Op(x)]) }

    fn a(l_map: &LMap, local: &Rc<str>, proc_name: &Rc<str>, expected_type: &Type) -> Result<usize, CompileError> {
        match l_map.get(local) {
            // TODO make sure expected type matches this local type
            Some((_, t)) => Ok(*t),
            None => Err(CompileError::AccessMissingLocal { proc: Rc::clone(proc_name), local: Rc::clone(local) }),
        }
    }

    match stmt {
        Stmt::Jump(x) => Ok(vec![LOp::Jump(Rc::clone(x))]),
        Stmt::BranchEqual { label, var } => Ok(vec![LOp::Branch { label: Rc::clone(label), var: Rc::clone(var) }]),
        Stmt::Label(x) => Ok(vec![LOp::Label(Rc::clone(x))]),
        Stmt::Return(local) => {
            let local = a(l_map, local, &proc.name, &proc.return_type)?;
            s(Op::ReturnLocal(local))
        },
        Stmt::Set { var, ttype, val: Expr::Lit(Lit::Int(x)) } => { s(Op::SetLocalData(a(l_map, &var, &proc.name, &ttype)?, RuntimeData::Int(*x))) },
        Stmt::Set { var, ttype, val: Expr::Lit(Lit::Float(x)) } => { s(Op::SetLocalData(a(l_map, &var, &proc.name, &ttype)?, RuntimeData::Float(*x))) },
        Stmt::Set { var, ttype, val: Expr::Lit(Lit::Bool(x)) } => { s(Op::SetLocalData(a(l_map, &var, &proc.name, &ttype)?, RuntimeData::Bool(*x))) },
        Stmt::Set { var, ttype, val: Expr::Lit(Lit::ConsType(x)) } => { s(Op::SetLocalData(a(l_map, &var, &proc.name, &ttype)?, RuntimeData::Symbol(Rc::clone(x)))) },
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

    // TODO
    /*

    Call { name : Rc<str>, params : Vec<Rc<str>> },
    DynCall { name : Rc<str>, params : Vec<Rc<str>> },
    Coroutine { name : Rc<str>, params : Vec<Rc<str>> },
    DynCoroutine { name : Rc<str>, params : Vec<Rc<str>> },
    Closure { name : Rc<str>, params : Vec<Rc<str>> },
    Cons { name : Rc<str>, params : Vec<Rc<str>> },
    Resume(Rc<str>),
    Length(Rc<str>),
    Type(Rc<str>),
    Var(Rc<str>),
    Slot { var: Rc<str>, index: usize },
    */
}
