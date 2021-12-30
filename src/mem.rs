use crate::node::NodeRef;

pub fn replace<T, R>(v: &mut T, change: impl FnOnce(T) -> (T, R)) -> R {
    use std::{panic, ptr};

    unsafe {
        let old_t = ptr::read(v);
        let (new_t, r) = panic::catch_unwind(panic::AssertUnwindSafe(|| change(old_t)))
            .unwrap_or_else(|_| ::std::process::abort());
        ptr::write(v, new_t);
        r
    }
}

pub struct NodeDropGuard<K, V, const N: usize>(pub [NodeRef<K, V>; N]);

impl<K, V, const N: usize> Drop for NodeDropGuard<K, V, N> {
    fn drop(&mut self) {
        for n in self.0 {
            unsafe {
                n.deallocate();
            }
        }
    }
}
