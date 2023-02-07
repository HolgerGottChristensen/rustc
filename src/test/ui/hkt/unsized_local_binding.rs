// HKT test

fn test<I<?j>>() {
    let i: I<[u32]> = todo!();
    // ~^ ERROR `[u32]` is not Sized
}

fn main() {}
