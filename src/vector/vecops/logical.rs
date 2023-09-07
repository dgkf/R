use crate::vector::coercion::{CoerceInto, zip_recycle};
use crate::vector::types::atomic::{Atomic, IntoAtomic};
use crate::vector::types::modes::Logical;
use crate::vector::Vector;

impl<L, R, O, A> std::ops::BitOr<Vector<R>> for Vector<L> 
where
    L: Atomic + CoerceInto<Logical>,
    R: Atomic + CoerceInto<Logical>,
    Logical: std::ops::BitOr<Logical, Output = O>,
    O: IntoAtomic<Atom = A>,
    A: Atomic,
{
    type Output = Vector<A>;
    fn bitor(self, rhs: Vector<R>) -> Self::Output {
        Vector::from(
            zip_recycle(self.inner(), rhs.inner())
                .map(|(l, r)| CoerceInto::<Logical>::coerce(l) | CoerceInto::<Logical>::coerce(r))
                .collect::<Vec<O>>()
        )
    }
}

impl<L, R, O, A> std::ops::BitAnd<Vector<R>> for Vector<L> 
where
    L: Atomic + CoerceInto<Logical>,
    R: Atomic + CoerceInto<Logical>,
    Logical: std::ops::BitAnd<Logical, Output = O>,
    O: IntoAtomic<Atom = A>,
    A: Atomic,
{
    type Output = Vector<A>;
    fn bitand(self, rhs: Vector<R>) -> Self::Output {
        Vector::from(
            zip_recycle(self.inner(), rhs.inner())
                .map(|(l, r)| CoerceInto::<Logical>::coerce(l) & CoerceInto::<Logical>::coerce(r))
                .collect::<Vec<O>>()
        )
    }
}
