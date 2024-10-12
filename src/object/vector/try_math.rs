use std::cell::RefCell;

use crate::error::Error;
use crate::lang::Signal;
use crate::object::coercion::{AtomicMode, CoercibleInto, CommonNum, MinimallyNumeric};
use crate::object::rep::Rep;
use crate::object::reptype::RepType;

pub trait TryAdd<Rhs = Self> {
    type Output;

    fn try_add(self, rhs: Rhs) -> Result<Self::Output, Signal>;
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
    fn try_add(self, rhs: Rep<R>) -> Result<Self::Output, Signal> {
        let mut lc = self.iter_values();
        let mut rc = rhs.iter_values();

        let x = lc.by_ref().zip(rc.by_ref()).map(|(l, r)| {
            (
                CoercibleInto::<LNum>::coerce_into(l.clone()),
                CoercibleInto::<RNum>::coerce_into(r.clone()),
            )
                .into_common()
        });

        let result = x.map(|(l, r)| l + r).collect::<Vec<O>>();

        // We check whether they were recyclable after the fact
        let l_extra = lc.count();
        let r_extra = rc.count();

        // we need to add one to the count because above we consumed more one than allocated
        // in the result vector (for the iterator to stop yielding)
        if l_extra != 0 {
            return Err(Signal::Error(Error::NonRecyclableLengths(
                l_extra + result.len() + 1,
                result.len(),
            )));
        } else if r_extra != 0 {
            return Err(Signal::Error(Error::NonRecyclableLengths(
                result.len(),
                r_extra + result.len() + 1,
            )));
        }

        Ok(Rep(RefCell::new(RepType::from(result))))
    }
}
