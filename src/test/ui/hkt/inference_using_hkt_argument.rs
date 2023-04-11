// HKT test
// check-pass

fn test<I<%J>>(in1: I<u32>, in2: I<String>) {}

fn main() {
    test(Some(5u32), Some("io".to_string())); //should infer to be Option<%J> and not Option<u32>
}

