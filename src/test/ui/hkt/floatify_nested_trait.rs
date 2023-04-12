// HKT inspect method
// check-pass

trait Collection<R, A<%B>>: IntoIterator<Item=R> {
    fn empty() -> A<R>;

    fn add(&mut self, value: R);
}


fn floatify<I<%J>: Collection<%J, I<%B>>>(input: I<u32>) -> I<f64> {
    let mut res = I::<f64>::empty();

    for i in input {
        res.add(i as f64);
    }

    res
}

impl<T> Collection<T, Vec<%B>> for Vec<T> {
    fn empty() -> Vec<T> {
        Vec::new()
    }

    fn add(&mut self, value: T) {
        self.push(value);
    }
}

impl<T> Collection<T, Option<%B>> for Option<T> {
    fn empty() -> Option<T> {
        None
    }

    fn add(&mut self, value: T) {
        *self = Some(value);
    }
}

fn main() {
    let res = floatify::<Vec<%J>>(vec![42, 43, 44, 45, 46]);
    println!("{:?}", res);
    let res2 = floatify::<Option<%J>>(Some(42));
    println!("{:?}", res2);
}

