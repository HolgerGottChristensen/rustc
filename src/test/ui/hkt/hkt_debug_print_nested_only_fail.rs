// HKT test that the nested debug bound is checked when using I<___> within the function body
// check-fail

use std::fmt::Debug;

struct Test;

fn test<I<%J: Debug>>(input: I<Test>) {} //~ ERROR: `Test` doesn't implement `Debug` [E0277]

fn main() {
    println!("Hello world!");
}
