
use crate::util::proj;
use crate::eval::data::RuntimeData;

use super::util::test;

#[test]
fn should_eq_symbols() {
    let input = r"
proc main() -> Bool {
    set z : Symbol = ~blah;
    set y : Symbol = ~blah;
    set x : Bool = call eq_symbol(z, y);
    return x;
}
"; 

    let output = proj!(test(input).unwrap(), RuntimeData::Bool(x), x);
    assert_eq!(output, true);
}

#[test]
fn should_eq_bools() {
    let input = r"
proc main() -> Bool {
    set z : Bool = false;
    set y : Bool = false;
    set x : Bool = call eq_bool(z, y);
    return x;
}
"; 

    let output = proj!(test(input).unwrap(), RuntimeData::Bool(x), x);
    assert_eq!(output, true);
}

#[test]
fn should_eq_ints() {
    let input = r"
proc main() -> Bool {
    set z : Int = 5;
    set y : Int = 5;
    set x : Bool = call eq_int(z, y);
    return x;
}
"; 

    let output = proj!(test(input).unwrap(), RuntimeData::Bool(x), x);
    assert_eq!(output, true);
}

#[test]
fn should_eq_floats() {
    let input = r"
proc main() -> Bool {
    set z : Float = 5.1;
    set y : Float = 5.1;
    set x : Bool = call eq_float(z, y);
    return x;
}
"; 

    let output = proj!(test(input).unwrap(), RuntimeData::Bool(x), x);
    assert_eq!(output, true);
}

#[test]
fn should_lt_floats() {
    let input = r"
proc main() -> Bool {
    set z : Float = 4.9;
    set y : Float = 5.1;
    set x : Bool = call lt_float(z, y);
    return x;
}
"; 

    let output = proj!(test(input).unwrap(), RuntimeData::Bool(x), x);
    assert_eq!(output, true);
}

#[test]
fn should_lt_ints() {
    let input = r"
proc main() -> Bool {
    set z : Int = 9;
    set y : Int = 10;
    set x : Bool = call lt_int(z, y);
    return x;
}
"; 

    let output = proj!(test(input).unwrap(), RuntimeData::Bool(x), x);
    assert_eq!(output, true);
}

#[test]
fn should_gt_floats() {
    let input = r"
proc main() -> Bool {
    set z : Float = 5.1;
    set y : Float = 4.9;
    set x : Bool = call gt_float(z, y);
    return x;
}
"; 

    let output = proj!(test(input).unwrap(), RuntimeData::Bool(x), x);
    assert_eq!(output, true);
}

#[test]
fn should_gt_ints() {
    let input = r"
proc main() -> Bool {
    set z : Int = 10;
    set y : Int = 9;
    set x : Bool = call gt_int(z, y);
    return x;
}
"; 

    let output = proj!(test(input).unwrap(), RuntimeData::Bool(x), x);
    assert_eq!(output, true);
}

#[test]
fn should_xor() {
    let input = r"
proc main() -> Bool {
    set z : Bool = false;
    set y : Bool = true;
    set x : Bool = call xor(z, y);
    return x;
}
"; 

    let output = proj!(test(input).unwrap(), RuntimeData::Bool(x), x);
    assert_eq!(output, true);
}

#[test]
fn should_or() {
    let input = r"
proc main() -> Bool {
    set z : Bool = false;
    set y : Bool = true;
    set x : Bool = call or(z, y);
    return x;
}
"; 

    let output = proj!(test(input).unwrap(), RuntimeData::Bool(x), x);
    assert_eq!(output, true);
}

#[test]
fn should_and() {
    let input = r"
proc main() -> Bool {
    set z : Bool = true;
    set y : Bool = true;
    set x : Bool = call and(z, y);
    return x;
}
"; 

    let output = proj!(test(input).unwrap(), RuntimeData::Bool(x), x);
    assert_eq!(output, true);
}

#[test]
fn should_not() {
    let input = r"
proc main() -> Bool {
    set y : Bool = false;
    set x : Bool = call not(y);
    return x;
}
"; 

    let output = proj!(test(input).unwrap(), RuntimeData::Bool(x), x);
    assert_eq!(output, true);
}

