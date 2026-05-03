
use std::rc::Rc;
use crate::parsing::lexer::dne::Token;
use super::data::*;
use super::util::*;


pub fn parse_let(input : &mut Input) -> Result<Def, ParseError> {
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

pub fn parse_expr(input : &mut Input) -> Result<Expr, ParseError> {
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

pub fn parse_pattern_var_follow_on(input : &mut Input, name : Rc<str>) -> Result<MatchPattern, ParseError> {
    if input.check(|x| x.eq(&Token::Colon))? {
        input.expect(|x| x.eq(&Token::Colon))?;
        let case = expect_sym(input)?;
        let params = if input.check(|x| x.eq(&Token::LParen))? {
            one_or_more(input, Token::RParen, parse_match_pattern, false)?
        }
        else {
            vec![]
        };
        Ok(MatchPattern::Enum { ttype: name, case, params } )
    }
    else if input.check(|x| x.eq(&Token::LCurl))? {
        // TODO needs to also handle, ..
        let fields = zero_or_more(input, Token::RCurl, |input| {
            let field = expect_sym(input)?;
            input.expect(|x| x.eq(&Token::Colon))?;
            let pattern = parse_match_pattern(input)?;
            Ok((field, pattern))
        }, true)?;
        Ok(MatchPattern::Struct { ttype: name, fields, total: true })
    }
    else {
        Ok(MatchPattern::Var(name))
    }
}

pub fn parse_match_pattern(input : &mut Input) -> Result<MatchPattern, ParseError> {
    if input.check(|x| x.eq(&Token::LSquare))? {
        todo!()
    }
    else if let Token::Symbol(x) = input.peek()? && x.as_ref() == "_" {
        input.take()?;
        Ok(MatchPattern::Wild)
    }
    else if let Token::Symbol(x) = input.peek()? {
        let x = Rc::clone(x);
        input.take()?;
        parse_pattern_var_follow_on(input, x)
    }
    else {
        let (s, e) = input.current()?;
        Err(ParseError::Fatal(s, e))
    }
}

pub fn parse_match_case(input : &mut Input) -> Result<MatchCase, ParseError> {
    let pat = parse_match_pattern(input)?;
    let pred = if input.check(|x| x.eq(&Token::If))? {
        Some(parse_expr(input)?)
    }
    else { 
        None 
    };
    input.expect(|x| x.eq(&Token::DArrow))?;
    let expr = parse_expr(input)?;
    Ok(MatchCase { pat, expr, pred })
}

pub fn parse_match(input : &mut Input) -> Result<Expr, ParseError> {
    let expr = parse_expr(input)?;
    input.expect(|x| x.eq(&Token::With))?;
    input.expect(|x| x.eq(&Token::LCurl))?;
    let cases = zero_or_more(input, Token::RCurl, parse_match_case, true)?;
    Ok(Expr::Match(Box::new(expr), cases))
}

pub fn parse_var_follow_on(input : &mut Input, name : Rc<str>) -> Result<Expr, ParseError> {
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

pub fn parse_type(input : &mut Input) -> Result<Type, ParseError> {
    let name = expect_sym(input)?; 
    let params = if input.check(|x| x.eq(&Token::LAngle))? {
        one_or_more(input, Token::RAngle, parse_type, false)?
    }
    else {
        vec![]
    };
    Ok(Type { name, params })
}

pub fn expect_sym(input : &mut Input) -> Result<Rc<str>, ParseError> {
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
    use crate::parsing::dne_parser::parse;
    use crate::parsing::lexer::dne::lex;

    #[test]
    fn should_parse_match() {
        let input = r#"
            fun y() -> Bool { true }
            fun x() -> Int {
                let y = 0;

                match y with {
                    _ if y() => 1,
                    _ => 1,
                }
            }
        "#;

        let output = parse(input).unwrap();
        assert_eq!(output.len(), 2);
    }
}
