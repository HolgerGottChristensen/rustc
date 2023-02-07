// HKT test
// pretty-compare-only

fn inspect<'a, A: 'a, G: FnOnce(A) -> A, I<?j>, F: FnOnce(&'a A)>(p: I<A>, f: F, map: fn(I<A>,G)->I<A>) -> I<A> {
    map(p, |a|{f(&a); a})
}

//fn main() { let i = 1 + "hejsa"; }
fn main() {

    fn f1(i: &u32) {
        println!("Hej verden!: {}", i);
    }

    inspect::<u32, u32, Option<?j>, _>(Some(42), f1, Option::<u32>::map::<u32>);
}
