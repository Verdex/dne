
use std::rc::Rc;

use crate::util::proj;

use super::data::*;
use super::error::*;

macro_rules! proj_type {
    ($self:expr, $local:expr, bool) => {{
        if $local >= $self.current.locals.len() {
           Err(VmError::AccessMissingLocal($local, $self.stack_trace()))
        }
        else if !matches!( $self.current.locals[$local], RuntimeData::Bool(_) ) {
            $self.local_unexpected_type($local, "bool")
        }
        else {
            Ok(proj!($self.current.locals[$local], RuntimeData::Bool(x), x))
        }
    }};
    ($self:expr, $local:expr, int) => {{
        if $local >= $self.current.locals.len() {
           Err(VmError::AccessMissingLocal($local, $self.stack_trace()))
        }
        else if !matches!( $self.current.locals[$local], RuntimeData::Int(_) ) {
            $self.local_unexpected_type($local, "int")
        }
        else {
            Ok(proj!($self.current.locals[$local], RuntimeData::Int(x), x))
        }
    }};
    ($self:expr, $local:expr, ref) => {{
        if $local >= $self.current.locals.len() {
           Err(VmError::AccessMissingLocal($local, $self.stack_trace()))
        }
        else if !matches!( $self.current.locals[$local], RuntimeData::Ref(_) ) {
            $self.local_unexpected_type($local, "ref")
        }
        else {
            Ok(proj!($self.current.locals[$local], RuntimeData::Ref(x), x))
        }
    }};
    ($self:expr, $local:expr, closure) => {{
        if $local >= $self.current.locals.len() {
           Err(VmError::AccessMissingLocal($local, $self.stack_trace()))
        }
        else if !matches!( $self.current.locals[$local], RuntimeData::Closure(_) ) {
            $self.local_unexpected_type($local, "closure")
        }
        else {
            Ok(proj!($self.current.locals[$local], RuntimeData::Closure(ref x), x))
        }
    }};
    ($self:expr, $local:expr, coroutine) => {{
        if $local >= $self.current.locals.len() {
           Err(VmError::AccessMissingLocal($local, $self.stack_trace()))
        }
        else if !matches!( $self.current.locals[$local], RuntimeData::Coroutine(_) ) {
            $self.local_unexpected_type($local, "coroutine")
        }
        else {
            Ok(proj!($self.current.locals[$local], RuntimeData::Coroutine(ref x), x))
        }
    }};
}

macro_rules! call {
    ($self:expr, $proc_id:expr, $params:expr) => {
        let mut new_locals = $self.clone_locals($params)?;
        $self.current.ip += 1;
        new_locals.append(&mut std::iter::repeat(RuntimeData::Nil).take($self.procs[$proc_id].stack_size - $params.len()).collect());
        let current = std::mem::replace(&mut $self.current, Frame { proc_id: $proc_id, ip: 0, locals: new_locals });
        $self.frames.push(current);
    }
}

/*fn dyn_call(&mut self, local: usize, params: &[usize]) -> Result<(), VmError> {
    let Closure { proc_id, env } = proj_type!(self, local, closure)?; 
    let proc_id = *proc_id;
    let env_and_param_len = env.len() + params.len();
    let mut new_locals = env.clone();
    let mut params = self.clone_locals(params)?;
    new_locals.append(&mut params);
    self.current.ip += 1;

    new_locals.append(&mut std::iter::repeat(RuntimeData::Nil).take(self.procs[proc_id].stack_size - env_and_param_len).collect());
    let current = std::mem::replace(&mut self.current, Frame { proc_id: proc_id, ip: 0, locals: new_locals });
    self.frames.push(current);
    Ok(())
}*/


