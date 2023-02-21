// The parameter given to the HKT is allowed to have one known hole
// check-pass

fn test<I<%J>>(input: I<u32>) {}

fn main() {
    test::<Option<%J>>(Some(42));
    // test::<&%J>(&42); // Does not currently compile
}
