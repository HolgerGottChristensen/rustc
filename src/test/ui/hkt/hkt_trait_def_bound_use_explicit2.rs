// HKT inspect method
// check-pass

trait Test<A<%B>> {
    fn identity(i: A<bool>) -> A<u32>;
}

fn test<I<%J>: Test<I<%B>>>(input: I<bool>) -> I<u32> {
    <I::<u32> as self::Test<I<%B>>>::identity(input)
}

fn main() {}

