use crate::node::NodeRef;

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
