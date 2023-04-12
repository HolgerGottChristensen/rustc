// HKT inspect method
// check-pass

trait Test<A<%B>> {
    fn identity(i: A<u32>) -> A<u32>;
}

fn test<I<%J>: Test<I<%B>>>(input: I<u32>) -> I<u32> {
    <I::<u32> as self::Test<I<%B>>>::identity(input)
}

fn main() {}

