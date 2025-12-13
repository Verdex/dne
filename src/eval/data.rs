
use std::rc::Rc;

#[derive(Debug, Clone)]
pub enum RuntimeData {
    Bool(bool),
    Int(i64),
    Float(f64),
    Symbol(Rc<str>),
    Ref(usize),
    // TODO closure, coroutine, string
}

pub enum Op {
    // TODO set global
    Call(usize, Vec<usize>),
    DynCall(usize, Vec<usize>),
    Resume(usize),
    DynResume(usize),
    ReturnLocal(usize), 
    Jump(usize),
    BranchEqual { label: usize, local: usize },
    SetLocalData(usize, RuntimeData),
    SetLocalReturn(usize),
    SetLocalVar { src: usize, dest: usize },
    GetLength(usize),
    GetType(usize),
    GetSlot { local: usize, index: usize },
    Closure { proc_id: usize, env: Vec<usize> },
    Cons { sym_var: usize, captures: Vec<usize> },
    Coroutine { proc_id: usize, params: Vec<usize> },
    DynCoroutine { proc_id: usize, params: Vec<usize> },
    Yield(usize),
    Break,
    InsertSlot { dest: usize, src: usize, index: usize },
    RemoveSlot { local: usize, index: usize },
}

pub struct Fun { // TODO rename proc
    pub name : Rc<str>,
    pub instrs : Vec<Op>,
}

#[derive(Clone)]
pub struct Frame {
    pub fun_id : usize,
    pub ip : usize,
    pub locals : Vec<RuntimeData>,
}

#[derive(Clone)]
pub enum Coroutine {
    Active(Frame),
    Running,
    Finished,
}

impl Coroutine {
    pub fn is_alive(&self) -> bool {
        match self { 
            Coroutine::Active(_) | Coroutine::Running => true,
            _ => false,
        }
    }
}
