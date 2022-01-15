use std::{fmt, iter::FusedIterator};

pub struct MergeIter<I: Iterator> {
    a: I,
    b: I,
    peeked: Option<Peeked<I>>,
}

#[derive(Clone, Debug)]
enum Peeked<I: Iterator> {
    A(I::Item),
    B(I::Item),
}

impl<I: Iterator> Clone for MergeIter<I>
where
    I: Clone,
    I::Item: Clone,
{
    fn clone(&self) -> Self {
        Self {
            a: self.a.clone(),
            b: self.b.clone(),
            peeked: self.peeked.clone(),
        }
    }
}

impl<I: Iterator> fmt::Debug for MergeIter<I>
where
    I: fmt::Debug,
    I::Item: fmt::Debug,
{
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("MergeIter")
            .field("a", &self.a)
            .field("b", &self.b)
            .field("peeked", &self.peeked)
            .finish()
    }
}

impl<I: Iterator> MergeIter<I> {
    pub fn new(a: I, b: I) -> Self {
        Self { a, b, peeked: None }
    }

    pub fn nexts<C>(&mut self, cmp: C) -> (Option<I::Item>, Option<I::Item>)
    where
        C: Fn(&I::Item, &I::Item) -> std::cmp::Ordering,
        I: FusedIterator,
    {
        let (mut a_next, mut b_next);
        match self.peeked.take() {
            Some(Peeked::A(next)) => {
                a_next = Some(next);
                b_next = self.b.next();
            }
            Some(Peeked::B(next)) => {
                a_next = self.a.next();
                b_next = Some(next);
            }
            None => {
                a_next = self.a.next();
                b_next = self.b.next();
            }
        }
        if let (Some(a1), Some(b1)) = (&a_next, &b_next) {
            match cmp(a1, b1) {
                std::cmp::Ordering::Less => self.peeked = b_next.take().map(Peeked::B),
                std::cmp::Ordering::Greater => self.peeked = a_next.take().map(Peeked::A),
                std::cmp::Ordering::Equal => {}
            }
        }
        (a_next, b_next)
    }

    pub fn lens(&self) -> (usize, usize)
    where
        I: ExactSizeIterator,
    {
        let (a_incr, b_incr) = match self.peeked {
            Some(Peeked::A(_)) => (1, 0),
            Some(Peeked::B(_)) => (0, 1),
            None => (0, 0),
        };
        (self.a.len() + a_incr, self.b.len() + b_incr)
    }
}
