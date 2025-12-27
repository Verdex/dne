
use crate::util::proj;
use crate::parsing::ir_parser::parse;
use crate::compiling::ir_compiler::compile;
use crate::eval::data::*;
use crate::eval::vm::*;

fn test(input : &str) -> Option<RuntimeData> {
    let ir = parse(input).unwrap();
    let procs = compile(&ir).unwrap();
    let main = procs.iter().enumerate().find(|(_, x)| *"main" == *x.name ).expect("cannot find main").0;
    let mut vm = Vm::new(procs);
    vm.run(main).unwrap()
}

#[test]
fn should_return_int() {
    let input = r"
proc main() -> Int {
    set x : Int = 1;
    return x;
}
"; 

    let output = proj!(test(input).unwrap(), RuntimeData::Int(x), x);
    assert_eq!(output, 1);
}

#[test]
fn should_return_bool() {
    let input = r"
proc main() -> Bool {
    set x : Bool = true;
    return x;
}
"; 

    let output = proj!(test(input).unwrap(), RuntimeData::Bool(x), x);
    assert_eq!(output, true);
}
