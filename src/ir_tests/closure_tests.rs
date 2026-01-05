
use crate::util::proj;
use crate::eval::data::RuntimeData;
use crate::eval::error::VmError;

use super::util::{ test, test_fails };

#[test]
fn should_closure_and_dyn_call() {
    let input = r"
proc target() -> Int {
    set x : Int = 7;
    return x;
}
proc main() -> Int {

    set f : Closure = closure target();

    set ret : Int = dyn_call f();

    return ret;
}
"; 

    let output = proj!(test(input).unwrap(), RuntimeData::Int(x), x);
    assert_eq!(output, 7);
}

#[test]
fn should_closure_and_dyn_call_with_env() {
    let input = r"
proc target(x : Int) -> Int {
    return x;
}
proc main() -> Int {

    set e1 : Int = 7;

    set f : Closure = closure target(e1);

    set ret : Int = dyn_call f();

    return ret;
}
"; 

    let output = proj!(test(input).unwrap(), RuntimeData::Int(x), x);
    assert_eq!(output, 7);
}

#[test]
fn should_closure_and_dyn_call_with_params() {
    let input = r"
proc target(x : Int) -> Int {
    return x;
}
proc main() -> Int {

    set p1 : Int = 7;

    set f : Closure = closure target();

    set ret : Int = dyn_call f(p1);

    return ret;
}
"; 

    let output = proj!(test(input).unwrap(), RuntimeData::Int(x), x);
    assert_eq!(output, 7);
}

#[test]
fn should_closure_and_dyn_call_with_env_and_params() {
    let input = r"
proc target(x1 : Int, x2 : Int, y1 : Int, y2 : Int) -> Int {
    set z : Int = 7;
    set ret : Int = call add_int(x1, y1);
    set ret : Int = call add_int(ret, x2);
    set ret : Int = call add_int(ret, y2);
    set ret : Int = call add_int(ret, z);
    return ret;
}
proc main() -> Int {

    set e1 : Int = 1;
    set e2 : Int = 2;
    set p1 : Int = 3;
    set p2 : Int = 4;

    set f : Closure = closure target(e1, e2);

    set ret : Int = dyn_call f(p1, p2);

    return ret;
}
"; 

    let output = proj!(test(input).unwrap(), RuntimeData::Int(x), x);
    assert_eq!(output, 17);
}

#[test]
fn should_return_closure() {
    let input = r"
proc target(x : Int, y : Int) -> Int {
    set ret : Int = call add_int(x, y);
    return ret;
}
proc returner() -> Closure {
    set x : Int = 1;
    set r : Closure = closure target(x);
    return r;
}
proc main() -> Int {

    set f : Closure = call returner();
    set x : Int = 2;

    set ret : Int = dyn_call f(x);

    return ret;
}
"; 

    let output = proj!(test(input).unwrap(), RuntimeData::Int(x), x);
    assert_eq!(output, 3);
}

#[test]
fn should_accept_closure() {
    let input = r"
proc target(x : Int, y : Int) -> Int {
    set ret : Int = call add_int(x, y);
    return ret;
}
proc caller(f : Closure) -> Int {
    set x : Int = 2;
    set r : Int = dyn_call f(x);
    return r;
}
proc main() -> Int {

    set x : Int = 1;
    set f : Closure = closure target(x);

    set ret : Int = call caller(f);

    return ret;
}
"; 

    let output = proj!(test(input).unwrap(), RuntimeData::Int(x), x);
    assert_eq!(output, 3);
}
