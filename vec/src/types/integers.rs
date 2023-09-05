use crate::atomic::{AtomicMode, IntoAtomic};
use crate::coercion::{OptionNa, IntoNaAble, IntoLogical, IntoNumeric};


impl AtomicMode for i8 { 
    fn is_integer() -> bool { 
        true 
    } 
}

impl IntoAtomic for i8 {
    type Output = OptionNa<i8>;
    fn into(self) -> Self::Output {
        OptionNa(Some(self))
    }
}

impl IntoNaAble for i8 {
    type Output = OptionNa<i8>;
    fn into(self) -> Self::Output {
        OptionNa(Some(self))
    }
}

impl IntoLogical for i8 {
    type Output = bool;
    fn as_logical(self) -> bool {
        match self {
            0 => false,
            _ => true,
        }
    }
}



impl AtomicMode for i16 { 
    fn is_integer() -> bool { 
        true
    }
}

impl IntoAtomic for i16 {
    type Output = OptionNa<i16>;
    fn into(self) -> Self::Output {
        OptionNa(Some(self))
    }
}

impl IntoNaAble for i16 {
    type Output = OptionNa<i16>;
    fn into(self) -> Self::Output {
        OptionNa(Some(self))
    }
}

impl IntoLogical for i16 {
    type Output = bool;
    fn as_logical(self) -> bool {
        match self {
            0 => false,
            _ => true,
        }
    }
}



impl AtomicMode for i32 {
    fn is_integer() -> bool {
        true
    }
}

impl IntoAtomic for i32 {
    type Output = OptionNa<i32>;
    fn into(self) -> Self::Output {
        OptionNa(Some(self))
    }
}

impl IntoNaAble for i32 {
    type Output = OptionNa<i32>;
    fn into(self) -> Self::Output {
        OptionNa(Some(self))
    }
}

impl IntoLogical for i32 {
    type Output = bool;
    fn as_logical(self) -> bool {
        match self {
            0 => false,
            _ => true,
        }
    }
}

impl IntoNumeric for i32 {
    type Output = i32;
    fn as_numeric(self) -> Self::Output {
        self
    }
}



impl AtomicMode for i64 {
    fn is_integer() -> bool {
        true
    }
}

impl IntoAtomic for i64 {
    type Output = OptionNa<i64>;
    fn into(self) -> Self::Output {
        OptionNa(Some(self))
    }
}

impl IntoNaAble for i64 {
    type Output = OptionNa<i64>;
    fn into(self) -> Self::Output {
        OptionNa(Some(self))
    }
}

impl IntoLogical for i64 {
    type Output = bool;
    fn as_logical(self) -> bool {
        match self {
            0 => false,
            _ => true,
        }
    }
}

impl IntoNumeric for i64 {
    type Output = i64;
    fn as_numeric(self) -> Self::Output {
        self
    }
}



impl AtomicMode for i128 {
    fn is_integer() -> bool {
        true
    }
}

impl IntoAtomic for i128 {
    type Output = OptionNa<i128>;
    fn into(self) -> Self::Output {
        OptionNa(Some(self))
    }
}

impl IntoNaAble for i128 {
    type Output = OptionNa<i128>;
    fn into(self) -> Self::Output {
        OptionNa(Some(self))
    }
}

impl IntoLogical for i128 {
    type Output = bool;
    fn as_logical(self) -> bool {
        match self {
            0 => false,
            _ => true,
        }
    }
}

impl IntoNumeric for i128 {
    type Output = i128;
    fn as_numeric(self) -> Self::Output {
        self
    }
}

