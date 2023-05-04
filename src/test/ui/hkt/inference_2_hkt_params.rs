// HKT test
// check-pass

fn test<I<%J>, P<%K>>(in1: I<u32>, in2: P<String>) {}

fn main() {
    //should infer to be Option<%J> and Option<%K>
    test(Some(5u32), Some("io".to_string()));
}

