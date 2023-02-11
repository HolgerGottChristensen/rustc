// HKT test
// check-fail

fn test<I<?j, ?k>>(input: I<u32>) {} //~ ERROR usage of `I` requires 2 parameters, but 1 was given

fn main() {}
