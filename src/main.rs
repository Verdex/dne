
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
            let file = File::open(path).expect("failed to open: {path}");
            let contents = std::io::read_to_string(file).expect("failure reading: {path}");
            let mut x = match parsing::ir_parser::parse(&contents) {
                Ok(x) => x,
                _ => todo!(),
            };
            ir.append(&mut x);
        }
        
        let procs = compiling::ir_compiler::compile(&ir).unwrap();

        let main = procs.iter().enumerate().find(|(_, x)| *"main" == *x.name ).expect("cannot find main").0;

        let mut vm = eval::vm::Vm::new(procs);

        let result = vm.run(main);
         
        println!("{:?}", result);
    }

}
