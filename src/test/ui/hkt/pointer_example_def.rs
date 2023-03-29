// HKT pointer example is positive function.
// check-pass

trait Pointer<T> {
    fn new(val: T) -> Self;
    fn val(&self) -> &T;
}

fn is_positive<I<%J>: Pointer<%J>>(input: I<i32>) -> I<bool> {
    todo!()
}

fn main() {}
