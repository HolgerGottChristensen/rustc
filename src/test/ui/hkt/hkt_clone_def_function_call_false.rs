// HKT check that if I is not clone, that we report a not clone error, function call syntax, definition only
// check-fail

fn test<I<%J>>(input: I<u32>) -> I<u32> {
    Clone::clone(&input) //~ ERROR the trait bound `I<u32>: Clone` is not satisfied [E0277]
}

fn main() {}