#[test]
fn should_neg_int() {
    let input = r"
proc main() -> Int {
    set y : Int = 3;
    set x : Int = call neg_int(y);
    return x;
}
"; 

    let output = proj!(test(input).unwrap(), RuntimeData::Int(x), x);
    assert_eq!(output, -3);
}

#[test]
fn should_neg_float() {
    let input = r"
proc main() -> Float {
    set y : Float = 3.0;
    set x : Float = call neg_float(y);
    return x;
}
"; 

    let output = proj!(test(input).unwrap(), RuntimeData::Float(x), x);
    assert_eq!(output, -3.0);
}

#[test]
fn should_mod_ints() {
    let input = r"
proc main() -> Int {
    set z : Int = 10;
    set y : Int = 3;
    set x : Int = call mod_int(z, y);
    return x;
}
"; 

    let output = proj!(test(input).unwrap(), RuntimeData::Int(x), x);
    assert_eq!(output, 1);
}

#[test]
fn should_mod_floats() {
    let input = r"
proc main() -> Float {
    set z : Float = 10.0;
    set y : Float = 3.0;
    set x : Float = call mod_float(z, y);
    return x;
}
"; 

    let output = proj!(test(input).unwrap(), RuntimeData::Float(x), x);
    assert_eq!(output, 1.0);
}

#[test]
fn should_mul_ints() {
    let input = r"
proc main() -> Int {
    set z : Int = 6;
    set y : Int = 2;
    set x : Int = call mul_int(z, y);
    return x;
}
"; 

    let output = proj!(test(input).unwrap(), RuntimeData::Int(x), x);
    assert_eq!(output, 12);
}

#[test]
fn should_mul_floats() {
    let input = r"
proc main() -> Float {
    set z : Float = 3.0;
    set y : Float = 2.0;
    set x : Float = call mul_float(z, y);
    return x;
}
"; 

    let output = proj!(test(input).unwrap(), RuntimeData::Float(x), x);
    assert_eq!(output, 6.0);
}

#[test]
fn should_div_ints() {
    let input = r"
proc main() -> Int {
    set z : Int = 6;
    set y : Int = 2;
    set x : Int = call div_int(z, y);
    return x;
}
"; 

    let output = proj!(test(input).unwrap(), RuntimeData::Int(x), x);
    assert_eq!(output, 3);
}

#[test]
fn should_div_floats() {
    let input = r"
proc main() -> Float {
    set z : Float = 3.0;
    set y : Float = 2.0;
    set x : Float = call div_float(z, y);
    return x;
}
"; 

    let output = proj!(test(input).unwrap(), RuntimeData::Float(x), x);
    assert_eq!(output, 1.5);
}

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

#[test]
fn should_to_string_and_concat() {
    let input = r#"
proc cl() -> Int { 
    set x : Int = 0;
    return x;
}
proc co() -> Int {
    break;
}
proc main() -> String {
    set i : Int = 1;
    set f : Float = 2.2;
    set b : Bool = true;
    set sym : Symbol = ~sym;
    set str : String = "str";
    set co : Coroutine = coroutine co();
    set cl : Closure = closure cl();
    set n : Int = resume co;
    set r : Ref = cons sym (i, f);
    set temp : String = to_string i;
    set ret : String = to_string f;
    set ret : String = concat temp ret;
    set temp : String = to_string b;
    set ret : String = concat ret temp;
    set temp : String = to_string sym;
    set ret : String = concat ret temp;
    set temp : String = to_string str;
    set ret : String = concat ret temp;
    set temp : String = to_string co;
    set ret : String = concat ret temp;
    set temp : String = to_string cl;
    set ret : String = concat ret temp;
    set temp : String = to_string n;
    set ret : String = concat ret temp;
    set temp : String = to_string r;
    set ret : String = concat ret temp;
    return ret;
}
"#; 

    let output = proj!(test(input).unwrap(), RuntimeData::String(x), x);
    assert_eq!(output, "12.2truesymstrcoroutineclosurenilref(0)".into());
}

