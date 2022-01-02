use std::{borrow, ops};

use crate::{
    node::{ChildIndex, NodeRef},
    RedBlackTree,
};

pub struct LeafRange<K, V> {
    start: Option<NodeRef<K, V>>,
    end: Option<NodeRef<K, V>>,
}

impl<K, V> LeafRange<K, V> {
    pub fn new(tree: RedBlackTree<K, V>) -> Self {
        let start = tree.first_node();
        let end = tree.last_node();
        std::mem::forget(tree);
        Self { start, end }
    }

    pub fn cut_left(&mut self) -> Option<(K, V)> {
        let start = self.start?;
        let next = start.child(ChildIndex::Right).or_else(|| start.parent());
        std::mem::replace(&mut self.start, next).map(|p| unsafe { p.deallocate() })
    }

    pub fn cut_right(&mut self) -> Option<(K, V)> {
        let end = self.end?;
        let next = end.child(ChildIndex::Left).or_else(|| end.parent());
        std::mem::replace(&mut self.end, next).map(|p| unsafe { p.deallocate() })
    }
}

#[derive(Debug, Clone, Copy)]
enum PreviousStep {
    Parent,
    LeftChild,
    RightChild,
}

pub struct RefLeafRange<K, V> {
    start: Option<NodeRef<K, V>>,
    start_prev: PreviousStep,
    end: Option<NodeRef<K, V>>,
    end_prev: PreviousStep,
}

impl<K, V> Clone for RefLeafRange<K, V> {
    fn clone(&self) -> Self {
        Self { ..*self }
    }
}

impl<K, V> RefLeafRange<K, V> {
    pub fn new<R, Q>(tree: &RedBlackTree<K, V>, range: R) -> Self
    where
        K: Ord + borrow::Borrow<Q>,
        Q: Ord + ?Sized,
        R: ops::RangeBounds<Q>,
    {
        let root = tree.root;
        Self {
            start: root.map(|r| search_lower(r, &range)),
            start_prev: PreviousStep::Parent,
            end: root.map(|r| search_upper(r, &range)),
            end_prev: PreviousStep::Parent,
        }
    }

    pub fn cut_left(&mut self) -> Option<NodeRef<K, V>> {
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
                    if let Some(right) = curr.right() {
                        // go to right
                        self.start_prev = PreviousStep::Parent;
                        self.start = Some(right);
                    } else {
                        self.start_prev = PreviousStep::RightChild;
                    }
                    return Some(curr);
                }
                PreviousStep::RightChild => {
                    // ascended from right, so ascend once more
                    self.start = curr.parent();
                }
            }
        }
        None
    }

    pub fn cut_right(&mut self) -> Option<NodeRef<K, V>> {
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
                    if let Some(left) = curr.left() {
                        // go to left
                        self.end_prev = PreviousStep::Parent;
                        self.end = Some(left);
                    } else {
                        self.end_prev = PreviousStep::LeftChild;
                    }
                    return Some(curr);
                }
                PreviousStep::LeftChild => {
                    // ascended from left, so ascend once more
                    self.end = curr.parent();
                }
            }
        }
        None
    }
}

fn search_lower<K, V, R, Q>(root: NodeRef<K, V>, range: &R) -> NodeRef<K, V>
where
    K: Ord + borrow::Borrow<Q>,
    Q: ?Sized + Ord,
    R: ops::RangeBounds<Q>,
{
    let contains_key = |node: &NodeRef<K, V>| range.contains(node.key());
    let mut current = root;
    while let Some(next) = current
        .left()
        .filter(contains_key)
        .or_else(|| current.right())
        .filter(contains_key)
    {
        current = next;
    }
    current
}

fn search_upper<K, V, R, Q>(root: NodeRef<K, V>, range: &R) -> NodeRef<K, V>
where
    K: Ord + borrow::Borrow<Q>,
    Q: ?Sized + Ord,
    R: ops::RangeBounds<Q>,
{
    let contains_key = |node: &NodeRef<K, V>| range.contains(node.key());
    let mut current = root;
    while let Some(next) = current
        .right()
        .filter(contains_key)
        .or_else(|| current.left())
        .filter(contains_key)
    {
        current = next;
    }
    current
}
