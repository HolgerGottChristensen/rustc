// The %j2 should not be able to be found, because it is not a parameter of the HKT
// check-fail

fn test<I<%J>>(input: I<u32>) {
}

fn main() {
    test::<Option<%J2>>(Some(42)); //~ ERROR could not resolve `%j2`
}
