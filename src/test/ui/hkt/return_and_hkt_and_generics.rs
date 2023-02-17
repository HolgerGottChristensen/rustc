// HKT mixing generics and hkt and returning them in a tuple from the function
// check-pass

fn test<T, I<%J>>(input: I<u32>, input2: T, f: fn(&I<u32>, &T)) -> (I<u32>, T) {
    f(&input, &input2);
    (input, input2)
}

fn print(i: &u32, j: &u32) {
    println!("Hej verden!: {}, {}", i, j);
}

fn main() {
    test::<u32, u32>(42, 45, print);
}
