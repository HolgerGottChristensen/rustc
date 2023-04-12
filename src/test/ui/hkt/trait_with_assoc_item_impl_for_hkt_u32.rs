// HKT inspect method
// check-pass

trait Test {
    type Item;

    fn t() -> Self::Item;
}

//fn ttt<T: Test<Item=u32>>() {
fn ttt<I<%J>: Test<Item=u32>>() {
    //let e: u32 = <T as Test>::t();
    let e: u32 = <I<u32> as Test>::t();
}

fn main() {

}

