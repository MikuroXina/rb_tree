#[cfg(test)]
mod tests;

use crate::node::{ChildIndex, Color, Node};

impl<K, V> Node<K, V> {
    pub(crate) fn rotate(self, pivot_idx: ChildIndex, root: &mut Option<Self>) -> Node<K, V> {
        //           [target]
        //            /   \
        //        [pivot] [be_fallen]
        //         /   \
        // [be_risen] [be_moved]
        //            ↓
        //        [pivot]
        //         /   \
        // [be_risen] [target]
        //             /   \
        //     [be_moved] [be_fallen]
        let pivot = self.child(pivot_idx).expect("pivot must be found");
        let be_moved = pivot.child(!pivot_idx);

        // SAFETY: The operations in this order is ok:
        // 1. Set `be_moved` into `target`'s child.
        // 2. Get parent of `target.
        // 3. Set `pivot` into `parent`'s child, or make it root.
        // 4. Set `target` into `pivot`'s child.
        unsafe {
            self.set_child(pivot_idx, be_moved);
            if let Some((idx, parent)) = self.index_and_parent() {
                parent.set_child(idx, pivot);
            } else {
                *root = pivot.make_root();
            }
            pivot.set_child(!pivot_idx, self);
        }

        pivot
    }

    pub(crate) fn balance_after_insert(mut self, root: &mut Option<Self>) {
        loop {
            if self.parent().map_or(true, |p| p.is_black()) {
                // if the parent is black or none, the tree is well balanced.
                break;
            }
            // the parent is red
            if self.grandparent().is_none() {
                // if the parent is red and no grandparent exists, the root parent will be black.
                self.parent().unwrap().set_color(Color::Black);
                break;
            }
            // the parent is red and the grandparent exists
            if self.uncle().map_or(false, |uncle| uncle.is_red()) {
                // if the parent and the uncle is red, they will be black and the grandparent will be red.
                self.parent().unwrap().set_color(Color::Black);
                self.uncle().unwrap().set_color(Color::Black);
                let grandparent = self.grandparent().unwrap();
                grandparent.set_color(Color::Red);
                // then nodes above the grandparent needs to re-balance.
                self = grandparent;
                continue;
            }
            // the parent is red and the uncle is black
            if self.parent().unwrap().index_on_parent() != self.index_on_parent() {
                let parent = self.parent().unwrap();
                // if the nodes are connected:
                //   [grandparent]  |  [grandparent]
                //     /     \      |     /     \
                // (parent) [uncle] | [uncle] (parent)
                //      \           |           /
                //     (target)     |      (target)
                parent.rotate(self.index_on_parent().unwrap(), root);
                // then rotated:
                //   [grandparent]  |  [grandparent]
                //     /     \      |     /     \
                // (target) [uncle] | [uncle] (target)
                //   /              |             \
                // (parent)         |          (parent)
                self = parent;
            }
            // the nodes are connected:
            //   [grandparent]  |  [grandparent]
            //     /     \      |     /     \
            // (parent) [uncle] | [uncle] (parent)
            //   /              |             \
            // (target)         |          (target)
            self.parent().unwrap().set_color(Color::Black);
            let grandparent = self.grandparent().unwrap();
            grandparent.set_color(Color::Red);
            // then colored:
            //   (grandparent)  |  (grandparent)
            //     /     \      |     /     \
            // [parent] [uncle] | [uncle] [parent]
            //   /              |             \
            // (target)         |          (target)
            grandparent.rotate(self.index_on_parent().unwrap(), root);
            // finished:
            //   [parent]             |          [parent]
            //    /    \              |           /    \
            // (target) (grandparent) | (grandparent) (target)
            //            \           |      /
            //          [uncle]       |   [uncle]
            break;
        }
        self.assert_tree(root);
    }

