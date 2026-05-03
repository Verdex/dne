
mod data;
mod util;
mod dne_parser;
mod top_level;

pub use data::*;


pub fn parse(input : &str) -> Result<Vec<Top>, ParseError> {
    let input = match crate::parsing::lexer::dne::lex(input) {
        Err(i) => { return Err(ParseError::Lex(i)); },
        Ok(ls) => ls,
    };
    let mut input = Input::new(input, ParseError::Eof, |s, e| ParseError::Fatal(s, e));

    top_level::parse_tops(&mut input)
}

