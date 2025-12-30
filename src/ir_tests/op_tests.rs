
use crate::util::proj;
use crate::eval::data::RuntimeData;

use super::util::test;

#[test]
fn should_sub_ints() {
    let input = r"
proc main() -> Int {
    set z : Int = 3;
    set y : Int = 2;
    set x : Int = call sub_int(z, y);
    return x;
}
"; 

    let output = proj!(test(input).unwrap(), RuntimeData::Int(x), x);
    assert_eq!(output, 1);
}

#[test]
fn should_sub_floats() {
    let input = r"
proc main() -> Float {
    set z : Float = 3.0;
    set y : Float = 2.0;
    set x : Float = call sub_float(z, y);
    return x;
}
"; 

    let output = proj!(test(input).unwrap(), RuntimeData::Float(x), x);
    assert_eq!(output, 1.0);
}

#[test]
fn should_add_ints() {
    let input = r"
proc main() -> Int {
    set z : Int = 3;
    set y : Int = 2;
    set x : Int = call add_int(z, y);
    return x;
}
"; 

    let output = proj!(test(input).unwrap(), RuntimeData::Int(x), x);
    assert_eq!(output, 5);
}

#[test]
fn should_add_floats() {
    let input = r"
proc main() -> Float {
    set z : Float = 3.0;
    set y : Float = 2.0;
    set x : Float = call add_float(z, y);
    return x;
}
"; 

    let output = proj!(test(input).unwrap(), RuntimeData::Float(x), x);
    assert_eq!(output, 5.0);
}

