// HKT The J inside I and the J with K inside are independent values and should not be resolved to
// the same things.
// check-pass

fn test<T, I<%J>, J<%K>>(int: T, input: I<u32>, input2: J<u32>) {
    println!("Hej verden!");
}

fn main() {
    test::<u32, Option<%J>, Option<%K>>(42, Some(42), Some(42));
}
