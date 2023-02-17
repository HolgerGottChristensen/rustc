// HKT Test that we can call the same function with multiple different types, and they all print
// check-pass

use std::fmt::Debug;

fn test<T, I<%J>>(int: T, input: I<u32>, f: fn(I<u32>)) {
    f(input)
}

fn print_t<T: Debug>(o: T) {
    println!("Hejsa verden!: {:?}", o)
}

fn main() {
    test::<u32, Option<%J>>(42, Some(42), print_t);
    test::<u32, Result<%J, String>>(42, Ok(42), print_t);
    test::<u32, Result<String, %J>>(42, Err(42), print_t);
    test::<u32, String>(42, "Oh".to_string(), print_t);
}
