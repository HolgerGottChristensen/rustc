// HKT We check that multiple parameters can be created for a function and that the function can be
// called
// check-pass

fn test<I<%J>>(input: I<u32>, input2: I<bool>) {
}

fn main() {

    fn print(i: u32) {
        println!("Hej verden!: {:#?}", i);
    }
    fn print1(i: bool) {
        println!("Hej verden!: {:#?}", i);
    }

    test(print, print1);
}
