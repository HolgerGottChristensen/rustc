// The parameter given is allowed to have 0 holes
// check-pass

fn test<I<?j>>(input: I<u32>) {}

fn main() {
    test::<u32>(42);
}
