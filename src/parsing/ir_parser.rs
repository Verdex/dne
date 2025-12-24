
use std::rc::Rc;
use super::lexer::ir::{self, Token};

type Input = super::parse_input::Input<Token, ParseError>;

#[derive(Debug, Clone)]
pub enum ParseError {
    Lex(usize),
    Fatal(usize, usize),
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

#[derive(Debug)]
pub enum Top {
    Global(Global),
    Proc(Proc),
}

#[derive(Debug)]
pub struct Global {
    pub name: Rc<str>, 
    pub ttype: Type,
    pub value: Lit,
}

#[derive(Debug)]
pub struct Proc {
    pub name: Rc<str>, 
    pub params: Vec<(Rc<str>, Type)>, 
    pub return_type : Type, 
    pub body : Vec<Stmt>,
}

#[derive(Debug)]
pub enum Stmt {
    Set { var: Rc<str>, ttype : Type, val: Expr },
    Jump(Rc<str>),
    BranchEqual { label: Rc<str>, var: Rc<str> },
    Return(Rc<str>),
    Yield(Rc<str>),
    Break,
    Label(Rc<str>),
    SlotInsert { var: Rc<str>, input: Rc<str>, index: usize },
    SlotRemove { var: Rc<str>, index: usize },
}

#[derive(Debug, PartialEq)]
pub enum Type {
    Int,
    Float,
    String,
    Bool,
    Symbol,
    Ref,
    Closure,
    Coroutine,
}

#[derive(Debug)]
pub enum Lit {
    Int(i64),
    Float(f64),
    Bool(bool),
    ConsType(Rc<str>),
    // TODO string
}

#[derive(Debug)]
pub enum Expr { 
    Lit(Lit), 
    Call { name : Rc<str>, params : Vec<Rc<str>> },
    DynCall { name : Rc<str>, params : Vec<Rc<str>> },
    Coroutine { name : Rc<str>, params : Vec<Rc<str>> },
    DynCoroutine { name : Rc<str>, params : Vec<Rc<str>> },
    // TODO params => captures
    Closure { name : Rc<str>, params : Vec<Rc<str>> },
    Cons { name : Rc<str>, params : Vec<Rc<str>> },
    Resume(Rc<str>),
    Length(Rc<str>),
    Type(Rc<str>),
    Var(Rc<str>),
    Slot { var: Rc<str>, index: usize },
}

pub fn parse(input : &str) -> Result<Vec<Top>, ParseError> {
    let input = match ir::lex(input) {
        Err(i) => { return Err(ParseError::Lex(i)); },
        Ok(ls) => ls,
    };
    let mut input = Input::new(input, ParseError::Eof, |s, e| ParseError::Fatal(s, e));

    parse_tops(&mut input)
}

fn parse_tops(input : &mut Input) -> Result<Vec<Top>, ParseError> {
    let mut ret = vec![];
    while !input.empty() {
        if input.check(|x| x.eq(&Token::Proc))? {
            ret.push(parse_proc(input)?);
        }
        else if input.check(|x| x.eq(&Token::Global))? {
            let name = expect_sym(input)?;
            input.expect(|x| x.eq(&Token::Colon))?;
            let ttype = parse_type(input)?;
            input.expect(|x| x.eq(&Token::Equal))?;
            let value = parse_lit(input)?;
            input.expect(|x| x.eq(&Token::SemiColon))?;
            ret.push(Top::Global(Global{ name, ttype, value }));
        }
        else {
            let (s, e) = input.current()?;
            return Err(ParseError::Fatal(s, e));
        }
    }
    Ok(ret)
}

fn parse_proc(input : &mut Input) -> Result<Top, ParseError> {
    let name = expect_sym(input)?;
    input.expect(|x| x.eq(&Token::LParen))?;
    let mut params = vec![];
    if !input.check(|x| x.eq(&Token::RParen))? {
        loop {
            let param = expect_sym(input)?;
            input.expect(|x| x.eq(&Token::Colon))?;
            let ttype = parse_type(input)?;
            params.push((param, ttype));

            if input.check(|x| x.eq(&Token::RParen))? {
                break;
            }
            else if input.check(|x| x.eq(&Token::Comma))? {
                continue;
            }
            else {
                let (s, e) = input.current()?;
                return Err(ParseError::Fatal(s, e));
            }
        }
    }
    input.expect(|x| x.eq(&Token::Arrow))?;
    let return_type = parse_type(input)?;
    input.expect(|x| x.eq(&Token::LCurl))?;
    let body = parse_stmts(input)?;
    input.expect(|x| x.eq(&Token::RCurl))?;
    Ok( Top::Proc(Proc{ name, params, return_type, body }))
}

fn parse_stmts(input : &mut Input) -> Result<Vec<Stmt>, ParseError> {
    let mut ret = vec![];
    loop {
        if input.check(|x| x.eq(&Token::Set))? {
            ret.push(parse_set(input)?);
        }
        else if input.check(|x| x.eq(&Token::Jump))? {
            let r = expect_sym(input)?;
            input.expect(|x| x.eq(&Token::SemiColon))?;
            ret.push(Stmt::Jump(r));
        }
        else if input.check(|x| x.eq(&Token::BranchTrue))? {
            let label = expect_sym(input)?;
            let var = expect_sym(input)?;
            input.expect(|x| x.eq(&Token::SemiColon))?;
            ret.push(Stmt::BranchEqual { label, var });
        }
        else if input.check(|x| x.eq(&Token::Return))? {
            let r = expect_sym(input)?;
            input.expect(|x| x.eq(&Token::SemiColon))?;
            ret.push(Stmt::Return(r));
        }
        else if input.check(|x| x.eq(&Token::Yield))? {
            let r = expect_sym(input)?;
            input.expect(|x| x.eq(&Token::SemiColon))?;
            ret.push(Stmt::Yield(r));
        }
        else if input.check(|x| x.eq(&Token::Break))? {
            input.expect(|x| x.eq(&Token::SemiColon))?;
            ret.push(Stmt::Break);
        }
        else if input.check(|x| x.eq(&Token::Label))? {
            let r = expect_sym(input)?;
            input.expect(|x| x.eq(&Token::SemiColon))?;
            ret.push(Stmt::Label(r));
        }
        else if input.check(|x| x.eq(&Token::SlotInsert))? {
            let var = expect_sym(input)?;
            let var_input = expect_sym(input)?;
            let index = expect_index(input)?;
            input.expect(|x| x.eq(&Token::SemiColon))?;
            ret.push(Stmt::SlotInsert { var, input: var_input, index })
        }
        else if input.check(|x| x.eq(&Token::SlotRemove))? {
            let var = expect_sym(input)?;
            let index = expect_index(input)?;
            input.expect(|x| x.eq(&Token::SemiColon))?;
            ret.push(Stmt::SlotRemove { var, index })
        }
        else {
            return Ok(ret);
        }
    }
}

fn parse_set(input : &mut Input) -> Result<Stmt, ParseError> {
    let var = expect_sym(input)?; 
    input.expect(|x| x.eq(&Token::Colon))?;
    let ttype = parse_type(input)?;
    input.expect(|x| x.eq(&Token::Equal))?;
    let val = parse_expr(input)?;
    input.expect(|x| x.eq(&Token::SemiColon))?;
    Ok(Stmt::Set { var, val, ttype })
}

fn parse_lit(input : &mut Input) -> Result<Lit, ParseError> {
    if let Token::Int(x) = input.peek()? {
        let x = *x;
        input.take()?;
        Ok(Lit::Int(x))
    }
    else if let Token::Float(x) = input.peek()? {
        let x = *x;
        input.take()?;
        Ok(Lit::Float(x))
    }
    else if let Token::Bool(x) = input.peek()? {
        let x = *x;
        input.take()?;
        Ok(Lit::Bool(x))
    }
    else if let Token::ConsType(x) = input.peek()? {
        let x = Rc::clone(x);
        input.take()?;
        Ok(Lit::ConsType(x))
    }
    else {
        let (s, e) = input.current()?;
        Err(ParseError::Fatal(s, e))
    }
}

fn parse_expr(input : &mut Input) -> Result<Expr, ParseError> {
    if let Token::Int(x) = input.peek()? {
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
    else if let Token::ConsType(x) = input.peek()? {
        let x = Rc::clone(x);
        input.take()?;
        Ok(Expr::Lit(Lit::ConsType(x)))
    }
    else if let Token::Symbol(x) = input.peek()? {
        let x = Rc::clone(x);
        input.take()?;
        Ok(Expr::Var(x))
    }
    else if input.check(|x| x.eq(&Token::Call))? {
        let name = expect_sym(input)?;
        let params = expect_params(input)?;
        Ok(Expr::Call { name, params })
    }
    else if input.check(|x| x.eq(&Token::DynCall))? {
        let name = expect_sym(input)?;
        let params = expect_params(input)?;
        Ok(Expr::DynCall { name, params })
    }
    else if input.check(|x| x.eq(&Token::Coroutine))? {
        let name = expect_sym(input)?;
        let params = expect_params(input)?;
        Ok(Expr::Coroutine { name, params })
    }
    else if input.check(|x| x.eq(&Token::DynCoroutine))? {
        let name = expect_sym(input)?;
        let params = expect_params(input)?;
        Ok(Expr::DynCoroutine { name, params })
    }
    else if input.check(|x| x.eq(&Token::Closure))? {
        let name = expect_sym(input)?;
        let params = expect_params(input)?;
        Ok(Expr::Closure { name, params })
    }
    else if input.check(|x| x.eq(&Token::Cons))? {
        let name = expect_sym(input)?;
        let params = expect_params(input)?;
        Ok(Expr::Cons { name, params })
    }
    else if input.check(|x| x.eq(&Token::Resume))? {
        Ok(Expr::Resume(expect_sym(input)?)) 
    }
    else if input.check(|x| x.eq(&Token::Length))? {
        Ok(Expr::Length(expect_sym(input)?)) 
    }
    else if input.check(|x| x.eq(&Token::Type))? {
        Ok(Expr::Type(expect_sym(input)?)) 
    }
    else if input.check(|x| x.eq(&Token::Slot))? {
        let var = expect_sym(input)?;
        let index = expect_index(input)?;
        Ok(Expr::Slot { var, index })
    }
    else {
        let (s, e) = input.current()?;
        Err(ParseError::Fatal(s, e))
    }
}

fn parse_type(input : &mut Input) -> Result<Type, ParseError> {
    let t = expect_sym(input)?; 
    match &*t {
        "Int" => Ok(Type::Int),
        "Float" => Ok(Type::Float),
        "String" => Ok(Type::String),
        "Bool" => Ok(Type::Bool),
        "Symbol" => Ok(Type::Symbol),
        "Ref" => Ok(Type::Ref),
        "Closure" => Ok(Type::Closure),
        "Coroutine" => Ok(Type::Coroutine),
        _ => {
            let (s, e) = input.current()?;
            Err(ParseError::Fatal(s, e))
        },
    }
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

fn expect_params(input : &mut Input) -> Result<Vec<Rc<str>>, ParseError> {
    input.expect(|x| x.eq(&Token::LParen))?;
    let mut ret = vec![];
    if input.check(|x| x.eq(&Token::RParen))? {
        return Ok(ret);
    }
    loop {
        ret.push(expect_sym(input)?);
        
        if input.check(|x| x.eq(&Token::RParen))? {
            break;
        }
        else if input.check(|x| x.eq(&Token::Comma))? {
            continue;
        }
        else {
            let (s, e) = input.current()?;
            return Err(ParseError::Fatal(s, e));
        }
    }
    Ok(ret)
}

fn expect_index(input : &mut Input) -> Result<usize, ParseError> {
    if let Token::Int(x) = input.peek()? {
        let x = *x;
        input.take()?;
        match usize::try_from(x) {
            Ok(x) => Ok(x),
            Err(_) => {
                let (s, e) = input.current()?;
                Err(ParseError::Fatal(s, e))
            },
        }
    }
    else {
        let (s, e) = input.current()?;
        Err(ParseError::Fatal(s, e))
    }
}

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn should_parse_globals() {
       let input = r#"
            global g_1 : Int = 0;
            global g_2 : Bool = true;
            global g_3 : Float = 0.1;
            global g_4 : Float = -0.1;
            global g_5 : Float = -0.1E-10;
            global g_5 : Float = -0.1E+10;
            global g_6 : Int = -1;
            global g_7 : Symbol = ~Symbol;
       "#; 

        let output = parse(input).unwrap();
        assert_eq!(output.len(), 8);
    }

    #[test]
    fn should_parse_empty_params_proc() {
        let input = r#"
            proc name() -> Int { set x : Int = 0; return x; } 
       "#; 

        let output = parse(input).unwrap();
        assert_eq!(output.len(), 1);
    }

    #[test]
    fn should_parse_params_proc() {
        let input = r#"
            proc name(x : Int, y : Int, z : Int) -> Int { return x; } 
       "#; 

        let output = parse(input).unwrap();
        assert_eq!(output.len(), 1);
    }

    #[test]
    fn should_parse_types() {
        let input = r#"
            proc name(x : Int) -> Int { return x; } 
            proc name(x : Float) -> Float { return x; } 
            proc name(x : String) -> String { return x; } 
            proc name(x : Bool) -> Bool { return x; } 
            proc name(x : Symbol) -> Symbol { return x; } 
            proc name(x : Ref) -> Ref { return x; } 
            proc name(x : Closure) -> Closure { return x; } 
            proc name(x : Coroutine) -> Coroutine { return x; } 
       "#; 

        let output = parse(input).unwrap();
        assert_eq!(output.len(), 8);
    }

    #[test]
    fn should_parse_statements() {
        let input = r#"
            proc name() -> Int { 
                set x : Int = 0;
                set b : Bool = false;
                yield x;
                jump location;
                label location;
                branch_true location b;
                break;
                slot_insert r i 1;
                slot_remove r 0;
                return x;
            } 
       "#; 

        let output = parse(input).unwrap();
        assert_eq!(output.len(), 1);
    }

    #[test]
    fn should_parse_exprs() {
        let input = r#"
            proc name() -> Int { 
                set x : Int = 0;
                set y : Float = 1.0;
                set z : Bool = true;
                set w : Int = call name (x, y, z);
                set a : Coroutine = coroutine name (x, y, z);
                set b : Int = resume a;
                set c : Int = x;
                set i : Ref = cons Blah (x, y, z);
                set j : Symbol = type i;
                set k : Int = slot i 0;
                set f : Closure = closure name (x, y, z);
                set g : Int = dyn_call f (x, y, z);
                set q : Coroutine = dyn_coroutine name (x, y, z);
                set r : Int = length i;
                set s : Symbol = ~Sym;
                return x;
            } 
       "#; 

        let output = parse(input).unwrap();
        assert_eq!(output.len(), 1);
    }

    fn d(input : &str, x : Result<Vec<Top>, ParseError>) {
        match x {
            Err(ParseError::Fatal(s, e)) => {
                let w = crate::util::underline(input, s, e);
                panic!("{w}");
            },
            _ => panic!("else"),
        }
    }
}
