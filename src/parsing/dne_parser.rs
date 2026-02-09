
use std::rc::Rc;
use super::lexer::dne::{self, Token};

type Input = super::parse_input::Input<Token, ParseError>;

#[derive(Debug, Clone)]
pub enum ParseError {
    Lex(usize),
    Fatal(usize, usize),
    Eof,
}

impl std::fmt::Display for ParseError {
    fn fmt(&self, f : &mut std::fmt::Formatter) -> std::fmt::Result {
        match self { 
            ParseError::Lex(x) => write!(f, "encountered unexpected lexing error at: {x}"),
            ParseError::Fatal(x, y) => write!(f, "encountered unexpected fatal parsing error at: {x}-{y}"),
            ParseError::Eof => write!(f, "encountered unexpected EOF"),
        }
    }
}

impl std::error::Error for ParseError { }

#[derive(Debug)]
pub struct Fun {
    pub name: Rc<str>, 
    pub type_params: Vec<Rc<str>>,
    pub params: Vec<(Rc<str>, Type)>, 
    pub return_type: Type, 
    pub defs: Vec<Def>,
    pub expr: Expr,
}

#[derive(Debug)]
pub enum Def {
    Let { 
        name: Rc<str>,
        ttype: Option<Type>,
        expr: Expr,
    },
    Fun(Fun),
}

#[derive(Debug, PartialEq, Clone, Copy)]
pub enum Type {
}

#[derive(Debug)]
pub enum Lit {
    Int(i64),
    Float(f64),
    Bool(bool),
    String(Rc<str>),
}

#[derive(Debug)]
pub enum Expr { 
    Lit(Lit), 
    Call { name : Rc<str>, params : Vec<Expr> },
}

pub fn parse(input : &str) -> Result<Vec<Fun>, ParseError> {
    let input = match dne::lex(input) {
        Err(i) => { return Err(ParseError::Lex(i)); },
        Ok(ls) => ls,
    };
    let mut input = Input::new(input, ParseError::Eof, |s, e| ParseError::Fatal(s, e));

    parse_funs(&mut input)
}

fn parse_funs(input : &mut Input) -> Result<Vec<Fun>, ParseError> {
    let mut ret = vec![];
    while !input.empty() {
        if input.check(|x| x.eq(&Token::Fun))? {
            ret.push(parse_fun(input)?);
        }
        else {
            let (s, e) = input.current()?;
            return Err(ParseError::Fatal(s, e));
        }
    }
    Ok(ret)
}

fn parse_fun(input : &mut Input) -> Result<Fun, ParseError> {
    todo!()
    /*let name = expect_sym(input)?;
    input.expect(|x| x.eq(&Token::LParen))?;
    let mut params = vec![];
    if !input.check(|x| x.eq(&Token::RParen))? {
        loop {
            let param = expect_sym(input)?;
            input.expect(|x| x.eq(&Token::Colon))?;
            let ttype = parse_type(input)?;
            params.push((param, ttype));

            if input.check(|x| x.eq(&Token::RParen))? {
                break;
            }
            else if input.check(|x| x.eq(&Token::Comma))? {
                continue;
            }
            else {
                let (s, e) = input.current()?;
                return Err(ParseError::Fatal(s, e));
            }
        }
    }
    input.expect(|x| x.eq(&Token::Arrow))?;
    let return_type = parse_type(input)?;
    input.expect(|x| x.eq(&Token::LCurl))?;
    let body = parse_stmts(input)?;
    input.expect(|x| x.eq(&Token::RCurl))?;
    Ok( Proc{ name, params, return_type, body })
    */
}

fn parse_defs(input : &mut Input) -> Result<Vec<Def>, ParseError> {
    todo!()
    /*
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
        else if input.check(|x| x.eq(&Token::BranchTrue))? {
            let label = expect_sym(input)?;
            let var = expect_sym(input)?;
            input.expect(|x| x.eq(&Token::SemiColon))?;
            ret.push(Stmt::BranchTrue { label, var });
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
        else if input.check(|x| x.eq(&Token::SlotInsert))? {
            let var = expect_sym(input)?;
            let var_input = expect_sym(input)?;
            let index = expect_index(input)?;
            input.expect(|x| x.eq(&Token::SemiColon))?;
            ret.push(Stmt::SlotInsert { var, input: var_input, index })
        }
        else if input.check(|x| x.eq(&Token::SlotRemove))? {
            let var = expect_sym(input)?;
            let index = expect_index(input)?;
            input.expect(|x| x.eq(&Token::SemiColon))?;
            ret.push(Stmt::SlotRemove { var, index })
        }
        else if input.check(|x| x.eq(&Token::Delete))? {
            let var = expect_sym(input)?;
            input.expect(|x| x.eq(&Token::SemiColon))?;
            ret.push(Stmt::Delete(var));
        }
        else {
            return Ok(ret);
        }
    }
    */
}

