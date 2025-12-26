
use std::rc::Rc;
use std::collections::HashMap;

use crate::parsing::ir_parser::{Lit, Expr, Type, Stmt, Top, Proc as PProc, Global};
use crate::eval::data::*;

type ProcMap<'a> = HashMap<Rc<str>, (&'a PProc, usize)>;
type LMap = HashMap<Rc<str>, (Type, usize)>;
type LabelMap = HashMap<Rc<str>, usize>;

#[derive(Debug)]
pub enum CompileError {
    AccessMissingLocal { proc: Rc<str>, local: Rc<str> },
    AccessMissingProc { caller_proc: Rc<str>, callee_proc: Rc<str> },
    AccessMissingLabel { proc: Rc<str>, label: Rc<str> },
    ProcCallArityMismatch { caller_proc: Rc<str>, callee_proc: Rc<str> },
    TypeMismatch { proc: Rc<str>, expected: Rc<str>, found : Rc<str> },
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

    let mut l_map : LMap = HashMap::from_iter(
        proc.params.iter().map(|(name, ttype)| (Rc::clone(name), *ttype))
        .chain(proc.body.iter().filter_map(|stmt| match stmt { Stmt::Set { var, ttype, .. } => Some((Rc::clone(var), *ttype)), _ => None} ))
        .enumerate()
        .map(|(i, (name, ttype))| (Rc::clone(&name), (ttype, i))));

    let mut stmts = vec![];
    for stmt in &proc.body {
        stmts.push(compile_stmt(proc, stmt, proc_map, &mut l_map)?);
    }

    let label_map : LabelMap = HashMap::from_iter(stmts.iter().flatten().enumerate().filter_map(|(index, op)| match op {
        LOp::Label(x) => Some((Rc::clone(x), index)),
        _ => None,
    }));

    let instrs = stmts.into_iter().flatten().map(|op| match op {
        LOp::Op(x) => Ok(x),
        LOp::Label(_) => Ok(Op::Nop),
        LOp::Branch { label, var } if label_map.contains_key(&label) => {
            let local = access(&l_map, &var, &proc.name, &Type::Bool)?;
            Ok(Op::BranchEqual { local, label: *label_map.get(&label).unwrap() })
        },
        LOp::Branch { label, .. } => Err(CompileError::AccessMissingLabel { proc: Rc::clone(&proc.name), label }),
        LOp::Jump(x) if label_map.contains_key(&x) => Ok(Op::Jump(*label_map.get(&x).unwrap())),
        LOp::Jump(x) => Err(CompileError::AccessMissingLabel { proc: Rc::clone(&proc.name), label: x}),
    }).collect::<Result<Vec<_>, CompileError>>()?;

    let stack_size = l_map.values().map(|(_, x)| *x).max().unwrap_or(0);
    Ok(Proc { name: Rc::clone(&proc.name), instrs, stack_size })
}

enum LOp {
    Op(Op),
    Label(Rc<str>),
    Branch { label: Rc<str>, var: Rc<str> },
    Jump(Rc<str>),
}

fn access(l_map: &LMap, local: &Rc<str>, proc_name: &Rc<str>, expected_type: &Type) -> Result<usize, CompileError> {
    match l_map.get(local) {
        Some((found_type, _)) if !expected_type.eq( found_type ) => Err(CompileError::TypeMismatch { 
            proc: Rc::clone(proc_name),
            expected: format!("{:?}", expected_type).into(),
            found: format!("{:?}", found_type).into()
        }),
        Some((_, t)) => Ok(*t),
        None => Err(CompileError::AccessMissingLocal { proc: Rc::clone(proc_name), local: Rc::clone(local) }),
    }
}

fn compile_stmt(proc: &PProc, stmt : &Stmt, proc_map : &ProcMap, l_map : &mut LMap) -> Result<Vec<LOp>, CompileError> {
    
    fn s(x : Op) -> Result<Vec<LOp>, CompileError> { Ok(vec![LOp::Op(x)]) }

    fn c<'a, 'b>(proc_map: &'b ProcMap<'a>, caller_proc_name: &Rc<str>, callee_proc_name: &Rc<str>) -> Result<&'b (&'a PProc, usize), CompileError> {
        match proc_map.get(callee_proc_name) {
            Some(t) => Ok(t),
            None => Err(CompileError::AccessMissingProc { caller_proc: Rc::clone(caller_proc_name), callee_proc: Rc::clone(callee_proc_name) }),
        }
    }

    match stmt {
        Stmt::Jump(x) => Ok(vec![LOp::Jump(Rc::clone(x))]),
        Stmt::BranchEqual { label, var } => Ok(vec![LOp::Branch { label: Rc::clone(label), var: Rc::clone(var) }]),
        Stmt::Label(x) => Ok(vec![LOp::Label(Rc::clone(x))]),
        Stmt::Return(local) => s(Op::ReturnLocal(access(l_map, local, &proc.name, &proc.return_type)?)),
        Stmt::Set { var, ttype, val: Expr::Lit(Lit::Int(x)) } => s(Op::SetLocalData(access(l_map, &var, &proc.name, &ttype)?, RuntimeData::Int(*x))),
        Stmt::Set { var, ttype, val: Expr::Lit(Lit::Float(x)) } => s(Op::SetLocalData(access(l_map, &var, &proc.name, &ttype)?, RuntimeData::Float(*x))),
        Stmt::Set { var, ttype, val: Expr::Lit(Lit::Bool(x)) } => s(Op::SetLocalData(access(l_map, &var, &proc.name, &ttype)?, RuntimeData::Bool(*x))),
        Stmt::Set { var, ttype, val: Expr::Lit(Lit::ConsType(x)) } => s(Op::SetLocalData(access(l_map, &var, &proc.name, &ttype)?, RuntimeData::Symbol(Rc::clone(x)))),
        Stmt::Set { var, ttype, val: Expr::Call { name, params } } => {
            let (callee_proc, callee_index) = c(proc_map, &proc.name, name)?;
            let local_index = access(l_map, &var, &proc.name, &callee_proc.return_type)?;

            if params.len() != callee_proc.params.len() {
                return Err(CompileError::ProcCallArityMismatch { caller_proc: Rc::clone(&proc.name), callee_proc: Rc::clone(&callee_proc.name) });
            }

            let local_indices = params.iter().zip(callee_proc.params.iter())
                         .map(|(local, (_, ttype))| access(l_map, local, &proc.name, ttype))
                         .collect::<Result<Vec<_>, CompileError>>()?;

            Ok(vec![LOp::Op(Op::Call(*callee_index, local_indices)), 
                    LOp::Op(Op::SetLocalReturn(local_index))
                    ])
        },
        Stmt::Set { var, ttype, val: Expr::Var(src) } => {
            let src = access(l_map, &src, &proc.name, ttype)?;
            let dest = access(l_map, &var, &proc.name, ttype)?;

            s(Op::SetLocalVar { src, dest })
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

    // TODO
    /*

    DynCall { name : Rc<str>, params : Vec<Rc<str>> },
    Coroutine { name : Rc<str>, params : Vec<Rc<str>> },
    DynCoroutine { name : Rc<str>, params : Vec<Rc<str>> },
    Closure { name : Rc<str>, params : Vec<Rc<str>> },
    Cons { name : Rc<str>, params : Vec<Rc<str>> },
    Resume(Rc<str>),
    Length(Rc<str>),
    Type(Rc<str>),
    Slot { var: Rc<str>, index: usize },
    */
}
