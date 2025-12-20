
use std::rc::Rc;

pub type StackTrace = Vec<(Rc<str>, usize)>;

#[derive(Debug)]
pub enum VmError {
    FunDoesNotExist(usize, StackTrace),
    InstrPointerOutOfRange(usize, StackTrace),
    AccessMissingReturn(StackTrace),
    AccessMissingLocal(usize, StackTrace),
    LocalUnexpectedType{local: usize, stack_trace: StackTrace, expected: &'static str, found: Box<str>},
    TopLevelYield(usize),
    AccessMissingCoroutine(usize, StackTrace),
    ResumeFinishedCoroutine(usize, StackTrace),
}

impl std::fmt::Display for VmError {
    fn fmt(&self, f : &mut std::fmt::Formatter) -> std::fmt::Result {
        fn d(x : &StackTrace) -> String {
            x.into_iter().map(|(n, i)| format!("    {} at index {}\n", n, i)).collect()
        }

        match self { 
            VmError::LocalUnexpectedType{local, stack_trace, expected, found } => 
                write!(f, "Local {} was unexpected type.  Expected: {}, but found {}: \n{}", local, expected, found, d(stack_trace) ),
            VmError::FunDoesNotExist(fun_index, trace) => 
                write!(f, "Fun Index {} does not exist: \n{}", fun_index, d(trace)),
            VmError::InstrPointerOutOfRange(instr, trace) => 
                write!(f, "Instr Index {} does not exist: \n{}", instr, d(trace)),
            VmError::AccessMissingReturn(trace) => 
                write!(f, "Attempting to access missing return: \n{}", d(trace)),
            VmError::AccessMissingLocal(local, trace) => 
                write!(f, "Attempting to access missing local {}: \n{}", local, d(trace)),
            VmError::TopLevelYield(ip) =>
                write!(f, "Top Level Yield no supported at instruction: {}", ip),
            VmError::AccessMissingCoroutine(coroutine, trace) =>
                write!(f, "Attempting to access missing coroutine {}: \n{}", coroutine, d(trace)),
            VmError::ResumeFinishedCoroutine(coroutine, trace) =>
                write!(f, "Attempting to resume finished coroutine {}: \n{}", coroutine, d(trace)),
        }
    }
}

impl std::error::Error for VmError { }

