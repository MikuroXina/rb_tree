use crate::RedBlackTree;

#[test]
fn simple_insert() {
    let mut tree = RedBlackTree::new();
    tree.insert(1, ());
    // (1)
    {
        let node1 = tree.root.inner().unwrap();
        assert_eq!(node1.key(), &1);
        assert!(node1.is_red());
    }

    tree.insert(4, ());
    // [1]
    //   \
    //   (4)
    {
        let node1 = tree.root.inner().unwrap();
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
        let node2 = tree.root.inner().unwrap();
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
        let node2 = tree.root.inner().unwrap();
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
        let node2 = tree.root.inner().unwrap();
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
        let node4 = tree.root.inner().unwrap();
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
        let node4 = tree.root.inner().unwrap();
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
        let node4 = tree.root.inner().unwrap();
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
        let node5 = tree.root.inner().unwrap();
        assert_eq!(node5.key(), &5);
        assert!(node5.is_black());
    }

    tree.remove(&5);
    assert!(tree.is_empty());
}
