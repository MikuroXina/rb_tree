use crate::{
    node::{ChildIndex, Color, NodeRef},
    RedBlackTree,
};

impl<K, V> RedBlackTree<K, V> {
    pub(crate) fn rotate(&mut self, target: NodeRef<K, V>, pivot_idx: ChildIndex) -> NodeRef<K, V> {
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
        let pivot = target.child(pivot_idx).expect("pivot must be found");
        let be_moved = pivot.child(!pivot_idx);

        // SAFETY:
        unsafe {
            if let Some((idx, parent)) = target.index_and_parent() {
                // update `parent`'s child
                parent.clear_child(idx);
                parent.set_child(idx, pivot);
            } else {
                self.root = pivot.make_root();
            }
            // update `pivot`'s child
            pivot.set_child(!pivot_idx, target);
            // update `node`'s child
            target.set_child(pivot_idx, be_moved);
        }

        if let Some((index, parent)) = pivot.index_and_parent() {
            debug_assert_eq!(parent.child(index), Some(pivot));
            debug_assert_eq!(pivot.parent(), Some(parent));
        } else {
            debug_assert_eq!(self.root, Some(pivot));
            debug_assert!(pivot.parent().is_none());
        }
        debug_assert_eq!(pivot.child(!pivot_idx), Some(target));
        debug_assert_eq!(target.parent(), Some(pivot));
        debug_assert_eq!(target.child(pivot_idx), be_moved);

        pivot
    }

    pub(crate) fn balance_after_insert(&mut self, mut target: NodeRef<K, V>) {
        loop {
            if target.parent().is_none() || target.parent().unwrap().is_black() {
                // if the parent is black or none, the tree is well balanced.
                break;
            }
            // the parent is red
            if target.grandparent().is_none() {
                // if the parent is red and no grandparent exists, the root parent will be black.
                target.parent().unwrap().set_color(Color::Black);
                break;
            }
            // the parent is red and the grandparent exists
            if target.uncle().map_or(false, |uncle| uncle.is_red()) {
                // if the parent and the uncle is red, they will be black and the grandparent will be red.
                target.parent().unwrap().set_color(Color::Black);
                target.uncle().unwrap().set_color(Color::Black);
                let grandparent = target.grandparent().unwrap();
                grandparent.set_color(Color::Red);
                // then nodes above the grandparent needs to re-balance.
                target = grandparent;
                continue;
            }
            // the parent is red and the uncle is black
            if target.parent().unwrap().index_on_parent() != target.index_on_parent() {
                let parent = target.parent().unwrap();
                // if the nodes are connected:
                //   [grandparent]  |  [grandparent]
                //     /     \      |     /     \
                // (parent) [uncle] | [uncle] (parent)
                //      \           |           /
                //     (target)     |      (target)
                self.rotate(parent, target.index_on_parent().unwrap());
                // then rotated:
                //   [grandparent]  |  [grandparent]
                //     /     \      |     /     \
                // (target) [uncle] | [uncle] (target)
                //   /              |             \
                // (parent)         |          (parent)
                target = parent;
            }
            // the nodes are connected:
            //   [grandparent]  |  [grandparent]
            //     /     \      |     /     \
            // (parent) [uncle] | [uncle] (parent)
            //   /              |             \
            // (target)         |          (target)
            target.parent().unwrap().set_color(Color::Black);
            let grandparent = target.grandparent().unwrap();
            grandparent.set_color(Color::Red);
            // then colored:
            //   (grandparent)  |  (grandparent)
            //     /     \      |     /     \
            // [parent] [uncle] | [uncle] [parent]
            //   /              |             \
            // (target)         |          (target)
            self.rotate(grandparent, target.index_on_parent().unwrap());
            // finished:
            //   [parent]             |          [parent]
            //    /    \              |           /    \
            // (target) (grandparent) | (grandparent) (target)
            //            \           |      /
            //          [uncle]       |   [uncle]
            break;
        }
        self.assert_tree();
    }

    /// Balances the tree for removing `target`. Then `target` will be removed from the tree. You must deallocate `target` or it leaks the memory.
    ///
    /// # Panics
    ///
    /// Panics if `target` is the root, red, or has no children.
    pub(crate) fn balance_after_remove(&mut self, target: NodeRef<K, V>) {
        debug_assert!(target.parent().is_some());
        debug_assert!(target.is_black());
        debug_assert!(target.left().is_none());
        debug_assert!(target.right().is_none());

        let (idx, mut parent) = target.index_and_parent().unwrap();
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
                self.rotate(parent, !idx);
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
                continue;
            }
            if distant_nephew.map_or(false, |n| n.is_red()) {
                // if the sibling is black and the distant nephew is red:
                //     parent
                //      /  \
                // target [sibling]
                //           \
                //    (distant_nephew)
                self.rotate(parent, !idx);
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
                self.rotate(sibling, idx);
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
        self.assert_tree();
    }

    #[cfg(not(test))]
    #[inline]
    fn assert_tree(&self) {}

    #[cfg(test)]
    fn assert_tree(&self) {
        if self.root.is_none() {
            return;
        }
        let mut stack = vec![(0usize, self.root.unwrap())];
        let mut node_count = 0;
        while let Some((black_count, node)) = stack.pop() {
            node_count += 1;
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
        self.root.unwrap().black_height();
        assert_eq!(self.len, node_count);
    }
}
