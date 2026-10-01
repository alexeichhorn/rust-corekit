/// Map borrowed elements of a slice, vector, or array into a new vector.
pub trait MapVec<T> {
    /// Eagerly map each element in order without consuming the input.
    ///
    /// The output may borrow from the input. No elements are cloned implicitly.
    ///
    /// ```
    /// use corekit::prelude::*;
    ///
    /// let names = vec![String::from("Alex"), String::from("Sam")];
    /// let mut index = 0;
    /// let indexed = names.map_vec(|name| {
    ///     index += 1;
    ///     (index, name.as_str())
    /// });
    ///
    /// assert_eq!(indexed, vec![(1, "Alex"), (2, "Sam")]);
    /// assert_eq!(names.len(), 2);
    /// ```
    fn map_vec<'a, U>(&'a self, f: impl FnMut(&'a T) -> U) -> Vec<U>
    where
        T: 'a;
}

impl<T> MapVec<T> for [T] {
    fn map_vec<'a, U>(&'a self, f: impl FnMut(&'a T) -> U) -> Vec<U>
    where
        T: 'a,
    {
        self.iter().map(f).collect()
    }
}
