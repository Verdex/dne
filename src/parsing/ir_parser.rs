
use super::lexer::ir;
use super::parse_input::Input;

#[derive(Debug)]
pub enum ParseError {
    Lex(usize),
    Fatal,
    Eof,
}


impl std::fmt::Display for ParseError {
    fn fmt(&self, f : &mut std::fmt::Formatter) -> std::fmt::Result {
        // TODO 
        match self { 
            _ => write!(f, ""),
        }
    }
}

impl std::error::Error for ParseError { }



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

