// HKT inspect method
// check-pass

trait Test<I<%A>> {
    fn identity<T>(i: I<T>) -> I<T>;
}

impl<F> Test<Option<%A>> for Option<F> {
    fn identity<G>(i: Option<G>) -> Option<G> {
        i
    }
}

/*trait Test<X> {
    fn identity<T>(i: T) -> T;
}

impl<F> Test<u32> for Option<F> {
    fn identity<G>(i: G) -> G {
        i
    }
}*/

fn main() {}

