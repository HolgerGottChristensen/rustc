// HKT test that something cloneable can be called with clone, method call syntax, definition and call.
// check-pass

use std::fmt::Debug;

fn test<I<%J: Debug>: Debug>(input: I<u32>) {
    println!("Hej verden: {:?}", input);
}

fn main() {
    test::<Option<u32>>(Some(42)); // Should pass even when %J is not required to be Debug
    test::<Option<%J>>(Some(42)); // Should only pass when %J is Debug
}
