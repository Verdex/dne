
use std::rc::Rc;
use std::collections::{ HashSet, HashMap };

use crate::parsing::dne_parser::*;

use super::unifier::{Term, Unify};


#[derive(Debug)]
pub enum StaticError {
    DupFunName(Rc<str>), 
    BuiltInCollision(Rc<str>),
    ExprIllTyped { expr: Rc<str>, found_type : Rc<str>, expected_type : Rc<str> },
    UnknownVar(Rc<str>),
    UnknownFun(Rc<str>),
    // TODO dup type names
    // TODO type name collides with built in
    // TODO dup type names
    // TODO type parameter shadows outer type (maybe a problem?)
    // TODO type names collide with function names (and/or builtins)
    
    // TODO FunStaticError { name: Rc<str>, error : FunStaticError },
}

/*
#[derive(Debug)]
pub enum FunStaticError {

}
*/

pub struct FunTypeInfo {
    name : Rc<str>,
    return_type : Type,
    param_types : Vec<Type>,
    type_params : Vec<Rc<str>>,
}

impl From<&Fun> for FunTypeInfo {
    fn from(item: &Fun) -> Self {
        FunTypeInfo { 
            name: Rc::clone(&item.name),
            param_types: item.params.iter().map(|(_, x)| x.clone()).collect(),
            return_type: item.return_type.clone(),
            type_params: item.type_params.clone(),
        }
    }
}

// TODO need to augment results such that the fun name is attached to each group
pub fn static_check(program : &[Fun], built_ins : Vec<FunTypeInfo>) -> Result<(), Vec<StaticError>> {
    let mut fun_names = program.iter().map(|x| Rc::clone(&x.name)).collect::<Vec<_>>();
    check( dup_fun( fun_names.clone() ).into_iter().map(StaticError::DupFunName).collect() )?;

    let mut built_in_names = built_ins.iter().map(|x| Rc::clone(&x.name)).collect::<Vec<_>>();
    fun_names.append(&mut built_in_names);
    check( dup_fun( fun_names ).into_iter().map(StaticError::BuiltInCollision).collect() )?;

    // TODO for each function determine if it has duplicate variable names

    check( types(program, built_ins) )?;

    Ok(())
}

fn types(program : &[Fun], built_ins : Vec<FunTypeInfo>) -> Vec<StaticError> {

    fn f(x : Result<(), Vec<StaticError>>) -> Vec<StaticError> { match x { Ok(_) => vec![], Err(x) => x } }

    let global_funs : HashMap<Rc<str>, FunTypeInfo> = HashMap::from_iter(
        built_ins.into_iter().map(|x| (Rc::clone(&x.name), x))
        .chain(program.iter().map(|x| (Rc::clone(&x.name), x.into()))));

    let mut env = Env { global_funs };

    program.iter().map(|x| check_fun(x, &mut env)).flat_map(f).collect()
}

fn check_fun(target : &Fun, env : &mut Env) -> Result<(), Vec<StaticError>> {

    let mut checker = Checker::new();

    for (n, t) in &target.params {
        checker.type_var(n, &type_to_term(&t, &HashSet::new()));
    }

    for def in &target.defs {
        match def {
            Def::Let { name, ttype: None, expr } => {
                checker.infer_var(name);
                let term = checker.get_type(name)?;
                check_expr(&mut checker, env, expr, &term)?;
            },
            Def::Let { name, ttype: Some(t), expr } => {
                let term = type_to_term(&t, &HashSet::new());
                check_expr(&mut checker, env, expr, &term)?;
                checker.type_var(name, &term);
            },
            Def::Fun(x) => todo!(),
        }
    }

    check_expr(&mut checker, env, &target.expr, &type_to_term(&target.return_type, &HashSet::new()))

    // TODO build up hashmap with variable to type (which i think is only internal to expr type)
    // Note target's param types don't really matter because they can be considered as some type X
    // called funs param types will matter because we'll be computing them
    // TODO also note that each time a function (which has already been called) with a type var is called it will need a fresh variable


    // TODO  local funs to ts

}

