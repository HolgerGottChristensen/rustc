// HKT test
// check-pass

fn test<I<%J>>(in1: I<u32>) {}

fn main() {
    //test::<Option<%J>>(Some(8u32), Some("Solo".to_string())); //should infer to be Option<%J> and not Option<u32>
    test(true); //should infer to be Option<%J> and not Option<u32>
}

