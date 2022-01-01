#![allow(dead_code)]

mod balance;
pub mod entry;
mod iter;
mod mem;
mod node;
#[cfg(test)]
mod tests;

pub use iter::{IntoIter, IntoKeys, IntoValues, Iter, IterMut, Keys, Values, ValuesMut};
use node::{ChildIndex, Node, NodeRef};

use std::{borrow::Borrow, fmt, marker::PhantomData};

pub struct RedBlackTree<K, V> {
    root: Option<NodeRef<K, V>>,
    len: usize,
    _phantom: PhantomData<Box<Node<K, V>>>,
}

// private methods
impl<K, V> RedBlackTree<K, V> {
    fn first_node(&self) -> Option<NodeRef<K, V>> {
        let mut current = self.root?;
        while let Some(left) = current.child(ChildIndex::Left) {
            current = left;
        }
        Some(current)
    }

    fn last_node(&self) -> Option<NodeRef<K, V>> {
        let mut current = self.root?;
        while let Some(right) = current.child(ChildIndex::Right) {
            current = right;
        }
        Some(current)
    }
}

// private methods
impl<K: Ord, V> RedBlackTree<K, V> {
    fn insert_node(&mut self, new_node: NodeRef<K, V>, (target, idx): (NodeRef<K, V>, ChildIndex)) {
        target.set_child(idx, Some(new_node));

        self.balance_after_insert(new_node);
    }

    fn remove_node(&mut self, node: NodeRef<K, V>) -> (K, V) {
        if node.parent().is_none() {
            // the node is root
            return unsafe { self.root.take().unwrap().deallocate() };
        }

        fn pop_then_promote<K, V>(node: NodeRef<K, V>, child: Option<NodeRef<K, V>>) -> (K, V) {
            if let Some(parent) = node.parent() {
                parent.set_child(node.index_on_parent().unwrap(), child);
            }
            unsafe { node.deallocate() }
        }

        let child = match (node.child(ChildIndex::Left), node.child(ChildIndex::Right)) {
            (Some(left), Some(right)) => {
                let mut min_in_right = right;
                while let Some(lesser) = min_in_right.child(ChildIndex::Left) {
                    min_in_right = lesser;
                }
                min_in_right
                    .parent()
                    .unwrap()
                    .set_child(min_in_right.index_on_parent().unwrap(), None);
                min_in_right.set_color(node.color());
                min_in_right.set_child(ChildIndex::Left, Some(left));
                let right_top = if min_in_right == right {
                    None
                } else {
                    Some(right)
                };
                min_in_right.set_child(ChildIndex::Right, right_top);
                Some(min_in_right)
            }
            (l, r) => l.xor(r),
        };

        self.balance_after_remove(node);

        pop_then_promote(node, child)
    }

    fn search_node<Q>(&self, key: &Q) -> Result<NodeRef<K, V>, (NodeRef<K, V>, ChildIndex)>
    where
        K: Borrow<Q>,
        Q: Ord + ?Sized,
    {
        self.root.unwrap().search(key)
    }
}

impl<K, V> Drop for RedBlackTree<K, V> {
    fn drop(&mut self) {
        drop(unsafe { std::ptr::read(self) }.into_iter());
    }
}

impl<K: fmt::Debug, V: fmt::Debug> fmt::Debug for RedBlackTree<K, V> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_map().entries(self.iter()).finish()
    }
}

impl<K, V> Default for RedBlackTree<K, V> {
    fn default() -> Self {
        Self::new()
    }
}

impl<K, V> RedBlackTree<K, V> {
    pub const fn new() -> Self {
        Self {
            root: None,
            len: 0,
            _phantom: PhantomData,
        }
    }

    pub fn clear(&mut self) {
        *self = Self::new();
    }

    pub const fn is_empty(&self) -> bool {
        self.root.is_none()
    }

    pub const fn len(&self) -> usize {
        self.len
    }

    pub fn iter(&self) -> Iter<K, V> {
        self.into_iter()
    }

    pub fn iter_mut(&mut self) -> IterMut<K, V> {
        self.into_iter()
    }

    pub fn into_keys(self) -> IntoKeys<K, V> {
        IntoKeys(self.into_iter())
    }

    pub fn keys(&self) -> Keys<K, V> {
        Keys(self.into_iter())
    }

    pub fn into_values(self) -> IntoValues<K, V> {
        IntoValues(self.into_iter())
    }

    pub fn values(&self) -> Values<K, V> {
        Values(self.into_iter())
    }

    pub fn values_mut(&mut self) -> ValuesMut<K, V> {
        ValuesMut(self.into_iter())
    }
}

impl<K: Ord, V> RedBlackTree<K, V> {
    pub fn insert(&mut self, key: K, value: V) -> Option<(K, V)> {
        if self.is_empty() {
            self.root = Some(NodeRef::new(key, value));
            self.len += 1;
            return None;
        }
        match self.search_node(&key) {
            Ok(found) => {
                // replace
                let old_v = std::mem::replace(found.value_mut(), value);
                Some((key, old_v))
            }
            Err(target) => {
                let new_node = NodeRef::new(key, value);
                self.insert_node(new_node, target);
                self.len += 1;
                None
            }
        }
    }

    pub fn remove<Q>(&mut self, key: &Q) -> Option<V>
    where
        K: Borrow<Q>,
        Q: Ord + ?Sized,
    {
        self.remove_entry(key).map(|(_, v)| v)
    }

    pub fn remove_entry<Q>(&mut self, key: &Q) -> Option<(K, V)>
    where
        K: Borrow<Q>,
        Q: Ord + ?Sized,
    {
        if self.is_empty() {
            return None;
        }
        let found = self.search_node(key).ok()?;
        self.len -= 1;
        Some(self.remove_node(found))
    }

    pub fn get<Q>(&self, key: &Q) -> Option<&V>
    where
        K: Borrow<Q>,
        Q: Ord + ?Sized,
    {
        self.get_key_value(key).map(|(_, v)| v)
    }

    pub fn get_mut<Q>(&mut self, key: &Q) -> Option<&mut V>
    where
        K: Borrow<Q>,
        Q: Ord + ?Sized,
    {
        if self.is_empty() {
            return None;
        }
        self.search_node(key).ok().map(|n| n.value_mut())
    }

    pub fn get_key_value<Q>(&self, key: &Q) -> Option<(&K, &V)>
    where
        K: Borrow<Q>,
        Q: Ord + ?Sized,
    {
        if self.is_empty() {
            return None;
        }
        self.search_node(key).ok().map(|n| n.key_value())
    }

    pub fn contains_key<Q>(&self, key: &Q) -> bool
    where
        K: Borrow<Q>,
        Q: Ord + ?Sized,
    {
        self.get(key).is_some()
    }
}
