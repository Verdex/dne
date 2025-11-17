
use super::lexer::ir;
use super::parse_input::Input;

pub enum ParseError {
    Lex(usize),
    Fatal,
    Eof,
}

pub enum Top {
    Global,
    Proc
}

pub fn parse(input : &str) -> Result<Vec<Top>, ParseError> {
    let input = match ir::lex(input) {
        Err(i) => { return Err(ParseError::Lex(i)); },
        Ok(ls) => ls,
    };
    let mut input = Input::new(input, ParseError::Eof, ParseError::Fatal);

    todo!()
}

