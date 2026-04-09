
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
pub enum MatchPattern {
    Wild,
    Var(Rc<str>),
    List(Vec<MatchPattern>, Box<MatchPattern>), 
    Struct { fields: Vec<(Rc<str>, MatchPattern)>, total: bool },
    Enum { ttype: Rc<str>, case: Rc<str>, params: Vec<MatchPattern> },
}

#[derive(Debug)]
pub struct MatchCase {
    pat : MatchPattern,
    expr : Expr,
    pred : Option<Expr>,
}

#[derive(Debug)]
pub enum Expr { 
    Lit(Lit), 
    Call { name : Rc<str>, params : Vec<Expr> },
    CaseCons { ttype : Rc<str>, case : Rc<str>, params : Vec<Expr> },
    StructCons { ttype : Rc<str>, params : Vec<(Rc<str>, Expr)> },
    Var(Rc<str>),
    List(Vec<Expr>),
    Match(Box<Expr>, Vec<MatchCase>),
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
        one_or_more(input, Token::RAngle, expect_sym, false)?
    }
    else {
        vec![]
    };
    input.expect(|x| x.eq(&Token::LCurl))?;
    let fields = zero_or_more(input, Token::RCurl, |input| {
        let field = expect_sym(input)?;
        input.expect(|x| x.eq(&Token::Colon))?;
        let ttype = parse_type(input)?;
        Ok((field, ttype))
    }, true)?;
    Ok(Struct { name, type_params, fields })
}

fn parse_enum(input : &mut Input) -> Result<Enum, ParseError> {
    let name = expect_sym(input)?;
    let type_params = if input.check(|x| x.eq(&Token::LAngle))? {
        one_or_more(input, Token::RAngle, expect_sym, false)?
    }
    else {
        vec![]
    };
    input.expect(|x| x.eq(&Token::LCurl))?;
    let cases = zero_or_more(input, Token::RCurl, parse_enum_case, true)?;
    Ok(Enum { name, type_params, cases })
}

fn parse_enum_case(input : &mut Input) -> Result<EnumCase, ParseError> {
    let name = expect_sym(input)?;
    if input.check(|x| x.eq(&Token::LParen))? {
        let params = one_or_more(input, Token::RParen, parse_type, true)?;
        Ok(EnumCase { name, params })
    }
    else {
        Ok(EnumCase { name, params: vec![] })
    }
}

