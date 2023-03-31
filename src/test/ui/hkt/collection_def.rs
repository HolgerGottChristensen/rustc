// HKT inspect method
// check-pass

trait Collection<R, I<%A>> {
    fn empty() -> I<R>;

    fn add(&mut self, value: R);

    fn clear(&mut self);
}

fn main() {}
