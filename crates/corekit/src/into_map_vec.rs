/// Map iterable items into a new vector.
pub trait IntoMapVec: IntoIterator + Sized {
    /// Eagerly map each item from `self.into_iter()` in order.
    ///
    /// An owned collection moves its items into the mapper; a slice or borrowing
    /// iterator yields references. No items are cloned implicitly.
    /// Use [`MapVec::map_vec`](crate::MapVec::map_vec) to borrow a collection directly.
    ///
    /// ```
    /// use corekit::prelude::*;
    ///
    /// let names = vec![String::from("Alex"), String::from("Sam")];
    /// let bytes = names.into_map_vec(|name| name.into_bytes());
    ///
    /// assert_eq!(bytes, vec![b"Alex".to_vec(), b"Sam".to_vec()]);
    /// ```
    fn into_map_vec<U>(self, f: impl FnMut(Self::Item) -> U) -> Vec<U> {
        self.into_iter().map(f).collect()
    }

    /// Map items from `self.into_iter()` in order, stopping at the first error.
    ///
    /// Ownership follows [`Self::into_map_vec`]. On error, collected values are
    /// dropped and later items are not mapped. Earlier mapper side effects remain.
    fn try_into_map_vec<U, E>(self, f: impl FnMut(Self::Item) -> Result<U, E>) -> Result<Vec<U>, E> {
        self.into_iter().map(f).collect()
    }
}

impl<I: IntoIterator> IntoMapVec for I {}
