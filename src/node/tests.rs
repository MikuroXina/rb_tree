use super::{ChildIndex, Node, NodeRef};

impl<'a, K: 'a, V: 'a> NodeRef<K, V> {
    pub fn assert_back_pointers(self) {
        let (left, right) = self.children();
        if let Some(left) = left {
            assert!(left.parent() == Some(self));
            left.assert_back_pointers();
        }
        if let Some(right) = right {
            assert!(right.parent() == Some(self));
            right.assert_back_pointers();
        }
    }
}

#[test]
fn test_partial_eq() {
    let root1 = Node::new(None, 1, ());
    let root2 = Node::new(None, 2, ());
    let root1_ref = NodeRef::from(&root1);
    root1_ref.assert_back_pointers();
    let root2_ref = NodeRef::from(&root2);
    root1_ref.set_child(ChildIndex::Left, Some(root2_ref));
    root2_ref.assert_back_pointers();

    let left = root1_ref.child(ChildIndex::Left);
    let right = root2_ref.child(ChildIndex::Right);
    let parent_1 = root1_ref.parent();
    let parent_2 = root2_ref.parent();

    assert!(left != right);
    assert!(left != parent_1);
    assert!(left != parent_2);
    assert!(parent_1 != parent_2);
}
