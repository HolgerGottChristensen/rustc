// HKT inspect method
// check-pass

trait Collection<R, A<%B>> {
    fn empty() -> A<R>;

    fn add(&mut self, value: R);
}


fn floatify<I<%J>: Collection<%J, I<%B>> + Iterator<Item=%J>>(input: I<u32>) -> I<f64> {
    let mut res = I::<f64>::empty();

    for i in input {
        res.add(i as f64);
    }

    res
}

fn main() {

}

