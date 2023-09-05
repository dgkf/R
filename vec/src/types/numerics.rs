use crate::atomic::{Atomic, AtomicMode};
use crate::coercion::{NaAble, IntoLogical, IntoNumeric};


impl Atomic for f32 {}

impl AtomicMode for f32 { 
    fn is_numeric() -> bool { 
        true
    }
}

impl NaAble for f32 {
    fn is_na(&self) -> bool { f32::is_nan(*self) }
    fn na() -> Self { std::f32::NAN }
}

impl IntoLogical for f32 {
    type Output = bool;
    fn as_logical(self) -> bool {
        match self.partial_cmp(&0.0) {
            Some(std::cmp::Ordering::Equal) => false,
            _ => true,
        }
    }
}

impl IntoNumeric for f32 {
    type Output = f32;
    fn as_numeric(self) -> Self::Output {
        self
    }
}



impl Atomic for f64 {}

impl AtomicMode for f64 { 
    fn is_numeric() -> bool {
        true
    }
}

impl NaAble for f64 {
    fn is_na(&self) -> bool { f64::is_nan(*self) }
    fn na() -> Self { std::f64::NAN }
}

impl IntoLogical for f64 {
    type Output = bool;
    fn as_logical(self) -> bool {
        match self.partial_cmp(&0.0) {
            Some(std::cmp::Ordering::Equal) => false,
            _ => true,
        }
    }
}

impl IntoNumeric for f64 {
    type Output = f64;
    fn as_numeric(self) -> Self::Output {
        self
    }
}

