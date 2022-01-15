use crate::{map::iter::DrainFilterNavigator, RbTreeSet};

use std::{fmt, iter::FusedIterator};

impl<T> RbTreeSet<T> {
    /// Creates an iterator that visits all values in ascending order and uses a closure to determine if a value should be removed.
    ///
    /// If the closure returns true, the value is removed from the set and yielded. If the closure returns false, or panics, the value remains in the set and will not be yielded.
    ///
    /// If the iterator is only partially consumed or not consumed at all, each of the remaining values is still subjected to the closure and removed and dropped if it returns true.
    ///
    /// It is unspecified how many more values will be subjected to the closure if a panic occurs in the closure, or if a panic occurs while dropping a value, or if the DrainFilter itself is leaked.
    ///
    /// # Examples
    ///
    /// Splitting a set into even and odd values, reusing the original set:
    ///
    /// ```
    /// use rb_tree::RbTreeSet;
    ///
    /// let mut set: RbTreeSet<i32> = (0..8).collect();
    /// let evens: RbTreeSet<_> = set.drain_filter(|v| v % 2 == 0).collect();
    /// let odds = set;
    /// assert_eq!(evens.into_iter().collect::<Vec<_>>(), vec![0, 2, 4, 6]);
    /// assert_eq!(odds.into_iter().collect::<Vec<_>>(), vec![1, 3, 5, 7]);
    /// ```
    pub fn drain_filter<'a, F>(&'a mut self, pred: F) -> DrainFilter<'a, T, F>
    where
        T: Ord,
        F: 'a + FnMut(&T) -> bool,
    {
        DrainFilter {
            pred,
            nav: DrainFilterNavigator::new(&mut self.map),
        }
    }
}

pub struct DrainFilter<'a, T: 'a + Ord, F: 'a + FnMut(&T) -> bool> {
    pred: F,
    nav: DrainFilterNavigator<'a, T, ()>,
}

impl<'a, T, F> Drop for DrainFilter<'a, T, F>
where
    T: 'a + Ord,
    F: 'a + FnMut(&T) -> bool,
{
    fn drop(&mut self) {
        let mut mapped_pred = |k: &T, _: &mut ()| (self.pred)(k);
        unsafe {
            self.nav.drop_nav(&mut mapped_pred);
        }
    }
}

impl<T, F> fmt::Debug for DrainFilter<'_, T, F>
where
    T: fmt::Debug + Ord,
    F: FnMut(&T) -> bool,
{
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_tuple("DrainFilter")
            .field(&self.nav.peek().map(|(k, _)| k))
            .finish()
    }
}

impl<'a, T, F> Iterator for DrainFilter<'a, T, F>
where
    T: 'a + Ord,
    F: 'a + FnMut(&T) -> bool,
{
    type Item = T;

    fn next(&mut self) -> Option<Self::Item> {
        let mut mapped_pred = |k: &T, _: &mut ()| (self.pred)(k);
        self.nav.next(&mut mapped_pred).map(|(k, _)| k)
    }

    fn size_hint(&self) -> (usize, Option<usize>) {
        self.nav.size_hint()
    }
}

impl<T, F> FusedIterator for DrainFilter<'_, T, F>
where
    T: Ord,
    F: FnMut(&T) -> bool,
{
}
