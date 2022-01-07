use crate::node::NodeRef;

pub struct NodeDropGuard<K, V, const N: usize>(pub [NodeRef<K, V>; N]);

impl<K, V, const N: usize> Drop for NodeDropGuard<K, V, N> {
    fn drop(&mut self) {
        // Safety: `self` will not be used after.
        unsafe {
            for n in self.0 {
                n.deallocate();
            }
        }
    }
}
