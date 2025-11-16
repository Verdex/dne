
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

    pub struct StringSegment {
        s : Rc<str>,
        var : Option<Rc<str>>,
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
        Equal,
        Int(i64),
        Float(f64),
        Type(Type),
        Symbol(Rc<str>),
        String(Vec<StringSegment>),
        GetType,
        Slot,
        Proc,
        Return,
        Yield,
        Resume,
        Break,
        Coroutine,
        DynCoroutine,
        Set,
        Jump,
        BranchEqual,
        Global,
        Call,
        DynCall,
        Closure,
        Cons,
        // TODO ? Op(Rc<str>),
    }

    pub fn lex(input : &str) -> Result<Vec<Token>, usize> {
        let mut input = input.char_indices().peekable();
        let mut ret = vec![];
 
        loop {
            match input.peek() {
                None => { return Ok(ret); },
                Some((_, c)) if c.is_whitespace() => {
                    whitespace(&mut input)?;
                },
                Some((_, c)) if c.is_alphabetic() || *c == '_' => {
                    ret.push(symbol(&mut input)?);
                },
                Some((_, c)) if c.is_numeric() || *c == '-' => { // TODO or arrow
                    //ret.push(number(&mut input)?);
                },
                /*Some((_, c)) if punct_char(*c) => {
                    //ret.append(&mut punct(&mut input)?);
                },*/
                Some((i, _)) => { return Err(*i); },
            }
        }
        todo!()
    }

    fn num_or_arrow(input : &mut Input) -> Result<Token, usize> {
        // TODO need backtrack
        // check arrow
    }

    fn symbol(input : &mut Input) -> Result<Token, usize> {
        let s = take_until(input, |c| c.is_alphanumeric() || c == '_');
        let s = s.into_iter().collect::<String>();

        match s.as_str() {
            "type" => Ok(Token::GetType),
            "slot" => Ok(Token::Slot),
            "proc" => Ok(Token::Proc),
            "return" =>  Ok(Token::Return),
            "yield" =>  Ok(Token::Yield),
            "resume" =>  Ok(Token::Resume),
            "break" =>  Ok(Token::Break),
            "coroutine" =>  Ok(Token::Coroutine),
            "dyn_coroutine" =>  Ok(Token::DynCoroutine),
            "set" =>  Ok(Token::Set),
            "jump" =>  Ok(Token::Jump),
            "branch_equal" =>  Ok(Token::BranchEqual),
            "global" =>  Ok(Token::Global),
            "call" =>  Ok(Token::Call),
            "dyn_call" =>  Ok(Token::DynCall),
            "closure" =>  Ok(Token::Closure),
            s => Ok(Token::Symbol(s.into())),
        }
    }

}

fn whitespace(input : &mut Input) -> Result<(), usize> {
    while let Some((_, c)) = input.peek() && c.is_whitespace() {
        input.next().unwrap();
    }
    Ok(())
}

enum Num { Int(i64), Float(f64) } 
fn number(input : &mut Input) -> Result<Num, usize> {
    let i : usize = input.peek().unwrap().0;
    let s = take_until(input, |c| c.is_numeric() || c == '.' || c == '-' || c == 'E' || c == 'e' || c == '+');
    let s = s.into_iter().collect::<String>();
    match s.parse::<i64>() {
        Ok(x) => Ok(Num::Int(x)),
        Err(_) => match s.parse::<f64>() {
            Ok(x) => Ok(Num::Float(x)),
            Err(_) => Err(i),
        },
    }
}

// Note:  Only call this function when you know the first char is what you want
fn take_until<F : FnMut(char) -> bool>(input : &mut Input, mut p : F) -> Vec<char> {
    let mut ret = vec![input.next().unwrap().1];

    loop {
        match input.peek() {
            Some((_, c)) if p(*c) => {
                ret.push(*c);
                input.next().unwrap();
            },
            Some(_) => { return ret; },
            None => { return ret; },
        }
    }
}

