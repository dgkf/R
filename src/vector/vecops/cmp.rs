use crate::vector::Vector; 
use crate::vector::types::{atomic::Atomic, OptionNa};
use crate::vector::coercion::{CoerceInto, CommonLog};

/// Vectorized Partial Comparison
///
/// This trait provides a vectorized equivalent to [PartialCmp]. It does
/// not provide implementations that will overload base operators, but does
/// provide new vectorized methods as substitutes.
///
///
pub trait VecPartialCmp<Rhs> {
    type Output;
    fn vec_gt(self, rhs: Rhs) -> Self::Output;
    fn vec_gte(self, rhs: Rhs) -> Self::Output;
    fn vec_lt(self, rhs: Rhs) -> Self::Output;
    fn vec_lte(self, rhs: Rhs) -> Self::Output;
    fn vec_eq(self, rhs: Rhs) -> Self::Output;
    fn vec_neq(self, rhs: Rhs) -> Self::Output;
}

impl<L, R, C> VecPartialCmp<Vector<R>> for Vector<L>
where
    L: Atomic + CoerceInto<C>,
    R: Atomic + CoerceInto<C>,
    (L, R): CommonLog<Common = C>,
    C: PartialOrd,
{
    type Output = Vector<OptionNa<bool>>;

    fn vec_gt(self, rhs: Vector<R>) -> Self::Output {
        use core::cmp::Ordering::*;
        Vector::from(
            self.vectorized_partial_cmp(rhs)
                .into_iter()
                .map(|i| match i {
                    Some(Greater) => OptionNa(Some(true)),
                    Some(_) => OptionNa(Some(false)),
                    None => OptionNa(None),
                })
                .collect::<Vec<_>>()
        )
    }

    fn vec_gte(self, rhs: Vector<R>) -> Self::Output {
        use core::cmp::Ordering::*;
        Vector::from(
            self.vectorized_partial_cmp(rhs)
                .into_iter()
                .map(|i| match i {
                    Some(Greater | Equal) => OptionNa(Some(true)),
                    Some(_) => OptionNa(Some(false)),
                    None => OptionNa(None),
                })
                .collect::<Vec<_>>()
        )
    }

    fn vec_lt(self, rhs: Vector<R>) -> Self::Output {
        use core::cmp::Ordering::*;
        Vector::from(
            self.vectorized_partial_cmp(rhs)
                .into_iter()
                .map(|i| match i {
                    Some(Less) => OptionNa(Some(true)),
                    Some(_) => OptionNa(Some(false)),
                    None => OptionNa(None),
                })
                .collect::<Vec<_>>()
        )
    }

    fn vec_lte(self, rhs: Vector<R>) -> Self::Output {
        use core::cmp::Ordering::*;
        Vector::from(
            self.vectorized_partial_cmp(rhs)
                .into_iter()
                .map(|i| match i {
                    Some(Less | Equal) => OptionNa(Some(true)),
                    Some(_) => OptionNa(Some(false)),
                    None => OptionNa(None),
                })
                .collect::<Vec<_>>()
        )
    }

    fn vec_eq(self, rhs: Vector<R>) -> Self::Output {
        use core::cmp::Ordering::*;
        Vector::from(
            self.vectorized_partial_cmp(rhs)
                .into_iter()
                .map(|i| match i {
                    Some(Equal) => OptionNa(Some(true)),
                    Some(_) => OptionNa(Some(false)),
                    None => OptionNa(None),
                })
                .collect::<Vec<_>>()
        )
    }

    fn vec_neq(self, rhs: Vector<R>) -> Self::Output {
        use core::cmp::Ordering::*;
        Vector::from(
            self.vectorized_partial_cmp(rhs)
                .into_iter()
                .map(|i| match i {
                    Some(Equal) => OptionNa(Some(false)),
                    Some(_) => OptionNa(Some(true)),
                    None => OptionNa(None),
                })
                .collect::<Vec<_>>()
        )
    }
}
