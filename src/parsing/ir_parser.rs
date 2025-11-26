
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
    Set { var: Rc<str>, ttype : Type, val: Expr },
    Jump(Rc<str>),
    BranchEqual { label: Rc<str>, var: Rc<str> },
    Return(Rc<str>),
    Yield(Rc<str>),
    Break,
    Label(Rc<str>),
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

pub enum Lit {
    Int(i64),
    Float(f64),
    // TODO bool
    // TODO string
}

pub enum Expr { 
    Lit(Lit), 
    Call { name : Rc<str>, params : Vec<Rc<str>> },
    DynCall { name : Rc<str>, params : Vec<Rc<str>> },
    Coroutine { name : Rc<str>, params : Vec<Rc<str>> },
    DynCoroutine { name : Rc<str>, params : Vec<Rc<str>> },
    Closure { name : Rc<str>, params : Vec<Rc<str>> },
    Cons { name : Rc<str>, params : Vec<Rc<str>> },
    Resume(Rc<str>),
    Length(Rc<str>),
    Type(Rc<str>),
    Var(Rc<str>),
    Slot { var: Rc<str>, index: usize },
    SlotInsert { var: Rc<str>, input: Rc<str>, index: usize },
    SlotRemove { var: Rc<str>, index: usize },
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
        if input.check(|x| x.eq(&Token::Set))? {
            ret.push(parse_set(input)?);
        }
        else if input.check(|x| x.eq(&Token::Jump))? {
            let r = expect_sym(input)?;
            input.expect(|x| x.eq(&Token::SemiColon))?;
            ret.push(Stmt::Jump(r));
        }
        else if input.check(|x| x.eq(&Token::BranchEqual))? {
            let label = expect_sym(input)?;
            let var = expect_sym(input)?;
            input.expect(|x| x.eq(&Token::SemiColon))?;
            ret.push(Stmt::BranchEqual { label, var });
        }
        else if input.check(|x| x.eq(&Token::Return))? {
            let r = expect_sym(input)?;
            input.expect(|x| x.eq(&Token::SemiColon))?;
            ret.push(Stmt::Return(r));
        }
        else if input.check(|x| x.eq(&Token::Yield))? {
            let r = expect_sym(input)?;
            input.expect(|x| x.eq(&Token::SemiColon))?;
            ret.push(Stmt::Yield(r));
        }
        else if input.check(|x| x.eq(&Token::Break))? {
            input.expect(|x| x.eq(&Token::SemiColon))?;
            ret.push(Stmt::Break);
        }
        else if input.check(|x| x.eq(&Token::Label))? {
            let r = expect_sym(input)?;
            input.expect(|x| x.eq(&Token::SemiColon))?;
            ret.push(Stmt::Label(r));
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
    if let Token::Int(x) = input.peek()? {
        let x = *x;
        input.take()?;
        Ok(Expr::Lit(Lit::Int(x)))
    }
    else if let Token::Float(x) = input.peek()? {
        let x = *x;
        input.take()?;
        Ok(Expr::Lit(Lit::Float(x)))
    }
    // TODO bool
    else if let Token::Symbol(x) = input.peek()? {
        let x = Rc::clone(x);
        input.take()?;
        Ok(Expr::Var(x))
    }
    else if input.check(|x| x.eq(&Token::Call))? {
        let name = expect_sym(input)?;
        let params = expect_params(input)?;
        Ok(Expr::Call { name, params })
    }
    else if input.check(|x| x.eq(&Token::DynCall))? {
        let name = expect_sym(input)?;
        let params = expect_params(input)?;
        Ok(Expr::DynCall { name, params })
    }
    else if input.check(|x| x.eq(&Token::Coroutine))? {
        let name = expect_sym(input)?;
        let params = expect_params(input)?;
        Ok(Expr::Coroutine { name, params })
    }
    else if input.check(|x| x.eq(&Token::DynCoroutine))? {
        let name = expect_sym(input)?;
        let params = expect_params(input)?;
        Ok(Expr::DynCoroutine { name, params })
    }
    else if input.check(|x| x.eq(&Token::Closure))? {
        let name = expect_sym(input)?;
        let params = expect_params(input)?;
        Ok(Expr::Closure { name, params })
    }
    else if input.check(|x| x.eq(&Token::Cons))? {
        let name = expect_sym(input)?;
        let params = expect_params(input)?;
        Ok(Expr::Cons { name, params })
    }
    else if input.check(|x| x.eq(&Token::Resume))? {
        Ok(Expr::Resume(expect_sym(input)?)) 
    }
    else if input.check(|x| x.eq(&Token::Length))? {
        Ok(Expr::Length(expect_sym(input)?)) 
    }
    else if input.check(|x| x.eq(&Token::Type))? {
        Ok(Expr::Type(expect_sym(input)?)) 
    }
    else if input.check(|x| x.eq(&Token::Slot))? {
        let var = expect_sym(input)?;
        let index = expect_index(input)?;
        Ok(Expr::Slot { var, index })
    }
    else if input.check(|x| x.eq(&Token::SlotInsert))? {
        let var = expect_sym(input)?;
        let var_input = expect_sym(input)?;
        let index = expect_index(input)?;
        Ok(Expr::SlotInsert { var, input: var_input, index })
    }
    else if input.check(|x| x.eq(&Token::SlotRemove))? {
        let var = expect_sym(input)?;
        let index = expect_index(input)?;
        Ok(Expr::SlotRemove { var, index })
    }
    else {
        Err(ParseError::Fatal)
    }
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

fn expect_sym(input : &mut Input) -> Result<Rc<str>, ParseError> {
    Ok(input.expect(|x| matches!(x, Token::Symbol(_)))?.value())
}

fn expect_params(input : &mut Input) -> Result<Vec<Rc<str>>, ParseError> {
    input.expect(|x| x.eq(&Token::LParen))?;
    let mut ret = vec![];
    loop {
        ret.push(expect_sym(input)?);
        if input.check(|x| x.eq(&Token::Comma))? {
            break;
        }
    }
    input.expect(|x| x.eq(&Token::RParen))?;
    Ok(ret)
}

fn expect_index(input : &mut Input) -> Result<usize, ParseError> {
    if let Token::Int(x) = input.peek()? {
        let x = *x;
        input.take()?;
        match usize::try_from(x) {
            Ok(x) => Ok(x),
            Err(_) => Err(ParseError::Fatal),
        }
    }
    else {
        Err(ParseError::Fatal)
    }
}
