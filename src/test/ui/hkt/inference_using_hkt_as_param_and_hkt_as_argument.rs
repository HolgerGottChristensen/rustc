// HKT test
// check-pass

fn test<I<%J>>(in1: I<u32>) {}

fn test2<L<%K>>(in1: L<u32>) {
    test(in1);
}

fn main() {

}

