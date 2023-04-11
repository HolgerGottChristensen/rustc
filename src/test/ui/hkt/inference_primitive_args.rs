// HKT test
// check-pass

fn test<I<%J>>(in1: I<u32>, in2: I<String>) {}

fn main() {
    test(false, true); //should infer to be Option<%J> and not Option<u32>
}

