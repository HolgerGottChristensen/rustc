// HKT test
// pretty-compare-only

fn test<I<?j>>(input: I<u32>) {
}

fn main() {
    test::<Option<%j2>>(Some(42));
    //           ~^ ERROR could not resolve `%j2`
}
