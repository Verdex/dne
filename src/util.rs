
use std::rc::Rc;
use std::collections::HashMap;

#[macro_export]
macro_rules! proj {
    ($input:expr, $p:pat, $output:expr) => {
        match $input {
            $p => $output,
            _ => panic!("proj failed"),
        }
    }
}

pub use proj;

pub fn underline(input : &str, start : usize, end : usize) -> String {
    let dashes = if start == end {
        "^".to_string()
    }
    else {
        "-".repeat(1 + end - start)
    };
    let mut prev = "";
    let mut c = 0;
    let mut line = 1;
    for x in input.lines(){
        let l = x.len();
        if c <= start && start <= c + l + 1 {
            let spaces = " ".repeat(start - c);
            return format!("error at {line}\n{prev}\n{x}\n{spaces}{dashes}");
        }
        prev = x;
        c += l + 1;
        line += 1; 
    }

    panic!("underline {start} and {end} are outside of the input string with length: {}", input.len());
}

#[derive(Debug)]
pub struct Unify {
    map : HashMap<Rc<str>, &Term>,
    gensym : usize,
}

#[derive(Debug, Clone)]
pub enum Term {
    Nil(usize),
    Var(Rc<str>),
    Rule(Rc<str>, Vec<Term>),
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
            self.map.insert(a.to_string(), Term::Nil(x));
            self.map.insert(b.to_string(), Term::Nil(x));
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

    fn bind(&mut self, key: &Rc<str>, input: &Term) {
        assert!(self.free(key)); // TODO 

        if !self.map.contains_key(key) {
            self.map.insert(key.to_string(), input);
        }
        else {
            let a = match self.map[key] { Term::Nil(x) => x, _ => panic!() };
            for (_, v) in self.map.iter_mut().filter(|(_, v)| match v { Term::Nil(x) => x == a, _ => false } ) {
                *v = input.clone();
            }
        }
    }

    fn get(&self, key : &Rc<str>) -> &Term {
        &self.map[key] // TODO
    }

    fn g(&mut self) -> usize {
        self.gensym += 1;
        self.gensym
    }

    fn unify(&mut self, a : &Term, b : &Term) -> bool {
        use Term::*;
        assert!( !matches!(a, Term::Nil(_)) && !matches!(b, Term::Nil(_)) ); // TODO
        match (a, b) {
            (Var(a), Var(b)) if self.free(&a) && self.free(&b) => { self.link(&a, &b); true },
            (Var(a), Var(b)) if self.free(&a) => { 
                let b = self.get(&b);
                self.bind(&a, b);
                true 
            },
            (Var(a), Var(b)) if self.free(&b) => { 
                let a = self.get(&a);
                self.bind(&b, a);
                true 
            },
            (Var(a), Var(b)) => { 
                let a = self.get(&a);
                let b = self.get(&b);
                self.unify(a, b)
            },
            (Var(a), b) if self.free(&a) => {
                self.bind(&a, b);
                true
            },
            (a, Var(b)) if self.free(&b) => {
                self.bind(&b, a);
                true
            },
            (Var(a), b) => {
                let a = self.get(&a);
                self.unify(a, b)
            },
            (a, Var(b)) => {
                let b = self.get(&b);
                self.unify(a, b)
            },
            (Rule(a_name, _), Rule(b_name, _)) if a_name != b_name => false,
            (Rule(_, a_rest), Rule(_, b_rest)) => {
                a_rest.into_iter().zip(b_rest.into_iter()).map(|(a, b)| self.unify(a, b)).all(|x| x)
            },
            (Nil(_), _) | (_, Nil(_)) => unreachable!(),
        }
    }
}

fn var(x : &Rc<str>) -> Term { Term::Var(Rc::clone(x)) }
fn atom(x : &Rc<str>) -> Term { Term::Rule(Rc::clone(x), vec![]) }
fn rule(x : &Rc<str>, xs : Vec<Term>) -> Term { Term::Rule(Rc::clone(x), xs) }

/*
fn main() {

    let mut env = Env::new();

    u(var("a"), var("b"), &mut env);
    u(var("a"), var("e"), &mut env);
    u(var("c"), var("d"), &mut env);
    u(var("b"), atom("blarg"), &mut env);
    u(var("e"), var("d"), &mut env);

    u(var("a0"), rule("x", vec![var("b"), atom("1"), var("c")]), &mut env);
    u(var("a1"), rule("x", vec![atom("7"), var("d"), var("w")]), &mut env);
    u(var("a2"), rule("x", vec![var("j"), var("d"), atom("11")]), &mut env);
    
    u(var("a0"), var("a1"), &mut env);
    u(var("a2"), var("a1"), &mut env);


    println!("{:?}", env);

}
    */


#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn should_underline_first_line() {
        let input = "one two three four\nfive six seven";
        let output = underline(input, 8, 12);
        assert_eq!(output, "error at 1\n\none two three four\n        -----");
    }

    #[test]
    fn should_underline_last_line() {
        let input = "one two three four\nfive six seven\neight nine ten";
        let output = underline(input, 40, 43);
        assert_eq!(output, "error at 3\nfive six seven\neight nine ten\n      ----");
    }

    #[test]
    fn should_underline_middle_line() {
        let input = "one two three four\nfive six seven\neight nine ten";
        let output = underline(input, 24, 26);
        assert_eq!(output, "error at 2\none two three four\nfive six seven\n     ---");
    }
}
