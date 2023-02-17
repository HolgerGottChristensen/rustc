// HKT check that the return type is Sized
// check-fail

fn test<I<%J>>() -> I<[u32]> { todo!() } //~ ERROR the size for values of type `[u32]` cannot be known at compilation time [E0277]

fn main() {}
