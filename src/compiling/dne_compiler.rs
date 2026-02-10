
use std::rc::Rc;
// TODO
use std::collections::{ HashSet, HashMap };

use crate::parsing::dne_parser::*;
//use crate::parsing::ir_parser::{Lit, Expr, Type, Stmt, Proc};


#[derive(Debug)]
pub enum CompileError {
}

impl std::fmt::Display for CompileError {
    fn fmt(&self, f : &mut std::fmt::Formatter) -> std::fmt::Result {
        match self { 
            _ => todo!(),
                //write!(f, "Access missing local {local} in proc {proc}"),
        }
    }
}

impl std::error::Error for CompileError { }


pub fn compile(input : &[Fun]) -> Result<Rc<str>, CompileError> {
    
//    let proc_map = HashMap::from_iter(op_sigs.iter().chain(procs.iter()).enumerate().map(|(v, k)| (Rc::clone(&k.name), (k, v))));
    todo!()
}


#[cfg(test)]
mod test {
    use super::*;

}
