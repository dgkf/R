use crate::vector::coercion::{CommonNum, map_common_numeric, zip_recycle};
use crate::vector::types::atomic::{Atomic, IntoAtomic};
use crate::vector::types::modes::AsMinimallyNumeric;
use crate::vector::Vector;

pub trait Pow {
    type Output;
    /// raise self to the rhs power
    fn power(self, rhs: Self) -> Self::Output;
}

impl<T, TNum, N, A> std::ops::Neg for Vector<T> 
where
    T: Atomic + AsMinimallyNumeric<As = TNum>,
    TNum: std::ops::Neg<Output = N>,
    N: IntoAtomic<Atom = A>,
    A: Atomic,
{
    type Output = Vector<A>;
    fn neg(self) -> Self::Output {
        Vector::from(
            self.inner().into_iter()
                .map(|i| i.as_minimally_numeric().neg())
                .collect::<Vec<_>>()
        )
    }
}

impl<L, R, C, O, A, LNum, RNum> std::ops::Add<Vector<R>> for Vector<L> 
where
    L: Atomic + AsMinimallyNumeric<As = LNum>,
    R: Atomic + AsMinimallyNumeric<As = RNum>,
    (LNum, RNum): CommonNum<Common = C>,
    C: std::ops::Add<Output = O>,
    O: IntoAtomic<Atom = A>,
    A: Atomic,
{
    type Output = Vector<A>;
    fn add(self, rhs: Vector<R>) -> Self::Output {
        Vector::from(
            map_common_numeric(zip_recycle(self.inner(), rhs.inner()))
                .map(|(l, r)| l + r)
                .collect::<Vec<_>>()
        )
    }
}

impl<L, R, C, O, A, LNum, RNum> std::ops::Sub<Vector<R>> for Vector<L> 
where
    L: Atomic + AsMinimallyNumeric<As = LNum>,
    R: Atomic + AsMinimallyNumeric<As = RNum>,
    (LNum, RNum): CommonNum<Common = C>,
    C: std::ops::Sub<Output = O>,
    O: IntoAtomic<Atom = A>,
    A: Atomic,
{
    type Output = Vector<A>;
    fn sub(self, rhs: Vector<R>) -> Self::Output {
        Vector::from(
            map_common_numeric(zip_recycle(self.inner(), rhs.inner()))
                .map(|(l, r)| l - r)
                .collect::<Vec<_>>()
        )
    }
}

impl<L, R, C, O, A, LNum, RNum> std::ops::Mul<Vector<R>> for Vector<L> 
where
    L: Atomic + AsMinimallyNumeric<As = LNum>,
    R: Atomic + AsMinimallyNumeric<As = RNum>,
    (LNum, RNum): CommonNum<Common = C>,
    C: std::ops::Mul<Output = O>,
    O: IntoAtomic<Atom = A>,
    A: Atomic,
{
    type Output = Vector<A>;
    fn mul(self, rhs: Vector<R>) -> Self::Output {
        Vector::from(
            map_common_numeric(zip_recycle(self.inner(), rhs.inner()))
                .map(|(l, r)| l * r)
                .collect::<Vec<_>>()
        )
    }
}

impl<L, R, C, O, A, LNum, RNum> std::ops::Div<Vector<R>> for Vector<L> 
where
    L: Atomic + AsMinimallyNumeric<As = LNum>,
    R: Atomic + AsMinimallyNumeric<As = RNum>,
    (LNum, RNum): CommonNum<Common = C>,
    C: std::ops::Div<Output = O>,
    O: IntoAtomic<Atom = A>,
    A: Atomic,
{
    type Output = Vector<A>;
    fn div(self, rhs: Vector<R>) -> Self::Output {
        Vector::from(
            map_common_numeric(zip_recycle(self.inner(), rhs.inner()))
                .map(|(l, r)| l / r)
                .collect::<Vec<_>>()
        )
    }
}

impl<L, O, A, LNum> Pow for Vector<L> 
where
    L: Atomic + AsMinimallyNumeric<As = LNum>,
    LNum: Pow<Output = O>,
    O: IntoAtomic<Atom = A>,
    A: Atomic,
{
    type Output = Vector<A>;
    fn power(self, rhs: Self) -> Self::Output {
        Vector::from(
            map_common_numeric(zip_recycle(self.inner(), rhs.inner()))
                .map(|(l, r)| l.power(r))
                .collect::<Vec<_>>()
        )
    }
}

impl<L, R, C, O, A, LNum, RNum> std::ops::Rem<Vector<R>> for Vector<L> 
where
    L: Atomic + AsMinimallyNumeric<As = LNum>,
    R: Atomic + AsMinimallyNumeric<As = RNum>,
    (LNum, RNum): CommonNum<Common = C>,
    C: std::ops::Rem<Output = O>,
    O: IntoAtomic<Atom = A>,
    A: Atomic,
{
    type Output = Vector<A>;
    fn rem(self, rhs: Vector<R>) -> Self::Output {
        Vector::from(
            map_common_numeric(zip_recycle(self.inner(), rhs.inner()))
                .map(|(l, r)| l.rem(r))
                .collect::<Vec<_>>()
        )
    }
}
