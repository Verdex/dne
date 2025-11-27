
use std::iter::Peekable;

pub struct Input<T, E> {
    ls : Peekable<std::vec::IntoIter<(T, usize, usize)>>,
    eof : E, 
    fatal : fn(usize, usize) -> E,
}

impl<T, E:Clone> Input<T, E> {
    pub fn new(input : Vec<(T, usize, usize)>, eof : E, fatal : fn(usize, usize) -> E) -> Self {
        Input { 
            ls: input.into_iter().peekable(),
            eof,
            fatal,
        }
    }
    pub fn current(&mut self) -> Result<(usize, usize), E> {
        match self.ls.peek() {
            Some((_, s, e)) => Ok((*s, *e)),
            None => Err(self.eof.clone()),
        }
    }
    pub fn check<F:Fn(&T) -> bool>(&mut self, f : F) -> Result<bool, E> {
        match self.ls.peek() {
            Some((l, _, _)) if f(l) => {
                self.ls.next().unwrap();
                Ok(true)
            },
            Some(_) => Ok(false),
            None => Err(self.eof.clone()),
        }
    }
    pub fn expect<F:Fn(&T) -> bool>(&mut self, f : F) -> Result<T, E> {
        match self.ls.peek() {
            Some((l, _, _)) if f(l) => {
                let (l, _, _) = self.ls.next().unwrap();
                Ok(l)
            },
            Some((_, s, e)) => Err((self.fatal)(*s, *e)),
            None => Err(self.eof.clone()),
        }
    }
    pub fn peek(&mut self) -> Result<&T, E> {
        match self.ls.peek() {
            Some((l, _, _)) => Ok(l),
            None => Err(self.eof.clone()),
        }
    }
    pub fn take(&mut self) -> Result<T, E> {
        match self.ls.next() {
            Some((l, _, _)) => Ok(l),
            None => Err(self.eof.clone()),
        }
    }
    pub fn empty(&mut self) -> bool {
        match self.ls.peek() {
            Some(_) => false,
            None => true,
        }
    }
}
