/// Whether the object has an NA representation
///
/// The NA-able trait provides common interfaces for all atomics to provide
/// NA values. Some atomic types like `f32` and `f64` are NA-able using their
/// built-in NaN representation. 
///
pub trait NaAble {
    type From: Default;
    fn inner(self) -> Self::From;
    fn is_na(&self) -> bool;
    fn na() -> Self;
}
