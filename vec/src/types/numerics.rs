use crate::atomic::{Atomic, AtomicMode};
use crate::coercion::{NaAble, AsMinimallyNumeric, Pow};


impl Atomic for f32 {}
impl AsMinimallyNumeric for f32 { type As = f32; }
impl AtomicMode for f32 { 
    fn is_numeric() -> bool { 
        true
    }
}

impl Pow for f32 {
    type Output = f32;
    fn power(self, rhs: Self) -> Self::Output {
        f32::powf(self, rhs)
    }
}

impl NaAble for f32 {
    type From = f32;
    fn inner(self) -> Self::From { self }
    fn is_na(&self) -> bool { f32::is_nan(*self) }
    fn na() -> Self { std::f32::NAN }
}


impl Atomic for f64 {}
impl AsMinimallyNumeric for f64 { type As = f64; }
impl AtomicMode for f64 { 
    fn is_numeric() -> bool {
        true
    }
}

impl NaAble for f64 {
    type From = f64;
    fn inner(self) -> Self::From { self }
    fn is_na(&self) -> bool { f64::is_nan(*self) }
    fn na() -> Self { std::f64::NAN }
}

impl Pow for f64 {
    type Output = f64;
    fn power(self, rhs: Self) -> Self::Output {
        f64::powf(self, rhs)
    }
}
