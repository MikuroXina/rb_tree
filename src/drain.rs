use crate::{
    node::{ChildIndex, NodeRef},
    RedBlackTree,
};

use std::{fmt, iter::FusedIterator};

impl<K: Ord, V> RedBlackTree<K, V> {
    pub fn drain_filter<F: FnMut(&K, &mut V) -> bool>(&mut self, f: F) -> DrainFilter<K, V, F> {
        let current = self.first_node();
        DrainFilter {
            tree: self,
            current,
            pred: f,
        }
    }
}

pub struct DrainFilter<'a, K: Ord, V, F: FnMut(&K, &mut V) -> bool> {
    tree: &'a mut RedBlackTree<K, V>,
    current: Option<NodeRef<K, V>>,
    pred: F,
}

impl<K: Ord, V, F: FnMut(&K, &mut V) -> bool> Drop for DrainFilter<'_, K, V, F> {
    fn drop(&mut self) {
        self.for_each(drop);
    }
}

impl<K: fmt::Debug + Ord, V: fmt::Debug, F: FnMut(&K, &mut V) -> bool> fmt::Debug
    for DrainFilter<'_, K, V, F>
{
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_tuple("DrainFilter").field(&self.peek()).finish()
    }
}

impl<'a, K: Ord, V, F: FnMut(&K, &mut V) -> bool> DrainFilter<'a, K, V, F> {
    fn peek(&self) -> Option<(&K, &V)> {
        self.current.map(|n| n.key_value())
    }
}

impl<'a, K: Ord, V, F: FnMut(&K, &mut V) -> bool> Iterator for DrainFilter<'a, K, V, F> {
    type Item = (K, V);

    fn next(&mut self) -> Option<Self::Item> {
        loop {
            let current = self.current?;
            let (k, v) = current.key_value_mut();
            self.current = current
                .child(ChildIndex::Right)
                .or_else(|| current.parent());
            if (self.pred)(k, v) {
                return self.tree.remove_entry(k);
            }
        }
    }

    fn size_hint(&self) -> (usize, Option<usize>) {
        (0, Some(self.tree.len()))
    }
}

impl<K: Ord, V, F: FnMut(&K, &mut V) -> bool> FusedIterator for DrainFilter<'_, K, V, F> {}
