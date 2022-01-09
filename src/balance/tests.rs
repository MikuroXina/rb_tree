use std::marker::PhantomData;

use crate::{
    node::{ChildIndex, Color, NodeRef},
    RedBlackTree,
};

#[test]
fn test_rotate() {
    let node1 = NodeRef::new(1, ());
    let node2 = NodeRef::new(2, ());
    let node3 = NodeRef::new(3, ());
    let node4 = NodeRef::new(4, ());
    let node5 = NodeRef::new(5, ());
    let node6 = NodeRef::new(6, ());

    // Safety: It is built as:
    //     node3
    //     /   \
    // node2   node4
    //   /      /  \
    // node1 node5 node6
    unsafe {
        node3.set_child(ChildIndex::Left, node2);
        node2.set_child(ChildIndex::Left, node1);
        node3.set_child(ChildIndex::Right, node4);
        node4.set_child(ChildIndex::Left, node5);
        node4.set_child(ChildIndex::Right, node6);
    }

    let mut tree = RedBlackTree {
        root: Some(node3),
        len: 3,
        _phantom: PhantomData,
    };

    tree.rotate(node3, ChildIndex::Right);

    // Rotated tree must be as:
    //       node4
    //       /   \
    //     node3 node6
    //     /   \
    // node2   node5
    //   /
    // node1
    //

    assert_eq!(tree.root, Some(node4));

    assert_eq!(node1.children(), (None, None));
    assert_eq!(node2.children(), (Some(node1), None));
    assert_eq!(node3.children(), (Some(node2), Some(node5)));
    assert_eq!(node4.children(), (Some(node3), Some(node6)));
    assert_eq!(node5.children(), (None, None));
    assert_eq!(node6.children(), (None, None));

    tree.rotate(node3, ChildIndex::Left);

    // Rotated tree must be as:
    //       node4
    //       /   \
    //     node2 node6
    //     /   \
    // node1   node3
    //           \
    //          node5
    //

    assert_eq!(tree.root, Some(node4));

    assert_eq!(node1.children(), (None, None));
    assert_eq!(node2.children(), (Some(node1), Some(node3)));
    assert_eq!(node3.children(), (None, Some(node5)));
    assert_eq!(node4.children(), (Some(node2), Some(node6)));
    assert_eq!(node5.children(), (None, None));
    assert_eq!(node6.children(), (None, None));
}

#[test]
fn test_balance_after_insert() {
    // All cases are inserting the node `1`.
    // Color is expressed like `[black]` or `(red)`.

    {
        // Case 1 - the parent is black:

        let node1 = NodeRef::new(1, ());
        let node3 = NodeRef::new(3, ());
        // Safety: Nodes are connected as:
        //   [3]
        //   /
        // (1)
        unsafe {
            node1.set_color(Color::Red);
            node3.set_color(Color::Black);

            node3.set_child(ChildIndex::Left, node1);
        }
        let mut tree = RedBlackTree {
            root: Some(node3),
            len: 2,
            _phantom: PhantomData,
        };
        tree.balance_after_insert(node1);
        // The tree must not balance for this.
        assert_eq!(tree.root, Some(node3));

        assert_eq!(node1.children(), (None, None));
        assert_eq!(node3.children(), (Some(node1), None));

        assert_eq!(node1.color(), Color::Red);
        assert_eq!(node3.color(), Color::Black);
    }
    {
        // Case 2 - the parent and uncle is red:
        let node1 = NodeRef::new(1, ());
        let node2 = NodeRef::new(2, ());
        let node3 = NodeRef::new(3, ());
        let node4 = NodeRef::new(4, ());
        // Safety: Nodes are connected as:
        //     [3]
        //     / \
        //   (2) (4)
        //   /
        // (1)
        unsafe {
            node1.set_color(Color::Red);
            node2.set_color(Color::Red);
            node3.set_color(Color::Black);
            node4.set_color(Color::Red);

            node3.set_child(ChildIndex::Left, node2);
            node3.set_child(ChildIndex::Right, node4);
            node2.set_child(ChildIndex::Left, node1);
        }
        let mut tree = RedBlackTree {
            root: Some(node3),
            len: 4,
            _phantom: PhantomData,
        };
        tree.balance_after_insert(node1);

        // The tree must color like:
        //     (3)
        //     / \
        //   [2] [4]
        //   /
        // (1)
        assert_eq!(tree.root, Some(node3));

        assert_eq!(node1.children(), (None, None));
        assert_eq!(node2.children(), (Some(node1), None));
        assert_eq!(node3.children(), (Some(node2), Some(node4)));
        assert_eq!(node4.children(), (None, None));

        assert_eq!(node1.color(), Color::Red);
        assert_eq!(node2.color(), Color::Black);
        assert_eq!(node3.color(), Color::Red);
        assert_eq!(node4.color(), Color::Black);
    }
    {
        // Case 3 - the tree is empty:

        let node1 = NodeRef::new(1, ());
        // Nodes are connected as:
        // (1)
        node1.set_color(Color::Red);
        let mut tree = RedBlackTree {
            root: Some(node1),
            len: 1,
            _phantom: PhantomData,
        };
        tree.balance_after_insert(node1);
        // The tree must not balance for this.
        assert_eq!(tree.root, Some(node1));

        assert_eq!(node1.children(), (None, None));

        assert_eq!(node1.color(), Color::Red);
    }
    {
        // Case 4 - the parent is root and red:

        let node1 = NodeRef::new(1, ());
        let node3 = NodeRef::new(3, ());
        // Safety: Nodes are connected as:
        //   (3)
        //   /
        // (1)
        unsafe {
            node1.set_color(Color::Red);
            node3.set_color(Color::Red);

            node3.set_child(ChildIndex::Left, node1);
        }
        let mut tree = RedBlackTree {
            root: Some(node3),
            len: 2,
            _phantom: PhantomData,
        };
        tree.balance_after_insert(node1);
        // The tree must color the root as black.
        assert_eq!(tree.root, Some(node3));

        assert_eq!(node1.children(), (None, None));
        assert_eq!(node3.children(), (Some(node1), None));

        assert_eq!(node1.color(), Color::Red);
        assert_eq!(node3.color(), Color::Black);
    }
    {
        // Case 5 - the parent is uncle, but the uncle is black.
        let node0 = NodeRef::new(0, ());
        let node1 = NodeRef::new(1, ());
        let node3 = NodeRef::new(3, ());
        let node4 = NodeRef::new(4, ());
        // Safety: Nodes are connected as:
        //   [3]
        //   / \
        // (0) [4]
        //   \
        //   (1)
        unsafe {
            node0.set_color(Color::Red);
            node1.set_color(Color::Red);
            node3.set_color(Color::Black);
            node4.set_color(Color::Black);

            node3.set_child(ChildIndex::Left, node0);
            node3.set_child(ChildIndex::Right, node4);
            node0.set_child(ChildIndex::Right, node1);
        }
        let mut tree = RedBlackTree {
            root: Some(node3),
            len: 4,
            _phantom: PhantomData,
        };
        tree.balance_after_insert(node1);

        // The tree must balance as:
        //   [1]
        //   / \
        // (0) (3)
        //       \
        //       [4]
        assert_eq!(tree.root, Some(node1));

        assert_eq!(node0.children(), (None, None));
        assert_eq!(node1.children(), (Some(node0), Some(node3)));
        assert_eq!(node3.children(), (None, Some(node4)));
        assert_eq!(node4.children(), (None, None));

        assert_eq!(node0.color(), Color::Red);
        assert_eq!(node1.color(), Color::Black);
        assert_eq!(node3.color(), Color::Red);
        assert_eq!(node4.color(), Color::Black);
    }
}

