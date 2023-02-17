// The %j2 should not be able to be found, because it is not a parameter of the HKT
// check-fail

fn test<I<%J>>(input: I<u32>) {
}

fn main() {
    test::<Option<%J2>>(Some(42)); //~ ERROR hkt argument `%J2` could not be found in the definition [E9999]
}
