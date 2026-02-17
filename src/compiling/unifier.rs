
use std::rc::Rc;
use std::collections::HashMap;

#[derive(Debug)]
pub enum Term {
    Var(Rc<str>),
    Data(Rc<str>, Vec<Term>),
}

#[derive(Debug)]
enum Vif<'a> {
    Value(&'a Term), 
    Index(usize),
    Free
}

#[derive(Debug)]
pub struct Unify<'a> {
    vars: Vec<Vif<'a>>,
    reg: HashMap<Rc<str>, usize>,
}

impl<'a> Unify<'a> {
    pub fn new() -> Self { Unify { reg: HashMap::new(), vars: vec![] } } 

    pub fn unify(&mut self, a : &'a Term, b : &'a Term) -> bool {
        match (a, b) {
            (Term::Data(a_name, _), Term::Data(b_name, _)) if a_name != b_name => false,
            (Term::Data(_, a_params), Term::Data(_, b_params)) => 
                a_params.iter().zip(b_params.iter()).map(|(a, b)| self.unify(a, b)).all(|x| x),
            (Term::Var(a), Term::Var(b)) => {
                let a = self.to_index(a);
                let b = self.to_index(b);
                let (a_index, a_value) = self.follow(a);
                let (b_index, b_value) = self.follow(b);

                match (a_value, b_value) {
                    (None, None) => { self.vars[a_index] = Vif::Index(b_index); true },
                    (Some(a), Some(b)) => self.unify(a, b),
                    (None, Some(b)) => { self.vars[a_index] = Vif::Value(b); true },
                    (Some(a), None) => { self.vars[b_index] = Vif::Value(a); true },
                }
            },
            (Term::Var(a), b) => {
                let a = self.to_index(a);
                let (a_index, a_value) = self.follow(a);

                match a_value {
                    None => { self.vars[a_index] = Vif::Value(b); true },
                    Some(a) => self.unify(a, b),
                }
            },
            (a, Term::Var(b)) => {
                let b = self.to_index(b);
                let (b_index, b_value) = self.follow(b);

                match b_value {
                    None => { self.vars[b_index] = Vif::Value(a); true },
                    Some(b) => self.unify(a, b),
                }
            },
        }
    }

    fn follow(&self, x : usize) -> (usize, Option<&'a Term>) {
        while let Vif::Index(x) = self.vars[x] { }
        match self.vars[x] {
            Vif::Value(v) => (x, Some(v)),
            Vif::Free => (x, None),
            Vif::Index(_) => unreachable!(),
        }
    }

    fn to_index(&mut self, input : &Rc<str>) -> usize {
        match self.reg.get(input) {
            Some(x) => *x,
            None => {
                let x = self.vars.len();
                self.reg.insert(Rc::clone(input), x);   
                self.vars.push(Vif::Free);
                x
            },
        }
    }
}

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn should_link_free_vars() {
        
    }
}

/*
#[derive(Debug)]
pub struct Unify {
    map : HashMap<Rc<str>, Rc<Term>>,
    gensym : usize,
}

#[derive(Debug)]
pub enum Term {
    Nil(usize),
    Var(Rc<str>),
    Rule(Rc<str>, Vec<Rc<Term>>),
}

impl Unify {
    pub fn new() -> Self { Unify { map : HashMap::new(), gensym: 0 } }

    fn free(&self, key : &Rc<str>) -> bool {
        !self.map.contains_key(key) || match self.map[key] { Term::Nil(_) => true, _ => false }
    }

    fn bound(&self, key : &Rc<str>) -> bool {  // TODO
        self.map.contains_key(key)
    }

    fn link(&mut self, a : &Rc<str>, b : &Rc<str>) {
        assert!( self.free(a) && self.free(b) ); // TODO

        if !self.map.contains_key(a) && !self.map.contains_key(b) {
            let x = self.g();
            self.map.insert(a, Term::Nil(x));
            self.map.insert(b, Term::Nil(x));
        }
        else if self.map.contains_key(a) && !self.map.contains_key(b) {
            let x = match self.map[a] {
                Term::Nil(x) => x,
                _ => panic!(),
            };
            self.map.insert(b.to_string(), Term::Nil(x));
        }
        else if !self.map.contains_key(a) && self.map.contains_key(b) {
            let x = match self.map[b] {
                Term::Nil(x) => x,
                _ => panic!(),
            };
            self.map.insert(a.to_string(), Term::Nil(x));
        }
        else {
            let a = match self.map[a] {
                Term::Nil(x) => x,
                _ => panic!(),
            };
            let b = match self.map[b] {
                Term::Nil(x) => x,
                _ => panic!(),
            };

            for (_, v) in self.map.iter_mut().filter(|(_, v)| match v { Term::Nil(x) => x == a, _ => false } ) {
                *v = Term::Nil(b);
            }
        }
    }

    fn bind(&mut self, key: &Rc<str>, input: &Rc<Term>) {
        assert!(self.free(key)); // TODO 

        if !self.map.contains_key(key) {
            self.map.insert(key, Rc::clone(input));
        }
        else {
            let a = match &*self.map[key] { Term::Nil(x) => *x, _ => panic!() };
            for (_, v) in self.map.iter_mut().filter(|(_, v)| match &***v { Term::Nil(x) => *x == a, _ => false } ) {
                *v = Rc::clone(input);
            }
        }
    }

    fn get(&self, key : &Rc<str>) -> Rc<Term> {
        Rc::clone(&self.map[key]) 
    }

    fn g(&mut self) -> usize {
        self.gensym += 1;
        self.gensym
    }

    fn unify(&mut self, a : &Rc<Term>, b : &Rc<Term>) -> bool {
        use Term::*;
        assert!( !matches!(&**a, Term::Nil(_)) && !matches!(&**b, Term::Nil(_)) ); // TODO
        match (&**a, &**b) {
            (Var(a), Var(b)) if self.free(&a) && self.free(&b) => { self.link(&a, &b); true },
            (Var(a), Var(b)) if self.free(&a) => { 
                let b = self.get(&b);
                self.bind(&a, &b);
                true 
            },
            (Var(a), Var(b)) if self.free(&b) => { 
                let a = self.get(&a);
                self.bind(&b, &a);
                true 
            },
            (Var(a), Var(b)) => { 
                let a = self.get(&a);
                let b = self.get(&b);
                self.unify(&a, &b)
            },
            (Var(a), _) if self.free(&a) => {
                self.bind(&a, &b);
                true
            },
            (_, Var(b)) if self.free(&b) => {
                self.bind(&b, &a);
                true
            },
            (Var(a), _) => {
                let a = self.get(&a);
                self.unify(&a, b)
            },
            (_, Var(b)) => {
                let b = self.get(&b);
                self.unify(a, &b)
            },
            (Rule(a_name, _), Rule(b_name, _)) if a_name != b_name => false,
            (Rule(_, a_rest), Rule(_, b_rest)) => {
                a_rest.into_iter().zip(b_rest.into_iter()).map(|(a, b)| self.unify(&a, &b)).all(|x| x)
            },
            (Nil(_), _) | (_, Nil(_)) => unreachable!(),
        }
    }
}

fn var(x : &Rc<str>) -> Term { Term::Var(Rc::clone(x)) }
fn atom(x : &Rc<str>) -> Term { Term::Rule(Rc::clone(x), vec![]) }
fn rule(x : &Rc<str>, xs : Vec<Rc<Term>>) -> Term { Term::Rule(Rc::clone(x), xs) }
*/
