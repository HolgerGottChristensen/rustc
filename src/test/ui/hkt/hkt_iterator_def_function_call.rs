// HKT test that something cloneable can be called with clone, function call syntax, definition only.
// check-pass

fn test<I<%J>: Iterator<Item=u32>>(mut input: I<bool>) -> Option<u32> {
    <I<bool> as Iterator>::next(&mut input)
}

fn main() {}
