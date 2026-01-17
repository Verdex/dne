
use std::rc::Rc;
use std::collections::{ HashSet, HashMap };

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
    ReuseParamName { proc: Rc<str>, param_name: Rc<str> },
}

// TODO define error for CompileName

pub fn compile(ir : &[Top]) -> Result<Vec<Proc>, CompileError> {
    let (op_sigs, mut op_code) = primitive_ops();

    let (procs, globals, proc_map) = {
        let mut ps : Vec<&PProc> = vec![];
        let mut gs = vec![];
        for t in ir {
            match t {
                Top::Global(x) => { gs.push(x); },
                Top::Proc(x) => { ps.push(x); },
            }
        }
        let pm = HashMap::from_iter(op_sigs.iter().chain(ps.iter().map(|x| *x)).enumerate().map(|(v, k)| (Rc::clone(&k.name), (k, v))));
        (ps, gs, pm)
    };

    // TODO handle globals (drop them all in an init proc?)

    let mut compiled = procs.into_iter().map(|x| compile_proc(x, &proc_map)).collect::<Result<Vec<_>, _>>()?;
    
    op_code.append(&mut compiled);
    
    Ok(op_code)
}

fn compile_proc(proc : &PProc, proc_map : &ProcMap) -> Result<Proc, CompileError> {

    let mut l_map : LMap = {

        let params : HashSet<Rc<str>> = {
            let mut x = HashSet::new();
            for (name, _) in &proc.params {
                if !x.insert(Rc::clone(name)) {
                    return Err(CompileError::ReuseParamName { proc: Rc::clone(&proc.name), param_name: Rc::clone(name) });
                }
            }
            x
        };

        let mut init_sets = proc.body.iter().filter_map(|stmt| 
            match stmt { 
                Stmt::Set { var, ttype, .. } if !params.contains(var) => Some((Rc::clone(var), *ttype)), 
                _ => None
            } ).collect::<Vec<_>>();

        init_sets.sort_by(|(a, _), (b, _)| a.cmp(b));
        init_sets.dedup_by(|(a, _), (b, _)| a.eq(&b));

        HashMap::from_iter(
            proc.params.iter().map(|(name, ttype)| (Rc::clone(name), *ttype))
            .chain(init_sets)
            .enumerate()
            .map(|(i, (name, ttype))| (Rc::clone(&name), (ttype, i))))
    };

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
            Ok(Op::BranchTrue { local, label: *label_map.get(&label).unwrap() })
        },
        LOp::Branch { label, .. } => Err(CompileError::AccessMissingLabel { proc: Rc::clone(&proc.name), label }),
        LOp::Jump(x) if label_map.contains_key(&x) => Ok(Op::Jump(*label_map.get(&x).unwrap())),
        LOp::Jump(x) => Err(CompileError::AccessMissingLabel { proc: Rc::clone(&proc.name), label: x}),
    }).collect::<Result<Vec<_>, CompileError>>()?;

    let stack_size = l_map.values().map(|(_, x)| *x + 1).max().unwrap_or(0);
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

