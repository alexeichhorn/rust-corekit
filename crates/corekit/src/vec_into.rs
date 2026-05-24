/// Extension trait for converting every element in a `Vec` with `Into`.
pub trait VecInto<D> {
    /// Convert a `Vec<E>` into a `Vec<D>` where each element implements `Into<D>`.
    fn vec_into(self) -> Vec<D>;
}

impl<E, D> VecInto<D> for Vec<E>
where
    E: Into<D>,
{
    fn vec_into(self) -> Vec<D> {
        self.into_iter().map(Into::into).collect()
    }
}
