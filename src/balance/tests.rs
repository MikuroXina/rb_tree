use std::marker::PhantomData;

use crate::{
    node::{ChildIndex, NodeRef},
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

    node3.rotate(ChildIndex::Right, &mut tree.root);

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

    node3.rotate(ChildIndex::Left, &mut tree.root);

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
fn simple_insert() {
    let mut tree = RedBlackTree::new();
    tree.insert(1, ());
    // (1)
    {
        let node1 = tree.root.unwrap();
        assert_eq!(node1.key(), &1);
        assert!(node1.is_red());
    }

    tree.insert(4, ());
    // [1]
    //   \
    //   (4)
    {
        let node1 = tree.root.unwrap();
        assert_eq!(node1.key(), &1);
        assert!(node1.is_black());
        let (_, node4) = node1.children();
        let node4 = node4.expect("node4 must exist");

        assert_eq!(node4.key(), &4);
        assert!(node4.is_red());
    }

    tree.insert(2, ());
    //   [2]
    //   / \
    // (1) (4)
    {
        let node2 = tree.root.unwrap();
        assert_eq!(node2.key(), &2);
        assert!(node2.is_black());
        let (node1, node4) = node2.children();
        let node1 = node1.expect("node1 must exist");
        let node4 = node4.expect("node4 must exist");

        assert_eq!(node1.key(), &1);
        assert!(node1.is_red());

        assert_eq!(node4.key(), &4);
        assert!(node4.is_red());
    }

    tree.insert(3, ());
    //   (2)
    //   / \
    // [1] [4]
    //     /
    //   (3)
    {
        let node2 = tree.root.unwrap();
        assert_eq!(node2.key(), &2);
        assert!(node2.is_red());
        let (node1, node4) = node2.children();
        let node1 = node1.expect("node1 must exist");
        let node4 = node4.expect("node4 must exist");

        assert_eq!(node1.key(), &1);
        assert!(node1.is_black());

        assert_eq!(node4.key(), &4);
        assert!(node4.is_black());

        let (node3, _) = node4.children();
        let node3 = node3.expect("node3 must exist");
        assert_eq!(node3.key(), &3);
        assert!(node3.is_red());
    }

    tree.insert(5, ());
    //   (2)
    //   / \
    // [1] [4]
    //     / \
    //   (3) (5)
    {
        let node2 = tree.root.unwrap();
        assert_eq!(node2.key(), &2);
        assert!(node2.is_red());
        let (node1, node4) = node2.children();
        let node1 = node1.expect("node1 must exist");
        let node4 = node4.expect("node4 must exist");

        assert_eq!(node1.key(), &1);
        assert!(node1.is_black());

        assert_eq!(node4.key(), &4);
        assert!(node4.is_black());

        let (node3, node5) = node4.children();
        let node3 = node3.expect("node3 must exist");
        let node5 = node5.expect("node5 must exist");
        assert_eq!(node3.key(), &3);
        assert!(node3.is_red());

        assert_eq!(node5.key(), &5);
        assert!(node5.is_red());
    }
}

#[test]
fn simple_remove() {
    let mut tree = RedBlackTree::new();
    tree.insert(1, ());
    tree.insert(4, ());
    tree.insert(2, ());
    tree.insert(3, ());
    tree.insert(5, ());

    tree.remove(&1);
    //   (4)
    //   / \
    // [2] [5]
    //   \
    //   (3)
    {
        let node4 = tree.root.unwrap();
        assert_eq!(node4.key(), &4);
        assert!(node4.is_red());
        let (node2, node5) = node4.children();
        let node2 = node2.expect("node2 must exist");
        let node5 = node5.expect("node5 must exist");

        assert_eq!(node2.key(), &2);
        assert!(node2.is_black());
        let (_, node3) = node2.children();
        let node3 = node3.expect("node3 must exist");

        assert_eq!(node3.key(), &3);
        assert!(node3.is_red());

        assert_eq!(node5.key(), &5);
        assert!(node5.is_black());
    }

    tree.remove(&2);
    //   (4)
    //   / \
    // [3] [5]
    {
        let node4 = tree.root.unwrap();
        assert_eq!(node4.key(), &4);
        assert!(node4.is_red());
        let (node3, node5) = node4.children();
        let node3 = node3.expect("node3 must exist");
        let node5 = node5.expect("node5 must exist");

        assert_eq!(node3.key(), &3);
        assert!(node3.is_black());

        assert_eq!(node5.key(), &5);
        assert!(node5.is_black());
    }

    tree.remove(&3);
    // [4]
    //   \
    //   (5)
    {
        let node4 = tree.root.unwrap();
        assert_eq!(node4.key(), &4);
        assert!(node4.is_black());
        let (_, node5) = node4.children();
        let node5 = node5.expect("node5 must exist");

        assert_eq!(node5.key(), &5);
        assert!(node5.is_red());
    }

    tree.remove(&4);
    // [5]
    {
        let node5 = tree.root.unwrap();
        assert_eq!(node5.key(), &5);
        assert!(node5.is_black());
    }

    tree.remove(&5);
    assert!(tree.is_empty());
}
