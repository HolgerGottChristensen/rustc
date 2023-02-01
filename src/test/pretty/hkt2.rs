// HKT test
// pretty-compare-only

//use std::fmt::Debug;

//trait Collection1<I<J,K<L>>> {}

//trait Collection2<I: Debug> {}

//trait Collection3<I<J>: Debug> {}

//trait Collection4<I<J>> {}

fn test<I<?J>>(input: I, f: fn(I)) {
    f(input)
}

//fn main() { let i = 1 + "hejsa"; }
fn main() {

    fn print(i: u32) {
        println!("Hej verden!: {}", i);
    }

    test(42, print);
}
