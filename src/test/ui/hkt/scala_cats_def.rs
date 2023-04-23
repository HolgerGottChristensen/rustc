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
    fn traverse<G<%P>: Applicative<%P, G<%K>>, U>(self, f: impl Fn(T)->G<U>);
}



fn main() {

}

