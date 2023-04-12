// HKT inspect method
// check-pass

trait Test<I<%A>> {
    fn identity<T>(i: I<T>) -> I<T>;
}

impl<T> Test<Option<%A>> for Option<T> {
    fn identity<G>(i: Option<G>) -> Option<G> {
        i
    }
}

fn main() {}

