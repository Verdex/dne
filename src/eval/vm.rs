
use std::rc::Rc;

use crate::util::proj;

use super::data::*;
use super::error::*;

enum Heap {
    Cons { name: Rc<str>, params: Vec<RuntimeData> },
    Nil,
}

pub struct Vm {
    procs: Vec<Proc>,
    heap: Vec<Heap>,
    frames : Vec<Frame>,
    current : Frame,
}

impl Vm {
    pub fn new(procs: Vec<Proc>) -> Self {
        let current = Frame { proc_id: 0, ip: 0, locals: vec![] };
        Vm { procs, heap: vec![], frames: vec![], current }
    }

    pub fn run(&mut self, entry : usize) -> Result<Option<RuntimeData>, VmError> {
        if entry >= self.procs.len() {
            return Err(VmError::ProcDoesNotExist(entry, self.stack_trace()));
        }

        self.current.proc_id = entry;
        self.current.locals = std::iter::repeat(RuntimeData::Nil).take(self.procs[entry].stack_size).collect();

        let mut ret : Option<RuntimeData> = None;
        loop {
            if self.current.ip >= self.procs[self.current.proc_id].instrs.len() {
                // Note:  if the current procedure isn't pushed onto the return stack, then the
                // stack trace will leave out the current procedure where the problem is occurring.
                return Err(VmError::InstrPointerOutOfRange(self.current.ip, self.stack_trace()));
            }

            match self.procs[self.current.proc_id].instrs[self.current.ip] {
                Op::Call(proc_id, _) if proc_id >= self.procs.len() => {
                    return Err(VmError::ProcDoesNotExist(self.current.proc_id, self.stack_trace()));
                },
                Op::Call(proc_id, ref params) => {
                    let mut new_locals = vec![];
                    for param in params {
                        match get_local(*param, &self.current.locals) {
                            Ok(v) => { new_locals.push(v); },
                            Err(f) => { 
                                return Err(f(self.stack_trace()));
                            },
                        }
                    }
                    self.current.ip += 1;
                    new_locals.append(&mut std::iter::repeat(RuntimeData::Nil).take(self.procs[proc_id].stack_size - params.len()).collect());
                    let current = std::mem::replace(&mut self.current, Frame { proc_id: proc_id, ip: 0, locals: new_locals });
                    self.frames.push(current);
                },
                Op::DynCall(local, _) if local >= self.current.locals.len() => {
                    return Err(VmError::AccessMissingLocal(local, self.stack_trace()));
                },
                Op::DynCall(local, _) if !matches!( self.current.locals[local], RuntimeData::Int(_) ) => {
                    return self.local_unexpected_type(local, "Int");
                },
                Op::DynCall(local, ref params) => {
                    let proc_id = {
                        let proc_id = proj!(self.current.locals[local], RuntimeData::Int(x), x); 
                        match usize::try_from(proc_id) {
                            Ok(v) => v, 
                            Err(_) => { return self.local_unexpected_type(local, "proc_id"); },
                        }
                    };
                    let mut new_locals = vec![];
                    for param in params {
                        match get_local(*param, &self.current.locals) {
                            Ok(v) => { new_locals.push(v); },
                            Err(f) => { 
                                return Err(f(self.stack_trace()));
                            },
                        }
                    }
                    self.current.ip += 1;
                    new_locals.append(&mut std::iter::repeat(RuntimeData::Nil).take(self.procs[proc_id].stack_size - params.len()).collect());
                    let current = std::mem::replace(&mut self.current, Frame { proc_id: proc_id, ip: 0, locals: new_locals });
                    self.frames.push(current);
                },
                Op::Jump(label) => {
                    self.current.ip = label;
                },
                Op::BranchTrue { local, .. } if local >= self.current.locals.len() => {
                    return Err(VmError::AccessMissingLocal(local, self.stack_trace()));
                },
                Op::BranchTrue { local, .. } if !matches!( self.current.locals[local], RuntimeData::Bool(_) ) => {
                    return self.local_unexpected_type(local, "bool");
                },
                Op::BranchTrue { label, local } => {
                    let test = proj!(self.current.locals[local], RuntimeData::Bool(x), x);
                    if test {
                        self.current.ip = label;
                    }
                    else {
                        self.current.ip += 1;
                    }
                },
                Op::ReturnLocal(local) if local >= self.current.locals.len() => {
                    return Err(VmError::AccessMissingLocal(local, self.stack_trace()));
                },
                Op::ReturnLocal(local) => {
                    ret = Some(self.current.locals.swap_remove(local));

                    match self.frames.pop() {
                        // Note:  if the stack is empty then all execution is finished
                        None => {
                            return Ok(ret);
                        },
                        Some(frame) => {
                            self.current = frame;
                        },
                    }
                },
                Op::SetLocalData(local, _) if local >= self.current.locals.len() => {
                    return Err(VmError::AccessMissingLocal(local, self.stack_trace()));
                },
                Op::SetLocalData(local, ref data) => {
                    self.current.locals[local] = data.clone();
                    self.current.ip += 1;
                },
                Op::SetLocalReturn(_) if ret.is_none()  => {
                    return Err(VmError::AccessMissingReturn(self.stack_trace()));
                },
                Op::SetLocalReturn(local) if local >= self.current.locals.len() => {
                    return Err(VmError::AccessMissingLocal(local, self.stack_trace()));
                },
                Op::SetLocalReturn(local) => {
                    self.current.locals[local] = ret.as_ref().unwrap().clone();
                    self.current.ip += 1;
                },
                Op::SetLocalVar { src, .. } if src >= self.current.locals.len() => {
                    return Err(VmError::AccessMissingLocal(src, self.stack_trace()));
                },
                Op::SetLocalVar { dest, .. } if dest >= self.current.locals.len() => {
                    return Err(VmError::AccessMissingLocal(dest, self.stack_trace()));
                },
                Op::SetLocalVar { src, dest } => {
                    self.current.locals[dest] = self.current.locals[src].clone();
                    self.current.ip += 1;
                },

                Op::Add(local, _) if local >= self.current.locals.len() => {
                    return Err(VmError::AccessMissingLocal(local, self.stack_trace()));
                },
                Op::Add(_, local) if local >= self.current.locals.len() => {
                    return Err(VmError::AccessMissingLocal(local, self.stack_trace()));
                },
                Op::Add(a, b) => { 
                    match (&self.current.locals[a], &self.current.locals[b]) {
                        (RuntimeData::Float(a), RuntimeData::Float(b)) => { ret = Some( RuntimeData::Float(a + b) ); },
                        (RuntimeData::Int(a), RuntimeData::Int(b)) => { ret = Some( RuntimeData::Int(a + b) ); },
                        (RuntimeData::Int(_), _) => { return self.local_unexpected_type(b, "int"); },
                        (RuntimeData::Float(_), _) => { return self.local_unexpected_type(b, "float"); },
                        _ => { return self.local_unexpected_type(a, "number"); },
                    }
                    self.current.ip += 1;
                },

                Op::Sub(local, _) if local >= self.current.locals.len() => {
                    return Err(VmError::AccessMissingLocal(local, self.stack_trace()));
                },
                Op::Sub(_, local) if local >= self.current.locals.len() => {
                    return Err(VmError::AccessMissingLocal(local, self.stack_trace()));
                },
                Op::Sub(a, b) => {
                    match (&self.current.locals[a], &self.current.locals[b]) {
                        (RuntimeData::Float(a), RuntimeData::Float(b)) => { ret = Some( RuntimeData::Float(a - b) ); },
                        (RuntimeData::Int(a), RuntimeData::Int(b)) => { ret = Some( RuntimeData::Int(a - b) ); },
                        (RuntimeData::Int(_), _) => { return self.local_unexpected_type(b, "int"); },
                        (RuntimeData::Float(_), _) => { return self.local_unexpected_type(b, "float"); },
                        _ => { return self.local_unexpected_type(a, "number"); },
                    }
                    self.current.ip += 1;
                },

                Op::Mul(local, _) if local >= self.current.locals.len() => {
                    return Err(VmError::AccessMissingLocal(local, self.stack_trace()));
                },
                Op::Mul(_, local) if local >= self.current.locals.len() => {
                    return Err(VmError::AccessMissingLocal(local, self.stack_trace()));
                },
                Op::Mul(a, b) => { 
                    match (&self.current.locals[a], &self.current.locals[b]) {
                        (RuntimeData::Float(a), RuntimeData::Float(b)) => { ret = Some( RuntimeData::Float(a * b) ); },
                        (RuntimeData::Int(a), RuntimeData::Int(b)) => { ret = Some( RuntimeData::Int(a * b) ); },
                        (RuntimeData::Int(_), _) => { return self.local_unexpected_type(b, "int"); },
                        (RuntimeData::Float(_), _) => { return self.local_unexpected_type(b, "float"); },
                        _ => { return self.local_unexpected_type(a, "number"); },
                    }
                    self.current.ip += 1;
                },

                Op::Div(local, _) if local >= self.current.locals.len() => {
                    return Err(VmError::AccessMissingLocal(local, self.stack_trace()));
                },
                Op::Div(_, local) if local >= self.current.locals.len() => {
                    return Err(VmError::AccessMissingLocal(local, self.stack_trace()));
                },
                Op::Div(a, b) => { 
                    match (&self.current.locals[a], &self.current.locals[b]) {
                        (RuntimeData::Float(a), RuntimeData::Float(b)) => { ret = Some( RuntimeData::Float(a / b) ); },
                        (RuntimeData::Int(a), RuntimeData::Int(b)) => { ret = Some( RuntimeData::Int(a / b) ); },
                        (RuntimeData::Int(_), _) => { return self.local_unexpected_type(b, "int"); },
                        (RuntimeData::Float(_), _) => { return self.local_unexpected_type(b, "float"); },
                        _ => { return self.local_unexpected_type(a, "number"); },
                    }
                    self.current.ip += 1;
                },

                Op::Mod(local, _) if local >= self.current.locals.len() => {
                    return Err(VmError::AccessMissingLocal(local, self.stack_trace()));
                },
                Op::Mod(_, local) if local >= self.current.locals.len() => {
                    return Err(VmError::AccessMissingLocal(local, self.stack_trace()));
                },
                Op::Mod(a, b) => {
                    match (&self.current.locals[a], &self.current.locals[b]) {
                        (RuntimeData::Float(a), RuntimeData::Float(b)) => { ret = Some( RuntimeData::Float(a % b) ); },
                        (RuntimeData::Int(a), RuntimeData::Int(b)) => { ret = Some( RuntimeData::Int(a % b) ); },
                        (RuntimeData::Int(_), _) => { return self.local_unexpected_type(b, "int"); },
                        (RuntimeData::Float(_), _) => { return self.local_unexpected_type(b, "float"); },
                        _ => { return self.local_unexpected_type(a, "number"); },
                    }
                    self.current.ip += 1;
                },

                Op::Neg(local) if local >= self.current.locals.len() => {
                    return Err(VmError::AccessMissingLocal(local, self.stack_trace()));
                },
                Op::Neg(x) => { 
                    match &self.current.locals[x] {
                        RuntimeData::Float(x) => { ret = Some( RuntimeData::Float(-x) ); },        
                        RuntimeData::Int(x) => { ret = Some( RuntimeData::Int(-x) ); },        
                        _ => { return self.local_unexpected_type(x, "number"); },
                    }
                    self.current.ip += 1;
                },

                Op::Eq(local, _) if local >= self.current.locals.len() => {
                    return Err(VmError::AccessMissingLocal(local, self.stack_trace()));
                },
                Op::Eq(_, local) if local >= self.current.locals.len() => {
                    return Err(VmError::AccessMissingLocal(local, self.stack_trace()));
                },
                Op::Eq(a, b) => { 
                    match (&self.current.locals[a], &self.current.locals[b]) {
                        (RuntimeData::Float(a), RuntimeData::Float(b)) => { ret = Some( RuntimeData::Bool(a == b) ); },
                        (RuntimeData::Int(a), RuntimeData::Int(b)) => { ret = Some( RuntimeData::Bool(a == b) ); },
                        (RuntimeData::Bool(a), RuntimeData::Bool(b)) => { ret = Some( RuntimeData::Bool(a == b) ); },
                        (RuntimeData::Symbol(a), RuntimeData::Symbol(b)) => { ret = Some( RuntimeData::Bool(a == b) ); },
                        (RuntimeData::Nil, RuntimeData::Nil) => { ret = Some( RuntimeData::Bool(true) ); },
                        (RuntimeData::Ref(a), RuntimeData::Ref(b)) => { ret = Some( RuntimeData::Bool(a == b) ); },

                        (RuntimeData::Float(a), _) => { return self.local_unexpected_type(b, "float"); },
                        (RuntimeData::Int(a), _) => { return self.local_unexpected_type(b, "int"); },
                        (RuntimeData::Bool(a), _) => { return self.local_unexpected_type(b, "bool"); },
                        (RuntimeData::Symbol(a), _) => { return self.local_unexpected_type(b, "symbol"); },
                        (RuntimeData::Nil, _) => { return self.local_unexpected_type(b, "nil"); },
                        (RuntimeData::Ref(_), _) => { return self.local_unexpected_type(b, "ref"); }, 
                    }
                    self.current.ip += 1;
                },

                Op::Gt(local, _) if local >= self.current.locals.len() => {
                    return Err(VmError::AccessMissingLocal(local, self.stack_trace()));
                },
                Op::Gt(_, local) if local >= self.current.locals.len() => {
                    return Err(VmError::AccessMissingLocal(local, self.stack_trace()));
                },
                Op::Gt(a, b) => { 
                    match (&self.current.locals[a], &self.current.locals[b]) {
                        (RuntimeData::Float(a), RuntimeData::Float(b)) => { ret = Some( RuntimeData::Bool(a > b) ); },
                        (RuntimeData::Int(a), RuntimeData::Int(b)) => { ret = Some( RuntimeData::Bool(a > b) ); },
                        (RuntimeData::Int(_), _) => { return self.local_unexpected_type(b, "int"); },
                        (RuntimeData::Float(_), _) => { return self.local_unexpected_type(b, "float"); },
                        _ => { return self.local_unexpected_type(a, "number"); },
                    }
                    self.current.ip += 1;
                },

                Op::Lt(local, _) if local >= self.current.locals.len() => {
                    return Err(VmError::AccessMissingLocal(local, self.stack_trace()));
                },
                Op::Lt(_, local) if local >= self.current.locals.len() => {
                    return Err(VmError::AccessMissingLocal(local, self.stack_trace()));
                },
                Op::Lt(a, b) => { 
                    match (&self.current.locals[a], &self.current.locals[b]) {
                        (RuntimeData::Float(a), RuntimeData::Float(b)) => { ret = Some( RuntimeData::Bool(a < b) ); },
                        (RuntimeData::Int(a), RuntimeData::Int(b)) => { ret = Some( RuntimeData::Bool(a < b) ); },
                        (RuntimeData::Int(_), _) => { return self.local_unexpected_type(b, "int"); },
                        (RuntimeData::Float(_), _) => { return self.local_unexpected_type(b, "float"); },
                        _ => { return self.local_unexpected_type(a, "number"); },
                    }
                    self.current.ip += 1;
                },

                Op::Not(local) if local >= self.current.locals.len() => {
                    return Err(VmError::AccessMissingLocal(local, self.stack_trace()));
                },
                Op::Not(x) => { 
                    match &self.current.locals[x] {
                        RuntimeData::Bool(x) => { ret = Some( RuntimeData::Bool(!x) ); },        
                        _ => { return self.local_unexpected_type(x, "bool"); },
                    }
                    self.current.ip += 1;
                },

                Op::And(local, _) if local >= self.current.locals.len() => {
                    return Err(VmError::AccessMissingLocal(local, self.stack_trace()));
                },
                Op::And(_, local) if local >= self.current.locals.len() => {
                    return Err(VmError::AccessMissingLocal(local, self.stack_trace()));
                },
                Op::And(a, b) => { 
                    match (&self.current.locals[a], &self.current.locals[b]) {
                        (RuntimeData::Bool(a), RuntimeData::Bool(b)) => { ret = Some( RuntimeData::Bool(*a && *b) ); },
                        (_, RuntimeData::Bool(_)) => { return self.local_unexpected_type(a, "bool");  },
                        _ => { return self.local_unexpected_type(b, "bool");  },
                    }
                    self.current.ip += 1;
                },

                Op::Or(local, _) if local >= self.current.locals.len() => {
                    return Err(VmError::AccessMissingLocal(local, self.stack_trace()));
                },
                Op::Or(_, local) if local >= self.current.locals.len() => {
                    return Err(VmError::AccessMissingLocal(local, self.stack_trace()));
                },
                Op::Or(a, b) => { 
                    match (&self.current.locals[a], &self.current.locals[b]) {
                        (RuntimeData::Bool(a), RuntimeData::Bool(b)) => { ret = Some( RuntimeData::Bool(*a || *b) ); },
                        (_, RuntimeData::Bool(_)) => { return self.local_unexpected_type(a, "bool");  },
                        _ => { return self.local_unexpected_type(b, "bool");  },
                    }
                    self.current.ip += 1;
                },

                Op::Xor(local, _) if local >= self.current.locals.len() => {
                    return Err(VmError::AccessMissingLocal(local, self.stack_trace()));
                },
                Op::Xor(_, local) if local >= self.current.locals.len() => {
                    return Err(VmError::AccessMissingLocal(local, self.stack_trace()));
                },
                Op::Xor(a, b) => { 
                    match (&self.current.locals[a], &self.current.locals[b]) {
                        (RuntimeData::Bool(a), RuntimeData::Bool(b)) => { ret = Some( RuntimeData::Bool(*a ^ *b) ); },
                        (_, RuntimeData::Bool(_)) => { return self.local_unexpected_type(a, "bool");  },
                        _ => { return self.local_unexpected_type(b, "bool");  },
                    }
                    self.current.ip += 1;
                },
                // TODO why not just do everything with a modified get local
                Op::Cons { sym_var, .. } if sym_var >= self.current.locals.len() => {
                    return Err(VmError::AccessMissingLocal(sym_var, self.stack_trace()));
                },
                Op::Cons { sym_var, ref params } => {
                    let params = {
                        let mut ret = vec![];
                        for param in params {
                            match get_local(*param, &self.current.locals) { // TODO get locals?
                                Ok(v) => { actual_params.push(v); },
                                Err(f) => { 
                                    return Err(f(self.stack_trace())); // TODO fix
                                }, 
                            }
                        }
                        ret
                    };
                    let name = match &self.current.locals[sym_var] {
                        RuntimeData::Symbol(x) => Rc::clone(x),
                        _ => { return self.local_unexpected_type(sym_var, "symbol"); },
                    };

                    // TODO pick first nil
                    self.heap.push(Heap::Cons { name, params });
                    ret = Some( RuntimeData::Ref( self.heap.len() ) );
                    self.current.ip += 1;
                },
                Op::Delete(local) => todo!(),

                Op::Nop => { self.current.ip += 1; },
                /*
                Op::Cons { sym_var, ref params } => todo!(),
                Op::Delete(local) => todo!(),
                Op::InsertSlot { dest, src, index } => todo!(),
                Op::RemoveSlot { local, index } => todo!(),
                Op::GetLength(local) => todo!(),
                Op::GetType(local) => todo!(),
                Op::GetSlot { local, index } => todo!(),

                Op::Resume(local) => todo!(),
                Op::Closure { proc_id, env } => todo!(),
                Op::Coroutine { proc_id, params } => todo!(),
                // TODO fix 
                Op::DynCoroutine { proc_id, params } => todo!(),
                Op::Yield(local) => todo!(),
                Op::Break => todo!(),
                */
                _ => todo!(),
                /*
                Op::CoYield(slot) => {

                    let ret_target = match get_local(slot, Cow::Borrowed(&self.current.locals)) {
                        Ok(v) => v,
                        Err(f) => { 
                            return Err(f(self.stack_trace()));
                        },
                    };

                    match self.frames.pop() {
                        None => {
                            // Note: Top level yields are not supported.
                            return Err(VmError::TopLevelYield(self.current.ip)); 
                        },
                        Some(frame) => {
                            self.current.ip += 1;
                            let coroutine = std::mem::replace(&mut self.current, frame);
                            self.current.ret = Some(ret_target);
                            match self.current.coroutines.iter().position(co_is_running) {
                                Some(index) => {
                                    let _ = std::mem::replace(&mut self.current.coroutines[index], Coroutine::Active(coroutine));
                                },
                                None => { 
                                    self.current.coroutines.push(Coroutine::Active(coroutine));
                                },
                            }
                        },
                    }
                },
                Op::CoFinish => {
                    match self.frames.pop() {
                        None => {
                            // Note: Top level yields are not supported.
                            return Err(VmError::TopLevelYield(self.current.ip)); 
                        },
                        Some(frame) => {
                            self.current = frame;
                            self.current.ret = None;

                            match self.current.coroutines.iter().position(co_is_running) {
                                Some(index) => {
                                    let _ = std::mem::replace(&mut self.current.coroutines[index], Coroutine::Finished);
                                },
                                None => { 
                                    self.current.coroutines.push(Coroutine::Finished);
                                },
                            }
                        },
                    }
                },
                Op::CoResume(coroutine) if coroutine < self.current.coroutines.len() => {
                    match std::mem::replace(&mut self.current.coroutines[coroutine], Coroutine::Running) { 
                        Coroutine::Active(frame) => {
                            self.current.ip += 1;
                            let old_current = std::mem::replace(&mut self.current, frame);
                            self.frames.push(old_current);
                        },
                        Coroutine::Finished => {
                            return Err(VmError::ResumeFinishedCoroutine(coroutine, self.stack_trace()))
                        },
                        Coroutine::Running => { unreachable!(); },
                    }
                },
                Op::CoResume(coroutine) => {
                    return Err(VmError::AccessMissingCoroutine(coroutine, self.stack_trace()));
                },
                Op::CoDrop(coroutine) if coroutine < self.current.coroutines.len() => {
                    self.current.coroutines.remove(coroutine);
                    self.current.ip += 1;
                },
                Op::CoDrop(coroutine) => {
                    return Err(VmError::AccessMissingCoroutine(coroutine, self.stack_trace()));
                },
                Op::CoDup(coroutine) if coroutine < self.current.coroutines.len() => {
                    let target = self.current.coroutines[coroutine].clone();
                    self.current.coroutines.push(target);
                    self.current.ip += 1;
                },
                Op::CoDup(coroutine) => {
                    return Err(VmError::AccessMissingCoroutine(coroutine, self.stack_trace()));
                },
                Op::CoSwap(a, b) if a < self.current.coroutines.len() && b < self.current.coroutines.len() => {
                    self.current.coroutines.swap(a, b);
                    self.current.ip += 1;
                },
                Op::CoSwap(a, b) if b < self.current.coroutines.len() => {
                    return Err(VmError::AccessMissingCoroutine(a, self.stack_trace()));
                },
                Op::CoSwap(_, b) => {
                    return Err(VmError::AccessMissingCoroutine(b, self.stack_trace()));
                },
                */
            }
        }
    }

