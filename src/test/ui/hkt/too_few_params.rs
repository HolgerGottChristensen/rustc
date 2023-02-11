// HKT test
// check-fail

fn test<I<?j, ?k>>(input: I<u32>) {} //~ ERROR this hkt parameter takes 2 generic arguments but 1 generic argument was supplied [E0107]

fn main() {}
