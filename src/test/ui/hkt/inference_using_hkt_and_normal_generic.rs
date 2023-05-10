// HKT test
// check-pass

fn test<I<%J>>(in1: I<u32>) {}

fn test2<A>(in1: A,) {
    test(in1);
}

fn main() {

}

