// HKT test
// pretty-compare-only

fn test<T, I<%J>, J<%K>>(int: T, input: I<u32>, input2: J<u32>) {
    println!("Hej verden!");
}

//fn main() { let i = 1 + "hejsa"; }
fn main() {
    test::<u32, Option<%J>, Option<%K>>(42, Some(42), Some(42));
}
