
use std::rc::Rc;
use std::str::CharIndices;
use std::iter::Peekable;

type Input<'a> = Peekable<CharIndices<'a>>;

pub mod Ir {

    use super::*;

    pub enum Type {
        Int,
        Float,
        String,
        Bool,
        Symbol,
        Ref,
        Closure,
        Coroutine,
    }

    pub enum Token {
        LParen,
        RParen,
        LCurl,
        RCurl,
        Comma,
        SemiColon,
        Colon,
        Arrow,
        Int(i64),
        Float(f64),
        Type(Type),
        Symbol(Rc<str>),
        // Type?
        Slot,
        Proc,
        Equal,
        Return,
        Yield,
        Resume,
        Break,
        CoStart,
        DynCoStart,
        Set,
        Jump,
        BranchEqual,
        Global,
        Call,
        DynCall,
        Closure,
        Cons,
        Op(Rc<str>),
    }

    pub fn lex_ir(input : &str) -> Result<Vec<IrToken>, usize> {
        let mut input = input.char_indices().peekable();
        todo!()
    }

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
