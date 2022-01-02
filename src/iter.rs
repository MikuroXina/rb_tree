mod entry;
mod keys;
mod range;
mod values;

pub use entry::*;
pub use keys::*;
pub use range::*;
pub use values::*;

use std::{borrow, ops};

use crate::{
    node::{ChildIndex, NodeRef},
    RedBlackTree,
};

struct LeafRange<K, V> {
    start: Option<NodeRef<K, V>>,
    end: Option<NodeRef<K, V>>,
}

impl<K, V> LeafRange<K, V> {
    fn cut_left(&mut self) -> Option<(K, V)> {
        let start = self.start?;
        let next = start.child(ChildIndex::Right).or_else(|| start.parent());
        std::mem::replace(&mut self.start, next).map(|p| unsafe { p.deallocate() })
    }

    fn cut_right(&mut self) -> Option<(K, V)> {
        let end = self.end?;
        let next = end.child(ChildIndex::Left).or_else(|| end.parent());
        std::mem::replace(&mut self.end, next).map(|p| unsafe { p.deallocate() })
    }
}

#[derive(Debug, Clone)]
struct SearchRange<R> {
    range: R,
}

impl<R> SearchRange<R> {
    fn contains<K>(&self, key: &K) -> bool
    where
        K: ?Sized + Ord,
        R: ops::RangeBounds<K>,
    {
        use ops::Bound::*;
        let lower = self.range.start_bound();
        let upper = self.range.end_bound();
        let is_lower_ok = match lower {
            Included(b) => b <= key,
            Excluded(b) => b < key,
            Unbounded => true,
        };
        let is_upper_ok = match upper {
            Included(b) => key <= b,
            Excluded(b) => key < b,
            Unbounded => true,
        };
        is_lower_ok && is_upper_ok
    }
}

struct RefLeafRange<K, V, R> {
    current: Option<NodeRef<K, V>>,
    range: SearchRange<R>,
    is_climbing: bool,
}

impl<K, V, R: Clone> Clone for RefLeafRange<K, V, R> {
    fn clone(&self) -> Self {
        Self {
            range: self.range.clone(),
            ..*self
        }
    }
}

impl<K, V, R> RefLeafRange<K, V, R>
where
    K: Ord,
{
    fn new<I>(tree: &RedBlackTree<K, V>, range: R) -> Self
    where
        K: borrow::Borrow<I>,
        I: Ord + ?Sized,
        R: ops::RangeBounds<I>,
    {
        Self {
            current: tree.root,
            range: SearchRange { range },
            is_climbing: false,
        }
    }

    fn cut_left<Q>(&mut self) -> Option<NodeRef<K, V>>
    where
        K: borrow::Borrow<Q>,
        Q: Ord + ?Sized,
        R: ops::RangeBounds<Q>,
    {
        let mut current = self.current?;
        let ret = if self.is_climbing {
            if let Some(right) = current.child(ChildIndex::Right) {
                self.current = Some(right);
                self.is_climbing = false;
            } else {
                self.current = current.parent();
            }
            current
        } else {
            while let Some(left) = current.child(ChildIndex::Left) {
                if !self.range.contains(left.key()) {
                    break;
                }
                current = left;
            }
            self.current = current.parent();
            self.is_climbing = true;
            current
        };
        if self.range.contains(ret.key()) {
            Some(ret)
        } else {
            self.current = None;
            None
        }
    }

    fn cut_right<Q>(&mut self) -> Option<NodeRef<K, V>>
    where
        K: borrow::Borrow<Q>,
        Q: Ord + ?Sized,
        R: ops::RangeBounds<Q>,
    {
        let mut current = self.current?;
        let ret = if self.is_climbing {
            if let Some(left) = current.child(ChildIndex::Left) {
                self.current = Some(left);
                self.is_climbing = false;
            } else {
                self.current = current.parent();
            }
            current
        } else {
            while let Some(right) = current.child(ChildIndex::Right) {
                if !self.range.contains(right.key()) {
                    break;
                }
                current = right;
            }
            self.current = current.parent();
            self.is_climbing = true;
            current
        };
        if self.range.contains(ret.key()) {
            Some(ret)
        } else {
            self.current = None;
            None
        }
    }
}
