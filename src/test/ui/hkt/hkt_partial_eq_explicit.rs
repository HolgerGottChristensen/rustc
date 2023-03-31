// HKT test that something cloneable can be called with clone, method call syntax, definition and call.
// check-pass

fn test<I<%J>: PartialEq<I<%J>>>(input: I<u32>, input2: I<String>) -> bool {
    input == input2
}

struct Test<T>(T);

impl<T> PartialEq for Test<T> {
    fn eq(&self, _other: &Self) -> bool {
        true
    }
}

fn main() {
    let res = test::<u32>(42, 42);
    let res2 = test::<Test<%J>>(Test(42), Test("Hello world!".to_string()));
    println!("{res}");
    println!("{res2}");
}
