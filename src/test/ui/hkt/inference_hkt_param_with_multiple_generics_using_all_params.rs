// HKT test
// check-pass

use std::collections::HashMap;

fn test<I<%J, %K, %L>>(in1: I<u32, bool, bool>, in2: I<String, bool, bool>) {}

fn main() {
    //should infer to be HashMap<%J, bool> or HashMap<%J, %K>
    test(
        HashMap::from([(5u32, true)]),
        HashMap::from([("ayo".to_string(), true)])
    );
}
