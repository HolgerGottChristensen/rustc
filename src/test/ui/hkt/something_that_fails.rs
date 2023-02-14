
#[derive(Copy)]
pub union MaybeUninit<T> {
    uninit: (),
    value: core::mem::ManuallyDrop<T>,
}

impl<T> core::fmt::Debug for MaybeUninit<T> {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        f.pad(core::any::type_name::<MaybeUninit<T>>())
    }
}

impl<T: Copy> Clone for MaybeUninit<T> {
    fn clone(&self) -> Self {
        *self
    }
}

fn main() {}
