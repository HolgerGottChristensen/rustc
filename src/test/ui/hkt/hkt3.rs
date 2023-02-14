// HKT test
// pretty-compare-only

fn test<I<%J, %K>, L<%T>>(input: I, input2: L, f: fn(I, L)) {
    f(input, input2)
}

fn main() {

    fn print(i: u32, j: u32) {
        println!("Hej verden!: {}, {}", i, j);
    }

    test(42, 45, print);
}