    /// Balances the tree for removing `target`. Then `target` will be removed from the tree. You must deallocate `target` or it leaks the memory.
    ///
    /// # Panics
    ///
    /// Panics if `target` is the root, red, or has no children.
    pub(crate) fn balance_after_remove(self, root: &mut Option<Self>) {
        debug_assert!(self.parent().is_some());
        debug_assert!(self.is_black());
        debug_assert!(self.left().is_none());
        debug_assert!(self.right().is_none());

        let (idx, mut parent) = self.index_and_parent().unwrap();
        let mut sibling = parent.child(!idx).unwrap();
        let mut close_nephew = sibling.child(idx);
        let mut distant_nephew = sibling.child(!idx);
        // Safety: `target` must be removed from the tree.
        unsafe {
            parent.clear_child(idx);
        }

        loop {
            if sibling.is_red() {
                // if the sibling is red, the parent and the nephews are black:
                //       [parent]
                //        /   \
                //    target (sibling)
                //            /    \
                // [close_nephew] [distant_nephew]
                debug_assert!(parent.is_black());
                debug_assert!(close_nephew.map_or(true, |n| n.is_black()));
                debug_assert!(distant_nephew.map_or(true, |n| n.is_black()));
                parent.rotate(!idx, root);
                parent.set_color(Color::Red);
                sibling.set_color(Color::Black);
                sibling = close_nephew.unwrap();
                distant_nephew = sibling.child(!idx);
                close_nephew = sibling.child(idx);
                // then:
                //       [sibling]
                //        /   \
                //   (parent) [distant_nephew]
                //    /    \
                // target [close_nephew]
            }
            if distant_nephew.map_or(false, |n| n.is_red()) {
                // if the sibling is black and the distant nephew is red:
                //     parent
                //      /  \
                // target [sibling]
                //           \
                //    (distant_nephew)
                parent.rotate(!idx, root);
                sibling.set_color(parent.color());
                parent.set_color(Color::Black);
                distant_nephew.unwrap().set_color(Color::Black);
                // then:
                //      sibling
                //       /  \
                // [parent] [distant_nephew]
                //     /
                // target
                break;
            }
            if close_nephew.map_or(false, |n| n.is_red()) {
                // if the sibling is black and the close nephew is red:
                //        parent
                //         /  \
                //    target [sibling]
                //             /   \
                // (close_nephew) [distant_nephew]
                sibling.rotate(idx, root);
                sibling.set_color(Color::Red);
                close_nephew.unwrap().set_color(Color::Black);
                distant_nephew = Some(sibling);
                sibling = close_nephew.unwrap();
                // then:
                //     parent
                //      /  \
                // target [close_nephew]
                //           \
                //      (sibling)
                continue;
            }
            if parent.is_red() {
                // if the parent is red, the sibling and the nephews are black:
                //       (parent)
                //        /   \
                //    target [sibling]
                //            /    \
                // [close_nephew] [distant_nephew]
                sibling.set_color(Color::Red);
                parent.set_color(Color::Black);
                break;
            }
            // if the parent and sibling and nephews are all black:
            sibling.set_color(Color::Red);
            // the parent node needs to re-balance.
            if let Some(grandparent) = parent.parent() {
                parent = grandparent;
            } else {
                // one black nodes are removed from all paths.
                break;
            }
        }
        self.assert_tree(root);
    }

    #[cfg(not(test))]
    #[inline]
    fn assert_tree(self, _: &Option<Self>) {}

    #[cfg(test)]
    fn assert_tree(self, root: &Option<Self>) {
        if root.is_none() {
            return;
        }
        let mut stack = vec![(0usize, root.unwrap())];
        while let Some((black_count, node)) = stack.pop() {
            if node.is_red() {
                assert!(node.left().map_or(true, |n| n.is_black()));
                assert!(node.right().map_or(true, |n| n.is_black()));
            }
            let is_black = node.is_black() as usize;
            let children = node.children();
            if let Some(c) = children.0 {
                let back_ptr = c.parent().unwrap();
                assert_eq!(back_ptr, node);
                stack.push((black_count + is_black, c));
            }
            if let Some(c) = children.1 {
                let back_ptr = c.parent().unwrap();
                assert_eq!(back_ptr, node);
                stack.push((black_count + is_black, c));
            }
        }
    }
}
