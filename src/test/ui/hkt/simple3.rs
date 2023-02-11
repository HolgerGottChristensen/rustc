// The parameter given is allowed to use the hole multiple times
// check-pass

fn test<I<?j>>(input: I<u32>) {}

fn main() {
    test::<Result<%j, %j>>(Ok(42));
}
