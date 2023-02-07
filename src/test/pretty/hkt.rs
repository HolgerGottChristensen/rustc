// HKT test
// pretty-compare-only

fn test<T, I<?j>>(int: T, input: I<u32>) {
    //println!("Hej verden!");
}

fn main() {
    test::<Option<String>, Option<%j>>(Some(String::new()), Some(42));
}
