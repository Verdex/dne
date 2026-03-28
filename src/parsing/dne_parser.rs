
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
pub enum Top {
    Fun(Fun),
    Enum(Enum),
    Struct(Struct),
}

#[derive(Debug)]
pub struct Enum {
    pub name: Rc<str>,
    pub type_params: Vec<Rc<str>>,
    pub cases : Vec<EnumCase>,
}

#[derive(Debug)]
pub struct EnumCase {
    pub name: Rc<str>,
    pub params: Vec<Type>,
}

#[derive(Debug)]
pub struct Struct {
    pub name: Rc<str>,
    pub type_params: Vec<Rc<str>>,
    pub fields: Vec<(Rc<str>, Type)>,
}

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

// TODO prioritize cons, enum, struct, and match
// Then get pattern, foreach_result, and pat_locals

#[derive(Debug)]
pub enum Expr { 
    Lit(Lit), 
    Call { name : Rc<str>, params : Vec<Expr> },
    CaseCons { ttype : Rc<str>, case : Rc<str>, params : Vec<Expr> },
    StructCons { ttype : Rc<str>, params : Vec<(Rc<str>, Expr)> },
    Var(Rc<str>),
    // TODO
    // list constructor
    // match
}

pub fn parse(input : &str) -> Result<Vec<Top>, ParseError> {
    let input = match dne::lex(input) {
        Err(i) => { return Err(ParseError::Lex(i)); },
        Ok(ls) => ls,
    };
    let mut input = Input::new(input, ParseError::Eof, |s, e| ParseError::Fatal(s, e));

    parse_tops(&mut input)
}

fn parse_tops(input : &mut Input) -> Result<Vec<Top>, ParseError> {
    let mut ret = vec![];
    while !input.empty() {
        if input.check(|x| x.eq(&Token::Fun))? {
            ret.push(Top::Fun(parse_fun(input)?));
        }
        else if input.check(|x| x.eq(&Token::Enum))? {
            ret.push(Top::Enum(parse_enum(input)?));
        }
        else if input.check(|x| x.eq(&Token::Struct))? {
            ret.push(Top::Struct(parse_struct(input)?));
        }
        else {
            let (s, e) = input.current()?;
            return Err(ParseError::Fatal(s, e));
        }
    }
    Ok(ret)
}

