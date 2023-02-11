
// HKT test
// pretty-compare-only

fn test<I<?j>>(input: I<u32>) {}

fn main() {
    test::<Option<%j>>(Some(42));
}
