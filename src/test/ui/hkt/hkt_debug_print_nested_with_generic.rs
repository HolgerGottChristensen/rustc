// HKT test that something cloneable can be called with clone, method call syntax, definition and call.
// check-pass

use std::fmt::Debug;

fn test<T: Debug, I<%J: Debug>: Debug>(input: I<T>) {
    println!("Hej verden: {:?}", input);
}

fn main() {
    test::<u32, Option<%J>>(Some(42));
}
