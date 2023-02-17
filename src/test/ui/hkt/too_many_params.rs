// HKT test for too many parameters
// check-fail

fn test<I<%J>>(input: I<u32, u32>) {} //~ ERROR this hkt parameter takes 1 generic argument but 2 generic arguments were supplied [E0107]

fn main() {
    test::<Option<%J>>(Some(42));
}
