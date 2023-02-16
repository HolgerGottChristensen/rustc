// HKT check that the value given to the I is sized,
// check-fail

fn test<I<%J>>(input: I<[u32]>, input2: Option<[u32]>) { } //~  ERROR `[u32]` is not Sized

fn main() {}
