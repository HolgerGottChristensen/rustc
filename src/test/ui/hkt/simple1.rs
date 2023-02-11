// The parameter given to the HKT is allowed to have one known hole
// check-pass

fn test<I<?j>>(input: I<u32>) {}

fn main() {
    test::<Option<%j>>(Some(42));
}
