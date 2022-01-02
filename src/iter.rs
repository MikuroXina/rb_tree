mod entry;
mod keys;
mod range;
mod values;

pub use entry::*;
pub use keys::*;
pub use range::*;
pub use values::*;

use std::marker::PhantomData;

use crate::node::{ChildIndex, NodeRef};

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

struct RefLeafRange<'a, K, V> {
    start: Option<NodeRef<K, V>>,
    end: Option<NodeRef<K, V>>,
    _phantom: PhantomData<&'a ()>,
}

impl<'a, K, V> RefLeafRange<'a, K, V> {
    fn cut_left(&mut self) -> Option<(&'a K, &'a V)> {
        let start = self.start?;
        let next = start.child(ChildIndex::Right).or_else(|| start.parent());
        std::mem::replace(&mut self.start, next).map(|p| p.key_value())
    }

    fn cut_right(&mut self) -> Option<(&'a K, &'a V)> {
        let end = self.end?;
        let next = end.child(ChildIndex::Left).or_else(|| end.parent());
        std::mem::replace(&mut self.end, next).map(|p| p.key_value())
    }
}

struct MutLeafRange<'a, K, V> {
    start: Option<NodeRef<K, V>>,
    end: Option<NodeRef<K, V>>,
    _phantom: PhantomData<&'a mut ()>,
}

impl<'a, K, V> MutLeafRange<'a, K, V> {
    fn cut_left(&mut self) -> Option<(&'a K, &'a mut V)> {
        let start = self.start?;
        let next = start.child(ChildIndex::Right).or_else(|| start.parent());
        std::mem::replace(&mut self.start, next).map(|p| p.key_value_mut())
    }

    fn cut_right(&mut self) -> Option<(&'a K, &'a mut V)> {
        let end = self.end?;
        let next = end.child(ChildIndex::Left).or_else(|| end.parent());
        std::mem::replace(&mut self.end, next).map(|p| p.key_value_mut())
    }
}
