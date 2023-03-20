// HKT check that if I is not clone, that we report a not clone error, function call syntax, definition only
// check-pass

fn test<I<%J>>(input: I<u32>) -> I<u32> where I<u32>: Clone {
    Clone::clone(&input)
}

fn main() {}
