
use std::rc::Rc;
use super::lexer::ir::{self, Token};

type Input = super::parse_input::Input<Token, ParseError>;

#[derive(Debug, Clone)]
pub enum ParseError {
    Lex(usize),
    Fatal,
    Eof,
}

impl std::fmt::Display for ParseError {
    fn fmt(&self, f : &mut std::fmt::Formatter) -> std::fmt::Result {
        // TODO 
        match self { 
            _ => write!(f, ""),
        }
    }
}

impl std::error::Error for ParseError { }

pub enum Top {
    Global,
    Proc
}

pub enum Stmt {
    Set { var: Rc<str>, ttype : Type, val: Expr }
}

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

pub enum Expr { 

}

pub fn parse(input : &str) -> Result<Vec<Top>, ParseError> {
    let input = match ir::lex(input) {
        Err(i) => { return Err(ParseError::Lex(i)); },
        Ok(ls) => ls,
    };
    let mut input = Input::new(input, ParseError::Eof, ParseError::Fatal);

    todo!()
}

fn parse_stmts(input : &mut Input) -> Result<Vec<Stmt>, ParseError> {
    let mut ret = vec![];
    loop {
        if input.check(|l| l.eq(&Token::Set))? {
            ret.push(parse_set(input)?);
        }
        else {
            return Ok(ret);
        }
    }
}

fn parse_set(input : &mut Input) -> Result<Stmt, ParseError> {
    let var = input.expect(|x| matches!(x, Token::Symbol(_)))?;
    input.expect(|x| x.eq(&Token::Colon))?;
    let ttype = parse_type(input)?;
    input.expect(|x| x.eq(&Token::Equal))?;
    let val = parse_expr(input)?;
    input.expect(|x| x.eq(&Token::SemiColon))?;
    Ok(Stmt::Set { var: var.value(), val, ttype })
}

fn parse_expr(input : &mut Input) -> Result<Expr, ParseError> {
    todo!()
}

fn parse_type(input : &mut Input) -> Result<Type, ParseError> {
    let t = input.expect(|x| matches!(x, Token::Symbol(_)))?.value();
    match &*t {
        "Int" => Ok(Type::Int),
        "Float" => Ok(Type::Float),
        "String" => Ok(Type::String),
        "Bool" => Ok(Type::Bool),
        "Symbol" => Ok(Type::Symbol),
        "Ref" => Ok(Type::Ref),
        "Closure" => Ok(Type::Closure),
        "Coroutine" => Ok(Type::Coroutine),
        _ => Err(ParseError::Fatal),
    }
}
