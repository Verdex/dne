
use crate::parsing::lexer::dne::Token;
use super::data::*;

pub fn zero_or_more<T>(
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

pub fn one_or_more<T>(
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
    use crate::parsing::dne_parser::parse;
    use crate::parsing::lexer::dne::lex;

    fn test_zero_or_more(input : &str, trail : bool, success : bool) {
        let input = lex(input).unwrap();
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
        let input = lex(target).unwrap();
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
}

