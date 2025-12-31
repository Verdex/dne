
use crate::util::proj;
use crate::eval::data::RuntimeData;

use super::util::test;

#[test]
fn should_loop_to_100() {
    let input = r"
proc main() -> Int {
    set one : Int = 1;
    set one_hundred : Int = 100;

    set x : Int = 0;
    label loop;
    set x : Int = call add_int(x, one);
    set b : Bool = call eq_int(x, one_hundred);
    branch_true end b;
    jump loop;
    label end;
    return x;
}
"; 

    let output = proj!(test(input).unwrap(), RuntimeData::Int(x), x);
    assert_eq!(output, 100);
}

#[test]
fn should_recurse_to_100() {
    let input = r"
proc main() -> Int {
    set start : Int = 4;
    set x : Int = call to_hundred(start);
    return x;
}

proc to_hundred(input : Int) -> Int {
    set one_hundred : Int = 100;
    set one : Int = 1;

    set b : Bool = call eq_int(input, one_hundred);
    set b : Bool = call not(b);
    branch_true continue b;
    return input;

    label continue;
    set input : Int = call add_int(one, input);
    set output : Int = call to_hundred(input);
    return output;
}
"; 

    let output = proj!(test(input).unwrap(), RuntimeData::Int(x), x);
    assert_eq!(output, 100);
}