fn any_access(l_map: &LMap, local: &Rc<str>, proc_name: &Rc<str>) -> Result<usize, CompileError> {
    match l_map.get(local) {
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
        Stmt::BranchTrue { label, var } => Ok(vec![LOp::Branch { label: Rc::clone(label), var: Rc::clone(var) }]),
        Stmt::Label(x) => Ok(vec![LOp::Label(Rc::clone(x))]),
        Stmt::Return(local) => s(Op::ReturnLocal(access(l_map, local, &proc.name, &proc.return_type)?)),
        Stmt::Set { var, val: Expr::Lit(Lit::Int(x)), .. } => s(Op::SetLocalData(access(l_map, &var, &proc.name, &Type::Int)?, RuntimeData::Int(*x))),
        Stmt::Set { var, val: Expr::Lit(Lit::Float(x)), .. } => s(Op::SetLocalData(access(l_map, &var, &proc.name, &Type::Float)?, RuntimeData::Float(*x))),
        Stmt::Set { var, val: Expr::Lit(Lit::Bool(x)), .. } => s(Op::SetLocalData(access(l_map, &var, &proc.name, &Type::Bool)?, RuntimeData::Bool(*x))),
        Stmt::Set { var, val: Expr::Lit(Lit::ConsType(x)), .. } => s(Op::SetLocalData(access(l_map, &var, &proc.name, &Type::Symbol)?, RuntimeData::Symbol(Rc::clone(x)))),
        Stmt::Set { var, val: Expr::Cons { name, params }, .. } => { 
            let target = access(l_map, &var, &proc.name, &Type::Ref)?;
            let sym_var = access(l_map, &name, &proc.name, &Type::Symbol)?;
            let params = params.iter().map(|x| any_access(l_map, x, &proc.name)).collect::<Result<Vec<usize>, _>>()?;
            Ok( vec![LOp::Op(Op::Cons { sym_var, params }),
                     LOp::Op(Op::SetLocalReturn(target))
                    ] )
        },
        Stmt::Set { var, val: Expr::Call { name, params }, .. } => {
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
        Stmt::Set { var, val: Expr::DynCall { name, params }, .. } => {
            let closure = access(l_map, &name, &proc.name, &Type::Closure)?;
            let dest = any_access(l_map, &var, &proc.name)?;

            let params = params.iter().map(|p| any_access(l_map, p, &proc.name)).collect::<Result<Vec<_>, CompileError>>()?;

            Ok(vec![LOp::Op(Op::DynCall(closure, params)), 
                    LOp::Op(Op::SetLocalReturn(dest))
                    ])
        },
        Stmt::Set { var, val: Expr::Closure { name, env }, .. } => {
            let (callee_proc, callee_index) = c(proc_map, &proc.name, name)?;
            let dest = access(l_map, &var, &proc.name, &Type::Closure)?;

            if env.len() > callee_proc.params.len() {
                return Err(CompileError::ProcCallArityMismatch { caller_proc: Rc::clone(&proc.name), callee_proc: Rc::clone(&callee_proc.name) });
            }
            // Note:  This is checking that the first params of the closure function accept the
            // capture environment locals.
            let env_indices = env.iter().zip(callee_proc.params.iter())
                         .map(|(local, (_, ttype))| access(l_map, local, &proc.name, ttype))
                         .collect::<Result<Vec<_>, CompileError>>()?;

            Ok(vec![LOp::Op(Op::Closure { proc_id: *callee_index, env: env_indices } ),
                    LOp::Op(Op::SetLocalReturn(dest))
                   ])
        },
        Stmt::Set { var, ttype, val: Expr::Var(src) } => {
            let src = access(l_map, &src, &proc.name, ttype)?;
            let dest = access(l_map, &var, &proc.name, ttype)?;

            s(Op::SetLocalVar { src, dest })
        },
        Stmt::Set { var: dest, val: Expr::Slot { var: src, index }, .. } => {
            let src = access(l_map, &src, &proc.name, &Type::Ref)?; 
            let dest = any_access(l_map, &dest, &proc.name)?;

            Ok(vec![LOp::Op(Op::GetSlot { local: src, index: *index }),
                    LOp::Op(Op::SetLocalReturn(dest))])
        },
        Stmt::Set { var: dest, val: Expr::Length(src), .. } => {
            let src = access(l_map, &src, &proc.name, &Type::Ref)?; 
            let dest = access(l_map, &dest, &proc.name, &Type::Int)?;

            Ok(vec![LOp::Op(Op::GetLength(src)),
                    LOp::Op(Op::SetLocalReturn(dest))])
        },
        Stmt::Set { var: dest, val: Expr::Type(src), .. } => {
            let src = access(l_map, &src, &proc.name, &Type::Ref)?; 
            let dest = access(l_map, &dest, &proc.name, &Type::Symbol)?;

            Ok(vec![LOp::Op(Op::GetType(src)),
                    LOp::Op(Op::SetLocalReturn(dest))])
        },
        Stmt::Set { var: dest, val: Expr::Coroutine { name, params }, .. } => {
            let (callee_proc, callee_index) = c(proc_map, &proc.name, name)?;
            let dest = access(l_map, &dest, &proc.name, &Type::Coroutine)?;

            if params.len() != callee_proc.params.len() {
                return Err(CompileError::ProcCallArityMismatch { caller_proc: Rc::clone(&proc.name), callee_proc: Rc::clone(&callee_proc.name) });
            }

            let local_indices = params.iter().zip(callee_proc.params.iter())
                         .map(|(local, (_, ttype))| access(l_map, local, &proc.name, ttype))
                         .collect::<Result<Vec<_>, CompileError>>()?;

            Ok(vec![LOp::Op(Op::Coroutine { proc_id: *callee_index, params: local_indices }),
                    LOp::Op(Op::SetLocalReturn(dest))])
        },
        Stmt::Set { var: dest, val: Expr::DynCoroutine { name, params }, .. } => {
            let closure = access(l_map, &name, &proc.name, &Type::Closure)?;
            let dest = access(l_map, &dest, &proc.name, &Type::Coroutine)?;

            let params = params.iter().map(|p| any_access(l_map, p, &proc.name)).collect::<Result<Vec<_>, CompileError>>()?;

            Ok(vec![LOp::Op(Op::DynCoroutine { local: closure, params }),
                    LOp::Op(Op::SetLocalReturn(dest))])
        },
        Stmt::SlotInsert { var, input, index } => {
            let src = any_access(l_map, &input, &proc.name)?;
            let dest = access(l_map, &var, &proc.name, &Type::Ref)?;

            s(Op::InsertSlot { dest, src, index: *index })
        },
        Stmt::SlotRemove { var, index } => {
            let local = access(l_map, &var, &proc.name, &Type::Ref)?;

            s(Op::RemoveSlot { local, index: *index })
        },
        Stmt::Delete(local) => s(Op::Delete(access(l_map, local, &proc.name, &Type::Ref)?)),
        Stmt::Break => s(Op::Break),
        Stmt::Yield(local) => s(Op::Yield(access(l_map, local, &proc.name, &proc.return_type)?)),
        _ => todo!(),
    }
    // TODO
    /*
    Set { var: Rc<str>, ttype : Type, val: Expr },
    */

    // TODO
    /*

    Resume(Rc<str>),
    */
}

fn primitive_ops() -> (Vec<PProc>, Vec<Proc>) {
    fn bin(input : Op) -> Vec<Op> { vec![input, Op::SetLocalReturn(2), Op::ReturnLocal(2)] }
    fn uni(input : Op) -> Vec<Op> { vec![input, Op::SetLocalReturn(1), Op::ReturnLocal(1)] }

    let sigs = vec![ 
        PProc { name: "add_float".into(), params: vec![("a".into(), Type::Float), ("b".into(), Type::Float)], return_type: Type::Float, body: vec![] },
        PProc { name: "add_int".into(), params: vec![("a".into(), Type::Int), ("b".into(), Type::Int)], return_type: Type::Int, body: vec![] },
        PProc { name: "sub_float".into(), params: vec![("a".into(), Type::Float), ("b".into(), Type::Float)], return_type: Type::Float, body: vec![] },
        PProc { name: "sub_int".into(), params: vec![("a".into(), Type::Int), ("b".into(), Type::Int)], return_type: Type::Int, body: vec![] },
        PProc { name: "mul_float".into(), params: vec![("a".into(), Type::Float), ("b".into(), Type::Float)], return_type: Type::Float, body: vec![] },
        PProc { name: "mul_int".into(), params: vec![("a".into(), Type::Int), ("b".into(), Type::Int)], return_type: Type::Int, body: vec![] },
        PProc { name: "div_float".into(), params: vec![("a".into(), Type::Float), ("b".into(), Type::Float)], return_type: Type::Float, body: vec![] },
        PProc { name: "div_int".into(), params: vec![("a".into(), Type::Int), ("b".into(), Type::Int)], return_type: Type::Int, body: vec![] },
        PProc { name: "mod_float".into(), params: vec![("a".into(), Type::Float), ("b".into(), Type::Float)], return_type: Type::Float, body: vec![] },
        PProc { name: "mod_int".into(), params: vec![("a".into(), Type::Int), ("b".into(), Type::Int)], return_type: Type::Int, body: vec![] },
        PProc { name: "neg_float".into(), params: vec![("a".into(), Type::Float)], return_type: Type::Float, body: vec![] },
        PProc { name: "neg_int".into(), params: vec![("a".into(), Type::Int)], return_type: Type::Int, body: vec![] },

        PProc { name: "and".into(), params: vec![("a".into(), Type::Bool), ("b".into(), Type::Bool)], return_type: Type::Bool, body: vec![] },
        PProc { name: "or".into(), params: vec![("a".into(), Type::Bool), ("b".into(), Type::Bool)], return_type: Type::Bool, body: vec![] },
        PProc { name: "xor".into(), params: vec![("a".into(), Type::Bool), ("b".into(), Type::Bool)], return_type: Type::Bool, body: vec![] },
        PProc { name: "not".into(), params: vec![("a".into(), Type::Bool)], return_type: Type::Bool, body: vec![] },

        PProc { name: "gt_float".into(), params: vec![("a".into(), Type::Float), ("b".into(), Type::Float)], return_type: Type::Bool, body: vec![] },
        PProc { name: "gt_int".into(), params: vec![("a".into(), Type::Int), ("b".into(), Type::Int)], return_type: Type::Bool, body: vec![] },
        PProc { name: "lt_float".into(), params: vec![("a".into(), Type::Float), ("b".into(), Type::Float)], return_type: Type::Bool, body: vec![] },
        PProc { name: "lt_int".into(), params: vec![("a".into(), Type::Int), ("b".into(), Type::Int)], return_type: Type::Bool, body: vec![] },

        PProc { name: "eq_float".into(), params: vec![("a".into(), Type::Float), ("b".into(), Type::Float)], return_type: Type::Bool, body: vec![] },
        PProc { name: "eq_int".into(), params: vec![("a".into(), Type::Int), ("b".into(), Type::Int)], return_type: Type::Bool, body: vec![] },
        PProc { name: "eq_bool".into(), params: vec![("a".into(), Type::Bool), ("b".into(), Type::Bool)], return_type: Type::Bool, body: vec![] },
        PProc { name: "eq_symbol".into(), params: vec![("a".into(), Type::Symbol), ("b".into(), Type::Symbol)], return_type: Type::Bool, body: vec![] },
        PProc { name: "eq_ref".into(), params: vec![("a".into(), Type::Ref), ("b".into(), Type::Ref)], return_type: Type::Bool, body: vec![] },
    ];

    let code = vec![ 
        Proc { name: "add_float".into(), instrs: bin(Op::Add(0, 1)), stack_size: 3 },
        Proc { name: "add_int".into(), instrs: bin(Op::Add(0, 1)), stack_size: 3 },
        Proc { name: "sub_float".into(), instrs: bin(Op::Sub(0, 1)), stack_size: 3 },
        Proc { name: "sub_int".into(), instrs: bin(Op::Sub(0, 1)), stack_size: 3 },
        Proc { name: "mul_float".into(), instrs: bin(Op::Mul(0, 1)), stack_size: 3 },
        Proc { name: "mul_int".into(), instrs: bin(Op::Mul(0, 1)), stack_size: 3 },
        Proc { name: "div_float".into(), instrs: bin(Op::Div(0, 1)), stack_size: 3 },
        Proc { name: "div_int".into(), instrs: bin(Op::Div(0, 1)), stack_size: 3 },
        Proc { name: "mod_float".into(), instrs: bin(Op::Mod(0, 1)), stack_size: 3 },
        Proc { name: "mod_int".into(), instrs: bin(Op::Mod(0, 1)), stack_size: 3 },
        Proc { name: "neg_float".into(), instrs: uni(Op::Neg(0)), stack_size: 2 },
        Proc { name: "neg_int".into(), instrs: uni(Op::Neg(0)), stack_size: 2 },

        Proc { name: "and".into(), instrs: bin(Op::And(0, 1)), stack_size: 3 },
        Proc { name: "or".into(), instrs: bin(Op::Or(0, 1)), stack_size: 3 },
        Proc { name: "xor".into(), instrs: bin(Op::Xor(0, 1)), stack_size: 3 },
        Proc { name: "not".into(), instrs: uni(Op::Not(0)), stack_size: 2 },

        Proc { name: "gt_float".into(), instrs: bin(Op::Gt(0, 1)), stack_size: 3 },
        Proc { name: "gt_int".into(), instrs: bin(Op::Gt(0, 1)), stack_size: 3 },
        Proc { name: "lt_float".into(), instrs: bin(Op::Lt(0, 1)), stack_size: 3 },
        Proc { name: "lt_int".into(), instrs: bin(Op::Lt(0, 1)), stack_size: 3 },

        Proc { name: "eq_float".into(), instrs: bin(Op::Eq(0, 1)), stack_size: 3 },
        Proc { name: "eq_int".into(), instrs: bin(Op::Eq(0, 1)), stack_size: 3 },
        Proc { name: "eq_bool".into(), instrs: bin(Op::Eq(0, 1)), stack_size: 3 },
        Proc { name: "eq_symbol".into(), instrs: bin(Op::Eq(0, 1)), stack_size: 3 },
        Proc { name: "eq_ref".into(), instrs: bin(Op::Eq(0, 1)), stack_size: 3 },
    ];

    (sigs, code)
}

#[cfg(test)]
mod test {
    use super::*;
    fn proc(params: Vec<(Rc<str>, Type)>, sets: Vec<(Rc<str>, Type, Lit)>) -> PProc {
        let body = sets.into_iter().map(|(n, t, v)| Stmt::Set { var: n, ttype: t, val: Expr::Lit(v) }).collect::<Vec<_>>();
        PProc { name: "a".into(), params, body, return_type: Type::Int }
    }
    
    #[test]
    fn should_calculate_zero_param_only_stack_size() {
        let input = proc(vec![], vec![]);
        let output = compile_proc(&input, &HashMap::from([])).unwrap(); 
        assert_eq!(output.stack_size, 0);
    }

    #[test]
    fn should_calculate_single_param_only_stack_size() {
        let input = proc(vec![("a".into(), Type::Int)], vec![]);
        let output = compile_proc(&input, &HashMap::from([])).unwrap(); 
        assert_eq!(output.stack_size, 1);
    }

    #[test]
    fn should_calculate_two_params_only_stack_size() {
        let input = proc(vec![("a".into(), Type::Int), ("b".into(), Type::Int)], vec![]);
        let output = compile_proc(&input, &HashMap::from([])).unwrap(); 
        assert_eq!(output.stack_size, 2);
    }

    #[test]
    fn should_calculate_single_local_only_stack_size() {
        let input = proc(vec![], vec![("a".into(), Type::Int, Lit::Int(0))]);
        let output = compile_proc(&input, &HashMap::from([])).unwrap(); 
        assert_eq!(output.stack_size, 1);
    }

    #[test]
    fn should_calculate_two_locals_only_stack_size() {
        let input = proc(vec![], vec![("a".into(), Type::Int, Lit::Int(0)), ("b".into(), Type::Int, Lit::Int(0))]);
        let output = compile_proc(&input, &HashMap::from([])).unwrap(); 
        assert_eq!(output.stack_size, 2);
    }

    #[test]
    fn should_error_with_duplicate_params() {
        let input = proc(vec![("a".into(), Type::Int), ("a".into(), Type::Int)], vec![]);
        let output = compile_proc(&input, &HashMap::from([])); 
        assert!(matches!(output, Err(CompileError::ReuseParamName { .. })));
    }

    #[test]
    fn should_calculate_stack_size_with_duplicate_set() {
        let input = proc(vec![], vec![("a".into(), Type::Int, Lit::Int(0)), ("a".into(), Type::Int, Lit::Int(0))]);
        let output = compile_proc(&input, &HashMap::from([])).unwrap(); 
        assert_eq!(output.stack_size, 1);
    }

    #[test]
    fn should_calculate_stack_size_with_setting_a_param() {
        let input = proc(vec![("a".into(), Type::Int)], vec![("a".into(), Type::Int, Lit::Int(0))]);
        let output = compile_proc(&input, &HashMap::from([])).unwrap(); 
        assert_eq!(output.stack_size, 1);
    }

    #[test]
    fn should_error_with_param_set_type_mismatch() {
        let input = proc(vec![("a".into(), Type::Float)], vec![("a".into(), Type::Int, Lit::Int(0))]);
        let output = compile_proc(&input, &HashMap::from([])); 
        assert!(matches!(output, Err(CompileError::TypeMismatch { .. })));
    }

    #[test]
    fn should_error_with_duplicate_set_type_mismatch() {
        let input = proc(vec![], vec![("a".into(), Type::Int, Lit::Int(0)), ("a".into(), Type::Float, Lit::Float(1.0))]);
        let output = compile_proc(&input, &HashMap::from([])); 
        assert!(matches!(output, Err(CompileError::TypeMismatch { .. })));
    }
}

