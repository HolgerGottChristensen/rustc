// HKT test
// check-pass

fn test<I<%J, %K>>(in1: I<u32, bool>, in2: I<String, bool>) {}

fn main() {
    //should infer to be Option<%J>
    test(Some(5u32), Some("io".to_string()));
}
