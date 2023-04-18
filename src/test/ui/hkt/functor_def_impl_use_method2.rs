// HKT inspect method
// check-pass

trait Functor<T, I<%J>> {
    fn fmap<U>(self, f: impl Fn(T)->U) -> I<U>;
}

impl<T> Functor<T, Vec<%J>> for Vec<T> {
    fn fmap<U>(self, f: impl Fn(T)->U) -> Vec<U> {
        self.into_iter().map(f).collect::<Vec<_>>()

        /*let mut res = Vec::new();
        for t in self {
            res.push(f(t));
        }
        res*/
    }
}


fn main() {
    let data = vec![1, 2, 3, 4];

    let res = data.fmap(|a| a as f64);
    println!("{:?}", res);
}

