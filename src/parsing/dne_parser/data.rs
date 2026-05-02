
use std::rc::Rc;
use crate::parsing::lexer::dne::{self, Token};

pub type Input = crate::parsing::parse_input::Input<Token, ParseError>;

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
    Struct { ttype: Rc<str>, fields: Vec<(Rc<str>, MatchPattern)>, total: bool },
    Enum { ttype: Rc<str>, case: Rc<str>, params: Vec<MatchPattern> },
}

#[derive(Debug)]
pub struct MatchCase {
    pub pat : MatchPattern,
    pub expr : Expr,
    pub pred : Option<Expr>,
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

