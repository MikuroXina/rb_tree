#![allow(dead_code)]

mod node;

use node::{ChildIndex, Color, Node};

use std::{marker::PhantomData, ptr::NonNull};

type Ptr<T> = Option<NonNull<T>>;

pub struct RedBlackTree<K, V> {
    root: Ptr<Node<K, V>>,
    len: usize,
    _phantom: PhantomData<Box<Node<K, V>>>,
}

// private methods
impl<K: Ord, V> RedBlackTree<K, V> {
    fn rotate(&mut self, node: NonNull<Node<K, V>>, pivot_idx: ChildIndex) -> NonNull<Node<K, V>> {
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
        let parent = Node::parent(node);
        let pivot = Node::child(node, pivot_idx).expect("pivot must be found");
        let be_moved = Node::child(pivot, !pivot_idx);

        match Node::index_on_parent(node) {
            // update `parent`'s child
            Some(idx) => {
                Node::set_child(parent.unwrap(), idx, Some(pivot));
            }
            None => {
                self.root = Some(pivot);
            }
        }
        // update `pivot`'s child
        Node::set_child(pivot, !pivot_idx, Some(node));
        // update `node`'s child
        Node::set_child(node, pivot_idx, be_moved);
        pivot
    }

    fn insert_node(&mut self, new_node: Box<Node<K, V>>, target: Ptr<Node<K, V>>) {
        let ptr = NonNull::new(Box::into_raw(new_node)).unwrap();
        if target.is_none() {
            self.root = Some(ptr);
            return;
        }
        let target = target.unwrap();
        let idx = Node::which_to_insert(ptr, target);
        Node::set_child(target, idx, Some(ptr));
        let mut new_node = ptr;

        // re-balance
        while let Some(parent) = Node::parent(new_node) {
            if Node::is_black(parent) {
                // if the parent is black, the tree is well balanced.
                return;
            }
            // the parent is red
            if Node::parent(parent).is_none() {
                // if the parent is red and no grandparent exists, the root parent will be black.
                Node::set_color(parent, Color::Black);
                return;
            }
            // the parent is red and the grandparent exists
            if Node::uncle(new_node).map_or(false, Node::is_red) {
                // if the parent and the uncle is red, they will be black and the grandparent will be red.
                Node::set_color(parent, Color::Black);
                Node::set_color(Node::uncle(new_node).unwrap(), Color::Black);
                let grandparent = Node::grandparent(new_node).unwrap();
                Node::set_color(grandparent, Color::Red);
                // then nodes above the grandparent needs to re-balance.
                new_node = grandparent;
                continue;
            }
            // the parent is red and the uncle is black
            if Node::index_on_parent(parent) != Node::index_on_parent(new_node) {
                // if the nodes are connected:
                //   [grandparent]  |  [grandparent]
                //     /     \      |     /     \
                // (parent) [uncle] | [uncle] (parent)
                //      \           |           /
                //    (new_node)    |      (new_node)
                self.rotate(parent, Node::index_on_parent(new_node).unwrap());
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
            Node::set_color(parent, Color::Black);
            Node::set_color(Node::grandparent(new_node).unwrap(), Color::Red);
            // then colored:
            //   (grandparent)  |  (grandparent)
            //     /     \      |     /     \
            // [parent] [uncle] | [uncle] [parent]
            //   /              |             \
            // (new_node)       |          (new_node)
            self.rotate(
                Node::grandparent(new_node).unwrap(),
                Node::index_on_parent(new_node).unwrap(),
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
    fn remove_node(&mut self, node: NonNull<Node<K, V>>) -> (K, V) {
        if self.root == Some(node) {
            return unsafe { Box::from_raw(self.root.take().unwrap().as_ptr()) }.into_element();
        }

        fn pop_then_promote<K, V>(node: NonNull<Node<K, V>>, child: Ptr<Node<K, V>>) -> (K, V) {
            if let Some(parent) = Node::parent(node) {
                Node::set_child(parent, Node::index_on_parent(node).unwrap(), child);
            }
            unsafe { Box::from_raw(node.as_ptr()) }.into_element()
        }

        let element = match (
            Node::child(node, ChildIndex::Left),
            Node::child(node, ChildIndex::Right),
        ) {
            (None, None) => return pop_then_promote(node, None),
            (None, Some(right)) => return pop_then_promote(node, Some(right)),
            (Some(left), None) => return pop_then_promote(node, Some(left)),
            (Some(left), Some(right)) => {
                let mut min_in_right = right;
                while let Some(lesser) = Node::child(min_in_right, ChildIndex::Left) {
                    min_in_right = lesser;
                }
                Node::set_child(
                    Node::parent(min_in_right).unwrap(),
                    Node::index_on_parent(min_in_right).unwrap(),
                    None,
                );
                Node::set_color(min_in_right, Node::color(node));
                Node::set_child(min_in_right, ChildIndex::Left, Some(left));
                let right_top = if min_in_right == right {
                    None
                } else {
                    Some(right)
                };
                Node::set_child(min_in_right, ChildIndex::Right, right_top);
                pop_then_promote(node, Some(min_in_right))
            }
        };

        // re-balance
        let mut node = node;
        while let Some(parent) = Node::parent(node) {
            let sibling = Node::sibling(node).unwrap();
            let close_nephew = Node::close_nephew(node);
            let distant_nephew = Node::distant_nephew(node);
            if Node::is_red(sibling) {
                // if the sibling is red, the parent and the nephews are black:
                //       [parent]
                //        /   \
                //      node (sibling)
                //            /    \
                // [close_nephew] [distant_nephew]
                self.rotate(parent, !Node::index_on_parent(node).unwrap());
                Node::set_color(parent, Color::Red);
                Node::set_color(sibling, Color::Black);
                // then:
                //       [sibling]
                //        /   \
                //   (parent) [distant_nephew]
                //    /    \
                // node [close_nephew]
                continue;
            }
            if distant_nephew.map_or(false, Node::is_red) {
                // if the sibling is black and the distant nephew is red:
                //   parent
                //    /  \
                // node [sibling]
                //         \
                //    (distant_nephew)
                self.rotate(parent, !Node::index_on_parent(node).unwrap());
                Node::set_color(sibling, Node::color(parent));
                Node::set_color(parent, Color::Black);
                Node::set_color(distant_nephew.unwrap(), Color::Black);
                // then:
                //      sibling
                //       /  \
                // [parent] [distant_nephew]
                //     /
                //   node
                break;
            }
            if close_nephew.map_or(false, Node::is_red) {
                // if the sibling is black and the close nephew is red:
                //        parent
                //         /  \
                //      node [sibling]
                //             /   \
                // (close_nephew) [distant_nephew]
                self.rotate(sibling, Node::index_on_parent(node).unwrap());
                Node::set_color(sibling, Color::Red);
                Node::set_color(close_nephew.unwrap(), Color::Black);
                // then:
                //   parent
                //    /  \
                // node [close_nephew]
                //         \
                //      (sibling)
                continue;
            }
            if Node::color(parent) == Color::Red {
                // if the parent is red, the sibling and the nephews are black:
                //       (parent)
                //        /   \
                //      node [sibling]
                //            /    \
                // [close_nephew] [distant_nephew]
                Node::set_color(sibling, Color::Red);
                Node::set_color(parent, Color::Black);
                break;
            }
            // if the parent and sibling and nephews are all black:
            Node::set_color(sibling, Color::Red);
            // the parent node needs to re-balance.
            node = parent;
        }
        element
    }
}

impl<K, V> Drop for RedBlackTree<K, V> {
    fn drop(&mut self) {
        todo!()
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
    pub fn insert(&mut self, key: K, value: V) -> Option<V> {
        todo!()
    }
}
