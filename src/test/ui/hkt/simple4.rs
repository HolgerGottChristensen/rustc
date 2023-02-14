// The HKT is allowed to be used in a return position
// check-pass

fn test<I<%J>>(input: I<u32>) -> I<u32> {}

fn main() {
    test::<Option<%J>>(Some(42));
}