    fn stack_trace(&self) -> StackTrace {
        struct RetAddr { proc: usize, instr : usize }

        let mut stack = self.frames.iter().map(|x| RetAddr { proc: x.proc_id, instr: x.ip }).collect::<Vec<_>>();
        stack.push(RetAddr { proc: self.current.proc_id, instr: self.current.ip + 1});

        let mut trace = vec![];
        for addr in stack {
            // Note:  if the procedure was already pushed into the stack, then
            // that means that it already resolved to a known procedure. Don't
            // have to check again that the proc map has it.
            let name = Rc::clone(&self.procs[addr.proc].name);
            trace.push((name, addr.instr - 1));
        }
        trace
    }

    fn local_unexpected_type<T>(&self, local : usize, expected : &'static str) -> Result<T, VmError> {
        return Err(VmError::LocalUnexpectedType { local, stack_trace: self.stack_trace(), expected, found: format!("{:?}", self.current.locals[local]).into() });
    }
}


fn get_local(index: usize, locals : &[RuntimeData]) -> Result<RuntimeData, Box<dyn Fn(StackTrace) -> VmError>> {
    if index >= locals.len() {
        Err(Box::new(move |trace| VmError::AccessMissingLocal(index, trace)))
    }
    else {
        Ok(locals[index].clone())
    }
}

fn co_is_running(coroutine : &Coroutine) -> bool {
    match coroutine { 
        Coroutine::Running => true,
        _ => false,
    }
}
