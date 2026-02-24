
use std::rc::Rc;
use std::collections::{ HashSet, HashMap };

use crate::parsing::dne_parser::*;

type TypeMap<'a> = HashMap<Rc<str>, FunTypeInfo<'a>>;

pub enum StaticError {
    DupFunName(Rc<str>), 
    BuiltInCollision(Rc<str>),
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
    todo!()
}

fn type_check(program : &[Fun], built_ins : Vec<FunTypeInfo>) -> Vec<StaticError> {



/*
    let fun_types : HashMap<Rc<str>, TypeInfo> = HashMap::from_iter(
        built_ins.into_iter().map(|x| (Rc::clone(&x.name), x))
        .chain(program.iter().map(|x| (Rc::clone(&x.name), x.into()))));

    // TODO look for duplicate fun names or names colliding with built ins
    //
    let _ = program.iter().flat_map(|x| check_fun(x, &fun_types));
    */
    
    todo!()
}

fn check_fun(fun : &Fun, fun_types : &TypeMap) -> Vec<StaticError> {
    // TODO param types
    // TODO let types
    // TODO check calls
    todo!()
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
