
use crate::util::proj;
use crate::eval::data::RuntimeData;

use super::util::test;

#[test]
fn blarg() {
    let input = r"
proc main() -> Bool {

    set name : Symbol = ~blah;
    set p1 : Int = 0;
    set p2 : Float = 0.1;

    set w : Ref = cons name (p1, p2);

    delete w;

    set ret : Bool = true;
    return ret;
}
"; 

    let output = proj!(test(input).unwrap(), RuntimeData::Bool(x), x);
    assert_eq!(output, true);
}
