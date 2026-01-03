
use crate::util::proj;
use crate::eval::data::RuntimeData;
use crate::eval::error::VmError;

use super::util::{ test, test_fails };

#[test]
fn should_remove_slot() {
    let input = r"
proc main() -> Int {

    set name : Symbol = ~blah;
    set p1 : Int = 2;
    set p2 : Float = 0.1;

    set cell : Ref = cons name (p2, p1);

    slot_remove cell 0;

    set a : Int = slot cell 0;
    set len : Int = length cell;

    set ret : Int = call add_int(len, a);

    return ret;
}
"; 

    let output = proj!(test(input).unwrap(), RuntimeData::Int(x), x);
    assert_eq!(output, 3);
}

#[test]
fn should_insert_slot() {
    let input = r"
proc main() -> Int {

    set name : Symbol = ~blah;
    set p1 : Int = 2;
    set p2 : Float = 0.1;
    set p3 : Int = 10;

    set cell : Ref = cons name (p1, p2);

    slot_insert cell p3 2;
    slot_insert cell p3 0;

    set a : Int = slot cell 0;
    set b : Int = slot cell 3;

    set ret : Int = call add_int(a, b);

    set len : Int = length cell;

    set ret : Int = call add_int(ret, len);

    return ret;
}
"; 

    let output = proj!(test(input).unwrap(), RuntimeData::Int(x), x);
    assert_eq!(output, 24);
}

#[test]
fn should_get_type() {
    let input = r"
proc main() -> Symbol {

    set name : Symbol = ~blah;
    set p1 : Int = 2;
    set p2 : Float = 0.1;

    set cell : Ref = cons name (p1, p2);

    set ret : Symbol = type cell;

    return ret;
}
"; 

    let output = proj!(test(input).unwrap(), RuntimeData::Symbol(x), x);
    assert_eq!(output, "blah".into());
}

#[test]
fn should_get_length() {
    let input = r"
proc main() -> Int {

    set name : Symbol = ~blah;
    set p1 : Int = 2;
    set p2 : Float = 0.1;

    set cell : Ref = cons name (p1, p2);

    set ret : Int = length cell;

    return ret;
}
"; 

    let output = proj!(test(input).unwrap(), RuntimeData::Int(x), x);
    assert_eq!(output, 2);
}

#[test]
fn should_get_slot() {
    let input = r"
proc main() -> Float {

    set name : Symbol = ~blah;
    set p1 : Int = 2;
    set p2 : Float = 0.1;

    set cell : Ref = cons name (p1, p2);

    set ret : Float = slot cell 1;

    delete cell;

    return ret;
}
"; 

    let output = proj!(test(input).unwrap(), RuntimeData::Float(x), x);
    assert_eq!(output, 0.1);
}

#[test]
fn should_delete() {
    let input = r"
proc main() -> Float {

    set name : Symbol = ~blah;
    set p1 : Int = 2;
    set p2 : Float = 0.1;

    set cell : Ref = cons name (p1, p2);

    delete cell;

    set ret : Float = slot cell 1;

    return ret;
}
"; 

    let output = test_fails(input);
    assert!(matches!(output, VmError::AccessNilHeap(_, _)));
}

