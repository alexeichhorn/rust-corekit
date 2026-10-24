use std::{collections::HashMap, hash::Hash};

/// Index iterable items by a key derived from each item.
pub trait KeyBy: IntoIterator + Sized {
    /// Consume the input into a map, keeping the last value for each equal key.
    ///
    /// Use `.iter().key_by(...)` or a slice to borrow values instead of moving them.
    /// No items are cloned implicitly. Map iteration order is unspecified.
    ///
    /// ```
    /// use corekit::prelude::*;
    ///
    /// let names = vec![String::from("Alex"), String::from("Sam")];
    /// let by_name = names.iter().key_by(|name| name.as_str());
    /// assert_eq!(by_name["Alex"], &names[0]);
    ///
    /// let by_length = names.key_by(|name| name.len());
    /// assert_eq!(by_length[&3], "Sam");
    /// ```
    fn key_by<K: Eq + Hash>(self, mut key: impl FnMut(&Self::Item) -> K) -> HashMap<K, Self::Item> {
        self.into_iter().map(|item| (key(&item), item)).collect()
    }
}

impl<I: IntoIterator> KeyBy for I {}
