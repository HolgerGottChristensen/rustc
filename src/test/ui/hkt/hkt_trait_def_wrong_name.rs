// HKT inspect method
// check-fail

trait Test<I<%A>> {
    fn identity(i: I<u32>) -> I<u32>;
}

impl<T> Test2<Option<%A>> for Option<T> {
    fn identity(i: Option<u32>) -> Option<u32> {
        i
    }
}

fn main() {}

