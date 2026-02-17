
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

    pub fn env(&self) -> Vec<(Rc<str>, &'a Term)> {
        self.reg.iter().filter_map(|(k, v)| {
            match &self.vars[*v] {
                Vif::Value(x) => Some((Rc::clone(k), *x)),
                Vif::Index(x) => {
                    match self.follow(*x).1 {
                        None => None,
                        Some(x) => Some((Rc::clone(k), x)),
                    }
                },
                Vif::Free => None,
            }
        }).collect()
    }

// TODO occurs
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

    fn follow(&self, mut x : usize) -> (usize, Option<&'a Term>) {
        while let Vif::Index(y) = self.vars[x] { x = y; }
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

    fn var(x : &str) -> Term { Term::Var(x.into()) }
    fn atom(x : &str) -> Term { Term::Data(x.into(), vec![]) }
    fn rule(x : &str, xs : Vec<Term>) -> Term { Term::Data(x.into(), xs) }

    #[test]
    fn should_link_free_vars() {
        let a = var("a"); 
        let b = var("b"); 
        let mut u = Unify::new();

        let r = u.unify(&a, &b);
        assert!(r);

        let c = atom("c");
        let r = u.unify(&a, &c);
        assert!(r);

        let r = u.env();
        assert_eq!( r.len(), 2 );
    }

    #[test]
    fn should_link_linked_free_vars() {
        let a = var("a"); 
        let b = var("b"); 
        let c = var("c"); 
        let d = var("d"); 
        let mut u = Unify::new();

        let r = u.unify(&a, &b);
        assert!(r);

        let r = u.unify(&c, &d);
        assert!(r);

        let r = u.unify(&b, &d);
        assert!(r);

        let x = atom("x");

        let r = u.unify(&d, &x);
        assert!(r);

        let r = u.env();
        assert_eq!( r.len(), 4 );
    }

    #[test]
    fn should_unify_rules() {
        let a = var("a"); 
        let b = var("b"); 
        let c = var("c"); 
        let d = atom("1"); 
        let f = atom("2"); 
        let e = rule("e", vec![b, f]);
        let g = rule("e", vec![d, c]);
        let mut u = Unify::new();

        let r = u.unify(&a, &e);
        assert!(r);

        let r = u.unify(&a, &g);
        assert!(r);

        let r = u.env();
        assert_eq!( r.len(), 3 );
    }
}

