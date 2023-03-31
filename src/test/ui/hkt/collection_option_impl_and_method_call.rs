// HKT inspect method
// check-pass

trait Collection<R, I<%A>> {
    fn empty() -> I<R>;

    fn add(&mut self, value: R);

    fn clear(&mut self);
}

impl<T> Collection<T, Option<%A>> for Option<T> {
    fn empty() -> Option<T> { None }

    fn add(&mut self, value: T) { *self = Some(value); }

    fn clear(&mut self) { *self = None; }
}

fn main() {
    let mut test = Option::<u32>::empty();
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
