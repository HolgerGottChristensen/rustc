// HKT test for generics inside hkt is still sized
// check-pass

fn test<A, I<%J>>(a: A, input: I<A>) {}

fn here<E>(e: E) {
    test::<u32, Result<%J, E>>(42, Err(e));
}

fn main() {

}
