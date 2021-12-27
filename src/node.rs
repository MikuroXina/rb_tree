use crate::Ptr;

use std::{borrow::Borrow, ptr::NonNull};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Color {
    Red,
    Black,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ChildIndex {
    Left,
    Right,
}

impl std::ops::Not for ChildIndex {
    type Output = Self;

    fn not(self) -> Self::Output {
        match self {
            ChildIndex::Left => ChildIndex::Right,
            ChildIndex::Right => ChildIndex::Left,
        }
    }
}

pub struct Node<K, V> {
    parent: Ptr<Node<K, V>>,
    #[allow(clippy::type_complexity)]
    children: (Ptr<Node<K, V>>, Ptr<Node<K, V>>),
    color: Color,
    key: K,
    value: V,
}

#[derive(Debug)]
pub struct NodeRef<K, V>(NonNull<Node<K, V>>);

impl<K, V> Clone for NodeRef<K, V> {
    fn clone(&self) -> Self {
        Self(self.0)
    }
}

impl<K, V> Copy for NodeRef<K, V> {}

impl<K, V> PartialEq for NodeRef<K, V> {
    fn eq(&self, other: &Self) -> bool {
        self.0 == other.0
    }
}

impl<K, V> Eq for NodeRef<K, V> {}

impl<K, V> Node<K, V> {
    pub fn new(parent: Option<NonNull<Self>>, key: K, value: V) -> Self {
        Self {
            parent,
            children: (None, None),
            color: Color::Red,
            key,
            value,
        }
    }

    pub fn into_element(self) -> (K, V) {
        (self.key, self.value)
    }
}

impl<K, V> From<NonNull<Node<K, V>>> for NodeRef<K, V> {
    fn from(ptr: NonNull<Node<K, V>>) -> Self {
        Self(ptr)
    }
}

impl<K, V> NodeRef<K, V> {
    pub fn as_raw(&self) -> NonNull<Node<K, V>> {
        self.0
    }

    pub fn key<Q>(&self) -> &Q
    where
        K: Borrow<Q>,
    {
        unsafe { self.0.as_ref() }.key.borrow()
    }

    pub fn is_red(&self) -> bool {
        unsafe { self.0.as_ref() }.color == Color::Red
    }

    pub fn is_black(&self) -> bool {
        !self.is_red()
    }

    pub fn color(&self) -> Color {
        unsafe { self.0.as_ref() }.color
    }

    pub fn set_color(&mut self, color: Color) {
        unsafe { self.0.as_mut() }.color = color;
    }

    pub fn parent(&self) -> Option<Self> {
        unsafe { self.0.as_ref() }.parent.map(Into::into)
    }

    pub fn grandparent(&self) -> Option<Self> {
        self.parent()?.parent()
    }

    pub fn uncle(&self) -> Option<Self> {
        self.parent()?.sibling()
    }

    pub fn sibling(&self) -> Option<Self> {
        let index = self.index_on_parent()?;
        let parent = self.parent()?;
        parent.child(!index)
    }

    pub fn close_nephew(&self) -> Option<Self> {
        let index = self.index_on_parent()?;
        let sibling = self.sibling()?;
        sibling.child(index)
    }

    pub fn distant_nephew(&self) -> Option<Self> {
        let index = self.index_on_parent()?;
        let sibling = self.sibling()?;
        sibling.child(!index)
    }

    pub fn children(&self) -> (Option<Self>, Option<Self>) {
        let this = unsafe { self.0.as_ref() };
        (
            this.children.0.map(Into::into),
            this.children.1.map(Into::into),
        )
    }

    pub fn child(&self, idx: ChildIndex) -> Option<Self> {
        let this = unsafe { self.0.as_ref() };
        match idx {
            ChildIndex::Left => this.children.0,
            ChildIndex::Right => this.children.1,
        }
        .map(Into::into)
    }

    pub fn set_child(&mut self, idx: ChildIndex, new_child: Option<Self>) {
        let this = unsafe { self.0.as_mut() };
        if let Some(mut child) = new_child {
            unsafe { child.0.as_mut() }.parent = Some(this.into());
        }
        match idx {
            ChildIndex::Left => this.children.0 = new_child.map(|p| p.0),
            ChildIndex::Right => this.children.1 = new_child.map(|p| p.0),
        }
    }

    pub fn index_on_parent(&self) -> Option<ChildIndex> {
        let parent = self.parent()?;
        let child = parent.child(ChildIndex::Left)?;
        Some(if *self == child {
            ChildIndex::Left
        } else {
            ChildIndex::Right
        })
    }
}

impl<K: Ord, V> NodeRef<K, V> {
    pub fn which_to_insert<Q>(&self, key: &Q) -> ChildIndex
    where
        K: Borrow<Q>,
        Q: Ord,
    {
        if key < unsafe { self.0.as_ref() }.key.borrow() {
            ChildIndex::Left
        } else {
            ChildIndex::Right
        }
    }
}
