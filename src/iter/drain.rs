use super::PreviousStep;
use crate::{
    node::{ChildIndex, NodeRef},
    RedBlackTree,
};

use std::{fmt, iter::FusedIterator};

impl<K: Ord, V> RedBlackTree<K, V> {
    /// Creates an iterator that visits all elements (key-value pairs) in ascending key order and uses a closure to determine if an element should be removed. If the closure returns true, the element is removed from the map and yielded. If the closure returns false, or panics, the element remains in the map and will not be yielded.
    ///
    /// The iterator also lets you mutate the value of each element in the closure, regardless of whether you choose to keep or remove it.
    ///
    /// If the iterator is only partially consumed or not consumed at all, each of the remaining elements is still subjected to the closure, which may change its value and, by returning true, have the element removed and dropped.
    ///
    /// It is unspecified how many more elements will be subjected to the closure if a panic occurs in the closure, or a panic occurs while dropping an element, or if the DrainFilter value is leaked.
    ///
    /// # Examples
    ///
    /// ```
    /// use rb_tree::RedBlackTree;
    ///
    /// let mut map: RedBlackTree<i32, i32> = (0..8).map(|x| (x, x)).collect();
    /// let evens: RedBlackTree<_, _> = map.drain_filter(|k, _| k % 2 == 0).collect();
    /// let odds = map;
    ///
    /// assert_eq!(evens.into_keys().collect::<Vec<_>>(), vec![0, 2, 4, 6]);
    /// assert_eq!(odds.into_keys().collect::<Vec<_>>(), vec![1, 3, 5, 7]);
    /// ```
    #[inline]
    pub fn drain_filter<F: FnMut(&K, &mut V) -> bool>(&mut self, f: F) -> DrainFilter<K, V, F> {
        // FIXME: to guarantee memory safety
        let current = self.root.map(|r| r.first_node());
        DrainFilter {
            tree: self,
            current,
            prev: PreviousStep::LeftChild,
            pred: f,
        }
    }
}

pub struct DrainFilter<'a, K: Ord, V, F: FnMut(&K, &mut V) -> bool> {
    tree: &'a mut RedBlackTree<K, V>,
    current: Option<NodeRef<K, V>>,
    prev: PreviousStep,
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
        // Safety: The reference will not live longer than `&self`.
        self.current.map(|n| unsafe { n.key_value() })
    }
}

impl<'a, K: Ord, V, F: FnMut(&K, &mut V) -> bool> Iterator for DrainFilter<'a, K, V, F> {
    type Item = (K, V);

    fn next(&mut self) -> Option<Self::Item> {
        while let Some(curr) = self.current {
            match self.prev {
                PreviousStep::Parent => {
                    // descended
                    if let Some(left) = curr.left() {
                        // go to left
                        self.current = Some(left);
                        continue;
                    }
                    self.prev = PreviousStep::LeftChild;
                }
                PreviousStep::LeftChild => {
                    // ascended from left
                    if let Some(right) = curr.right() {
                        // go to right
                        self.prev = PreviousStep::Parent;
                        self.current = Some(right);
                    } else {
                        // ascended from right, so ascend again
                        self.prev = if let Some(ChildIndex::Left) = curr.index_on_parent() {
                            PreviousStep::LeftChild
                        } else {
                            PreviousStep::RightChild
                        };
                        self.current = curr.parent();
                    }
                    // Safety: The mutable reference will not live longer than `pred`.
                    let (k, v) = unsafe { curr.key_value_mut() };
                    if (self.pred)(k, v) {
                        self.prev = PreviousStep::Parent;
                        return self.tree.remove_entry(k);
                    }
                }
                PreviousStep::RightChild => {
                    // ascended from right, so ascend again
                    self.current = curr.parent();
                    if let Some(ChildIndex::Left) = curr.index_on_parent() {
                        self.prev = PreviousStep::LeftChild;
                    }
                }
            }
        }
        None
    }

    fn size_hint(&self) -> (usize, Option<usize>) {
        (0, Some(self.tree.len()))
    }
}

impl<K: Ord, V, F: FnMut(&K, &mut V) -> bool> FusedIterator for DrainFilter<'_, K, V, F> {}
