// HKT inspect method
// check-pass

trait Functor<T, I<%J>> {
    fn fmap<U>(self, f: impl Fn(T)->U) -> I<U>;

    fn inspect(self, f: impl Fn(&T)) -> I<T> where Self: Sized {
        self.fmap(|t| {f(&t); t})
    }
}

fn main() {
}

