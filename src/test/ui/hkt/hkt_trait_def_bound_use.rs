// HKT inspect method
// check-pass

trait Test<A<%B>> {
    fn identity(i: A<u32>) -> A<u32>;
}

// We know that I<%J> (I with every %J) should implement the trait Test<I<%B>>.
// This means that if we call `identity` on I<u32>, we get that I<u32>: Test<I<u32>> since
// the identity function specifies that %B has to be u32.
// It also means that if we call identity on I<bool>, we get that I<u32>: Test<I<u32>> since
// the identity function still specify that %B has to be u32.

fn test<I<%J>: Test<I<%B>>>(input: I<u32>) -> I<u32> {
    I::<u32>::identity(input)
}

fn main() {}

