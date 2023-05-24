// HKT test
// check-pass

fn test<I<%J>, P<%K>>(in1: I<u32>, in2: P<bool>) {}

fn main() {
    test([5u32, 6u32], [false, true, false]);
}

