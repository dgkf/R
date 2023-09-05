/// Atomic 
///
/// Intended to be a generic trait for all vector-able elements. Ideally
/// would encompass things like coercion traits and operator implementations.
/// For now just requires Clone.
/// 
use super::coercion::*;
pub trait Atomic: 
    Clone + 
    NaAble + 
    IntoLogical + 
    IntoNumeric + 
    AtomicMode {}

pub trait AtomicMode {
    fn is_numeric() -> bool { false }
    fn is_logical() -> bool { false }
    fn is_integer() -> bool { false }
    fn is_character() -> bool { false }
}

pub trait IntoAtomic {
    type Output;
    fn into(self) -> Self::Output;
}

impl<T> IntoAtomic for T 
where
    T: Atomic,
{
    type Output = T;
    fn into(self) -> Self::Output {
        self
    }
}
