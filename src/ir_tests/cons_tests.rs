
use crate::util::proj;
use crate::eval::data::RuntimeData;

use super::util::test;

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
fn blarg() {
    let input = r"
proc main() -> Int {

    set name : Symbol = ~blah;
    set p1 : Int = 2;
    set p2 : Float = 0.1;

    set w : Ref = cons name (p1, p2);

    set ret : Int = slot w 0;

    delete w;

    return ret;
}
"; 

    let output = proj!(test(input).unwrap(), RuntimeData::Int(x), x);
    assert_eq!(output, 0);
}
