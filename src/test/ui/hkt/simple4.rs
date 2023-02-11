// The HKT is allowed to be used in a return position
// check-pass

fn test<I<?j>>(input: I<u32>) -> I<u32> {}

fn main() {
    test::<Option<%j>>(Some(42));
}
