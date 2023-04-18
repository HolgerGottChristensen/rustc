// HKT We check that multiple parameters can be created for a function and that the function can be
// called
// check-pass * -> * -> *

fn test<I<%J>>(input: I<u32>, input2: I<bool>, f: fn(I<u32>, I<bool>)) {
    f(input, input2)
}

fn main() {

    fn print(i: u32, j: bool) {
        println!("Hej verden!: {}, {}", i, j);
    }

    test(42u32, true, |i: u32, j: bool| print(i, j));
}
