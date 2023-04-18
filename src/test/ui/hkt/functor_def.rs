// HKT inspect method
// check-pass

trait Functor<T, I<%J>> {
    fn fmap<U>(i: I<T>, f: impl FnOnce(T)->U) -> I<U>;
}

/*impl<F> Test<Option<%A>> for Option<F> {
    fn identity<G>(i: Option<G>) -> Option<G> {
        i
    }
}*/


fn main() {}

