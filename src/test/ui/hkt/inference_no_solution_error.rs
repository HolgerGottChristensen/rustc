//check-fail
fn test<I<%J>, L<%K>>(input: I<u32>, i2: I<bool>) {
}

fn main() {
    test(Some(32u32), Some("dff".to_string()));
}
