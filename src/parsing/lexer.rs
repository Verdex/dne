
use std::rc::Rc;
use std::str::CharIndices;
use std::iter::Peekable;

type Input<'a> = Peekable<CharIndices<'a>>;

pub mod ir {

    use super::*;

    #[derive(Debug, PartialEq)]
    pub struct StringSegment {
        s : Rc<str>,
        var : Option<Rc<str>>,
    }

    #[derive(Debug, PartialEq)]
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
        Bool(bool),
        Symbol(Rc<str>),
        String(Vec<StringSegment>),
        Type,
        Slot,
        SlotSet,
        SlotInsert,
        SlotRemove,
        Length,
        Proc,
        Return,
        Yield,
        Resume,
        Break,
        Coroutine,
        DynCoroutine,
        Set,
        Jump,
        Label,
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
            // TODO string
            // TODO comment
            match input.peek() {
                None => { return Ok(ret); },
                Some((_, c)) if c.is_whitespace() => {
                    whitespace(&mut input)?;
                },
                Some((_, c)) if c.is_alphabetic() || *c == '_' => {
                    ret.push(symbol(&mut input)?);
                },
                Some((_, c)) if c.is_numeric() || *c == '-' => { 
                    let x = match number_or_arrow(&mut input)? {
                        Num::Int(x) => Token::Int(x),
                        Num::Float(x) => Token::Float(x),
                        Num::Arrow => Token::Arrow,
                    };

                    ret.push(x);
                },
                Some((_, '(')) => { input.next().unwrap(); ret.push(Token::LParen); },
                Some((_, ')')) => { input.next().unwrap(); ret.push(Token::RParen); },
                Some((_, '{')) => { input.next().unwrap(); ret.push(Token::LCurl); },
                Some((_, '}')) => { input.next().unwrap(); ret.push(Token::RCurl); },
                Some((_, ',')) => { input.next().unwrap(); ret.push(Token::Comma); },
                Some((_, ';')) => { input.next().unwrap(); ret.push(Token::SemiColon); },
                Some((_, ':')) => { input.next().unwrap(); ret.push(Token::Colon); },
                Some((_, '=')) => { input.next().unwrap(); ret.push(Token::Equal); },
                Some((i, _)) => { return Err(*i); },
            }
        }
    }

    fn symbol(input : &mut Input) -> Result<Token, usize> {
        let s = take_until(input, |c| c.is_alphanumeric() || c == '_');
        let s = s.into_iter().collect::<String>();

        match s.as_str() {
            "type" => Ok(Token::Type),
            "slot" => Ok(Token::Slot),
            "slot_set" => Ok(Token::SlotSet),
            "slot_insert" => Ok(Token::SlotInsert),
            "slot_remove" => Ok(Token::SlotRemove),
            "length" => Ok(Token::Length),
            "proc" => Ok(Token::Proc),
            "return" =>  Ok(Token::Return),
            "yield" =>  Ok(Token::Yield),
            "resume" =>  Ok(Token::Resume),
            "break" =>  Ok(Token::Break),
            "coroutine" =>  Ok(Token::Coroutine),
            "dyn_coroutine" =>  Ok(Token::DynCoroutine),
            "set" =>  Ok(Token::Set),
            "jump" =>  Ok(Token::Jump),
            "label" => Ok(Token::Label),
            "branch_equal" =>  Ok(Token::BranchEqual),
            "global" =>  Ok(Token::Global),
            "call" =>  Ok(Token::Call),
            "dyn_call" =>  Ok(Token::DynCall),
            "closure" =>  Ok(Token::Closure),
            "true" => Ok(Token::Bool(true)),
            "false" => Ok(Token::Bool(false)),
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

enum Num { Int(i64), Float(f64), Arrow } 
fn number_or_arrow(input : &mut Input) -> Result<Num, usize> {
    let (i, c) = *input.peek().unwrap();

    if c == '-' {
        input.next().unwrap();
        if matches!( input.peek(), Some((_, '>'))) {
            input.next().unwrap();
            return Ok(Num::Arrow);
        }
    }

    let s = take_until(input, |c| c.is_numeric() || c == '.' || c == '-' || c == 'E' || c == 'e' || c == '+');
    let mut s = s.into_iter().collect::<String>();

    if c == '-' {
        s.insert(0, '-');
    }

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


#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn should_lex_punct() {
        let input = "(){}:;,=";
        let output = ir::lex(input).unwrap();
        assert_eq!(output.len(), 8);
    }
}
