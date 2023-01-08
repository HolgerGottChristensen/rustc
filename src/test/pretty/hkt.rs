// HKT test

// pretty-compare-only

use std::fmt::Debug;

trait Collection1<I<J,K<L>>> {}

trait Collection2<I: Debug> {}

trait Collection3<I<J>: Debug> {}

trait Collection4<I<J: Debug>> {}

fn main() { }
