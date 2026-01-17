
use crate::util::proj;
use crate::eval::data::RuntimeData;
use crate::eval::error::VmError;

use super::util::{ test, test_fails };

// TODO need nil predicate check


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
    yield x;
    yield y;
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
