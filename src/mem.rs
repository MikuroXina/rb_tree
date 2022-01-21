use crate::node::Node;

pub struct NodeDropGuard<K, V, const N: usize>(pub [Node<K, V>; N]);

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
