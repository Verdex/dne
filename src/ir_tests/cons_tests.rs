
use crate::util::proj;
use crate::eval::data::RuntimeData;

use super::util::test;

#[test]
fn blarg() {
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
