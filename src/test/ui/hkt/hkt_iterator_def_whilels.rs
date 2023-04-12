// HKT test that something cloneable can be called with clone, function call syntax, definition only.
// check-pass

fn test<I<%J>: Iterator<Item=u32>>(mut input: I<bool>) -> u32 {
    let mut i = 0;

    while let Some(n) = input.next() {
        i += n;
    }

    i
}

fn main() {}
