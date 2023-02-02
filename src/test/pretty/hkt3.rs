// HKT test
// pretty-compare-only

//use std::fmt::Debug;

//trait Collection1<I<J,K<L>>> {}

//trait Collection2<I: Debug> {}

//trait Collection3<I<J>: Debug> {}

//trait Collection4<I<J>> {}

fn test<I<?J, ?K>, L<?T>>(input: I, input2: L, f: fn(I, L)) {
    f(input, input2)
}

//fn main() { let i = 1 + "hejsa"; }
fn main() {

    fn print(i: u32, j: u32) {
        println!("Hej verden!: {}, {}", i, j);
    }

    test(42, 45, print);
}
