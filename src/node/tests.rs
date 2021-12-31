use crate::mem::NodeDropGuard;

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
    let root1 = NodeRef::new(1, ());
    let root2 = NodeRef::new(2, ());
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
    //    ( 3 )
    //    /   \
    // [ 1 ] [ 7 ]
    //       /   \
    //    ( 5 ) ( 9 )
    let n1 = NodeRef::new(1, ());
    let n3 = NodeRef::new(3, ());
    let n5 = NodeRef::new(5, ());
    let n7 = NodeRef::new(7, ());
    let n9 = NodeRef::new(9, ());

    let _guard = NodeDropGuard([n1, n3, n7, n5, n9]);

    n3.set_child(ChildIndex::Left, Some(n1));
    n3.set_child(ChildIndex::Right, Some(n7));
    n7.set_child(ChildIndex::Left, Some(n5));
    n7.set_child(ChildIndex::Right, Some(n9));
    n1.balance_after_insert();
    n7.balance_after_insert();
    n5.balance_after_insert();
    n9.balance_after_insert();

    assert_eq!(n1.key(), &1);
    assert_eq!(n3.key(), &3);
    assert_eq!(n5.key(), &5);
    assert_eq!(n7.key(), &7);
    assert_eq!(n9.key(), &9);

    assert!(n1.color() == Color::Black);
    assert!(n3.color() == Color::Red);
    assert!(n5.color() == Color::Red);
    assert!(n7.color() == Color::Black);
    assert!(n9.color() == Color::Red);

    assert!(n3.parent().is_none());
    assert_eq!(n3.child(ChildIndex::Left), Some(n1));
    assert_eq!(n3.child(ChildIndex::Right), Some(n7));
    assert_eq!(n7.child(ChildIndex::Left), Some(n5));
    assert_eq!(n7.child(ChildIndex::Right), Some(n9));

    assert_eq!(n3.search(&0), Err((n1, ChildIndex::Left)));
    assert_eq!(n3.search(&1), Ok(n1));
    assert_eq!(n3.search(&2), Err((n1, ChildIndex::Right)));
    assert_eq!(n3.search(&3), Ok(n3));
    assert_eq!(n3.search(&4), Err((n5, ChildIndex::Left)));
    assert_eq!(n3.search(&5), Ok(n5));
    assert_eq!(n3.search(&6), Err((n5, ChildIndex::Right)));
    assert_eq!(n3.search(&7), Ok(n7));
    assert_eq!(n3.search(&8), Err((n9, ChildIndex::Left)));
    assert_eq!(n3.search(&9), Ok(n9));
    assert_eq!(n3.search(&10), Err((n9, ChildIndex::Right)));
}