fn parse_fun(input : &mut Input) -> Result<Fun, ParseError> {
    let name = expect_sym(input)?;
    let type_params = if input.check(|x| x.eq(&Token::LAngle))? {
        one_or_more(input, Token::RAngle, expect_sym, false)?
    }
    else {
        vec![]
    };
    input.expect(|x| x.eq(&Token::LParen))?;
    let params = zero_or_more(input, Token::RParen, |input| {
        let param = expect_sym(input)?;
        input.expect(|x| x.eq(&Token::Colon))?;
        let ttype = parse_type(input)?;
        Ok((param, ttype))
    }, false)?;
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
        else if input.check(|x| x.eq(&Token::Yield))? {
            let r = expect_sym(input)?;
            input.expect(|x| x.eq(&Token::SemiColon))?;
            ret.push(Stmt::Yield(r));
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
    if let Token::Symbol(x) = input.peek()? {
        let x = Rc::clone(x);
        input.take()?;
        parse_var_follow_on(input, x)
    }
    else if input.check(|x| x.eq(&Token::Match))? {
        parse_match(input)
    }
    else if input.check(|x| x.eq(&Token::LSquare))? {
        let items = zero_or_more(input, Token::RSquare, parse_expr, true)?;
        Ok(Expr::List(items))
        // TODO this will need follow on as well
    }
    else if let Token::Int(x) = input.peek()? {
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
    else {
        let (s, e) = input.current()?;
        Err(ParseError::Fatal(s, e))
    }
}

fn parse_match_pattern(input : &mut Input) -> Result<MatchPattern, ParseError> {
    todo!()
}

fn parse_match_case(input : &mut Input) -> Result<MatchCase, ParseError> {
    let pat = parse_match_pattern(input)?;
    let pred = if input.check(|x| x.eq(&Token::If))? {
        Some(parse_expr(input)?)
    }
    else { 
        None 
    };
    let expr = parse_expr(input)?;
    Ok(MatchCase { pat, expr, pred })
}

fn parse_match(input : &mut Input) -> Result<Expr, ParseError> {
    let expr = parse_expr(input)?;
    input.expect(|x| x.eq(&Token::LCurl))?;
    let cases = zero_or_more(input, Token::RCurl, parse_match_case, true)?;
    Ok(Expr::Match(Box::new(expr), cases))
}

fn parse_var_follow_on(input : &mut Input, name : Rc<str>) -> Result<Expr, ParseError> {
    if input.check(|x| x.eq(&Token::LParen))? {
        let params = zero_or_more(input, Token::RParen, parse_expr, false)?;
        Ok(Expr::Call { name, params })
    }
    else if input.check(|x| x.eq(&Token::Colon))? {
        input.expect(|x| x.eq(&Token::Colon))?;
        let case = expect_sym(input)?;
        let params = if input.check(|x| x.eq(&Token::LParen))? {
            one_or_more(input, Token::RParen, parse_expr, false)?
        }
        else {
            vec![]
        };
        Ok(Expr::CaseCons { ttype: name, case, params } )
    }
    else if input.check(|x| x.eq(&Token::LCurl))? {
        let params = zero_or_more(input, Token::RCurl, |input| {
            let field = expect_sym(input)?;
            input.expect(|x| x.eq(&Token::Colon))?;
            let expr = parse_expr(input)?;
            Ok((field, expr))
        }, true)?;
        Ok(Expr::StructCons { ttype: name, params })
    }
    else {
        Ok(Expr::Var(name))
    }
}

fn parse_type(input : &mut Input) -> Result<Type, ParseError> {
    let name = expect_sym(input)?; 
    let params = if input.check(|x| x.eq(&Token::LAngle))? {
        one_or_more(input, Token::RAngle, parse_type, false)?
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

fn zero_or_more<T>(
    input : &mut Input, 
    end : Token, 
    item : impl Fn(&mut Input) -> Result<T, ParseError>,
    trail : bool) -> Result<Vec<T>, ParseError> {

    if input.check(|x| x.eq(&end))? {
        return Ok(vec![]);
    }
    let mut xs = vec![];
    xs.push(item(input)?);
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

    fn test_zero_or_more(input : &str, trail : bool, success : bool) {
        let input = dne::lex(input).unwrap();
        let mut input = Input::new(input, ParseError::Eof, |s, e| ParseError::Fatal(s, e));
        let out = zero_or_more(&mut input, Token::RCurl, |x| x.expect(|x| x.eq(&Token::Let)), trail);
        if success {
            assert!(matches!(out, Ok(_)));
        }
        else {
            assert!(matches!(out, Err(_)));
        }
    }

    fn test_one_or_more(target: &str, trail : bool, success : bool) {
        let input = dne::lex(target).unwrap();
        let mut input = Input::new(input, ParseError::Eof, |s, e| ParseError::Fatal(s, e));
        let out = one_or_more(&mut input, Token::RCurl, |x| x.expect(|x| x.eq(&Token::Let)), trail);
        if success {
            assert!(matches!(out, Ok(_)), "{}", target);
        }
        else {
            assert!(matches!(out, Err(_)), "{}", target);
        }
    }

    #[test]
    fn should_handle_zero_or_more() {
        test_zero_or_more("}", false, true);
        test_zero_or_more("let }", false, true);
        test_zero_or_more("let, let }", false, true);
        test_zero_or_more("let, let, let }", false, true);
        test_zero_or_more("let, let, let, }", false, false);
        test_zero_or_more("let, let,, let }", false, false);
        test_zero_or_more("}", true, true);
        test_zero_or_more("let }", true, true);
        test_zero_or_more("let, let }", true, true);
        test_zero_or_more("let, let, let }", true, true);
        test_zero_or_more("let, let, let, }", true, true);
        test_zero_or_more("let, let,, let }", true, false);
        test_zero_or_more("let, }", true, true);
        test_zero_or_more("let, let, }", true, true);
        test_zero_or_more("let, let, let, }", true, true);
        test_zero_or_more("let, let,, let, }", true, false);
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
