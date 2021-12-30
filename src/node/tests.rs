use super::{ChildIndex, NodeRef};

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
    let root1 = NodeRef::new(None, 1, ());
    let root2 = NodeRef::new(None, 2, ());
    root1.assert_back_pointers();
    root1.set_child(ChildIndex::Left, Some(root2));
    root2.assert_back_pointers();

    let left = root1.child(ChildIndex::Left);
    let right = root2.child(ChildIndex::Right);
    let parent_1 = root1.parent();
    let parent_2 = root2.parent();

    assert!(left != right);
    assert!(left != parent_1);
    assert!(left != parent_2);
    assert!(parent_1 != parent_2);
}
