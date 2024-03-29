// HKT We check that multiple parameters can be created for a function and that the function can be
// called
// check-pass * -> * -> *

fn test<I<%J, %K>, L<%T>>(input: I<u32, u32>, input2: L<u32>, f: fn(I<u32, u32>, L<u32>, bool)) {
    f(input, input2, true)
}

fn main() {

    fn print<A>(i: u32, j: u32, k: A) {
        println!("Hej verden!: {}, {}", i, j);
    }

    test::<u32, u32>(42u32, 45u32, print);
}
