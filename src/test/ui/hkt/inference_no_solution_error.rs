// should fail because there should be no solution for hkt inference
// check-fail
fn test<I<%J>>(input: I<u32>, i2: I<bool>) {
}

fn main() {
    test(Some(32u32), Some("dff".to_string())); //~ ERROR cannot infer HKT parameters [E10001]
}
