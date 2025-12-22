
use std::rc::Rc;

use super::data::*;
use super::error::*;

pub struct Vm {
    procs: Vec<Proc>,
    globals: Vec<RuntimeData>,
    frames : Vec<Frame>,
    current : Frame,
}

macro_rules! proj {
    ($input:expr, $p:pat, $output:expr) => {
        match $input {
            $p => $output,
            _ => panic!("proj failed"),
        }
    }
}

impl Vm {
    pub fn new(procs: Vec<Proc>) -> Self {
        let current = Frame { proc_id: 0, ip: 0, locals: vec![] };
        Vm { procs, globals: vec![], frames: vec![], current }
    }

    pub fn run(&mut self, entry : usize) -> Result<Option<RuntimeData>, VmError> {
        if entry >= self.procs.len() {
            return Err(VmError::ProcDoesNotExist(entry, self.stack_trace()));
        }

        self.current.proc_id = entry;

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
                Op::BranchEqual { local, .. } if local >= self.current.locals.len() => {
                    return Err(VmError::AccessMissingLocal(local, self.stack_trace()));
                },
                Op::BranchEqual { local, .. } if !matches!( self.current.locals[local], RuntimeData::Bool(_) ) => {
                    return self.local_unexpected_type(local, "bool");
                },
                Op::BranchEqual { label, local } => {
                    let test = proj!(self.current.locals[local], RuntimeData::Bool(x), x);
                    if test {
                        self.current.ip = label;
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
                /*
                Op::SetLocalData(local, data) => 
                Op::SetLocalData(local, data) => {
                    
                },
                Op::SetLocalReturn(local) => todo!(),
                Op::SetLocalVar { src, dest } => todo!(),
                */
                /*
                Op::Resume(local) => todo!(),
                Op::GetLength(local) => todo!(),
                Op::GetType(local) => todo!(),
                Op::GetSlot { local, index } => todo!(),
                Op::Closure { proc_id, env } => todo!(),
                Op::Cons { sym_var, captures } => todo!(),
                Op::Coroutine { proc_id, params } => todo!(),
                // TODO fix 
                Op::DynCoroutine { proc_id, params } => todo!(),
                Op::Yield(local) => todo!(),
                Op::Break => todo!(),
                Op::InsertSlot { dest, src, index } => todo!(),
                Op::RemoveSlot { local, index } => todo!(),
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
                Op::Drop(local) if local < self.current.locals.len() => {
                    self.current.locals.remove(local);
                    self.current.ip += 1;
                },
                Op::Drop(local) => {
                    return Err(VmError::AccessMissingLocal(local, self.stack_trace()));
                },
                Op::Dup(local) if local < self.current.locals.len() => {
                    let target = self.current.locals[local].clone();
                    self.current.locals.push(target);
                    self.current.ip += 1;
                },
                Op::Dup(local) => {
                    return Err(VmError::AccessMissingLocal(local, self.stack_trace()));
                },
                Op::Swap(a, b) if a < self.current.locals.len() && b < self.current.locals.len() => {
                    self.current.locals.swap(a, b);
                    self.current.ip += 1;
                },
                Op::Swap(a, b) if b < self.current.locals.len() => {
                    return Err(VmError::AccessMissingLocal(a, self.stack_trace()));
                },
                Op::Swap(_, b) => {
                    return Err(VmError::AccessMissingLocal(b, self.stack_trace()));
                },
                Op::PushRet if self.current.ret.is_some() => {
                    let ret = std::mem::replace(&mut self.current.ret, None);
                    self.current.locals.push(ret.unwrap());
                    self.current.ret = None;
                    self.current.ip += 1;
                },
                Op::PushRet => {
                    return Err(VmError::AccessMissingReturn(self.stack_trace()));
                },
                Op::PushLocal(ref t) => {
                    self.current.locals.push(t.clone());
                    self.current.ip += 1;
                }*/
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