fn check_expr(checker : &mut Checker, env : &Env, expr : &Expr, expected_type : &Rc<Term>) -> Result<(), Vec<StaticError>> { 
    let t = match expr {
        Expr::Lit(Lit::Int(_)) => u_atom(&"Int".into()),
        Expr::Lit(Lit::Float(_)) => u_atom(&"Float".into()),
        Expr::Lit(Lit::Bool(_)) => u_atom(&"Bool".into()),
        Expr::Lit(Lit::String(_)) => u_atom(&"String".into()),
        Expr::Var(x) => checker.get_type(x)?,
        Expr::Call { name, params } => { 
            let info = env.get_fun_info(&name)?;
            let type_vars : HashSet<Rc<str>> = HashSet::from_iter(info.type_params.iter().map(Rc::clone));

            check( params.iter().zip(info.param_types.iter())
                .map(|(p, t)| check_expr(checker, env, p, &type_to_term(t, &type_vars)))
                .flat_map(|x| match x { Err(x) => x, _ => vec![] }).collect() )?;
            
            type_to_term( &info.return_type, &type_vars )
        },
        _ => todo!(),
    };
    if !checker.unify_types(&t, expected_type) {
        Err(vec![ StaticError::ExprIllTyped { 
            expr: format!("{:?}", expr).into(), 
            found_type: format!("{:?}", t).into(),
            expected_type: format!("{:?}", expected_type).into(),
        }])
    }
    else {
        Ok(())
    }
}

fn type_to_term(t : &Type, type_vars : &HashSet<Rc<str>>) -> Rc<Term> {
    if type_vars.contains(&t.name) && t.params.len() == 0 {
        u_var(&t.name)
    }
    else {
        u_data(&t.name, t.params.iter().map(|x| type_to_term(x, type_vars)).collect())
    }
}

fn dup_fun(mut x : Vec<Rc<str>>) -> Vec<Rc<str>> {
    x.sort(); 
    let mut x = x.iter().zip(x.iter().skip(1)).filter_map(|(a, b)| if a == b { Some(Rc::clone(a)) } else { None }).collect::<Vec<_>>();
    x.dedup();
    x
}

fn check(x : Vec<StaticError>) -> Result<(), Vec<StaticError>> {
    if x.len() == 0 { Ok(()) }
    else { Err(x) }
}

struct Env { 
    global_funs: HashMap<Rc<str>, FunTypeInfo>,
}

impl Env {
    pub fn get_fun_info(&self, name : &Rc<str>) -> Result<&FunTypeInfo, Vec<StaticError>> {
        match self.global_funs.get(name) {
            Some(v) => Ok(v),
            None => todo!(), // TODO fun name does not exist
        }
    }
}

struct Checker {
    unifier : Unify,
    vars : HashMap<Rc<str>, Rc<Term>>,
    gen_value : usize,
}

impl Checker {
    pub fn new() -> Self { Checker { unifier: Unify::new(), vars: HashMap::new(), gen_value: 0 } }

    pub fn infer_var(&mut self, var : &Rc<str>) {
        let x = self.gensym();
        self.vars.insert(Rc::clone(var), u_var(&x));
    }

    pub fn type_var(&mut self, var : &Rc<str>, t : &Rc<Term>) {
        self.vars.insert(Rc::clone(var), Rc::clone(t));
    }

    pub fn get_type(&mut self, var : &Rc<str>) -> Result<Rc<Term>, Vec<StaticError>> {
        match self.vars.get(var) {
            None => Err(vec![StaticError::UnknownVar(Rc::clone(var))]),
            Some(x) => Ok(Rc::clone(x)),
        }
    }

    pub fn unify_types(&mut self, a : &Rc<Term>, b : &Rc<Term>) -> bool {
        self.unifier.unify(a, b)
    }

    fn gensym(&mut self) -> Rc<str> {
        self.gen_value += 1;
        format!("gen_{}", self.gen_value).into()
    }
}

fn u_var(x : &Rc<str>) -> Rc<Term> { Term::Var(Rc::clone(x)).into() }
fn u_atom(x : &Rc<str>) -> Rc<Term> { Term::Data(Rc::clone(x), vec![]).into() }
fn u_data(x : &Rc<str>, xs : Vec<Rc<Term>>) -> Rc<Term> { Term::Data(Rc::clone(x), xs).into() }

#[cfg(test)]
mod test {
    use super::*;
    use crate::parsing::dne_parser::*;

    #[test]
    fn should_type_check_param_with_return() {
        let x = r#"fun x(y : Int) -> Int { y }"#;
        let y = parse(x).unwrap();
        let z = static_check(&y, vec![]);
        assert!(z.is_ok());
    }

    #[test]
    fn should_fail_type_check_param_with_return() {
        let x = r#"fun x(y : Bool) -> Int { y }"#;
        let y = parse(x).unwrap();
        let z = static_check(&y, vec![]);
        assert!(z.is_err());
        let z = z.unwrap_err();
        assert!(z.len() == 1);
        assert!(matches!(z[0], StaticError::ExprIllTyped { .. }));
    }

