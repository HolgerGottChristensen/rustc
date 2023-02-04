// HKT test
// pretty-compare-only

// test::<u32, %j>(42, 42);
// I: * -> *
// I (j) -> j
// I<u32> -> u32

// test::<u32, Option<%j>>(42, Some(42));
// I: * -> *
// I (j) -> Option<j>
// I<u32> -> Option<u32>

// test::<u32, Result<%j, String>>(42, Ok(42));
// I: * -> *
// I (j) -> Result<j, String>
// I<u32> -> Result<u32, String>

// test::<u32, String>(42, "Hejsa".to_string());
// I: * -> *
// I (j) -> String
// I<u32> -> String


fn test<T, I<?j>>(int: T, input: I<u32>) {
    //println!("Hej verden!");
}

//fn main() { let i = 1 + "hejsa"; }
fn main() {
    test::<u32, Option<%j>>(42, Some(42));
}

// HKT(I/1, {%j: u32, %k: String})
