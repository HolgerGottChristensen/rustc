// HKT test
// check-fail

fn test<I<%J>, L<%K>>(input: I<L<u32>>) {
}

fn main() {
    test(Some(Some(5u32))); //~ERROR: too many possible types to infer HKT parameters
    //~^ ERROR: too many possible types to infer HKT parameters
}
