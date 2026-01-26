
use crate::util::proj;
use crate::eval::data::RuntimeData;

use super::util::test;

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

#[test]
fn should_return_string() {
    let input = r#"
proc main() -> String {
    set x : String = "this is a string";
    return x;
}
"#; 

    let output = proj!(test(input).unwrap(), RuntimeData::String(x), x);
    assert_eq!(output, "this is a string".into());
}

