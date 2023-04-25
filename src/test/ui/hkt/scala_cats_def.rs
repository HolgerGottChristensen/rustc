// HKT inspect method
// check-pass

use std::ops::{Div, Mul, Add, Sub};

trait Functor<T, I<%J>> {
    fn map<U>(self, f: impl Fn(T)->U) -> I<U>;

    fn fproduct<U>(self, f: impl Fn(&T)->U) -> I<(T, U)> where Self: Sized {
        self.map(|t| {
            let u = f(&t);
            (t, u)
        })
    }

    fn fproduct_left<U>(self, f: impl Fn(&T)->U) -> I<(U, T)> where Self: Sized {
        self.map(|t| {
            let u = f(&t);
            (u, t)
        })
    }

    fn aas<U: Clone>(self, u: U) -> I<U> where Self: Sized {
        self.map(|_| u.clone())
    }

    fn funit(self) -> I<()> where Self: Sized {
        self.aas(())
    }

    //def lift[A, B](f: A => B): F[A] => F[B] = map(_)(f)

    fn tuple_left<U: Clone>(self, u: U) -> I<(U, T)> where Self: Sized {
        self.map(|t| (u.clone(), t))
    }

    fn tuple_right<U: Clone>(self, u: U) -> I<(T, U)> where Self: Sized {
        self.map(|t| (t, u.clone()))
    }

    // def unzip[A, B](fab: F[(A, B)]): (F[A], F[B]) = (map(fab)(_._1), map(fab)(_._2))

    // def ifF[A](fb: F[Boolean])(ifTrue: => A, ifFalse: => A): F[A] = map(fb)(x => if (x) ifTrue else ifFalse)
}

trait Applicative<T, I<%K>>: Functor<T, I<%J>> {
    fn pure(t: T) -> I<T>;

    fn product<U>(self, other: I<U>) -> I<(T, U)>;

    // def replicateA[A](n: Int, fa: F[A]): F[List[A]] =

    /*fn unit() -> I<()> {
        Self::pure(())
    }*/

    fn map2<U, V>(self, other: I<U>, f: impl Fn(T, U)->V) -> I<V> where Self: Sized {
        todo!()
        //self.product(other).map(f)
    }
}

trait Monad<T, I<%M>>: Functor<T, I<%J>> {
    fn flatmap<U>(self, f: impl Fn(T)->I<U>) -> I<U>;

    //def flatten[A](ffa: F[F[A]]): F[A] = flatMap(ffa)(fa => fa)

    /*fn mproduct<U>(self, f: impl Fn(&T)->I<U>) -> I<(T, U)> where Self: Sized {
        self.flatmap(|t| f(&t).map(|u| (t, u)))
    }*/
}

trait Foldable<T, I<%N>> {
    /// Fold from left to right
    fn foldleft<U>(self, state: U, f: impl Fn(U, T)->U) -> U;

    /// Fold from right to left
    fn foldright<U>(self, state: U, f: impl Fn(T, U)->U) -> U;

    fn reduce_left_to_option<U>(self, f: impl Fn(T)->U, g: impl Fn(U, T)->U) -> Option<U> where Self: Sized {
        self.foldleft(None, |u, t| {
            match (u, t) {
                (Some(u), t) => Some(g(u, t)),
                (None, a) => Some(f(a))
            }
        })
    }

    fn reduce_right_to_option<U>(self, f: impl Fn(T)->U, g: impl Fn(T, U)->U) -> Option<U> where Self: Sized {
        self.foldright(None, |t, u| {
            match (u, t) {
                (Some(u), t) => Some(g(t, u)),
                (None, a) => Some(f(a))
            }
        })
    }

    fn reduce_left_option(self, f: impl Fn(T, T) -> T) -> Option<T> where Self: Sized {
        self.reduce_left_to_option(|x| x, f)
    }

    fn reduce_right_option(self, f: impl Fn(T, T) -> T) -> Option<T> where Self: Sized {
        self.reduce_right_to_option(|x| x, f)
    }
}

trait FoldableOrd<T, I<%R>>: Foldable<T, I<%N>> where T: Ord {
    fn minimum_option(self) -> Option<T> where Self: Sized {
        self.reduce_left_option(Ord::min)
    }

    fn maximum_option(self) -> Option<T> where Self: Sized {
        self.reduce_left_option(Ord::max)
    }
}

trait Numeric: Add<Output = Self> + Sub<Output = Self> + Mul<Output = Self> + Div<Output = Self> where Self: Sized {
    fn zero() -> Self;
    fn one() -> Self;
}

impl Numeric for u32 {
    fn one() -> Self {
        1
    }

    fn zero() -> Self {
        0
    }
}

trait FoldableNumeric<T, I<%S>>: Foldable<T, I<%N>> where T: Numeric {
    fn sum_all(self) -> T where Self: Sized {
        self.foldleft(T::zero(), |a, b| a + b)
    }

