use std::{borrow, ops};

use super::PreviousStep;
use crate::{
    node::{ChildIndex, Node},
    RbTreeMap,
};

#[derive(Debug)]
pub struct DyingLeafRange<K, V> {
    start: Option<Node<K, V>>,
    start_prev: PreviousStep,
    end: Option<Node<K, V>>,
    end_prev: PreviousStep,
}

impl<K, V> DyingLeafRange<K, V> {
    pub fn new(tree: RbTreeMap<K, V>) -> Self {
        let start = tree.root.inner().map(|r| r.min_child());
        let end = tree.root.inner().map(|r| r.max_child());
        std::mem::forget(tree);
        Self {
            start,
            start_prev: PreviousStep::LeftChild,
            end,
            end_prev: PreviousStep::RightChild,
        }
    }

    pub fn cut_left(&mut self) -> Option<(K, V)> {
        while let Some(curr) = self.start {
            match self.start_prev {
                PreviousStep::Parent => {
                    // descended
                    if let Some(left) = curr.left() {
                        // go to left
                        self.start = Some(left);
                        continue;
                    }
                    self.start_prev = PreviousStep::LeftChild;
                }
                PreviousStep::LeftChild => {
                    // ascended from left
                    if self.start == self.end && self.end_prev.is_right_child() {
                        // finish
                        self.start = None;
                        self.end = None;
                    } else if let Some(right) = curr.right() {
                        // go to right
                        self.start_prev = PreviousStep::Parent;
                        self.start = Some(right);
                    } else {
                        self.start_prev = PreviousStep::RightChild;
                    }
                    unsafe {
                        return Some((std::ptr::read(curr.key()), std::ptr::read(curr.value())));
                    }
                }
                PreviousStep::RightChild => {
                    // ascended from right, so ascend again
                    self.start = curr.parent();
                    if let Some(ChildIndex::Left) = curr.index_on_parent() {
                        self.start_prev = PreviousStep::LeftChild;
                    }
                    // deallocate now and forget, because it will be dropped on outside.
                    std::mem::forget(unsafe { curr.deallocate() });
                }
            }
        }
        None
    }

    pub fn cut_right(&mut self) -> Option<(K, V)> {
        while let Some(curr) = self.end {
            match self.end_prev {
                PreviousStep::Parent => {
                    // descended
                    if let Some(right) = curr.right() {
                        // go to right
                        self.end = Some(right);
                        continue;
                    }
                    self.end_prev = PreviousStep::RightChild;
                }
                PreviousStep::RightChild => {
                    // ascended from right
                    if self.start == self.end && self.start_prev.is_left_child() {
                        // finish
                        self.start = None;
                        self.end = None;
                    } else if let Some(left) = curr.left() {
                        // go to left
                        self.end_prev = PreviousStep::Parent;
                        self.end = Some(left);
                    } else {
                        self.end_prev = PreviousStep::LeftChild;
                    }
                    unsafe {
                        return Some((std::ptr::read(curr.key()), std::ptr::read(curr.value())));
                    }
                }
                PreviousStep::LeftChild => {
                    // ascended from left, so ascend again
                    self.end = curr.parent();
                    if let Some(ChildIndex::Right) = curr.index_on_parent() {
                        self.start_prev = PreviousStep::RightChild;
                    }
                    // deallocate now and forget, because it will be dropped on outside.
                    std::mem::forget(unsafe { curr.deallocate() });
                }
            }
        }
        None
    }
}

#[derive(Debug)]
pub struct RefLeafRange<K, V> {
    start: Option<Node<K, V>>,
    start_prev: PreviousStep,
    end: Option<Node<K, V>>,
    end_prev: PreviousStep,
}

impl<K, V> Clone for RefLeafRange<K, V> {
    fn clone(&self) -> Self {
        Self { ..*self }
    }
}

impl<K, V> RefLeafRange<K, V> {
    pub fn all(tree: &RbTreeMap<K, V>) -> Self {
        let root = tree.root.inner();
        let (start, end) = if let Some((start, end)) =
            root.map(|r| r.min_child()).zip(root.map(|r| r.max_child()))
        {
            (Some(start), Some(end))
        } else {
            (None, None)
        };
        Self {
            start,
            start_prev: PreviousStep::LeftChild,
            end,
            end_prev: PreviousStep::RightChild,
        }
    }

