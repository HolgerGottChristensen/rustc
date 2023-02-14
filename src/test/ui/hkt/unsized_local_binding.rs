// HKT test

fn test<I<%J>>() {
    let i: I<[u32]> = todo!();
    // ~^ ERROR `[u32]` is not Sized
}

fn main() {}
