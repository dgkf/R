use crate::vector::coercion::CoerceInto;
use super::OptionNa;

/// Defines a Minimally Numeric representation
///
/// For a type to be atomic, it must declare a minimally numeric
/// representation. This representation is the lowest-overhead representation
/// that represents the value with the least data loss. When performing 
/// mathematical operations, the two minimal numeric types are used to 
/// determine what output type should be preferred for the calculation.
///
/// Just because a minimally numeric type is defined doesn't mean that we 
/// will implicitly define math on all objects. For example `"3" + 1`
/// may not be expected to succeed. Whether 
///
pub trait AsMinimallyNumeric: CoerceInto<Self::As> + Sized {
    type As;
    #[inline]
    fn as_minimally_numeric(self) -> Self::As {
        self.coerce()
    }
}

pub type Logical = OptionNa<bool>;
pub type Numeric = f64;
pub type Integer = OptionNa<i32>;
pub type Character = OptionNa<String>;
