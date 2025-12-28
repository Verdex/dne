
use std::rc::Rc;

#[derive(Debug, Clone)]
pub enum RuntimeData {
    Bool(bool),
    Int(i64),
    Float(f64),
    Symbol(Rc<str>),
    Ref(usize),
    Nil,
    // TODO closure, coroutine, string
}

pub enum Op {
    // TODO set global
    // TODO set local global
    Call(usize, Vec<usize>),
    DynCall(usize, Vec<usize>),
    Resume(usize),
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
    Cons { sym_var: usize, params: Vec<usize> },
    Coroutine { proc_id: usize, params: Vec<usize> },
    // TODO proc_id for dyn coroutine has to be a local that is a coroutine
    DynCoroutine { proc_id: usize, params: Vec<usize> },
    Yield(usize),
    Break,
    InsertSlot { dest: usize, src: usize, index: usize },
    RemoveSlot { local: usize, index: usize },
    Nop,
    Add(usize, usize),
    Sub(usize, usize),
    Mul(usize, usize),
    Div(usize, usize),
    Mod(usize, usize),
    Neg(usize),
    Eq(usize, usize),
    NEq(usize, usize),
    Gt(usize, usize),
    Lt(usize, usize),
    Not(usize),
    And(usize, usize),
    Or(usize, usize),
    Xor(usize, usize),
}

pub struct Proc { 
    pub name : Rc<str>,
    pub instrs : Vec<Op>,
    pub stack_size : usize,
}

#[derive(Clone)]
pub struct Frame {
    pub proc_id : usize,
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
