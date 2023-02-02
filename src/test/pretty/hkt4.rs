// HKT test
// pretty-compare-only

fn test<T, I<?J>>(input: I, input2: T, f: fn(&I, &T)) -> (I, T) {
f(&input, &input2);
(input, input2)
}

//fn main() { let i = 1 + "hejsa"; }
fn main() {

    fn print(i: &u32, j: &u32) {
        println!("Hej verden!: {}, {}", i, j);
    }

    test(42, 45, print);
}
