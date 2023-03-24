// HKT pointer example is positive function.
// check-pass

use std::sync::Arc;
use std::rc::Rc;


trait Pointer<T> {
    fn new(val: T) -> Self;
    fn val(&self) -> &T;
}

impl<T> Pointer<T> for Rc<T> {
    fn new(val: T) -> Self {
        Rc::new(val)
    }
    fn val(&self) -> &T {
        &*self
    }
}

impl<T> Pointer<T> for Arc<T> {
    fn new(val: T) -> Self {
        Arc::new(val)
    }
    fn val(&self) -> &T {
        &*self
    }
}

impl<T> Pointer<T> for Box<T> {
    fn new(val: T) -> Self {
        Box::new(val)
    }
    fn val(&self) -> &T {
        &*self
    }
}

/*impl<T> Pointer<T> for T {
    fn new(val: T) -> Self {
        Box::new(val)
    }
    fn val(&self) -> &T {
        &*self
    }
}*/

fn is_positive<I<%J>: Pointer<%J>>(input: I<i32>) -> I<bool> {
    let is_positive = *input.val() > 0;
    I::<bool>::new(is_positive)
}

fn main() {
    println!("{:#?}", is_positive::<Rc<%J>>(Rc::new(42)));
    println!("{:#?}", is_positive::<Arc<%J>>(Arc::new(-42)));
    println!("{:#?}", is_positive::<Box<%J>>(Box::new(42)));
}
