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
    AsMinimallyNumeric + 
    AtomicMode {}

pub trait AtomicMode {
    fn is_numeric() -> bool { false }
    fn is_logical() -> bool { false }
    fn is_integer() -> bool { false }
    fn is_character() -> bool { false }
}

pub trait IntoAtomic: CoerceInto<Self::Atom> + Sized {
    type Atom;
    fn atomic(self) -> Self::Atom {
        self.coerce()
    }
}

impl<T> IntoAtomic for T 
where
    T: Atomic,
    T: CoerceInto<T>,
{
    type Atom = T;
}
