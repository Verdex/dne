
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
    set target : Closure = closure target();
    set co : Coroutine = dyn_coroutine target();
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
    set target : Closure = closure target();
    set co : Coroutine = dyn_coroutine target();
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

    set target : Closure = closure target();
    set co : Coroutine = dyn_coroutine target();

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
    set target : Closure = closure target();
    set co1 : Coroutine = dyn_coroutine target(i1);
    set co2 : Coroutine = dyn_coroutine target(i2);

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

#[test]
fn should_handle_coroutine_as_param() {
    let input = r"
proc target_inner() -> Int {
    set x : Int = 1;
    set y : Int = 2;
    yield x;
    yield y;
    break;
}
proc target(c : Coroutine) -> Int {
    set x : Int = 3;
    set y : Int = 5;
    yield x;
    set z : Int = resume c;
    yield z;
    yield y;
    break;
}
proc main() -> Int {
    set target_inner : Closure = closure target_inner();
    set target : Closure = closure target();
    set co1 : Coroutine = dyn_coroutine target_inner();
    set trash : Int = resume co1;
    set co2 : Coroutine = dyn_coroutine target(co1);

    set a : Int = resume co2;
    set b : Int = resume co2;
    set c : Int = resume co2;

    set ret : Int = call mul_int(a, b); 
    set ret : Int = call sub_int(ret, c);

    return ret;
}
"; 

    let output = proj!(test(input).unwrap(), RuntimeData::Int(x), x);
    assert_eq!(output, 1);
}

#[test]
fn should_handle_coroutine_as_return() {
    let input = r"
proc target(x : Int, y : Int) -> Int {
    yield x;
    yield y;
    break;
}
proc p() -> Coroutine {
    set a : Int = 10;
    set b : Int = 20;
    set target : Closure = closure target();
    set c : Coroutine = dyn_coroutine target(a, b);
    set trash : Int = resume c; 
    return c;
}
proc main() -> Int {
    set c : Coroutine = call p();
    set ret : Int = resume c;
    return ret;
}
"; 

    let output = proj!(test(input).unwrap(), RuntimeData::Int(x), x);
    assert_eq!(output, 20);
}

#[test]
fn should_handle_dyn_coroutine_with_env_closure() {
    let input = r"
proc target(x : Int, y : Int, z : Int, w : Int) -> Int {
    yield x;
    yield y;
    yield z;
    yield w;
    break;
}
proc main() -> Int {
    set a : Int = 1;
    set b : Int = 2;
    set c : Int = 3;
    set d : Int = 4;
    set target : Closure = closure target(a, b);
    set co : Coroutine = dyn_coroutine target(c, d);
    
    set r1 : Int = resume co;
    set r2 : Int = resume co;
    set r3 : Int = resume co;
    set r4 : Int = resume co;

    set ret : Int = call add_int(r1, r2);
    set ret : Int = call add_int(ret, r3);
    set ret : Int = call add_int(ret, r4);

    return ret;
}
"; 

    let output = proj!(test(input).unwrap(), RuntimeData::Int(x), x);
    assert_eq!(output, 10);
}



