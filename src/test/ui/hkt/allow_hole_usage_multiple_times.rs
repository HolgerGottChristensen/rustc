// The parameter given is allowed to use the hole multiple times
// check-pass

fn test<I<%J>>(input: I<u32>) {}

fn main() {
    test::<Result<%J, %J>>(Ok(42));
}
