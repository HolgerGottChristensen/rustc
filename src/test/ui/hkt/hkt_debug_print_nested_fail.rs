// HKT test that something cloneable can be called with clone, method call syntax, definition and call.
// check-fail

use std::fmt::{Debug};

struct Test;

fn test<I<%J: Debug>: Debug>(input: I<Test>) { //~ ERROR: `Test` doesn't implement `Debug`
    println!("Hej verden: {:?}", input);
}

fn main() {
    test::<Option<%J>>(Some(Test));
}
