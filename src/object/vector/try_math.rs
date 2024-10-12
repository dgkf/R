use std::cell::RefCell;

use crate::object::rep::Rep;
use crate::object::reptype::RepType;
use crate::object::coercion::{AtomicMode, CommonNum, CoercibleInto, MinimallyNumeric};
use crate::object::iterators::{map_common_numeric, zip_recycle};


pub trait TryAdd<Rhs>
{
    type Output;

    fn try_add(self, rhs: Rhs) -> Result<Self::Output, (usize, usize)>;
}

impl<L, R, C, O, LNum, RNum> TryAdd<Rep<R>> for Rep<L>
where
    L: AtomicMode + Default + Clone + MinimallyNumeric<As = LNum> + CoercibleInto<LNum>,
    R: AtomicMode + Default + Clone + MinimallyNumeric<As = RNum> + CoercibleInto<RNum>,
    (LNum, RNum): CommonNum<Common = C>,
    C: Clone + std::ops::Add<Output = O>,
    RepType<C>: From<Vec<O>>,
{
    type Output = Rep<C>;
    fn try_add(self, rhs: Rep<R>) -> Result<Self::Output, (usize, usize)> {
        let lc = self.inner().clone();
        let lb = lc.borrow();
        let lhs = lb.iter();

        let rc = rhs.inner().clone();
        let rb = rc.borrow();
        let rhs = rb.iter();

        let result = RepType::from(
            map_common_numeric(zip_recycle(lhs, rhs))
                .map(|(l, r)| l + r)
                .collect::<Vec<O>>(),
        );

        Ok(Rep(RefCell::new(result)))
    }
}
