// HKT test
// check-pass

fn test<I<%J>, P<%K>>(in1: I<u32>, in2: P<String>) {}

fn main() {
    //should infer to be Option<%J> and not Option<u32>
    test((5u32, true), ("io".to_string(), 5u32, true));
}

