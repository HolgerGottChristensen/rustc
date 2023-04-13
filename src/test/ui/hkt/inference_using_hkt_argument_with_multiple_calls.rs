// HKT test
// check-pass

fn test<I<%J>>(in1: I<u32>, in2: I<String>) {}

fn main() {
    //should infer to be Option<%J> and not Option<u32>
    test(Some(5u32), Some("io".to_string()));
    test(true, true);
    test(5u32, "io".to_string());
}

