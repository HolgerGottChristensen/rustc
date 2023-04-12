// HKT inspect method
// check-pass

trait Test {
    type Item;

    fn t() -> Self::Item;
}

fn ttt<I<%J>: Test<Item=%J>>() {
    let _e: bool = <I<bool> as Test>::t();
}

fn main() {

}

