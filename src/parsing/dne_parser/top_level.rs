
use crate::parsing::lexer::dne::Token;
use super::data::*;
use super::expr::*;
use super::util::*;

pub fn parse_tops(input : &mut Input) -> Result<Vec<Top>, ParseError> {
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

#[cfg(test)]
mod test {
    use super::*;
    use crate::parsing::dne_parser::parse;
    use crate::parsing::lexer::dne::lex;

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
