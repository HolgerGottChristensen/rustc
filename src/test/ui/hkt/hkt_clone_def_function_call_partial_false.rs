// HKT check that if I<bool> is Clone, that we do not also think that I<u32> is automatically Clone
// check-fail

fn test<I<%J>>(input: I<u32>) -> I<u32> where I<bool>: Clone {
    Clone::clone(&input) //~ ERROR the trait bound `I<u32>: Clone` is not satisfied
}

fn main() {}
