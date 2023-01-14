// HKT test
// pretty-compare-only

use std::fmt::Debug;

//trait Collection1<I<J,K<L>>> {}

//trait Collection2<I: Debug> {}

//trait Collection3<I<J>: Debug> {}

//trait Collection4<I<J>> {}

fn test<I<?J>: Debug>(input: I) {
    println!("Hej verden! {:?}", input);
}

//fn main() { let i = 1 + "hejsa"; }
fn main() {
    test(42);
}
