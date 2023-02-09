// HKT test
// pretty-compare-only

fn test<T, I<?j>>(int: T, input: I<u32>) {
    //println!("Hej verden!");
}

fn main() {
    test::<u32, Option<%j>>(42, Some(42));
}
