
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
    pub params: Vec<(Expr, Type)>, 
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

#[derive(Debug, Clone)]
pub struct Type {
    pub name : Rc<str>,
    pub params : Vec<Type>,
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
    Cons { ttype: Rc<str>, cons: Rc<str>, params: Vec<Expr> },
    Var(Rc<str>),
    // TODO lambda
    // list constructor
    // tuple constructor
    // match
}

pub fn parse(input : &str) -> Result<Vec<Fun>, ParseError> {
    let input = match dne::lex(input) {
        Err(i) => { return Err(ParseError::Lex(i)); },
        Ok(ls) => ls,
    };
    let mut input = Input::new(input, ParseError::Eof, |s, e| ParseError::Fatal(s, e));

    parse_funs(&mut input)
}

// TODO also parse struct and enum
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
    let name = expect_sym(input)?;
    let type_params = if input.check(|x| x.eq(&Token::LAngle))? {
        let mut ret = vec![expect_sym(input)?];
        loop {
            if input.check(|x| x.eq(&Token::RAngle))? {
                break ret;
            }
            input.expect(|x| x.eq(&Token::Comma))?; 
            ret.push(expect_sym(input)?);
        }
    }
    else {
        vec![]
    };
    input.expect(|x| x.eq(&Token::LParen))?;
    let mut params = vec![];
    if !input.check(|x| x.eq(&Token::RParen))? {
        loop {
            let param = parse_expr(input)?;
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
    let defs = parse_defs(input)?;
    let expr = parse_expr(input)?;
    input.expect(|x| x.eq(&Token::RCurl))?;
    Ok( Fun{ name, type_params, params, return_type, defs, expr })
}

fn parse_defs(input : &mut Input) -> Result<Vec<Def>, ParseError> {
    let mut ret = vec![];
    loop {
        if input.check(|x| x.eq(&Token::Let))? {
            ret.push(parse_let(input)?);
        }
        // TODO fun
        // TODO pat 
    /*
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
    */
        else {
            return Ok(ret);
        }
    }
}

fn parse_let(input : &mut Input) -> Result<Def, ParseError> {
    let name = expect_sym(input)?; 
    let ttype = if input.check(|x| x.eq(&Token::Colon))? {
        Some(parse_type(input)?)
    }
    else {
        None
    };
    input.expect(|x| x.eq(&Token::Equal))?;
    let expr = parse_expr(input)?;
    input.expect(|x| x.eq(&Token::SemiColon))?;
    Ok(Def::Let { name, ttype, expr })
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
    else if let Token::Bool(x) = input.peek()? {
        let x = *x;
        input.take()?;
        Ok(Expr::Lit(Lit::Bool(x)))
    }
    else if let Token::String(x) = input.peek()? {
        let x = Rc::clone(x);
        input.take()?;
        Ok(Expr::Lit(Lit::String(x)))
    }
    else if let Token::Symbol(x) = input.peek()? {
        // TODO: can also be cons or function call
        let x = Rc::clone(x);
        input.take()?;
        Ok(Expr::Var(x))
    }
    else {
        let (s, e) = input.current()?;
        Err(ParseError::Fatal(s, e))
    }
}

fn parse_type(input : &mut Input) -> Result<Type, ParseError> {
    let name = expect_sym(input)?; 
    let params = if input.check(|x| x.eq(&Token::LAngle))? {
        let mut ret = vec![parse_type(input)?];
        loop {
            if input.check(|x| x.eq(&Token::RAngle))? {
                break ret;
            }
            input.expect(|x| x.eq(&Token::Comma))?; 
            ret.push(parse_type(input)?);
        }
    }
    else {
        vec![]
    };
    Ok(Type { name, params })
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


#[cfg(test)]
mod test {
    use super::*;

}
