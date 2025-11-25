
use std::iter::Peekable;

pub struct Input<T, E> {
    ls : Peekable<std::vec::IntoIter<T>>,
    eof : E, 
    fatal : E,
}

impl<T, E:Clone> Input<T, E> {
    pub fn new(input : Vec<T>, eof : E, fatal : E) -> Self {
        Input { 
            ls: input.into_iter().peekable(),
            eof,
            fatal,
        }
    }
    pub fn check<F:Fn(&T) -> bool>(&mut self, f : F) -> Result<bool, E> {
        match self.ls.peek() {
            Some(l) if f(l) => {
                self.ls.next().unwrap();
                Ok(true)
            },
            Some(_) => Ok(false),
            None => Err(self.eof.clone()),
        }
    }
    pub fn expect<F:Fn(&T) -> bool>(&mut self, f : F) -> Result<T, E> {
        match self.ls.peek() {
            Some(l) if f(l) => {
                let l = self.ls.next().unwrap();
                Ok(l)
            },
            Some(_) => Err(self.fatal.clone()),
            None => Err(self.eof.clone()),
        }
    }
    pub fn peek(&mut self) -> Result<&T, E> {
        match self.ls.peek() {
            Some(l) => Ok(l),
            None => Err(self.eof.clone()),
        }
    }
    pub fn take(&mut self) -> Result<T, E> {
        match self.ls.next() {
            Some(l) => Ok(l),
            None => Err(self.eof.clone()),
        }
    }
}