fn parse_struct(input : &mut Input) -> Result<Struct, ParseError> {
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
    input.expect(|x| x.eq(&Token::LCurl))?;
    let mut fields = vec![];
    if !input.check(|x| x.eq(&Token::RCurl))? {
        loop {
            let field = expect_sym(input)?;
            input.expect(|x| x.eq(&Token::Colon))?;
            let ttype = parse_type(input)?;
            fields.push((field, ttype));

            if input.check(|x| x.eq(&Token::RCurl))? {
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
    Ok(Struct { name, type_params, fields })
}

fn parse_enum(input : &mut Input) -> Result<Enum, ParseError> {
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
    input.expect(|x| x.eq(&Token::LCurl))?;
    let mut cases = vec![];
    if !input.check(|x| x.eq(&Token::RCurl))? {
        loop {
            cases.push(parse_enum_case(input)?);

            if input.check(|x| x.eq(&Token::RCurl))? {
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
    Ok(Enum { name, type_params, cases })
}

fn parse_enum_case(input : &mut Input) -> Result<EnumCase, ParseError> {
    let name = expect_sym(input)?;
    if input.check(|x| x.eq(&Token::LParen))? {
        let mut params = vec![parse_type(input)?];
        while !input.check(|x| x.eq(&Token::RParen))? {
            input.expect(|x| x.eq(&Token::Comma))?;
            params.push(parse_type(input)?);
        }
        Ok(EnumCase { name, params })
    }
    else {
        Ok(EnumCase { name, params: vec![] })
    }
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
        let x = Rc::clone(x);
        input.take()?;
        parse_var_follow_on(input, x)
    }
    else {
        let (s, e) = input.current()?;
        Err(ParseError::Fatal(s, e))
    }
}

fn parse_var_follow_on(input : &mut Input, name : Rc<str>) -> Result<Expr, ParseError> {
    if input.check(|x| x.eq(&Token::LParen))? {
        if input.check(|x| x.eq(&Token::RParen))? {
            Ok(Expr::Call { name, params: vec![] } )
        }
        else {
            let mut params = vec![parse_expr(input)?];
            loop {
                if input.check(|x| x.eq(&Token::RParen))? {
                    break Ok(Expr::Call { name, params } );
                }
                input.expect(|x| x.eq(&Token::Comma))?;
                params.push(parse_expr(input)?);
            }
        }
    }
    else if input.check(|x| x.eq(&Token::Colon))? {
        let mut params = vec![];
        input.expect(|x| x.eq(&Token::Colon))?;
        let case = expect_sym(input)?;
        if input.check(|x| x.eq(&Token::LParen))? {
            params.push(parse_expr(input)?);
            while !input.check(|x| x.eq(&Token::RParen))? {
                input.expect(|x| x.eq(&Token::Comma))?;
                params.push(parse_expr(input)?);
            }
        }
        Ok(Expr::CaseCons { ttype: name, case, params } )
    }
    else if input.check(|x| x.eq(&Token::LCurl))? {
        if input.check(|x| x.eq(&Token::RCurl))? {
            Ok(Expr::StructCons { ttype: name, params: vec![] })
        }
        else {
            let mut params = vec![];
            let field = expect_sym(input)?;
            input.expect(|x| x.eq(&Token::Colon))?;
            let expr = parse_expr(input)?;
            params.push((field, expr));
            while !input.check(|x| x.eq(&Token::RCurl))? {
                input.expect(|x| x.eq(&Token::Comma))?;
                let field = expect_sym(input)?;
                input.expect(|x| x.eq(&Token::Colon))?;
                let expr = parse_expr(input)?;
                params.push((field, expr));
            }
            Ok(Expr::StructCons { ttype: name, params })
        }
    }
    else {
        Ok(Expr::Var(name))
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

// empty_one_or_more
// zom

fn one_or_more<T>(
    input : &mut Input, 
    end : Token, 
    item : impl Fn(&mut Input) -> Result<T, ParseError>,
    trail : bool) -> Result<Vec<T>, ParseError> {

    let mut xs = vec![item(input)?];
    loop { 
        if input.check(|x| x.eq(&end))? {
            return Ok(xs);
        }
        input.expect(|x| x.eq(&Token::Comma))?;
        if trail && input.check(|x| x.eq(&end))? {
            return Ok(xs);
        }
        xs.push(item(input)?);
    }
}

#[cfg(test)]
mod test {
    use super::*;

    fn test_one_or_more(input : &str, trail : bool, success : bool) {
        let input = dne::lex(input).unwrap();
        let mut input = Input::new(input, ParseError::Eof, |s, e| ParseError::Fatal(s, e));
        let out = one_or_more(&mut input, Token::RCurl, |x| x.expect(|x| x.eq(&Token::Let)), trail);
        if success {
            assert!(matches!(out, Ok(_)));
        }
        else {
            assert!(matches!(out, Err(_)));
        }
    }

    #[test]
    fn should_handle_one_or_more() {
        test_one_or_more("}", false, false);
        test_one_or_more("let }", false, true);
        test_one_or_more("let, let }", false, true);
        test_one_or_more("let, let, let }", false, true);
        test_one_or_more("let, let, let, }", false, false);
        test_one_or_more("let, let,, let }", false, false);
        test_one_or_more("}", true, false);
        test_one_or_more("let }", true, true);
        test_one_or_more("let, let }", true, true);
        test_one_or_more("let, let, let }", true, true);
        test_one_or_more("let, let, let, }", true, true);
        test_one_or_more("let, let,, let }", true, false);
        test_one_or_more("let, }", true, true);
        test_one_or_more("let, let, }", true, true);
        test_one_or_more("let, let, let, }", true, true);
        test_one_or_more("let, let,, let, }", true, false);
    }

    #[test]
    fn should_parse_top_level_type_defs() {
        let input = r#"
            struct X { }
            struct Y { a : Int }
            struct Z<T> { a : T }
            struct W<T, S> { a : T, b : S }

            enum A { }
            enum B { L }
            enum C { L, M }
            enum D { L(Int), M, N(Int, Float) }
            enum E<T> { L(T) }
            enum E<T, S> { L(T, S) }
        "#;

        let output = parse(input).unwrap();
        assert_eq!(output.len(), 10);
    }
}