    fn product_all(self) -> T where Self: Sized {
        self.foldleft(T::one(), |a, b| a * b)
    }
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

impl<T> Functor<T, Vec<%J>> for Vec<T> {
    fn map<U>(self, f: impl Fn(T)->U) -> Vec<U> {
        let mut res = vec![];

        for element in self {
            res.push(f(element));
        }

        res
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

//impl<T, I<%P>> Monad<T, I<%M>> for I<T> where I<T>: Functor<T, I<%J>> + FlatMap<T, I<%L>> {}

impl<T> Monad<T, Option<%M>> for Option<T> {
    fn flatmap<U>(self, f: impl Fn(T)->Option<U>) -> Option<U> {
        match self {
            Some(t) => f(t),
            None => None,
        }
    }
}

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

impl<T> Foldable<T, Vec<%N>> for Vec<T> {
    fn foldleft<U>(self, state: U, f: impl Fn(U, T)->U) -> U {
        let mut state = state;

        for element in self {
            state = f(state, element);
        }

        state
    }

    fn foldright<U>(self, state: U, f: impl Fn(T, U)->U) -> U {
        let mut state = state;

        for element in self.into_iter().rev() {
            state = f(element, state);
        }

        state
    }
}

impl<T: Ord> FoldableOrd<T, Vec<%R>> for Vec<T> {}
impl<T: Ord> FoldableOrd<T, Option<%R>> for Option<T> {}
impl<T: Numeric> FoldableNumeric<T, Vec<%S>> for Vec<T> {}
impl<T: Numeric> FoldableNumeric<T, Option<%S>> for Option<T> {}

//impl<T: Ord, G<%J>: Foldable<T, G<%N>>> FoldableOrd<T, G<%R>> for G<T> {}

impl<T> Traverse<T, Option<%O>> for Option<T> {
    fn traverse<H<%Q>: Applicative<%Q, H<%K>>, U>(self, f: impl Fn(T)->H<U>) -> H<Option<U>> {
        match self {
            None => H::<Option<U>>::pure(None),
            Some(t) => f(t).map(|a| Some(a)),
        }
    }
}

impl<T> Traverse<T, Vec<%O>> for Vec<T> {
    fn traverse<H<%Q>: Applicative<%Q, H<%K>>, U>(self, f: impl Fn(T)->H<U>) -> H<Vec<U>> {
        self.foldleft(H::<Vec<U>>::pure(vec![]), |acc, t| {
            let h: H<U> = f(t);

            h.map2(acc, |a: U, mut b: Vec<U>| {
                b.push(a);
                b
            })
        })
    }
}



fn main() {
    let val = Some(42);
    let res = val.fproduct(|t| t.to_string()); //let res: Option<(u32, String)> = val.fproduct(|t| t.to_string());
    println!("fproduct: {:?}", res);

    let val = Some(42);
    let res = val.fproduct_left(|t| t.to_string()); //let res: Option<(String, u32)> = val.fproduct(|t| t.to_string());
    println!("fproduct_left: {:?}", res);

    let val = Some(42);
    let res = val.aas("Hello world".to_string());
    println!("aas: {:?}", res);

    let val = Some(42);
    let res = val.funit();
    println!("funit: {:?}", res);

    let val = Some(42);
    let res = val.tuple_left(24);
    println!("tuple_left: {:?}", res);

    let val = Some(42);
    let res = val.tuple_right(24);
    println!("tuple_right: {:?}", res);

    let res = Option::pure(42);
    println!("pure: {:?}", res);

    /*let val1 = Some(42);
    let val2 = Some(24);
    let res = val1.product(val2);
    println!("product: {:?}", res);*/

    /*let val1 = Some(42);
    let val2 = Some(24);
    let res = val1.map2(val2, |t, u| t + u);
    println!("map2: {:?}", res);*/

    let val = Some(42);
    let res = val.flatmap(|a| Some(a));
    println!("flatmap: {:?}", res);

    /*let val = Some(42);
    let res = val.mproduct(|a| Some(a));
    println!("mproduct: {:?}", res);*/

    let val = vec![1, 1, 2, 5, 8];
    let res = val.foldleft(0, |u, t| {
        u + t
    });
    println!("foldleft: {:?}", res);

    let val = vec![1, 1, 2, 5, 8];
    let res = val.foldright(0, |t, u| {
        u + t
    });
    println!("foldright: {:?}", res);

    let val = vec![6, 3, 2];
    let res = val.reduce_left_to_option(|t| t, |u, t| {
        //println!("  {} - {} = {}", u, t, u - t);
        u - t
    });
    println!("reduce_left_to_option: {:?}", res);

    let val = vec![2, 3, 6];
    let res = val.reduce_right_to_option(|t| t, |t, u| {
        //println!("  {} - {} = {}", u, t, u - t);
        u - t
    });
    println!("reduce_right_to_option: {:?}", res);

    let val = vec![6, 3, 2];
    let res = val.reduce_left_option(|u, t| {
        //println!("  {} - {} = {}", u, t, u - t);
        u - t
    });
    println!("reduce_left_option: {:?}", res);

    let val = vec![2, 3, 6];
    let res = val.reduce_right_option(|t, u| {
        //println!("  {} - {} = {}", u, t, u - t);
        u - t
    });
    println!("reduce_right_option: {:?}", res);

    let val = vec![2, 3, 6];
    let res = val.minimum_option();
    println!("minimum_option: {:?}", res);

    let val = vec![2, 3, 6];
    let res = val.maximum_option();
    println!("maximum_option: {:?}", res);

    let val = vec![2, 3, 6];
    let res = val.sum_all();
    println!("sum_all: {:?}", res);

    let val = vec![2, 3, 6];
    let res = val.product_all();
    println!("product_all: {:?}", res);

    /*let val = vec!["1", "2", "3"];
    let res = val.product_all();
    println!("product_all: {:?}", res);*/
}

