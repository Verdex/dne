
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
    
    todo!()
}


#[cfg(test)]
mod test {
    use super::*;

}
