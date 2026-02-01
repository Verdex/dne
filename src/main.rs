
mod util;
mod parsing;
mod compiling;
mod eval;

#[cfg(test)]
mod ir_tests;

use std::fs::File;

fn main() {

    let args = std::env::args().into_iter().collect::<Vec<_>>();

    if args.len() <= 1 {
        println!("usage: dne file+");
    }
    else {

        let mut ir = vec![];
        for path in args {
            use crate::parsing::ir_parser::ParseError;

            let file = File::open(path).expect("failed to open: {path}");
            let contents = std::io::read_to_string(file).expect("failure reading: {path}");
            let mut x = match parsing::ir_parser::parse(&contents) {
                Ok(x) => x,
                Err(ParseError::Lex(x)) => {
                    let x = crate::util::underline(&contents, x, x);
                    panic!("encountered lexer error:\n\n{x}");
                },
                Err(ParseError::Fatal(x, y)) => {
                    let x = crate::util::underline(&contents, x, y);
                    panic!("encountered parser error:\n\n{x}");
                },
                Err(x) => { panic!("{x}"); },
            };
            ir.append(&mut x);
        }
        
        let procs = match compiling::ir_compiler::compile(&ir) {
            Ok(x) => x,
            Err(x) => { panic!("{x}"); }, 
        };

        let main = procs.iter().enumerate().find(|(_, x)| *"main" == *x.name ).expect("cannot find main").0;
        let mut vm = eval::vm::Vm::new(procs);  

        let result = match vm.run(main) {
            Ok(x) => x,
            Err(x) => { panic!("{x}"); },
        };
         
        println!("{:?}", result);
    }
}
