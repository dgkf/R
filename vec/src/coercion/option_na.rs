use crate::atomic::{Atomic, AtomicMode};
use super::{NaAble, IntoLogical, IntoNumeric};


/// OptionNa Types
///
/// OptionNa is used as a bespoke Option type that represents whether a value
/// is NA or not. Some types (f32, f64) have NaN values that can be used
/// in place of an NA, while others need to be wrapped in an OptionNa to 
/// provide this behavior.
///
#[derive(Debug, Clone, PartialEq)]
pub struct OptionNa<T>(pub Option<T>);

impl<T> OptionNa<T> {
    #[inline]
    pub fn map<U, F>(self, f: F) -> OptionNa<U>
    where
        F: FnOnce(T) -> U,
    {
        let OptionNa(inner) = self;
        OptionNa(inner.map(f))
    }
}

impl<T> Atomic for OptionNa<T> 
where
    T: Clone + IntoLogical + IntoNumeric + AtomicMode
{
}

impl<T> AtomicMode for OptionNa<T>
where
    T: AtomicMode
{
    fn is_numeric() -> bool { T::is_numeric() }
    fn is_logical() -> bool { T::is_logical() }
    fn is_integer() -> bool { T::is_integer() }
    fn is_character() -> bool { T::is_character() }
}

impl<T> NaAble for OptionNa<T> {
    #[inline]
    fn na() -> Self {
        OptionNa(None)
    }

    #[inline]
    fn is_na(&self) -> bool {
        let OptionNa(x) = self;
        x.is_none()
    }
}

impl<T, U> IntoLogical for OptionNa<T> 
where
    T: IntoLogical<Output = U>
{
    type Output = OptionNa<U>;

    #[inline]
    fn as_logical(self) -> Self::Output {
        self.map(|x| x.as_logical())
    }
}

impl<T, O> IntoNumeric for OptionNa<T> 
where
    T: IntoNumeric<Output = O>
{
    type Output = OptionNa<O>;
    fn as_numeric(self) -> Self::Output {
        self.map(|x| x.as_numeric())
    }
}
