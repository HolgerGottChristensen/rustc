// HKT inspect method
// check-pass

trait Functor<T, I<%J>> {
    fn fmap<U>(self, f: impl Fn(T)->U) -> I<U>;

    fn finspect(self, f: impl Fn(&T)) -> I<T> where Self: Sized {
        self.fmap(|t| {f(&t); t})
    }
}

impl<T> Functor<T, Vec<%J>> for Vec<T> {
    fn fmap<U>(self, f: impl Fn(T)->U) -> Vec<U> {
        self.into_iter().map(f).collect::<Vec<_>>()
    }
}

impl<T> Functor<T, Option<%J>> for Option<T> {
    fn fmap<U>(self, f: impl FnOnce(T)->U) -> Option<U> {
        Option::map(self, f)
    }
}

fn main() {
    Some(42).finspect(|a| println!("{:?}", a));
    vec![32, 43, 67].finspect(|a| println!("{:?}", a));
    Some("Hello world").finspect(|a| println!("{:?}", a));
}

