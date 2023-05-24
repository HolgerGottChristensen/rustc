// HKT test
// check-pass

fn test<I<%J>>(in1: I<u32>, in2: I<bool>) {}

fn main() {
    test([5u32, 6u32], [false, true]);
}

