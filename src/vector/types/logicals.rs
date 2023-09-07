use crate::vector::vecops::Pow;
use super::atomic::{AtomicMode, IntoAtomic};
use super::modes::AsMinimallyNumeric;
use super::OptionNa;

impl AsMinimallyNumeric for bool { type As = i8; }
impl IntoAtomic for bool { type Atom = OptionNa<Self>; }
impl AtomicMode for bool {
    fn is_logical() -> bool {
        true
    }
}

impl Pow for bool {
    type Output = bool;
    fn power(self, _: Self) -> Self::Output {
        self
    }
}
