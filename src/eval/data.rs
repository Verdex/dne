
use std::rc::Rc;

#[derive(Debug)]
pub enum RuntimeData {
    Int(i64),
    Float(f64),
    Symbol(Rc<str>),
    Ref(usize),
}

pub enum Op {
    Gen(usize, Vec<usize>),
    Call(usize, Vec<usize>),
    ReturnLocal(usize), 
    Return,
    Branch(usize),
    DynCall(Vec<usize>),
    Drop(usize),
    Dup(usize),
    Swap(usize, usize),
    PushRet,
    PushLocal(RuntimeData),
    CoYield(usize),
    CoFinish,
    CoResume(usize),
    CoDrop(usize),
    CoDup(usize), 
    CoSwap(usize, usize),
}

pub struct Fun {
    pub name : Rc<str>,
    pub instrs : Vec<Op>,
}

#[derive(Clone)]
pub struct Frame {
    pub fun_id : usize,
    pub ip : usize,
    pub ret : Option<RuntimeData>,
    pub locals : Vec<RuntimeData>,
    pub coroutines : Vec<Coroutine>,
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
