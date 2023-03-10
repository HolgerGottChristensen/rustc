// HKT test that something cloneable can be called with clone, method call syntax, definition and call.
// check-pass

use std::fmt::{Debug, Formatter};

fn test<I<%J>: Debug>(input: I<u32>) {
    println!("Hej verden: {:?}", input);
}

struct FooBar<T> {
    t: T,
}

impl<T> Debug for FooBar<T> {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("Test").finish_non_exhaustive()
    }
}

fn main() {
    let l = FooBar { t: 42 };
    test::<FooBar<%J>>(l);

    let l2 = FooBar { t: 42 };
    test::<FooBar<u32>>(l2);

    test::<FooBar<%J>>(FooBar { t: 42 });
    test::<Option<u32>>(Some(42));
    test::<FooBar<u32>>(FooBar { t: 42 });
}
