// HKT test that something cloneable can be called with clone, method call syntax, definition and call.
// check-fail

use std::fmt::Debug;

fn test<T, I<%J: Debug>: Debug>(input: I<T>) { //~ ERROR: `T` doesn't implement `Debug`
    println!("Hej verden: {:?}", input);
}

fn main() {
    test::<u32, Option<%J>>(Some(42));
}
