
use crate::util::proj;
use crate::parsing::ir_parser::parse;
use crate::compiling::ir_compiler::compile;
use crate::eval::data::*;
use crate::eval::vm::*;

fn test(input : &str) -> Option<RuntimeData> {
    let ir = parse("").unwrap();
    let procs = compile(&ir).unwrap();
    let main = procs.iter().enumerate().find(|(_, x)| *"main" == *x.name ).expect("cannot find main").0;
    let mut vm = Vm::new(procs);
    vm.run(main).unwrap()
}

#[test]
fn blarg() {
    let input = r"
proc main() {
    return 0;
}
"; 
    let output = test(input).unwrap();

    let w = proj!(output, RuntimeData::Int(x), x);

    assert_eq!(w, 0);
}
