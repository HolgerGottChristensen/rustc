// HKT test that something cloneable can be called with clone, function call syntax, definition only.
// check-pass

fn test<I<%J>: Clone>(input: I<u32>) -> I<u32> {
    Clone::clone(&input)
}

fn main() {}
