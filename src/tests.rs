use crate::RbTreeMap;

#[test]
fn simple() {
    let mut tree = RbTreeMap::new();
    tree.insert(1, 'a');
    tree.insert(4, 'b');
    tree.insert(2, 'c');
    tree.insert(3, 'd');
    tree.insert(5, 'e');

    assert_eq!(tree.remove(&0), None);
    assert_eq!(tree.remove(&1), Some('a'));
    assert_eq!(tree.remove(&2), Some('c'));
    assert_eq!(tree.remove(&3), Some('d'));
    assert_eq!(tree.remove(&4), Some('b'));
    assert_eq!(tree.remove(&5), Some('e'));
    assert_eq!(tree.remove(&6), None);
}

#[test]
fn retain() {
    let mut tree = RbTreeMap::new();
    tree.insert(1, ());
    tree.insert(4, ());
    tree.insert(2, ());
    tree.insert(3, ());
    tree.insert(5, ());

    assert_eq!(
        tree.drain_filter(|k, _| k % 2 != 0).collect::<Vec<_>>(),
        vec![(1, ()), (3, ()), (5, ())]
    );
    assert_eq!(tree.remove(&1), None);
    assert_eq!(tree.remove(&2), Some(()));
    assert_eq!(tree.remove(&3), None);
    assert_eq!(tree.remove(&4), Some(()));
    assert_eq!(tree.remove(&5), None);
}
