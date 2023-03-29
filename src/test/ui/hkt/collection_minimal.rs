// HKT inspect method
// check-pass

// We need some where: I<R> = Self.
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
    //Option::<u32>::add(&mut test, 42);
    test.add(42);
    println!("{:?}", test);
    test.add(41);
    println!("{:?}", test);
    test.add(40);
    println!("{:?}", test);
    test.clear();
    println!("{:?}", test);
}

/*fn test<I<%J>: Collection<%J, I<%A>>>(h: I<u32>) {
    ...
}*/

/*trait Test<R: Debug> {
    fn hejsa(i: R);
}*/
/*fn collection<R, I<%A>>() {

}*/
