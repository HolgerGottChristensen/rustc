// HKT inspect method
// check-pass

trait Functor<T, I<%J>> {
    fn fmap<U>(self, f: impl FnOnce(T)->U) -> I<U>;
}

impl<T> Functor<T, Option<%J>> for Option<T> {
    fn fmap<U>(self, f: impl FnOnce(T)->U) -> Option<U> {
        Option::map(self, f)
    }
}


fn main() {
    let res = Some(42).fmap(|a| a as f64);
    //let res: Option<f64> = Some(42).fmap(|a| a as f64);
    //let res: Option<u32> = Some(42).fmap(|a| a);
    println!("{:?}", res);
}

