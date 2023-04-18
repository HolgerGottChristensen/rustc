// HKT We check that multiple parameters can be created for a function and that the function can be
// called
// check-pass * -> * -> *

fn test<I<%J>, L<%K>>(input: I<L<u32>>) {
}

fn main() {

    test::<Option<%J>, Option<%K>>(Some(Some(5u32)));
}
