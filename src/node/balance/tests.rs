use crate::{
    mem::NodeDropGuard,
    node::{ChildIndex, Color, NodeRef},
};

#[test]
fn test_after_insert() {
    // when `2` has just inserted:
    //    [ 3 ]
    //    /   \
    // ( 1 ) [ 5 ]
    //    \
    //   ( 2 )
    let n1 = NodeRef::new(1, ());
    let n2 = NodeRef::new(2, ());
    let n3 = NodeRef::new(3, ());
    let n5 = NodeRef::new(5, ());

    let _guard = NodeDropGuard([n1, n2, n3, n5]);

    n3.set_color(Color::Black);
    n5.set_color(Color::Black);

    n3.set_child(ChildIndex::Left, Some(n1));
    n3.set_child(ChildIndex::Right, Some(n5));
    n1.set_child(ChildIndex::Right, Some(n2));

    n2.balance_after_insert();

    // it must be balanced as:
    //    [ 2 ]
    //    /   \
    // ( 1 ) ( 3 )
    //          \
    //         [ 5 ]

    assert_eq!(n2.children(), (Some(n1), Some(n3)));
    assert_eq!(n1.children(), (None, None));
    assert_eq!(n3.children(), (None, Some(n5)));
    assert_eq!(n5.children(), (None, None));

    assert_eq!(n1.color(), Color::Red);
    assert_eq!(n2.color(), Color::Black);
    assert_eq!(n3.color(), Color::Red);
    assert_eq!(n5.color(), Color::Black);
}
