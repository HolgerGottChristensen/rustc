const fn failure() {
    panic!("{:?}", 0);
    //~^ ERROR cannot call non-const formatting macro in constant functions
}

const fn print() {
    println!("{:?}", 0);
    //~^ ERROR cannot call non-const formatting macro in constant functions
    //~| ERROR `Arguments::<'a>::new_v1` is not yet stable as a const fn
    //~| ERROR cannot call non-const fn `_print` in constant functions
}

fn main() {}
