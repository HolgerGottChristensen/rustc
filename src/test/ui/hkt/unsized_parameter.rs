// HKT test

fn test<I<?J>>(input: I<[u32]>) {
    // ~^ ERROR `[u32]` is not Sized
}

fn main() {}
