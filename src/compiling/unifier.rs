

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
