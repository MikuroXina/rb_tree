#![allow(dead_code)]

mod iter;
mod mem;
mod node;
#[cfg(test)]
mod tests;

pub use iter::IntoIter;

use node::{ChildIndex, Color, Node, NodeRef};

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
    fn rotate(&mut self, node: NodeRef<K, V>, pivot_idx: ChildIndex) -> NodeRef<K, V> {
        //           [node]
        //            /   \
        //        [pivot] [be_fallen]
        //         /   \
        // [be_risen] [be_moved]
        //            ↓
        //        [pivot]
        //         /   \
        // [be_risen] [node]
        //             /   \
        //     [be_moved] [be_fallen]
        let parent = node.parent();
        let pivot = node.child(pivot_idx).expect("pivot must be found");
        let be_moved = pivot.child(!pivot_idx);

        match node.index_on_parent() {
            // update `parent`'s child
            Some(idx) => {
                parent.unwrap().set_child(idx, Some(pivot));
            }
            None => {
                self.root = Some(pivot.as_raw());
            }
        }
        // update `pivot`'s child
        pivot.set_child(!pivot_idx, Some(node));
        // update `node`'s child
        node.set_child(pivot_idx, be_moved);
        pivot
    }

    fn insert_node(&mut self, new_node: Box<Node<K, V>>, target: Option<NodeRef<K, V>>) {
        if target.is_none() {
            let ptr = NonNull::new(Box::into_raw(new_node)).unwrap();
            self.root = Some(ptr);
            return;
        }
        let target = target.unwrap();
        let ptr = NonNull::new(Box::into_raw(new_node)).unwrap();
        let mut new_node: NodeRef<K, V> = ptr.into();
        let idx = target.which_to_insert(new_node.key());
        target.set_child(idx, Some(new_node));

        // re-balance
        while let Some(parent) = new_node.parent() {
            if parent.is_black() {
                // if the parent is black, the tree is well balanced.
                return;
            }
            // the parent is red
            if parent.parent().is_none() {
                // if the parent is red and no grandparent exists, the root parent will be black.
                parent.set_color(Color::Black);
                return;
            }
            // the parent is red and the grandparent exists
            if new_node.uncle().map_or(false, |uncle| uncle.is_red()) {
                // if the parent and the uncle is red, they will be black and the grandparent will be red.
                parent.set_color(Color::Black);
                new_node.uncle().unwrap().set_color(Color::Black);
                let grandparent = new_node.grandparent().unwrap();
                grandparent.set_color(Color::Red);
                // then nodes above the grandparent needs to re-balance.
                new_node = grandparent;
                continue;
            }
            // the parent is red and the uncle is black
            if parent.index_on_parent() != new_node.index_on_parent() {
                // if the nodes are connected:
                //   [grandparent]  |  [grandparent]
                //     /     \      |     /     \
                // (parent) [uncle] | [uncle] (parent)
                //      \           |           /
                //    (new_node)    |      (new_node)
                self.rotate(parent, new_node.index_on_parent().unwrap());
                // then rotated:
                //   [grandparent]    |  [grandparent]
                //     /     \        |     /     \
                // (new_node) [uncle] | [uncle] (new_node)
                //   /                |             \
                // (parent)           |          (parent)
                new_node = parent;
            }
            // the nodes are connected:
            //   [grandparent]  |  [grandparent]
            //     /     \      |     /     \
            // (parent) [uncle] | [uncle] (parent)
            //   /              |             \
            // (new_node)       |          (new_node)
            parent.set_color(Color::Black);
            new_node.grandparent().unwrap().set_color(Color::Red);
            // then colored:
            //   (grandparent)  |  (grandparent)
            //     /     \      |     /     \
            // [parent] [uncle] | [uncle] [parent]
            //   /              |             \
            // (new_node)       |          (new_node)
            self.rotate(
                new_node.grandparent().unwrap(),
                new_node.index_on_parent().unwrap(),
            );
            // finished:
            //       [parent]           |          [parent]
            //        /    \            |           /    \
            // (new_node) (grandparent) | (grandparent) (new_node)
            //                \         |      /
            //              [uncle]     |   [uncle]
            return;
        }
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

        let element = match (node.child(ChildIndex::Left), node.child(ChildIndex::Right)) {
            (None, None) => return pop_then_promote(node, None),
            (None, Some(right)) => return pop_then_promote(node, Some(right)),
            (Some(left), None) => return pop_then_promote(node, Some(left)),
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
                pop_then_promote(node, Some(min_in_right))
            }
        };

        // re-balance
        let mut node = node;
        while let Some(parent) = node.parent() {
            let sibling = node.sibling().unwrap();
            let close_nephew = node.close_nephew();
            let distant_nephew = node.distant_nephew();
            if sibling.is_red() {
                // if the sibling is red, the parent and the nephews are black:
                //       [parent]
                //        /   \
                //      node (sibling)
                //            /    \
                // [close_nephew] [distant_nephew]
                self.rotate(parent, !node.index_on_parent().unwrap());
                parent.set_color(Color::Red);
                sibling.set_color(Color::Black);
                // then:
                //       [sibling]
                //        /   \
                //   (parent) [distant_nephew]
                //    /    \
                // node [close_nephew]
                continue;
            }
            if distant_nephew.map_or(false, |n| n.is_red()) {
                // if the sibling is black and the distant nephew is red:
                //   parent
                //    /  \
                // node [sibling]
                //         \
                //    (distant_nephew)
                self.rotate(parent, !node.index_on_parent().unwrap());
                sibling.set_color(parent.color());
                parent.set_color(Color::Black);
                distant_nephew.unwrap().set_color(Color::Black);
                // then:
                //      sibling
                //       /  \
                // [parent] [distant_nephew]
                //     /
                //   node
                break;
            }
            if close_nephew.map_or(false, |n| n.is_red()) {
                // if the sibling is black and the close nephew is red:
                //        parent
                //         /  \
                //      node [sibling]
                //             /   \
                // (close_nephew) [distant_nephew]
                self.rotate(sibling, node.index_on_parent().unwrap());
                sibling.set_color(Color::Red);
                close_nephew.unwrap().set_color(Color::Black);
                // then:
                //   parent
                //    /  \
                // node [close_nephew]
                //         \
                //      (sibling)
                continue;
            }
            if parent.is_red() {
                // if the parent is red, the sibling and the nephews are black:
                //       (parent)
                //        /   \
                //      node [sibling]
                //            /    \
                // [close_nephew] [distant_nephew]
                sibling.set_color(Color::Red);
                parent.set_color(Color::Black);
                break;
            }
            // if the parent and sibling and nephews are all black:
            sibling.set_color(Color::Red);
            // the parent node needs to re-balance.
            node = parent;
        }
        element
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
