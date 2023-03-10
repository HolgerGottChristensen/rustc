// HKT test that something cloneable can be called with clone, method call syntax, definition and call.
// check-fail

use std::fmt::{Debug};

fn test<I<%J>: Debug>(input: I<u32>) {
    println!("Hej verden: {:?}", input);
}

fn main() {
    test::<Option<%J>>(Some(42)); //~ ERROR `%J` doesn't implement `Debug`
}
