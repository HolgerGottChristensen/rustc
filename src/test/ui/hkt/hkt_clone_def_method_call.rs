// HKT test
// check-pass

fn test<I<%J>: Clone>(input: I<u32>) { input.clone(); }
//fn test<A: Clone>(input: A) {}// input.clone(); }
//fn test<I<%J>>(input: I<u32>) {}

/*use std::marker::PhantomData;

struct I<J> {
    // This is only needed for Rust to not complain over J not being used
    i: PhantomData<J>,
}

impl<J: Clone> Clone for I<J> {
    fn clone(&self) -> Self {
        todo!()
    }
}

// Function 2
fn test(i: I<u32>) -> I<u32> {
    i.clone()
}*/

fn main() {}
