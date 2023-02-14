// HKT inspect method
// check-pass

//fn inspect<A, I<%J>, F: FnOnce(&A)>(i: I<A>, f: F, map: fn(I<A>, &dyn FnOnce(A)->A)->I<A>) -> I<A> {
//    map(i, &|a| {f(&a); a})
//}

fn inspect<I<%J>, F: FnOnce(&u32) + 'static>(i: I<u32>, f: F, map: fn(I<u32>, Box<dyn FnOnce(u32)->u32>)->I<u32>) -> I<u32> {
    map(i, Box::new(|a| {f(&a); a}))
}

fn f1(i: &u32) {
    println!("Hej verden!: {}", i);
}

fn map(i: Option<u32>, m: Box<dyn FnOnce(u32)->u32>) -> Option<u32> {
    i.map(|a| {m(a)})
}

//fn main() { let i = 1 + "hejsa"; }
fn main() {
    inspect::<Option<%J>, _>(Some(42), f1, map);
    //inspect::<u32, Option<%J>, _>(Some(42), f1, Option::<u32>::map::<u32>);
}
