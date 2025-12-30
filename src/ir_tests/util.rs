
use crate::parsing::ir_parser::parse;
use crate::compiling::ir_compiler::compile;
use crate::eval::data::RuntimeData;
use crate::eval::vm::*;

pub fn test(input : &str) -> Option<RuntimeData> {
    let ir = parse(input).unwrap();
    let procs = compile(&ir).unwrap();
    let main = procs.iter().enumerate().find(|(_, x)| *"main" == *x.name ).expect("cannot find main").0;
    let mut vm = Vm::new(procs);
    vm.run(main).unwrap()
}
