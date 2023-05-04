// HKT test
// pretty-compare-only

fn test<T, I<%J>>(int: T, input: I<u32>) {
    //println!("Hej verden!");
}

fn main() {
    test::<u32, Option<%j>>(30, Some(42));
}
