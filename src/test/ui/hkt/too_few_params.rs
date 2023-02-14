// HKT test for too few parameters
// check-fail

fn test<I<%J, %K>>(input: I<u32>) {} //~ ERROR this hkt parameter takes 2 generic arguments but 1 generic argument was supplied [E0107]

fn main() {}
