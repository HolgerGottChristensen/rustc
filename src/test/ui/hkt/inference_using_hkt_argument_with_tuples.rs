// HKT test
// check-pass

fn test<I<%J>>(in1: I<u32>, in2: I<String>) {}

fn main() {
    test((5u32, true), ("io".to_string(), true));
}

