// HKT test that something cloneable can be called with clone, function call syntax, definition only.
// check-pass

fn test<I<%J>: Iterator<Item=u32>>(input: I<bool>) -> u32 {
    let mut i = 0;

    for n in input {
        i += n;
    }

    i
}

fn main() {}
