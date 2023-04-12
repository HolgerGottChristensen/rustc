// HKT inspect method
// check-fail

trait Test<I<%A>> {
    fn identity(i: I<u32>) -> I<u32>;
}

impl<T> Test<Option<%B>> for Option<T> { //~ERROR: hkt argument `%B` could not be found in the definition
    fn identity(i: Option<u32>) -> Option<u32> {
        i
    }
}

fn main() {}

