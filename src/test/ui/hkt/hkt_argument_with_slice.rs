// HKT test
// check-pass

fn test<I<%J>>(in1: I<u32>, in2: I<bool>) {}

fn main() {
    //should infer to be Option<%J> and not Option<u32>
    test::<&[%J]>(&[5u32][..], &[true][..]);
}

