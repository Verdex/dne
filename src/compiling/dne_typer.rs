
use std::rc::Rc;
use std::collections::{ HashSet, HashMap };

use crate::parsing::dne_parser::*;

type TypeMap<'a> = HashMap<Rc<str>, TypeInfo<'a>>;

pub enum StaticError {
    DupFunName(Rc<str>), 
}

pub struct TypeInfo<'a> {
    name : Rc<str>,
    return_type : &'a Type,
    param_types : Vec<&'a Type>,
    type_params : HashSet<Rc<str>>,
}

impl<'a> From<&'a Fun> for TypeInfo<'a> {
    fn from(item: &'a Fun) -> Self {
        TypeInfo { 
            name: Rc::clone(&item.name),
            param_types: item.params.iter().map(|(_, t)| t).collect(),
            return_type: &item.return_type,
            type_params: HashSet::from_iter(item.type_params.iter().map(|x| Rc::clone(x)))
        }
    }
}

pub fn type_check(program : &[Fun], built_ins : Vec<TypeInfo>) -> Vec<StaticError> {
    let fun_types : HashMap<Rc<str>, TypeInfo> = HashMap::from_iter(
        built_ins.into_iter().map(|x| (Rc::clone(&x.name), x))
        .chain(program.iter().map(|x| (Rc::clone(&x.name), x.into()))));

    // TODO look for duplicate fun names or names colliding with built ins
    //
    let _ = program.iter().flat_map(|x| check_fun(x, &fun_types));
    
    todo!()
}

fn check_fun(fun : &Fun, fun_types : &TypeMap) -> Vec<StaticError> {
    // TODO param types
    // TODO let types
    // TODO check calls
    todo!()
}
