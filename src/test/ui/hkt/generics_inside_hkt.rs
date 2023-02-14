// HKT test for generics inside hkt is still sized
// check-pass

fn test<A, I<%J>>(input: I<A>) {}

fn main() {
    test::<u32, Option<%J>>(Some(42));
}