#[derive(Debug)]
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
                    call!(self, proc_id, params);
                },
                Op::DynCall(local, ref params) => {
                    let Closure { proc_id, env } = proj_type!(self, local, closure)?; 
                    let proc_id = *proc_id;
                    let env_and_param_len = env.len() + params.len();
                    let mut new_locals = env.clone();
                    let mut params = self.clone_locals(params)?;
                    new_locals.append(&mut params);
                    self.current.ip += 1;

                    new_locals.append(&mut std::iter::repeat(RuntimeData::Nil).take(self.procs[proc_id].stack_size - env_and_param_len).collect());
                    let current = std::mem::replace(&mut self.current, Frame { proc_id: proc_id, ip: 0, locals: new_locals });
                    self.frames.push(current);
                },
                Op::Jump(label) => {
                    self.current.ip = label;
                },
                Op::BranchTrue { label, local } => {
                    let test = proj_type!(self, local, bool)?;
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
                Op::SetLocalData(local, ref data) => {
                    *self.mut_local(local)? = data.clone();
                    self.current.ip += 1;
                },
                Op::SetLocalReturn(_) if ret.is_none()  => {
                    return Err(VmError::AccessMissingReturn(self.stack_trace()));
                },
                Op::SetLocalReturn(local) => {
                    *self.mut_local(local)? = std::mem::replace(&mut ret, None).unwrap();
                    self.current.ip += 1;
                },
                Op::SetLocalVar { src, dest } => {
                    *self.mut_local(dest)? = self.get_local(src)?.clone();
                    self.current.ip += 1;
                },

                Op::Add(a, b) => { 
                    match (self.get_local(a)?, self.get_local(b)?) {
                        (RuntimeData::Float(a), RuntimeData::Float(b)) => { ret = Some( RuntimeData::Float(a + b) ); },
                        (RuntimeData::Int(a), RuntimeData::Int(b)) => { ret = Some( RuntimeData::Int(a + b) ); },
                        (RuntimeData::Int(_), _) => { return self.local_unexpected_type(b, "int"); },
                        (RuntimeData::Float(_), _) => { return self.local_unexpected_type(b, "float"); },
                        _ => { return self.local_unexpected_type(a, "number"); },
                    }
                    self.current.ip += 1;
                },

                Op::Sub(a, b) => {
                    match (self.get_local(a)?, self.get_local(b)?) {
                        (RuntimeData::Float(a), RuntimeData::Float(b)) => { ret = Some( RuntimeData::Float(a - b) ); },
                        (RuntimeData::Int(a), RuntimeData::Int(b)) => { ret = Some( RuntimeData::Int(a - b) ); },
                        (RuntimeData::Int(_), _) => { return self.local_unexpected_type(b, "int"); },
                        (RuntimeData::Float(_), _) => { return self.local_unexpected_type(b, "float"); },
                        _ => { return self.local_unexpected_type(a, "number"); },
                    }
                    self.current.ip += 1;
                },

                Op::Mul(a, b) => { 
                    match (self.get_local(a)?, self.get_local(b)?) {
                        (RuntimeData::Float(a), RuntimeData::Float(b)) => { ret = Some( RuntimeData::Float(a * b) ); },
                        (RuntimeData::Int(a), RuntimeData::Int(b)) => { ret = Some( RuntimeData::Int(a * b) ); },
                        (RuntimeData::Int(_), _) => { return self.local_unexpected_type(b, "int"); },
                        (RuntimeData::Float(_), _) => { return self.local_unexpected_type(b, "float"); },
                        _ => { return self.local_unexpected_type(a, "number"); },
                    }
                    self.current.ip += 1;
                },

                Op::Div(a, b) => { 
                    match (self.get_local(a)?, self.get_local(b)?) {
                        (RuntimeData::Float(a), RuntimeData::Float(b)) => { ret = Some( RuntimeData::Float(a / b) ); },
                        (RuntimeData::Int(a), RuntimeData::Int(b)) => { ret = Some( RuntimeData::Int(a / b) ); },
                        (RuntimeData::Int(_), _) => { return self.local_unexpected_type(b, "int"); },
                        (RuntimeData::Float(_), _) => { return self.local_unexpected_type(b, "float"); },
                        _ => { return self.local_unexpected_type(a, "number"); },
                    }
                    self.current.ip += 1;
                },

                Op::Mod(a, b) => {
                    match (self.get_local(a)?, self.get_local(b)?) {
                        (RuntimeData::Float(a), RuntimeData::Float(b)) => { ret = Some( RuntimeData::Float(a % b) ); },
                        (RuntimeData::Int(a), RuntimeData::Int(b)) => { ret = Some( RuntimeData::Int(a % b) ); },
                        (RuntimeData::Int(_), _) => { return self.local_unexpected_type(b, "int"); },
                        (RuntimeData::Float(_), _) => { return self.local_unexpected_type(b, "float"); },
                        _ => { return self.local_unexpected_type(a, "number"); },
                    }
                    self.current.ip += 1;
                },

                Op::Neg(x) => { 
                    match self.get_local(x)? {
                        RuntimeData::Float(x) => { ret = Some( RuntimeData::Float(-x) ); },        
                        RuntimeData::Int(x) => { ret = Some( RuntimeData::Int(-x) ); },        
                        _ => { return self.local_unexpected_type(x, "number"); },
                    }
                    self.current.ip += 1;
                },

                Op::Eq(a, b) => { 
                    match (self.get_local(a)?, self.get_local(b)?) {
                        (RuntimeData::Float(a), RuntimeData::Float(b)) => { ret = Some( RuntimeData::Bool(a == b) ); },
                        (RuntimeData::Int(a), RuntimeData::Int(b)) => { ret = Some( RuntimeData::Bool(a == b) ); },
                        (RuntimeData::Bool(a), RuntimeData::Bool(b)) => { ret = Some( RuntimeData::Bool(a == b) ); },
                        (RuntimeData::Symbol(a), RuntimeData::Symbol(b)) => { ret = Some( RuntimeData::Bool(a == b) ); },
                        (RuntimeData::Nil, RuntimeData::Nil) => { ret = Some( RuntimeData::Bool(true) ); },
                        (RuntimeData::Ref(a), RuntimeData::Ref(b)) => { ret = Some( RuntimeData::Bool(a == b) ); },

                        (RuntimeData::Closure { .. }, RuntimeData::Closure { .. }) => { ret = Some( RuntimeData::Bool(false) ); },
                        (RuntimeData::Coroutine(_), RuntimeData::Coroutine(_)) => { ret = Some( RuntimeData::Bool(false) ); },

                        (RuntimeData::Float(a), _) => { return self.local_unexpected_type(b, "float"); },
                        (RuntimeData::Int(a), _) => { return self.local_unexpected_type(b, "int"); },
                        (RuntimeData::Bool(a), _) => { return self.local_unexpected_type(b, "bool"); },
                        (RuntimeData::Symbol(a), _) => { return self.local_unexpected_type(b, "symbol"); },
                        (RuntimeData::Nil, _) => { return self.local_unexpected_type(b, "nil"); },
                        (RuntimeData::Ref(_), _) => { return self.local_unexpected_type(b, "ref"); }, 
                        (RuntimeData::Closure { .. }, _) => { return self.local_unexpected_type(b, "closure"); },
                        (RuntimeData::Coroutine(_), _) => { return self.local_unexpected_type(b, "coroutine"); },
                    }
                    self.current.ip += 1;
                },

                Op::Gt(a, b) => { 
                    match (self.get_local(a)?, self.get_local(b)?) {
                        (RuntimeData::Float(a), RuntimeData::Float(b)) => { ret = Some( RuntimeData::Bool(a > b) ); },
                        (RuntimeData::Int(a), RuntimeData::Int(b)) => { ret = Some( RuntimeData::Bool(a > b) ); },
                        (RuntimeData::Int(_), _) => { return self.local_unexpected_type(b, "int"); },
                        (RuntimeData::Float(_), _) => { return self.local_unexpected_type(b, "float"); },
                        _ => { return self.local_unexpected_type(a, "number"); },
                    }
                    self.current.ip += 1;
                },

                Op::Lt(a, b) => { 
                    match (self.get_local(a)?, self.get_local(b)?) {
                        (RuntimeData::Float(a), RuntimeData::Float(b)) => { ret = Some( RuntimeData::Bool(a < b) ); },
                        (RuntimeData::Int(a), RuntimeData::Int(b)) => { ret = Some( RuntimeData::Bool(a < b) ); },
                        (RuntimeData::Int(_), _) => { return self.local_unexpected_type(b, "int"); },
                        (RuntimeData::Float(_), _) => { return self.local_unexpected_type(b, "float"); },
                        _ => { return self.local_unexpected_type(a, "number"); },
                    }
                    self.current.ip += 1;
                },

                Op::Not(x) => { 
                    match self.get_local(x)? {
                        RuntimeData::Bool(x) => { ret = Some( RuntimeData::Bool(!x) ); },        
                        _ => { return self.local_unexpected_type(x, "bool"); },
                    }
                    self.current.ip += 1;
                },

                Op::And(a, b) => { 
                    match (self.get_local(a)?, self.get_local(b)?) {
                        (RuntimeData::Bool(a), RuntimeData::Bool(b)) => { ret = Some( RuntimeData::Bool(*a && *b) ); },
                        (_, RuntimeData::Bool(_)) => { return self.local_unexpected_type(a, "bool");  },
                        _ => { return self.local_unexpected_type(b, "bool");  },
                    }
                    self.current.ip += 1;
                },

                Op::Or(a, b) => { 
                    match (self.get_local(a)?, self.get_local(b)?) {
                        (RuntimeData::Bool(a), RuntimeData::Bool(b)) => { ret = Some( RuntimeData::Bool(*a || *b) ); },
                        (_, RuntimeData::Bool(_)) => { return self.local_unexpected_type(a, "bool");  },
                        _ => { return self.local_unexpected_type(b, "bool");  },
                    }
                    self.current.ip += 1;
                },

                Op::Xor(a, b) => { 
                    match (self.get_local(a)?, self.get_local(b)?) {
                        (RuntimeData::Bool(a), RuntimeData::Bool(b)) => { ret = Some( RuntimeData::Bool(*a ^ *b) ); },
                        (_, RuntimeData::Bool(_)) => { return self.local_unexpected_type(a, "bool");  },
                        _ => { return self.local_unexpected_type(b, "bool");  },
                    }
                    self.current.ip += 1;
                },

                Op::Cons { sym_var, ref params } => {
                    let params = self.clone_locals(params)?;
                    let name = match self.get_local(sym_var)? {
                        RuntimeData::Symbol(x) => Rc::clone(x),
                        _ => { return self.local_unexpected_type(sym_var, "symbol"); },
                    };

                    match self.heap.iter_mut().enumerate().find(|(_, x)| matches!(x, Heap::Nil)) {
                        Some((addr, x)) => { 
                            *x = Heap::Cons { name, params }; 
                            ret = Some( RuntimeData::Ref( addr ) );
                        },
                        None => {
                            self.heap.push(Heap::Cons { name, params });
                            ret = Some( RuntimeData::Ref( self.heap.len() - 1 ) );
                        },
                    }
                    self.current.ip += 1;
                },

                Op::Delete(local) => {  
                    let addr = proj_type!(self, local, ref)?;
                    self.heap[addr] = Heap::Nil;
                    self.current.ip += 1;
                },

                Op::InsertSlot { dest, src, index } => {
                    let addr = proj_type!(self, dest, ref)?;
                    let input = self.get_local(src)?.clone();
                    match &mut self.heap[addr] { 
                        Heap::Nil => { return Err(VmError::AccessNilHeap(addr, self.stack_trace())); },
                        Heap::Cons { params, .. } if index > params.len() => { return Err(VmError::AccessMissingSlotIndex { index, addr, stack_trace: self.stack_trace() }); },
                        Heap::Cons { params, .. } => {
                            params.insert(index, input); 
                        },
                    }
                    self.current.ip += 1;
                },
                
                Op::RemoveSlot { local, index } => {
                    let addr = proj_type!(self, local, ref)?;
                    match &mut self.heap[addr] { 
                        Heap::Nil => { return Err(VmError::AccessNilHeap(addr, self.stack_trace())); },
                        Heap::Cons { params, .. } if index > params.len() => { return Err(VmError::AccessMissingSlotIndex { index, addr, stack_trace: self.stack_trace() }); },
                        Heap::Cons { params, .. } => {
                            params.remove(index); 
                        },
                    }
                    self.current.ip += 1;
                },

                Op::GetLength(local) => {
                    let addr = proj_type!(self, local, ref)?;
                    match &mut self.heap[addr] { 
                        Heap::Nil => { return Err(VmError::AccessNilHeap(addr, self.stack_trace())); },
                        Heap::Cons { params, .. } => {
                            ret = Some(RuntimeData::Int(params.len().try_into().unwrap()));
                        },
                    }
                    self.current.ip += 1;
                },

                Op::GetType(local) => {
                    let addr = proj_type!(self, local, ref)?;
                    match &mut self.heap[addr] { 
                        Heap::Nil => { return Err(VmError::AccessNilHeap(addr, self.stack_trace())); },
                        Heap::Cons { name, .. } => {
                            ret = Some(RuntimeData::Symbol(Rc::clone(name)));
                        },
                    }
                    self.current.ip += 1;
                },

                Op::GetSlot { local, index } => {
                    let addr = proj_type!(self, local, ref)?;
                    match &mut self.heap[addr] { 
                        Heap::Nil => { return Err(VmError::AccessNilHeap(addr, self.stack_trace())); },
                        Heap::Cons { params, .. } if index > params.len() => { return Err(VmError::AccessMissingSlotIndex { index, addr, stack_trace: self.stack_trace() }); },
                        Heap::Cons { params, .. } => {
                            ret = Some(params[index].clone());
                        },
                    }
                    self.current.ip += 1;
                },

                Op::Closure { proc_id, .. } if proc_id >= self.procs.len() => {
                    return Err(VmError::ProcDoesNotExist(self.current.proc_id, self.stack_trace()));
                },
                Op::Closure { proc_id, ref env } => {
                    let env = self.clone_locals(env)?;
                    ret = Some(RuntimeData::Closure (Closure { proc_id, env }));
                    self.current.ip += 1;
                },

                Op::Coroutine { proc_id, .. } if proc_id >= self.procs.len() => {
                    return Err(VmError::ProcDoesNotExist(self.current.proc_id, self.stack_trace()));
                },
                Op::Coroutine { proc_id, ref params } => {
                    let params = self.clone_locals(params)?;
                    ret = Some(RuntimeData::Coroutine(Coroutine::Start{ proc_id, params }));
                    self.current.ip += 1;
                },

                Op::DynCoroutine { local, ref params } => {
                    let closure = proj_type!(self, local, closure)?.clone();
                    let params = self.clone_locals(params)?;
                    ret = Some(RuntimeData::Coroutine(Coroutine::DynStart { closure, params }));
                    self.current.ip += 1;
                },

                Op::Resume(local) => {
                    match proj_type!(self, local, coroutine)? {
                        Coroutine::Active(frame) => {
                           /* self.current.ip += 1;
                            let current = std::mem::replace(&mut self.current, frame);
                            self.frames.push(current);
                            */
                        },
                        Coroutine::Start { proc_id, params } => {
                            self.current.ip += 1;
                            todo!()
                        },
                        Coroutine::DynStart { closure, params } => {
                            self.current.ip += 1;
                            todo!()
                        },
                        Coroutine::Ended => {
                            ret = Some(RuntimeData::Nil);
                            self.current.ip += 1;
                        },
                    }
                },
                Op::Nop => { self.current.ip += 1; },
                /*

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

    fn get_local<'a>(&'a self, local: usize) -> Result<&'a RuntimeData, VmError> {
        if local >= self.current.locals.len() {
            return Err(VmError::AccessMissingLocal(local, self.stack_trace()));
        }
        Ok(&self.current.locals[local])
    }

    fn mut_local<'a>(&'a mut self, local: usize) -> Result<&'a mut RuntimeData, VmError> {
        if local >= self.current.locals.len() {
            return Err(VmError::AccessMissingLocal(local, self.stack_trace()));
        }
        Ok(&mut self.current.locals[local])
    }

    fn clone_locals(&self, locals: &[usize]) -> Result<Vec<RuntimeData>, VmError> {
        let mut ret = vec![];
        for local in locals {
            let local = *local;
            if local >= self.current.locals.len() {
                return Err(VmError::AccessMissingLocal(local, self.stack_trace()));
            }
            else {
                ret.push(self.current.locals[local].clone());
            }
        }
        Ok(ret)
    }
}

