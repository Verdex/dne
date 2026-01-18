
use crate::util::proj;
use crate::eval::data::RuntimeData;
use crate::eval::error::VmError;

use super::util::{ test, test_fails };


#[test]
fn should_detect_nil() {
    let input = r"
proc target() -> Int {
    break;
}
proc main() -> Bool {
    set co : Coroutine = coroutine target();
    set a : Int = resume co;
    set ret : Bool = is_nil a;
    return ret;
}
"; 

    let output = proj!(test(input).unwrap(), RuntimeData::Bool(x), x);
    assert_eq!(output, true);
}

#[test]
fn should_not_detect_nil() {
    let input = r"
proc target() -> Int {
    set x : Int = 7;
    yield x;
    break;
}
proc main() -> Bool {
    set co : Coroutine = coroutine target();
    set a : Int = resume co;
    set ret : Bool = is_nil a;
    return ret;
}
"; 

    let output = proj!(test(input).unwrap(), RuntimeData::Bool(x), x);
    assert_eq!(output, false);
}

#[test]
fn should_yield_twice() {
    let input = r"
proc target() -> Int {
    set x : Int = 7;
    set y : Int = 8;
    yield y;
    yield x;
    break;
}
proc main() -> Int {

    set co : Coroutine = coroutine target();

    set a : Int = resume co;
    set b : Int = resume co;
    set ret : Int = call add_int(a, b);

    return ret;
}
"; 

    let output = proj!(test(input).unwrap(), RuntimeData::Int(x), x);
    assert_eq!(output, 15);
}

#[test]
fn should_handle_two_coroutines_with_same_function() {
    let input = r"
proc target(y : Int) -> Int {
    set x : Int = 2;
    yield x;
    yield y;
    break;
}
proc main() -> Int {

    set i1 : Int = 3;
    set i2 : Int = 4;
    set co1 : Coroutine = coroutine target(i1);
    set co2 : Coroutine = coroutine target(i2);

    set a : Int = resume co1;
    set b : Int = resume co2;
    set c : Int = resume co2;
    set d : Int = resume co1;

    set r1 : Int = call add_int(a, b);
    set r2 : Int = call add_int(c, d);
    set r3 : Int = call mul_int(r1, r2);

    return r3;
}
"; 

    let output = proj!(test(input).unwrap(), RuntimeData::Int(x), x);
    assert_eq!(output, 28);
}

// TODO coroutine with coroutine inside of it
// coroutine as parameter
// coroutine as return value
// coroutine with inputs
// coroutine with different functions