#[test]
fn test_balance_after_remove() {
    // All cases are removing the node `1`.
    // Color is expressed like `[black]` or `(red)`.

    {
        // Case 1 - the parent, sibling and nephews are black:

        let node1 = NodeRef::new(1, ());
        let node2 = NodeRef::new(2, ());
        let node3 = NodeRef::new(3, ());
        let node4 = NodeRef::new(4, ());
        let node5 = NodeRef::new(5, ());
        // Safety: It is built as:
        //   [2]
        //   / \
        // [1] [4]
        //     / \
        //   [3] [5]
        unsafe {
            node2.set_child(ChildIndex::Left, node1);
            node2.set_child(ChildIndex::Right, node4);
            node4.set_child(ChildIndex::Left, node3);
            node4.set_child(ChildIndex::Right, node5);

            node1.set_color(Color::Black);
            node2.set_color(Color::Black);
            node3.set_color(Color::Black);
            node4.set_color(Color::Black);
            node5.set_color(Color::Black);
        }
        let mut tree = RedBlackTree {
            root: Some(node2),
            len: 4,
            _phantom: PhantomData,
        };

        tree.balance_after_remove(node1);
        // Safety: Removed node must be deallocated.
        unsafe {
            node1.deallocate();
        }

        // Balanced tree must be as:
        //   [2]
        //     \
        //     (4)
        //     / \
        //   [3] [5]
        assert_eq!(tree.root, Some(node2));

        assert_eq!(node2.children(), (None, Some(node4)));
        assert_eq!(node3.children(), (None, None));
        assert_eq!(node4.children(), (Some(node3), Some(node5)));
        assert_eq!(node5.children(), (None, None));

        assert!(node2.is_black());
        assert!(node3.is_black());
        assert!(node4.is_red());
        assert!(node5.is_black());
    }
    {
        // Case 2 - the sibling is red:

        let node1 = NodeRef::new(1, ());
        let node2 = NodeRef::new(2, ());
        let node3 = NodeRef::new(3, ());
        let node4 = NodeRef::new(4, ());
        let node5 = NodeRef::new(5, ());
        // Safety: It is built as:
        //   [2]
        //   / \
        // [1] (4)
        //     / \
        //   [3] [5]
        unsafe {
            node2.set_child(ChildIndex::Left, node1);
            node2.set_child(ChildIndex::Right, node4);
            node4.set_child(ChildIndex::Left, node3);
            node4.set_child(ChildIndex::Right, node5);

            node1.set_color(Color::Black);
            node2.set_color(Color::Black);
            node3.set_color(Color::Black);
            node4.set_color(Color::Red);
            node5.set_color(Color::Black);
        }
        let mut tree = RedBlackTree {
            root: Some(node2),
            len: 4,
            _phantom: PhantomData,
        };

        tree.balance_after_remove(node1);
        // Safety: Removed node must be deallocated.
        unsafe {
            node1.deallocate();
        }

        // Balanced tree must be as:
        //   [4]
        //   / \
        // [2] [5]
        //   \
        //   (3)
        assert_eq!(tree.root, Some(node4));

        assert_eq!(node2.children(), (None, Some(node3)));
        assert_eq!(node3.children(), (None, None));
        assert_eq!(node4.children(), (Some(node2), Some(node5)));
        assert_eq!(node5.children(), (None, None));

        assert!(node2.is_black());
        assert!(node3.is_red());
        assert!(node4.is_black());
        assert!(node5.is_black());
    }
    {
        // Case 3 - the sibling and nephews are black, but the parent is red:

        let node1 = NodeRef::new(1, ());
        let node2 = NodeRef::new(2, ());
        let node3 = NodeRef::new(3, ());
        let node4 = NodeRef::new(4, ());
        let node5 = NodeRef::new(5, ());
        // Safety: It is built as:
        //   (2)
        //   / \
        // [1] [4]
        //     / \
        //   [3] [5]
        unsafe {
            node2.set_child(ChildIndex::Left, node1);
            node2.set_child(ChildIndex::Right, node4);
            node4.set_child(ChildIndex::Left, node3);
            node4.set_child(ChildIndex::Right, node5);

            node1.set_color(Color::Black);
            node2.set_color(Color::Red);
            node3.set_color(Color::Black);
            node4.set_color(Color::Black);
            node5.set_color(Color::Black);
        }
        let mut tree = RedBlackTree {
            root: Some(node2),
            len: 4,
            _phantom: PhantomData,
        };

        tree.balance_after_remove(node1);
        // Safety: Removed node must be deallocated.
        unsafe {
            node1.deallocate();
        }

        // Balanced tree must be as:
        // [2]
        //   \
        //   (4)
        //   / \
        // [3] [5]
        assert_eq!(tree.root, Some(node2));

        assert_eq!(node2.children(), (None, Some(node4)));
        assert_eq!(node3.children(), (None, None));
        assert_eq!(node4.children(), (Some(node3), Some(node5)));
        assert_eq!(node5.children(), (None, None));

        assert!(node2.is_black());
        assert!(node3.is_black());
        assert!(node4.is_red());
        assert!(node5.is_black());
    }
    {
        // Case 4 - the sibling and distant nephew are black, but the close nephew is red:

        let node1 = NodeRef::new(1, ());
        let node2 = NodeRef::new(2, ());
        let node3 = NodeRef::new(3, ());
        let node4 = NodeRef::new(4, ());
        let node5 = NodeRef::new(5, ());
        // Safety: It is built as:
        //   (2)
        //   / \
        // [1] [4]
        //     / \
        //   (3) [5]
        unsafe {
            node2.set_child(ChildIndex::Left, node1);
            node2.set_child(ChildIndex::Right, node4);
            node4.set_child(ChildIndex::Left, node3);
            node4.set_child(ChildIndex::Right, node5);

            node1.set_color(Color::Black);
            node2.set_color(Color::Red);
            node3.set_color(Color::Red);
            node4.set_color(Color::Black);
            node5.set_color(Color::Black);
        }
        let mut tree = RedBlackTree {
            root: Some(node2),
            len: 4,
            _phantom: PhantomData,
        };

        tree.balance_after_remove(node1);
        // Safety: Removed node must be deallocated.
        unsafe {
            node1.deallocate();
        }

        // Balanced tree must be as:
        //   (3)
        //   / \
        // [2] [4]
        //       \
        //       [5]
        assert_eq!(tree.root, Some(node3));

        assert_eq!(node2.children(), (None, None));
        assert_eq!(node3.children(), (Some(node2), Some(node4)));
        assert_eq!(node4.children(), (None, Some(node5)));
        assert_eq!(node5.children(), (None, None));

        assert!(node2.is_black());
        assert!(node3.is_red());
        assert!(node4.is_black());
        assert!(node5.is_black());

        // then try to delete node 2.
        tree.len -= 1;
        tree.balance_after_remove(node2);
        // Safety: Removed node must be deallocated.
        unsafe {
            node2.deallocate();
        }

        // Balanced tree must be as:
        //   (4)
        //   / \
        // [3] [5]
        assert_eq!(tree.root, Some(node4));

        assert_eq!(node3.children(), (None, None));
        assert_eq!(node4.children(), (Some(node3), Some(node5)));
        assert_eq!(node5.children(), (None, None));

        assert!(node3.is_black());
        assert!(node4.is_red());
        assert!(node5.is_black());
    }
}
