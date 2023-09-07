//! Atomic 
//!
//! Intended to be a generic trait for all vector-able elements. The atomic
//! trait is relatively minimal, expecting only that values have an `NA` 
//! representation and that they declare a mode.
//!
//! Many internal operations on vectors are constrained to atomic values to
//! rely on these basic capabilities
//! 

use crate::vector::coercion::*;
use super::NaAble;

pub trait Atomic: Sized {}

pub trait AtomicMode {
    fn is_numeric(&self) -> bool { false }
    fn is_logical(&self) -> bool { false }
    fn is_integer(&self) -> bool { false }
    fn is_character(&self) -> bool { false }
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
