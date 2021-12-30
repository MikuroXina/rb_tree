#![allow(dead_code)]

mod iter;
mod mem;
mod node;
#[cfg(test)]
mod tests;

pub use iter::IntoIter;

use node::{ChildIndex, Node, NodeRef};

use std::{borrow::Borrow, marker::PhantomData};

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
    fn insert_node(&mut self, new_node: NodeRef<K, V>, target: NodeRef<K, V>) {
        let idx = match new_node.key().cmp(target.key()) {
            std::cmp::Ordering::Less => ChildIndex::Left,
            std::cmp::Ordering::Equal => unreachable!(),
            std::cmp::Ordering::Greater => ChildIndex::Right,
        };
        target.set_child(idx, Some(new_node));

        new_node.balance_after_insert();
    }

    fn remove_node(&mut self, node: NodeRef<K, V>) -> (K, V) {
        if self.root == Some(node) {
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

        node.balance_after_remove();

        pop_then_promote(node, child)
    }

    fn search_node<Q>(&self, key: &Q) -> Result<NodeRef<K, V>, (NodeRef<K, V>, ChildIndex)>
    where
        K: Borrow<Q>,
        Q: Ord + ?Sized,
    {
        let mut current = self.root.unwrap();
        loop {
            let idx = match key.cmp(current.key()) {
                std::cmp::Ordering::Less => ChildIndex::Left,
                std::cmp::Ordering::Equal => return Ok(current),
                std::cmp::Ordering::Greater => ChildIndex::Right,
            };
            current = current.child(idx).ok_or((current, idx))?
        }
    }
}

impl<K, V> Drop for RedBlackTree<K, V> {
    fn drop(&mut self) {
        drop(unsafe { std::ptr::read(self) }.into_iter());
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

    pub const fn is_empty(&self) -> bool {
        self.root.is_none()
    }

    pub const fn len(&self) -> usize {
        self.len
    }
}

impl<K: Ord, V> RedBlackTree<K, V> {
    pub fn insert(&mut self, key: K, value: V) -> Option<(K, V)> {
        if self.root.is_none() {
            self.root = Some(NodeRef::new_root(key, value));
            return None;
        }
        match self.search_node(&key) {
            Ok(found) => {
                // replace
                let parent = found.parent().unwrap();
                let new_node = NodeRef::new(parent, key, value);
                let ret = self.remove_node(found);
                self.insert_node(new_node, parent);
                Some(ret)
            }
            Err((target, _)) => {
                let new_node = NodeRef::new(target, key, value);
                self.insert_node(new_node, target);
                None
            }
        }
    }

    pub fn remove<Q>(&mut self, key: &Q) -> Option<V>
    where
        K: Borrow<Q>,
        Q: Ord + ?Sized,
    {
        let found = self.search_node(key).ok()?;
        Some(self.remove_node(found).1)
    }

    pub fn get<Q>(&self, key: &Q) -> Option<&V>
    where
        K: Borrow<Q>,
        Q: Ord + ?Sized,
    {
        self.search_node(key).ok().map(|n| n.value())
    }
}
