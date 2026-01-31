
use std::rc::Rc;

pub type StackTrace = Vec<(Rc<str>, usize)>;

#[derive(Debug)]
pub enum VmError {
    AccessNilHeap(usize, StackTrace),
    AccessMissingSlotIndex { addr: usize, index: usize, stack_trace: StackTrace },
    ProcDoesNotExist(usize, StackTrace),
    InstrPointerOutOfRange(usize, StackTrace),
    AccessMissingReturn(StackTrace),
    AccessMissingLocal(usize, StackTrace),
    LocalUnexpectedType{local: usize, stack_trace: StackTrace, expected: &'static str, found: Box<str>},
    TopLevelYield(usize),
}

impl std::fmt::Display for VmError {
    fn fmt(&self, f : &mut std::fmt::Formatter) -> std::fmt::Result {
        fn d(x : &StackTrace) -> String {
            x.into_iter().map(|(n, i)| format!("    {} at index {}\n", n, i)).collect()
        }

        match self { 
            VmError::AccessMissingSlotIndex { addr, index, stack_trace } => 
                write!(f, "Access missing slot index {} at address {}:  \n{}", index, addr, d(stack_trace)),
            VmError::AccessNilHeap(addr, stack_trace) => 
                write!(f, "Access nil heap at address {}:  \n{}", addr, d(stack_trace)),
            VmError::LocalUnexpectedType{local, stack_trace, expected, found } => 
                write!(f, "Local {} was unexpected type.  Expected: {}, but found {}: \n{}", local, expected, found, d(stack_trace) ),
            VmError::ProcDoesNotExist(proc_index, trace) => 
                write!(f, "Proc Index {} does not exist: \n{}", proc_index, d(trace)),
            VmError::InstrPointerOutOfRange(instr, trace) => 
                write!(f, "Instr Index {} does not exist: \n{}", instr, d(trace)),
            VmError::AccessMissingReturn(trace) => 
                write!(f, "Attempting to access missing return: \n{}", d(trace)),
            VmError::AccessMissingLocal(local, trace) => 
                write!(f, "Attempting to access missing local {}: \n{}", local, d(trace)),
            VmError::TopLevelYield(ip) =>
                write!(f, "Top Level Yield no supported at instruction: {}", ip),
        }
    }
}

impl std::error::Error for VmError { }

