// HKT test
// check-pass

struct MyStruct<A, B> {
    left: A,
    right: B,
}

impl<A,B> MyStruct<A, B> {
    pub fn new(a:A, b:B) -> Self {
        Self{
            left: a,
            right: b,
        }
    }
}

fn test<I<%J, %K>>(in1: I<u32, String>, in2: I<f32, String>) {}

fn main() {
    //should infer to be MyStruct<%J,String> or MyStruct<%J,%K>
    test(MyStruct::new(5u32, "Hej".to_string()), MyStruct::new(8f32, "Hejsas".to_string()));
}

