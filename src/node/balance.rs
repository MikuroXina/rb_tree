#[cfg(test)]
mod tests;

use super::{ChildIndex, Color, NodeRef};

impl<K, V> NodeRef<K, V> {
    fn rotate(self, pivot_idx: ChildIndex) -> Self {
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
        let mut pivot = self.child(pivot_idx).expect("pivot must be found");
        let be_moved = pivot.child(!pivot_idx);

        if let Some((parent, idx)) = self.parent().zip(self.index_on_parent()) {
            // update `parent`'s child
            parent.set_child(idx, Some(pivot));
        } else {
            unsafe { pivot.0.as_mut() }.parent = None;
        }
        // update `pivot`'s child
        pivot.set_child(!pivot_idx, Some(self));
        // update `node`'s child
        self.set_child(pivot_idx, be_moved);
        pivot
    }

    pub fn balance_after_insert(mut self) {
        loop {
            if self.parent().is_none() || self.parent().unwrap().is_black() {
                // if the parent is black or none, the tree is well balanced.
                return;
            }
            // the parent is red
            if self.grandparent().is_none() {
                // if the parent is red and no grandparent exists, the root parent will be black.
                self.parent().unwrap().set_color(Color::Black);
                return;
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
                //     (self)       |      (self)
                parent.rotate(self.index_on_parent().unwrap());
                // then rotated:
                //   [grandparent] |  [grandparent]
                //     /     \     |     /     \
                // (self) [uncle]  | [uncle] (self)
                //   /             |             \
                // (parent)        |          (parent)
                self = parent;
            }
            // the nodes are connected:
            //   [grandparent]  |  [grandparent]
            //     /     \      |     /     \
            // (parent) [uncle] | [uncle] (parent)
            //   /              |             \
            // (self)           |          (self)
            self.parent().unwrap().set_color(Color::Black);
            let grandparent = self.grandparent().unwrap();
            grandparent.set_color(Color::Red);
            // then colored:
            //   (grandparent)  |  (grandparent)
            //     /     \      |     /     \
            // [parent] [uncle] | [uncle] [parent]
            //   /              |             \
            // (self)           |          (self)
            grandparent.rotate(self.index_on_parent().unwrap());
            // finished:
            //   [parent]           |          [parent]
            //    /    \            |           /    \
            // (self) (grandparent) | (grandparent) (self)
            //            \         |      /
            //          [uncle]     |   [uncle]
            return;
        }
    }

    pub fn balance_after_remove(mut self) {
        while let Some(parent) = self.parent() {
            let sibling = self.sibling();
            let close_nephew = self.close_nephew();
            let distant_nephew = self.distant_nephew();
            if sibling.map_or(false, NodeRef::is_red) {
                // if the sibling is red, the parent and the nephews are black:
                //       [parent]
                //        /   \
                //      node (sibling)
                //            /    \
                // [close_nephew] [distant_nephew]
                parent.rotate(!self.index_on_parent().unwrap());
                parent.set_color(Color::Red);
                sibling.unwrap().set_color(Color::Black);
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
                parent.rotate(!self.index_on_parent().unwrap());
                sibling.unwrap().set_color(parent.color());
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
                let sibling = sibling.unwrap();
                sibling.rotate(self.index_on_parent().unwrap());
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
            self = parent;
        }
    }
}
