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
