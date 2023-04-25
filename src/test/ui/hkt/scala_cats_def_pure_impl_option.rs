// HKT inspect method
// check-pass

trait Functor<T, I<%J>> {
    fn map<U>(self, f: impl Fn(T)->U) -> I<U>;
}

trait Applicative<T, I<%K>>: Functor<T, I<%J>> {
    fn pure(t: T) -> I<T>;

    fn product<U>(self, other: I<U>) -> I<(T, U)>;
}

trait FlatMap<T, I<%L>> {
    fn flatmap<U>(self, f: impl Fn(T)->I<U>) -> I<U>;
}

trait Monad<T, I<%M>>: Functor<T, I<%J>> + FlatMap<T, I<%L>> {}

trait Foldable<T, I<%N>> {
    fn foldleft<U>(self, state: U, f: impl Fn(U, T)->U) -> U;

    fn foldright<U>(self, state: U, f: impl Fn(T, U)->U) -> U;
}

trait Traverse<T, I<%O>>: Functor<T, I<%J>> + Foldable<T, I<%N>> {
    fn traverse<G<%P>: Applicative<%P, G<%K>>, U>(self, f: impl Fn(T)->G<U>) -> G<I<U>>;
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


impl<T> FlatMap<T, Option<%L>> for Option<T> {
    fn flatmap<U>(self, f: impl Fn(T)->Option<U>) -> Option<U> {
        match self {
            Some(t) => f(t),
            None => None,
        }
    }
}

//impl<T, I<%P>> Monad<T, I<%M>> for I<T> where I<T>: Functor<T, I<%J>> + FlatMap<T, I<%L>> {}

impl<T> Monad<T, Option<%M>> for Option<T> {}

impl<T> Foldable<T, Option<%N>> for Option<T> {
    fn foldleft<U>(self, state: U, f: impl Fn(U, T)->U) -> U {
        match self {
            Some(t) => f(state, t),
            None => state,
        }
    }

    fn foldright<U>(self, state: U, f: impl Fn(T, U)->U) -> U {
        match self {
            Some(t) => f(t, state),
            None => state,
        }
    }
}

impl<T> Traverse<T, Option<%O>> for Option<T> {
    fn traverse<H<%Q>: Applicative<%Q, H<%K>>, U>(self, f: impl Fn(T)->H<U>) -> H<Option<U>> {
        match self {
            None => H::<Option<U>>::pure(None),
            //None => <H<Option<U>> as Applicative<Option<U>, H<%K>>>::pure(None),
            Some(t) => f(t).map(|a| Some(a)),
        }
    }
}

fn main() {

}