/*
fn parse_set(input : &mut Input) -> Result<Stmt, ParseError> {
    let var = expect_sym(input)?; 
    input.expect(|x| x.eq(&Token::Colon))?;
    let ttype = parse_type(input)?;
    input.expect(|x| x.eq(&Token::Equal))?;
    let val = parse_expr(input)?;
    input.expect(|x| x.eq(&Token::SemiColon))?;
    Ok(Stmt::Set { var, val, ttype })
}
*/

fn parse_expr(input : &mut Input) -> Result<Expr, ParseError> {
    todo!()
    /*
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
    else if let Token::Bool(x) = input.peek()? {
        let x = *x;
        input.take()?;
        Ok(Expr::Lit(Lit::Bool(x)))
    }
    else if let Token::ConsType(x) = input.peek()? {
        let x = Rc::clone(x);
        input.take()?;
        Ok(Expr::Lit(Lit::ConsType(x)))
    }
    else if let Token::Symbol(x) = input.peek()? {
        let x = Rc::clone(x);
        input.take()?;
        Ok(Expr::Var(x))
    }
    else if let Token::String(x) = input.peek()? {
        let x = Rc::clone(x);
        input.take()?;
        Ok(Expr::Lit(Lit::String(x)))
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
        let env = expect_params(input)?;
        Ok(Expr::Closure { name, env })
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
    else if input.check(|x| x.eq(&Token::IsNil))? {
        let var = expect_sym(input)?;
        Ok(Expr::IsNil(var))
    }
    else if input.check(|x| x.eq(&Token::ToString))? {
        let var = expect_sym(input)?;
        Ok(Expr::ToString(var))
    }
    else if input.check(|x| x.eq(&Token::Concat))? {
        let var1 = expect_sym(input)?;
        let var2 = expect_sym(input)?;
        Ok(Expr::Concat(var1, var2))
    }
    else {
        let (s, e) = input.current()?;
        Err(ParseError::Fatal(s, e))
    }
    */
}

fn parse_type(input : &mut Input) -> Result<Type, ParseError> {
    todo!()
    /*
    let t = expect_sym(input)?; 
    match &*t {
        "Int" => Ok(Type::Int),
        "Float" => Ok(Type::Float),
        "String" => Ok(Type::String),
        "Bool" => Ok(Type::Bool),
        "Symbol" => Ok(Type::Symbol),
        "Ref" => Ok(Type::Ref),
        "Closure" => Ok(Type::Closure),
        "Coroutine" => Ok(Type::Coroutine),
        _ => {
            let (s, e) = input.current()?;
            Err(ParseError::Fatal(s, e))
        },
    }
    */
}

fn expect_sym(input : &mut Input) -> Result<Rc<str>, ParseError> {
    if let Token::Symbol(x) = input.peek()? {
        let x = Rc::clone(x);
        input.take()?;
        Ok(x)     
    }
    else {
        let (s, e) = input.current()?;
        Err(ParseError::Fatal(s, e))
    }
}

fn expect_params(input : &mut Input) -> Result<Vec<Rc<str>>, ParseError> {
    input.expect(|x| x.eq(&Token::LParen))?;
    let mut ret = vec![];
    if input.check(|x| x.eq(&Token::RParen))? {
        return Ok(ret);
    }
    loop {
        ret.push(expect_sym(input)?);
        
        if input.check(|x| x.eq(&Token::RParen))? {
            break;
        }
        else if input.check(|x| x.eq(&Token::Comma))? {
            continue;
        }
        else {
            let (s, e) = input.current()?;
            return Err(ParseError::Fatal(s, e));
        }
    }
    Ok(ret)
}


#[cfg(test)]
mod test {
    use super::*;

}
