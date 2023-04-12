// HKT inspect method
// check-pass

trait Test<A<%B>> {
    fn identity(self) -> A<u32>;
}

fn test<I<%J>: Test<I<%B>>>(input: I<u32>) -> I<u32> {
    input.identity()
}

fn main() {}

