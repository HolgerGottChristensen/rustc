// HKT The local binding should report that I<[u32]> is not sized.

fn test<I<%J>>() {
    let i: I<[u32]> = todo!(); //~ ERROR the size for values of type `[u32]` cannot be known at compilation time [E0277]
    //~^ ERROR the size for values of type
}

fn main() {}
