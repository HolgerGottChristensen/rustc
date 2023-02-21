// HKT test that something cloneable can be called with clone, function call syntax, definition and call.
// check-fail

use std::rc::Rc;

fn test<I<%J>: Clone>(input: I<u32>) -> I<u32> {
    Clone::clone(&input)
}

fn main() {
    test::<Option<u32>>(Some(43));
    //test::<&%J>(&42);
    test::<Rc<%J>>(Rc::new(42));

    test::<Option<%J>>(Some(41)); //~ ERROR the trait bound `%J: Clone` is not satisfied
}
