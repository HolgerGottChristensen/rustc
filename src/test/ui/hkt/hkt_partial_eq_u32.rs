// HKT test that something cloneable can be called with clone, method call syntax, definition and call.
// check-pass

fn test<I<%J: PartialEq<u32>>: PartialEq<u32>>(input: I<u32>, input2: u32) -> bool {
    input != input2
}

struct Test(u32);

impl PartialEq<u32> for Test {
    fn eq(&self, other: &u32) -> bool {
        &self.0 == other
    }
}

struct Test2<T>(T);

impl<T: PartialEq<u32>> PartialEq<u32> for Test2<T> {
    fn eq(&self, other: &u32) -> bool {
        &self.0 == other
    }
}


fn main() {
    let res = test::<u32>(42, 42);
    println!("{res}");
    let res2 = test::<Test>(Test(32), 42);
    println!("{res2}");
    let res3 = test::<Test2<%J>>(Test2(32), 42);
    println!("{res3}");
}
