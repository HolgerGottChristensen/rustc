// HKT test
// check-fail

//fn test<I<%J, %J>>() {}

fn test<J, J>() {} //~ ERROR: the name `J` is already used for a generic parameter in this item's generic parameters

fn main() {}
