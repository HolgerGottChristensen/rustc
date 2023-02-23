// HKT test
// check-pass

use std::collections::HashMap;

fn test<I<%J>>(in1: I<u32>, in2: I<String>) {

}
/*
fn test2<I<%J, %K>>(input: I<u32, u32>, f: fn(I<u32, u32>)) {
    f(input)
}

 */



fn main() {

    /*fn print(i: Option<u32>) {
        println!("Hej verden!: {:#?}", i);
    }
    fn print2(i: HashMap<u32, u32>) {
        println!("Hej verden!: {:#?}", i);
    }*/

    test::<Option<u32>>(Some(8u32), Some("Solo".to_string())); //should infer to be Option<%J> and not Option<u32>
    //test2(HashMap::from([(1, 2), (3, 4)]), print2);
}

