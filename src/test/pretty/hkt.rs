// HKT test
// pretty-compare-only

fn test<T, I<?J>>(int: T, input: I) {
    println!("Hej verden!");
}

//fn main() { let i = 1 + "hejsa"; }
fn main() {
    test(42, 42);
}
