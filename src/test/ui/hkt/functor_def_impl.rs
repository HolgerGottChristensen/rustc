// HKT inspect method
// check-pass

trait Functor<T, I<%J>> {
    fn fmap<U>(i: I<T>, f: impl FnOnce(T)->U) -> I<U>;
}

impl<T> Functor<T, Option<%J>> for Option<T> {
    fn fmap<U>(i: Option<T>, f: impl FnOnce(T)->U) -> Option<U> {
        Option::map(i, f)
    }
}


fn main() {}

