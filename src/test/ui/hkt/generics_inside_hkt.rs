// HKT test for generics inside hkt is still sized
// check-pass

fn test<A, I<%J>>(a: A, input: I<A>) {}

fn main() {
    test::<u32, Option<%J>>(42, Some(42));
}
