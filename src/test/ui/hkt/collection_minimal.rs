// HKT inspect method
// check-pass

trait Collection where Self<%A> {
    fn empty() -> Self<%A>;
}

impl Collection for Option<%A> {
    fn empty() -> Option<%A> {
        None
    }
}

/*impl Collection for Option<%A> {
    fn empty() -> Self<%A> {
        None
    }
}*/

fn main() {

}
