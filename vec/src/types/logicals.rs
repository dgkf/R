use crate::atomic::{AtomicMode, IntoAtomic};
use crate::coercion::{OptionNa, IntoLogical, IntoNumeric};


impl AtomicMode for bool {
    fn is_logical() -> bool {
        true
    }
}

impl IntoAtomic for bool {
    type Output = OptionNa<bool>;
    fn into(self) -> Self::Output {
        OptionNa(Some(self))
    }
}

impl IntoLogical for bool {
    type Output = bool;
    fn as_logical(self) -> bool {
        self
    }
}

impl IntoNumeric for bool {
    type Output = i32;
    fn as_numeric(self) -> Self::Output {
        self as i32
    }
}

