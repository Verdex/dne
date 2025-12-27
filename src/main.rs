
mod util;
mod parsing;
mod compiling;
mod eval;

#[cfg(test)]
mod ir_tests;

fn main() {

    let ir = parsing::ir_parser::parse("").unwrap();

    let procs = compiling::ir_compiler::compile(&ir).unwrap();

    let main = procs.iter().enumerate().find(|(_, x)| *"main" == *x.name ).expect("cannot find main").0;

    let mut vm = eval::vm::Vm::new(procs);

    let result = vm.run(main);
     
    println!("{:?}", result);
}
