// HKT test that something cloneable can be called with clone, method call syntax, definition and call.
// check-pass

use std::fmt::Debug;

fn test<I<%J: Debug>>(input: I<u32>) {}

fn main() {
    println!("Hello world!");
}
