
use std::rc::Rc;
use std::collections::{ HashSet, HashMap };

use crate::parsing::dne_parser::*;

use super::unifier::{Term, Unify};


type TypeMap<'a> = HashMap<Rc<str>, FunTypeInfo<'a>>;

pub enum StaticError {
    DupFunName(Rc<str>), 
    BuiltInCollision(Rc<str>),
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

    let fun_types : HashMap<Rc<str>, FunTypeInfo> = HashMap::from_iter(
        built_ins.into_iter().map(|x| (Rc::clone(&x.name), x))
        .chain(program.iter().map(|x| (Rc::clone(&x.name), x.into()))));

    program.iter().flat_map(|x| check_fun(x, &fun_types)).collect()
}

fn check_fun(target : &Fun, global_fun_types : &TypeMap) -> Vec<StaticError> {

    // TODO rename
    fn var(x : &Rc<str>) -> Term { Term::Var(Rc::clone(x)) }
    fn atom(x : &Rc<str>) -> Term { Term::Data(Rc::clone(x), vec![]) }
    fn rule(x : &Rc<str>, xs : Vec<Term>) -> Term { Term::Data(Rc::clone(x), xs) }
    
    let mut unifier = Unify::new();

    let mut ts : HashMap<Rc<str>, Term> = 
        HashMap::from_iter(target.params.iter().map(|(n, t)| (Rc::clone(n), type_to_term(t, &HashSet::new()))));
    

    // TODO build up hashmap with variable to type (which i think is only internal to expr type)
    // Note target's param types don't really matter because they can be considered as some type X
    // called funs param types will matter because we'll be computing them
    // TODO also note that each time a function (which has already been called) with a type var is called it will need a fresh variable


    // TODO add lets and local funs to ts

    // TODO compute expr and make sure its valid and matches return type

    todo!()
}

// TODO will have ts and global fun types as inputs
// TODO i think can have it's own unifier because when we're done we need a real type
// TODO will need the return type (fun x<T>() -> Fun<T, T> { fun a<S>( x : S ) -> S { x } a }
// that example unifies 'a' that has effectively forall s . s -> s into T -> T
fn check_expr(expr : Expr) -> Result<Type, Vec<StaticError>> { 

    todo!()
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
