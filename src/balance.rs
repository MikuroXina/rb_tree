use crate::{
    node::{ChildIndex, Color, NodeRef},
    RedBlackTree,
};

impl<K, V> RedBlackTree<K, V> {
    fn rotate(&mut self, target: NodeRef<K, V>, pivot_idx: ChildIndex) -> NodeRef<K, V> {
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

        if let Some((parent, idx)) = target.parent().zip(target.index_on_parent()) {
            // update `parent`'s child
            parent.set_child(idx, Some(pivot));
        } else {
            pivot.make_root();
            self.root = Some(pivot);
        }
        // update `pivot`'s child
        pivot.set_child(!pivot_idx, Some(target));
        // update `node`'s child
        target.set_child(pivot_idx, be_moved);
        pivot
    }

    pub(crate) fn balance_after_insert(&mut self, mut target: NodeRef<K, V>) {
        loop {
            if target.parent().is_none() || target.parent().unwrap().is_black() {
                // if the parent is black or none, the tree is well balanced.
                return;
            }
            // the parent is red
            if target.grandparent().is_none() {
                // if the parent is red and no grandparent exists, the root parent will be black.
                target.parent().unwrap().set_color(Color::Black);
                return;
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
            return;
        }
    }

    pub(crate) fn balance_after_remove(&mut self, mut target: NodeRef<K, V>) {
        while let Some(parent) = target.parent() {
            let sibling = target.sibling();
            let close_nephew = target.close_nephew();
            let distant_nephew = target.distant_nephew();
            if sibling.map_or(false, NodeRef::is_red) {
                // if the sibling is red, the parent and the nephews are black:
                //       [parent]
                //        /   \
                //    target (sibling)
                //            /    \
                // [close_nephew] [distant_nephew]
                self.rotate(parent, !target.index_on_parent().unwrap());
                parent.set_color(Color::Red);
                sibling.unwrap().set_color(Color::Black);
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
                self.rotate(parent, !target.index_on_parent().unwrap());
                sibling.unwrap().set_color(parent.color());
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
                let sibling = sibling.unwrap();
                self.rotate(sibling, target.index_on_parent().unwrap());
                sibling.set_color(Color::Red);
                close_nephew.unwrap().set_color(Color::Black);
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
                if let Some(sibling) = sibling {
                    sibling.set_color(Color::Red)
                }
                parent.set_color(Color::Black);
                break;
            }
            // if the parent and sibling and nephews are all black:
            if let Some(sibling) = sibling {
                sibling.set_color(Color::Red)
            }
            // the parent node needs to re-balance.
            target = parent;
        }
    }
}
