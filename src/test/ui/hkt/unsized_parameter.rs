// HKT check that the value given to the I is sized,
// check-fail

fn test<I<%J>>(_input: I<[u32]>) { } //~ ERROR the size for values of type `[u32]` cannot be known at compilation time [E0277]

fn main() {}
