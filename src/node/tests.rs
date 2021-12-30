use super::{ChildIndex, Color, NodeRef};

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
    let root1 = NodeRef::new_root(1, ());
    let root2 = NodeRef::new_root(2, ());
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

    unsafe {
        root2.deallocate();
        root1.deallocate();
    }
}

#[test]
fn test_insert() {
    //    ( 2 )
    //    /   \
    // [ 1 ] ( 4 )
    //       /   \
    //    [ 3 ] [ 5 ]
    let n1 = NodeRef::new_root(1, ());
    let n2 = NodeRef::new_root(2, ());
    let n3 = NodeRef::new_root(3, ());
    let n4 = NodeRef::new_root(4, ());
    let n5 = NodeRef::new_root(5, ());
    n2.set_child(ChildIndex::Left, Some(n1));
    n2.set_child(ChildIndex::Right, Some(n4));
    n4.set_child(ChildIndex::Left, Some(n3));
    n4.set_child(ChildIndex::Right, Some(n5));
    n1.balance_after_insert();
    n4.balance_after_insert();
    n3.balance_after_insert();
    n5.balance_after_insert();

    assert!(n2.color() == Color::Red);
    assert!(n4.color() == Color::Red);
    assert!(n1.color() == Color::Black);
    assert!(n3.color() == Color::Black);
    assert!(n5.color() == Color::Black);
}
