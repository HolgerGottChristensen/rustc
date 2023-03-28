// HKT inspect method
// check-pass

//use std::fmt::Debug;

trait Collection<R, I<%A>> {
    fn empty() -> I<R>;

    //fn add(&mut self, value: R);
}

impl<T> Collection<T, Option<%A>> for Option<T> {
    fn empty() -> Option<T> { None }

    // fn add(&mut self, value: T) { *self = Some(value); }
}

/*fn test<I<%J>: Collection<%J, I<%A>>>(h: I<u32>) {
    ...
}*/

/*trait Test<R: Debug> {
    fn hejsa(i: R);
}*/
/*fn collection<R, I<%A>>() {

}*/


fn main() {
    let test = Option::<u32>::empty();
    println!("{:?}", test);
}
