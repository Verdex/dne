
use std::str::CharIndices;
use std::iter::Peekable;

type Input<'a> = Peekable<CharIndices<'a>>;

pub enum IrToken {

}

pub fn lex_ir() -> Result<Vec<IrToken>, usize> {
    todo!()
}




/*
 * global name = lit;
 *
 *  proc name(a : T, ..) -> T {
 *      set name : T = expr;
 *      jump label_name;
 *      bz label_name var;
 *      yield var;
 *      resume var;
 *      label name;
 *      return var; 
 *  }
 *
 *
 * lit = int | float | string
 *
 *  expr = lit
 *       | var
 *       | call name (var, ...)
 *       | dyn_call var(var, ...)
 *
 * // TODO: syntax for calling a coroutine, dynamic coroutine, and closure
 *  
 *
 *
 */
