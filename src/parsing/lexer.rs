
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
        ConsType(Rc<str>),
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
        BranchTrue,
        Global,
        Call,
        DynCall,
        Closure,
        Cons,
    }

    pub fn lex(input : &str) -> Result<Vec<(Token, usize, usize)>, usize> {
        macro_rules! punct {
            ($input:ident, $ret:ident, $i:ident, $t:expr) => { { let i = *$i; $input.next().unwrap(); $ret.push(($t, i, i)); } }
        }

        let mut input = input.char_indices().peekable();
        let mut ret : Vec<(Token, usize, usize)> = vec![];
 
        loop {
            // TODO string
            // TODO comment
            match input.peek() {
                None => { return Ok(ret); },
                Some((_, c)) if c.is_whitespace() => {
                    whitespace(&mut input)?;
                },
                Some((s, c)) if c.is_alphabetic() || *c == '_' => {
                    let s = *s;
                    let (t, l) = symbol(&mut input)?;
                    ret.push((t, s, s + l));
                },
                Some((s, c)) if c.is_numeric() || *c == '-' => { 
                    let s = *s;
                    let (x, e) = number_or_arrow(&mut input)?;
                    let x = match x {
                        Num::Int(x) => Token::Int(x),
                        Num::Float(x) => Token::Float(x),
                        Num::Arrow => Token::Arrow,
                    };

                    ret.push((x, s, e));
                },
                Some((s, c)) if *c == '~' => {
                    let s = *s;
                    let (t, l) = cons_type(&mut input)?;
                    ret.push((t, s, s + l));
                },
                Some((i, '(')) => punct!(input, ret, i, Token::LParen),
                Some((i, ')')) => punct!(input, ret, i, Token::RParen), 
                Some((i, '{')) => punct!(input, ret, i, Token::LCurl), 
                Some((i, '}')) => punct!(input, ret, i, Token::RCurl), 
                Some((i, ',')) => punct!(input, ret, i, Token::Comma),
                Some((i, ';')) => punct!(input, ret, i, Token::SemiColon),
                Some((i, ':')) => punct!(input, ret, i, Token::Colon), 
                Some((i, '=')) => punct!(input, ret, i, Token::Equal), 
                Some((i, _)) => { return Err(*i); },
            }
        }
    }

    fn cons_type(input : &mut Input) -> Result<(Token, usize), usize> {
        input.next().unwrap();
        let s = take_until(input, |c| c.is_alphanumeric() || c == '_');
        let s = s.into_iter().collect::<String>();
        let l = s.len() - 1;
        Ok((Token::ConsType(s.into()), l))
    }

    fn symbol(input : &mut Input) -> Result<(Token, usize), usize> {
        let s = take_until(input, |c| c.is_alphanumeric() || c == '_');
        let s = s.into_iter().collect::<String>();
        let l = s.len() - 1;

        let r = match s.as_str() {
            "type" => Token::Type,
            "slot" => Token::Slot,
            "slot_set" => Token::SlotSet,
            "slot_insert" => Token::SlotInsert,
            "slot_remove" => Token::SlotRemove,
            "length" => Token::Length,
            "proc" => Token::Proc,
            "return" => Token::Return,
            "yield" => Token::Yield,
            "resume" => Token::Resume,
            "break" => Token::Break,
            "coroutine" => Token::Coroutine,
            "dyn_coroutine" => Token::DynCoroutine,
            "set" => Token::Set,
            "jump" => Token::Jump,
            "label" => Token::Label,
            "branch_true" => Token::BranchTrue,
            "global" => Token::Global,
            "call" => Token::Call,
            "dyn_call" => Token::DynCall,
            "closure" => Token::Closure,
            "true" => Token::Bool(true),
            "false" => Token::Bool(false),
            "cons" => Token::Cons,
            s => Token::Symbol(s.into()),
        };

        Ok((r, l))
    }
}

fn whitespace(input : &mut Input) -> Result<(), usize> {
    while let Some((_, c)) = input.peek() && c.is_whitespace() {
        input.next().unwrap();
    }
    Ok(())
}

enum Num { Int(i64), Float(f64), Arrow } 
fn number_or_arrow(input : &mut Input) -> Result<(Num, usize), usize> {
    let (i, c) = *input.peek().unwrap();

    if c == '-' {
        input.next().unwrap();
        if matches!( input.peek(), Some((_, '>'))) {
            input.next().unwrap();
            return Ok((Num::Arrow, i + 1));
        }
    }

    let s = take_until(input, |c| c.is_numeric() || c == '.' || c == '-' || c == 'E' || c == 'e' || c == '+');
    let mut s = s.into_iter().collect::<String>();

    if c == '-' {
        s.insert(0, '-');
    }

    let end = i + s.len() - 1;

    match s.parse::<i64>() {
        Ok(x) => Ok((Num::Int(x), end)),
        Err(_) => match s.parse::<f64>() {
            Ok(x) => Ok((Num::Float(x), end)),
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
