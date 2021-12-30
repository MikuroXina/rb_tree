#![allow(dead_code)]

mod iter;
mod mem;
mod node;
#[cfg(test)]
mod tests;

pub use iter::IntoIter;

use node::{ChildIndex, Node, NodeRef};

use std::{borrow::Borrow, marker::PhantomData, ptr::NonNull};

type Ptr<T> = Option<NonNull<T>>;

pub struct RedBlackTree<K, V> {
    root: Ptr<Node<K, V>>,
    len: usize,
    _phantom: PhantomData<Box<Node<K, V>>>,
}

// private methods
impl<K, V> RedBlackTree<K, V> {
    fn first_node(&self) -> Option<NodeRef<K, V>> {
        let mut current = NodeRef::from(self.root?);
        while let Some(left) = current.child(ChildIndex::Left) {
            current = left;
        }
        Some(current)
    }

    fn last_node(&self) -> Option<NodeRef<K, V>> {
        let mut current = NodeRef::from(self.root?);
        while let Some(right) = current.child(ChildIndex::Right) {
            current = right;
        }
        Some(current)
    }
}

// private methods
impl<K: Ord, V> RedBlackTree<K, V> {
    fn insert_node(&mut self, new_node: Box<Node<K, V>>, target: Option<NodeRef<K, V>>) {
        if target.is_none() {
            let ptr = NonNull::new(Box::into_raw(new_node)).unwrap();
            self.root = Some(ptr);
            return;
        }
        let target = target.unwrap();
        let ptr = NonNull::new(Box::into_raw(new_node)).unwrap();
        let new_node: NodeRef<K, V> = ptr.into();
        let idx = target.which_to_insert(new_node.key());
        target.set_child(idx, Some(new_node));

        new_node.balance_after_insert();
    }
    fn remove_node(&mut self, node: NodeRef<K, V>) -> (K, V) {
        if self.root == Some(node.as_raw()) {
            return unsafe { Box::from_raw(self.root.take().unwrap().as_ptr()) }.into_element();
        }

        fn pop_then_promote<K, V>(node: NodeRef<K, V>, child: Option<NodeRef<K, V>>) -> (K, V) {
            if let Some(parent) = node.parent() {
                parent.set_child(node.index_on_parent().unwrap(), child);
            }
            unsafe { Box::from_raw(node.as_raw().as_ptr()) }.into_element()
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

    fn search_node<Q>(&self, key: &Q) -> Option<NodeRef<K, V>>
    where
        K: Borrow<Q>,
        Q: Ord + ?Sized,
    {
        let mut current = NodeRef::from(self.root?);
        loop {
            let index = current.which_to_insert(key);
            if let Some(child) = current.child(index) {
                current = child;
            } else {
                return Some(current);
            }
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
        let found = self.search_node(&key);
        if found.as_ref().map(|found| found.key().borrow()) == Some(&key) {
            // replace
            let found = found.unwrap();
            let parent = found.parent();
            let new_node = Node::new(parent.as_ref().map(|p| p.as_raw()), key, value);
            let ret = self.remove_node(found);
            self.insert_node(new_node.into(), parent);
            return Some(ret);
        }
        let parent = found.and_then(|f| f.parent());
        let new_node = Node::new(parent.as_ref().map(|p| p.as_raw()), key, value);
        self.insert_node(new_node.into(), parent);
        None
    }

    pub fn remove<Q>(&mut self, key: &Q) -> Option<V>
    where
        K: Borrow<Q>,
        Q: Ord + ?Sized,
    {
        let found = self.search_node(key)?;
        (found.key() == key).then(|| self.remove_node(found).1)
    }

    pub fn get<Q>(&self, key: &Q) -> Option<&V>
    where
        K: Borrow<Q>,
        Q: Ord + ?Sized,
    {
        self.search_node(key).map(|n| n.value())
    }
}
