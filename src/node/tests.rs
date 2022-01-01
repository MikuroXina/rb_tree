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
    let n1 = NodeRef::new(1, ());
    let n4 = NodeRef::new(4, ());
    let n2 = NodeRef::new(2, ());
    let n3 = NodeRef::new(3, ());

    let _guard = NodeDropGuard([n1, n4, n2, n3]);

    assert_eq!(n1.search(&4), Err((n1, ChildIndex::Right)));
    n1.set_child(ChildIndex::Right, Some(n4));
    n4.balance_after_insert();
    assert_eq!(n1.search(&4), Ok(n4));
    assert_eq!(n1.children(), (None, Some(n4)));
    assert_eq!(n4.children(), (None, None));
    assert_eq!(n1.color(), Color::Black);
    assert_eq!(n4.color(), Color::Red);

    // [ 1 ]
    //    \
    //   ( 4 )
    //    /
    // ( 2 )
    assert_eq!(n1.search(&2), Err((n4, ChildIndex::Left)));
    n4.set_child(ChildIndex::Left, Some(n2));
    n2.balance_after_insert();
    //    [ 2 ]
    //    /   \
    // ( 1 ) ( 4 )
    assert_eq!(n2.search(&2), Ok(n2));
    assert_eq!(n2.children(), (Some(n1), Some(n4)));
    assert_eq!(n1.children(), (None, None));
    assert_eq!(n4.children(), (None, None));
    assert_eq!(n2.color(), Color::Black);
    assert_eq!(n1.color(), Color::Red);
    assert_eq!(n4.color(), Color::Red);

    //    [ 2 ]
    //    /   \
    // ( 1 ) ( 4 )
    //         /
    //      ( 3 )
    assert_eq!(n2.search(&3), Err((n4, ChildIndex::Left)));
    n4.set_child(ChildIndex::Left, Some(n3));
    n3.balance_after_insert();
    //    ( 2 )
    //    /   \
    // [ 1 ] [ 4 ]
    //         /
    //      ( 3 )
    assert_eq!(n2.search(&3), Ok(n3));
    assert_eq!(n1.children(), (None, None));
    assert_eq!(n2.children(), (Some(n1), Some(n4)));
    assert_eq!(n3.children(), (None, None));
    assert_eq!(n4.children(), (Some(n3), None));
    assert_eq!(n2.color(), Color::Red);
    assert_eq!(n1.color(), Color::Black);
    assert_eq!(n4.color(), Color::Black);
    assert_eq!(n3.color(), Color::Red);
}
