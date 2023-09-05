/// Whether the object has an NA representation
///
/// The NA-able trait provides common interfaces for all atomics to provide
/// NA values. Some atomic types like `f32` and `f64` are NA-able using their
/// built-in NaN representation. 
///
pub trait NaAble {
    fn is_na(&self) -> bool;
    fn na() -> Self;
}

/// Specifies how a type can be coerced into an NA-able type
pub trait IntoNaAble: Sized {
    type Output;
    fn into(self) -> Self::Output;
}

impl<T> IntoNaAble for T 
where
    T: NaAble
{
    type Output = T;
    fn into(self) -> Self::Output {
        self
    }
}
