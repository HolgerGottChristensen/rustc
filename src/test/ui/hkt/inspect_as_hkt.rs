// HKT inspect method
// check-pass


fn inspect<A, I<%J>, F: FnOnce(&A) + 'static>(i: I<A>, f: F, map: fn(I<A>, Box<dyn FnOnce(A)->A>)->I<A>) -> I<A> {
    map(i, Box::new(|a| {f(&a); a}))
}

fn f1(i: &u32) {
    println!("Hej verden!: {}", i);
}

fn f2(i: &String) {
    println!("Hej verden!: {}", i);
}

fn map<A>(i: Option<A>, m: Box<dyn FnOnce(A)->A>) -> Option<A> {
    i.map(|a| {m(a)})
}

fn map2<A>(i: Result<A, String>, m: Box<dyn FnOnce(A)->A>) -> Result<A, String> {
    i.map(|a| {m(a)})
}

fn map3<A>(i: A, m: Box<dyn FnOnce(A)->A>) -> A {
    m(i)
}

fn main() {
    inspect::<u32, Option<%J>, _>(Some(42), f1, map);
    inspect::<String, Option<%J>, _>(Some("Det er nice!".to_string()), f2, map);
    inspect::<u32, Result<%J, String>, _>(Ok(42), f1, map2);
    inspect::<u32, Result<%J, String>, _>(Err("Hejsa".to_string()), f1, map2);
    inspect::<u32, u32, _>(32, f1, map3);
}