    pub fn new<R, Q>(tree: &RbTreeMap<K, V>, range: R) -> Self
    where
        K: Ord + borrow::Borrow<Q>,
        Q: Ord + ?Sized,
        R: ops::RangeBounds<Q>,
    {
        let (start, end) = if let Some((start, end)) =
            tree.root.inner().and_then(|root| search_range(root, range))
        {
            (Some(start), Some(end))
        } else {
            (None, None)
        };
        Self {
            start,
            start_prev: PreviousStep::LeftChild,
            end,
            end_prev: PreviousStep::RightChild,
        }
    }

    pub fn cut_left(&mut self) -> Option<Node<K, V>> {
        while let Some(curr) = self.start {
            match self.start_prev {
                PreviousStep::Parent => {
                    // descended
                    if let Some(left) = curr.left() {
                        // go to left
                        self.start = Some(left);
                        continue;
                    }
                    self.start_prev = PreviousStep::LeftChild;
                }
                PreviousStep::LeftChild => {
                    // ascended from left
                    if self.start == self.end && self.end_prev.is_right_child() {
                        // finish
                        self.start = None;
                        self.end = None;
                    } else if let Some(right) = curr.right() {
                        // go to right
                        self.start_prev = PreviousStep::Parent;
                        self.start = Some(right);
                    } else {
                        self.start_prev = PreviousStep::RightChild;
                    }
                    return Some(curr);
                }
                PreviousStep::RightChild => {
                    // ascended from right, so ascend again
                    self.start = curr.parent();
                    if let Some(ChildIndex::Left) = curr.index_on_parent() {
                        self.start_prev = PreviousStep::LeftChild;
                    }
                }
            }
        }
        None
    }

    pub fn cut_right(&mut self) -> Option<Node<K, V>> {
        while let Some(curr) = self.end {
            match self.end_prev {
                PreviousStep::Parent => {
                    // descended
                    if let Some(right) = curr.right() {
                        // go to right
                        self.end = Some(right);
                        continue;
                    }
                    self.end_prev = PreviousStep::RightChild;
                }
                PreviousStep::RightChild => {
                    // ascended from right
                    if self.start == self.end && self.start_prev.is_left_child() {
                        // finish
                        self.start = None;
                        self.end = None;
                    } else if let Some(left) = curr.left() {
                        // go to left
                        self.end_prev = PreviousStep::Parent;
                        self.end = Some(left);
                    } else {
                        self.end_prev = PreviousStep::LeftChild;
                    }
                    return Some(curr);
                }
                PreviousStep::LeftChild => {
                    // ascended from left, so ascend again
                    self.end = curr.parent();
                    if let Some(ChildIndex::Right) = curr.index_on_parent() {
                        self.start_prev = PreviousStep::RightChild;
                    }
                }
            }
        }
        None
    }
}

fn search_range<K, V, R, Q>(root: Node<K, V>, range: R) -> Option<(Node<K, V>, Node<K, V>)>
where
    K: Ord + borrow::Borrow<Q>,
    Q: ?Sized + Ord,
    R: ops::RangeBounds<Q>,
{
    use std::cmp::Ordering;
    let lower = {
        let cmp = |key: &Q| match range.start_bound() {
            ops::Bound::Included(b) => b.cmp(key),
            ops::Bound::Excluded(b) => b.cmp(key).then(Ordering::Less),
            ops::Bound::Unbounded => Ordering::Less,
        };
        let mut current = root;
        loop {
            match cmp(current.key()) {
                Ordering::Less => {
                    if let Some(left) = current.left().filter(|left| cmp(left.key()).is_le()) {
                        current = left;
                        continue;
                    }
                }
                Ordering::Equal => {}
                Ordering::Greater => {
                    if let Some(right) = current.right() {
                        current = right;
                        continue;
                    }
                }
            }
            break;
        }
        current
    };
    let upper = {
        let cmp = |key: &Q| match range.end_bound() {
            ops::Bound::Included(b) => key.cmp(b),
            ops::Bound::Excluded(b) => key.cmp(b).then(Ordering::Less),
            ops::Bound::Unbounded => Ordering::Less,
        };
        let mut current = root;
        loop {
            match cmp(current.key()) {
                Ordering::Greater => {
                    if let Some(left) = current.left() {
                        current = left;
                        continue;
                    }
                }
                Ordering::Equal => {}
                Ordering::Less => {
                    if let Some(right) = current.right().filter(|right| cmp(right.key()).is_le()) {
                        current = right;
                        continue;
                    }
                }
            }
            break;
        }
        current
    };
    if upper.key() < lower.key() {
        // if empty range
        None
    } else {
        Some((lower, upper))
    }
}
