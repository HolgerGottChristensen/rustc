// HKT test
// check-pass

fn test<I<%J>>(in1: I<u32>, in2: I<String>) {}

fn main() {
    //test::<Option<%J>>(Some(8u32), Some("Solo".to_string())); //should infer to be Option<%J> and not Option<u32>
    test(Some(8u32), Some("Solo".to_string())); //should infer to be Option<%J> and not Option<u32>
}

