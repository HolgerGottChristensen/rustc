// HKT inspect method
// check-pass

// We need some where: I<R> = Self.
trait Collection<R, I<%A>> {
    fn empty() -> I<R>;

    fn add(&mut self, value: R);

    fn clear(&mut self);
}

impl<T, E: Default> Collection<T, Result<%A, E>> for Result<T, E> {
    fn empty() -> Result<T, E> { Err(E::default()) }

    fn add(&mut self, value: T) { *self = Ok(value); }

    fn clear(&mut self) { *self = Err(E::default()); }
}

fn main() {
    let mut test = Result::<u32, u32>::empty();
    println!("{:?}", test);
    test.add(42);
    println!("{:?}", test);
    test.add(41);
    println!("{:?}", test);
    test.add(40);
    println!("{:?}", test);
    test.clear();
    println!("{:?}", test);
}

