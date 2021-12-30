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
    // [ 1 ] [ 4 ]
    //       /   \
    //    ( 3 ) ( 5 )
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

    assert_eq!(n1.key(), &1);
    assert_eq!(n2.key(), &2);
    assert_eq!(n3.key(), &3);
    assert_eq!(n4.key(), &4);
    assert_eq!(n5.key(), &5);

    assert!(n1.color() == Color::Black);
    assert!(n2.color() == Color::Red);
    assert!(n3.color() == Color::Red);
    assert!(n4.color() == Color::Black);
    assert!(n5.color() == Color::Red);

    assert!(n2.parent().is_none());
    assert_eq!(n2.child(ChildIndex::Left), Some(n1));
    assert_eq!(n2.child(ChildIndex::Right), Some(n4));
    assert_eq!(n4.child(ChildIndex::Left), Some(n3));
    assert_eq!(n4.child(ChildIndex::Right), Some(n5));

    assert_eq!(n2.search(&0), Err((n1, ChildIndex::Left)));
    assert_eq!(n2.search(&1), Ok(n1));
    assert_eq!(n2.search(&2), Ok(n2));
    assert_eq!(n2.search(&3), Ok(n3));
    assert_eq!(n2.search(&4), Ok(n4));
    assert_eq!(n2.search(&5), Ok(n5));
    assert_eq!(n2.search(&6), Err((n5, ChildIndex::Right)));

    for n in [n1, n2, n3, n4, n5] {
        unsafe {
            n.deallocate();
        }
    }
}