    #[test]
    fn should_fail_type_check_lit_with_return() {
        let x = r#"fun x() -> Int { true }"#;
        let y = parse(x).unwrap();
        let z = static_check(&y, vec![]);
        assert!(z.is_err());
        let z = z.unwrap_err();
        assert!(z.len() == 1);
        assert!(matches!(z[0], StaticError::ExprIllTyped { .. }));
    }

    #[test]
    fn should_type_check_single_int_lit_with_return() {
        let x = r#"fun x() -> Int { 0 }"#;
        let y = parse(x).unwrap();
        let z = static_check(&y, vec![]);
        assert!(z.is_ok());
    }

    #[test]
    fn should_type_check_single_float_lit_with_return() {
        let x = r#"fun x() -> Float { -1.1 }"#;
        let y = parse(x).unwrap();
        let z = static_check(&y, vec![]);
        assert!(z.is_ok());
    }

    #[test]
    fn should_type_check_single_string_lit_with_return() {
        let x = r#"fun x() -> String { "string" }"#;
        let y = parse(x).unwrap();
        let z = static_check(&y, vec![]);
        assert!(z.is_ok());
    }

    #[test]
    fn should_type_check_single_bool_lit_with_return() {
        let x = r#"fun x() -> Bool { true }"#;
        let y = parse(x).unwrap();
        let z = static_check(&y, vec![]);
        assert!(z.is_ok());
    }

    #[test]
    fn should_type_check_let_with_lit() {
        let x = r#"
            fun x() -> Bool { 
                let y : Bool = true; 
                y
            }
        "#;
        let y = parse(x).unwrap();
        let z = static_check(&y, vec![]);
        assert!(z.is_ok());
    }

    #[test]
    fn should_fail_type_check_let_with_lit() {
        let x = r#"
            fun x() -> Bool { 
                let y : Bool = 0; 
                y
            }
        "#;
        let y = parse(x).unwrap();
        let z = static_check(&y, vec![]);
        assert!(z.is_err());
        let z = z.unwrap_err();
        assert!(z.len() == 1);
        assert!(matches!(z[0], StaticError::ExprIllTyped { .. }));
    }

    #[test]
    fn should_fail_type_check_let_with_param() {
        let x = r#"
            fun x(z : Int) -> Bool { 
                let y : Bool = z; 
                y
            }
        "#;
        let y = parse(x).unwrap();
        let z = static_check(&y, vec![]);
        assert!(z.is_err());
        let z = z.unwrap_err();
        assert!(z.len() == 1);
        assert!(matches!(z[0], StaticError::ExprIllTyped { .. }));
    }

    #[test]
    fn should_fail_type_check_let_with_var() {
        let x = r#"
            fun x() -> Int { 
                let y : Bool = true; 
                let z : Int = y; 
                y
            }
        "#;
        let y = parse(x).unwrap();
        let z = static_check(&y, vec![]);
        assert!(z.is_err());
        let z = z.unwrap_err();
        assert!(z.len() == 1);
        assert!(matches!(z[0], StaticError::ExprIllTyped { .. }));
    }

    #[test]
    fn should_fail_type_check_return_with_var() {
        let x = r#"
            fun x() -> Bool { 
                let y : Int = 0; 
                y
            }
        "#;
        let y = parse(x).unwrap();
        let z = static_check(&y, vec![]);
        assert!(z.is_err());
        let z = z.unwrap_err();
        assert!(z.len() == 1);
        assert!(matches!(z[0], StaticError::ExprIllTyped { .. }));
    }

    #[test]
    fn should_type_check_with_inferred_var() {
        let x = r#"
            fun x() -> Bool { 
                let y = true; 
                let z = y; 
                z
            }
        "#;
        let y = parse(x).unwrap();
        let z = static_check(&y, vec![]);
        assert!(z.is_ok());
    }
    
    #[test]
    fn should_fail_type_check_with_inferred_var() {
        let x = r#"
            fun x() -> Int { 
                let y = true; 
                let z = y; 
                z
            }
        "#;
        let y = parse(x).unwrap();
        let z = static_check(&y, vec![]);
        assert!(z.is_err());
        let z = z.unwrap_err();
        assert!(z.len() == 1);
        assert!(matches!(z[0], StaticError::ExprIllTyped { .. }));
    }

    #[test]
    fn should_type_check_call_with_inferred_fun_info() {
        let x = r#"
            fun y<T>( x : T ) -> T { x }

            fun x() -> Bool { 
                let z = true;
                y( z )
            }
        "#;
        let y = parse(x).unwrap();
        let z = static_check(&y, vec![]);
        assert!(z.is_ok());
    }
}
