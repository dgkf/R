use crate::vector::vecops::Pow;
use super::modes::AsMinimallyNumeric;
use super::atomic::{IntoAtomic, AtomicMode};
use super::OptionNa;

impl AsMinimallyNumeric for i8 { type As = Self; }
impl IntoAtomic for i8 { type Atom = OptionNa<Self>; }
impl AtomicMode for i8 { fn is_integer() -> bool { true } }
impl Pow for i8 {
    type Output = f32;
    fn power(self, rhs: Self) -> Self::Output {
        f32::powf(self as f32, rhs as f32)
    }
}

impl AsMinimallyNumeric for i16 { type As = Self; }
impl IntoAtomic for i16 { type Atom = OptionNa<Self>; }
impl AtomicMode for i16 { fn is_integer() -> bool { true } }
impl Pow for i16 {
    type Output = f32;
    fn power(self, rhs: Self) -> Self::Output {
        f32::powf(self as f32, rhs as f32)
    }
}

impl AsMinimallyNumeric for i32 { type As = Self; }
impl IntoAtomic for i32 { type Atom = OptionNa<Self>; }
impl AtomicMode for i32 { fn is_integer() -> bool { true } }
impl Pow for i32 {
    type Output = f64;
    fn power(self, rhs: Self) -> Self::Output {
        f64::powf(self as f64, rhs as f64)
    }
}

impl AsMinimallyNumeric for i64 { type As = Self; }
impl IntoAtomic for i64 { type Atom = OptionNa<Self>; }
impl AtomicMode for i64 { fn is_integer() -> bool { true } }
impl Pow for i64 {
    type Output = f64;
    fn power(self, rhs: Self) -> Self::Output {
        f64::powf(self as f64, rhs as f64)
    }
}

impl AsMinimallyNumeric for i128 { type As = Self; }
impl IntoAtomic for i128 { type Atom = OptionNa<Self>; }
impl AtomicMode for i128 { fn is_integer() -> bool { true } }
impl Pow for i128 {
    type Output = f64;
    fn power(self, rhs: Self) -> Self::Output {
        f64::powf(self as f64, rhs as f64)
    }
}
