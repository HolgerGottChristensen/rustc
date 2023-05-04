// HKT inspect method
// check-pass

use std::ops::{Div, Mul, Add, Sub};

trait Functor<T, I<%J>> {
    fn map<U>(self, f: impl Fn(T)->U) -> I<U>;
}

trait Applicative<T, I<%K>> {//: Functor<T, I<%J>> {
    fn pure(t: T) -> I<T>;

    fn product<U>(self, other: I<U>) -> I<(T, U)>;
}

trait Traverse<T, I<%O>> {
    fn traverse<G<%P>: Applicative<%P, G<%K>>, U>(self) -> Option<Option<U>>;//G<I<U>>;
}

impl<T> Functor<T, Option<%J>> for Option<T> {
    fn map<U>(self, f: impl Fn(T)->U) -> Option<U> {
        match self {
            Some(x) => Some(f(x)),
            None => None,
        }
    }
}

impl<T> Applicative<T, Option<%K>> for Option<T> {
    fn pure(t: T) -> Option<T> {
        Some(t)
    }

    fn product<U>(self, other: Option<U>) -> Option<(T, U)> {
        match (self, other) {
            (Some(t), Some(u)) => Some((t, u)),
            _ => None,
        }
    }
}

impl<T> Traverse<T, Option<%O>> for Option<T> {
    fn traverse<H<%Q>: Applicative<%Q, H<%K>>, U>(self) -> Option<Option<U>> {//H<Option<U>> {
        /*match self {
            None => H::<Option<U>>::pure(None),
            Some(t) => f(t).map(|a| Some(a)),
        }*/
        todo!()
    }
}




fn main() {
    let val = Some(32);
    let res: Option<Option<u32>> = <Option<u32> as Traverse<u32, Option<%O>>>::traverse::<Option<%P>, u32>(val);
    println!("traverse: {:?}", res);
}

