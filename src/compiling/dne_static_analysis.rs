
use std::rc::Rc;
use std::collections::{ HashSet, HashMap };

use crate::parsing::dne_parser::*;

use super::unifier::{Term, Unify};


type TypeMap<'a> = HashMap<Rc<str>, FunTypeInfo<'a>>;

pub enum StaticError {
    DupFunName(Rc<str>), 
    BuiltInCollision(Rc<str>),
    ExprIllTyped { expr: Rc<str>, found_type : Rc<str>, expected_type : Rc<str> },
    // TODO dup type names
    // TODO type name collides with built in
    // TODO type parameter shadows outer type (maybe a problem?)
}

pub struct FunTypeInfo<'a> {
    name : Rc<str>,
    return_type : &'a Type,
    param_types : Vec<&'a Type>,
    type_params : HashSet<Rc<str>>,
}

impl<'a> From<&'a Fun> for FunTypeInfo<'a> {
    fn from(item: &'a Fun) -> Self {
        FunTypeInfo { 
            name: Rc::clone(&item.name),
            param_types: item.params.iter().map(|(_, t)| t).collect(),
            return_type: &item.return_type,
            type_params: HashSet::from_iter(item.type_params.iter().map(|x| Rc::clone(x)))
        }
    }
}

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

    let fun_types : HashMap<Rc<str>, FunTypeInfo> = HashMap::from_iter(
        built_ins.into_iter().map(|x| (Rc::clone(&x.name), x))
        .chain(program.iter().map(|x| (Rc::clone(&x.name), x.into()))));

    program.iter().map(|x| check_fun(x, &fun_types)).flat_map(f).collect()
}

fn check_fun(target : &Fun, global_fun_types : &TypeMap) -> Result<(), Vec<StaticError>> {

    // TODO rename
    fn var(x : &Rc<str>) -> Term { Term::Var(Rc::clone(x)) }
    fn atom(x : &Rc<str>) -> Term { Term::Data(Rc::clone(x), vec![]) }
    fn rule(x : &Rc<str>, xs : Vec<Term>) -> Term { Term::Data(Rc::clone(x), xs) }
    
    let mut checker = Checker::new();

    for (n, t) in &target.params {
        checker.type_var(n, type_to_term(&t, &HashSet::new()));
    }

    check(expr(&mut checker, &target.expr, type_to_term(&target.return_type, &HashSet::new())))?;

    // TODO build up hashmap with variable to type (which i think is only internal to expr type)
    // Note target's param types don't really matter because they can be considered as some type X
    // called funs param types will matter because we'll be computing them
    // TODO also note that each time a function (which has already been called) with a type var is called it will need a fresh variable


    // TODO add lets and local funs to ts

    // TODO compute expr and make sure its valid and matches return type

    todo!()
}

fn expr(checker : &mut Checker, expr : &Expr, expected_type : Term) -> Vec<StaticError> { 
    let t = match expr {
        Expr::Lit(Lit::Int(_)) => Term::Data("Int".into(), vec![]),
        Expr::Lit(Lit::Float(_)) => Term::Data("Float".into(), vec![]),
        Expr::Lit(Lit::Bool(_)) => Term::Data("Bool".into(), vec![]),
        Expr::Lit(Lit::String(_)) => Term::Data("String".into(), vec![]),
        _ => todo!(),
    };
    // TODO:  This seems unfortunate
    let error_expected_type : Rc<str> = format!("{:?}", expected_type).into();
    let error_found_type : Rc<str> = format!("{:?}", t).into();
    if !checker.unify_types(t, expected_type) {
        vec![ StaticError::ExprIllTyped { 
            expr: format!("{:?}", expr).into(), 
            found_type: error_found_type,
            expected_type: error_expected_type,
        }]
    }
    else {
        vec![]
    }
}

fn type_to_term(t : &Type, vars : &HashSet<Rc<str>>) -> Term {
    if vars.contains(&t.name) && t.params.len() == 0 {
        Term::Var(Rc::clone(&t.name))
    }
    else {
        Term::Data(Rc::clone(&t.name), t.params.iter().map(|x| type_to_term(x, vars)).collect())
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

struct Checker<'a> {
    unifier : Unify<'a>,
    vars : HashMap<Rc<str>, Term>,
}

impl<'a> Checker<'a> {
    pub fn new() -> Self { Checker { unifier: Unify::new(), vars: HashMap::new() } }

    pub fn infer_var(&mut self, var : &Rc<str>) {
        todo!()
    }

    pub fn type_var(&mut self, var : &Rc<str>, t : Term) {
        todo!()
    }

    pub fn unify_types(&mut self, a : Term, b : Term) -> bool {
        todo!()
    }
}
