
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
fn should_return_symbol() {
    let input = r"
proc main() -> Symbol {
    set x : Symbol = ~symbol;
    return x;
}
"; 

    let output = proj!(test(input).unwrap(), RuntimeData::Symbol(x), x);
    assert_eq!(*output, *"symbol");
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

#[test]
fn should_return_float() {
    let input = r"
proc main() -> Float {
    set x : Float = 18.01E-5;
    return x;
}
"; 

    let output = proj!(test(input).unwrap(), RuntimeData::Float(x), x);
    assert_eq!(output, 18.01E-5);
}
