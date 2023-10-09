use std::str::FromStr;

use super::OptionNA;

pub trait AtomicMode {
    fn is_numeric() -> bool {
        false
    }
    fn is_integer() -> bool {
        false
    }
    fn is_logical() -> bool {
        false
    }
    fn is_character() -> bool {
        false
    }
}

pub trait CoercibleInto<T>: Sized {
    fn coerce_into(self) -> T;
}

impl CoercibleInto<bool> for bool {
    #[inline]
    fn coerce_into(self) -> bool {
        self
    }
}

impl CoercibleInto<i32> for bool {
    #[inline]
    fn coerce_into(self) -> i32 {
        self as i32
    }
}

impl CoercibleInto<i32> for i32 {
    #[inline]
    fn coerce_into(self) -> i32 {
        self
    }
}

impl CoercibleInto<i32> for f64 {
    #[inline]
    fn coerce_into(self) -> i32 {
        self as i32
    }
}

impl CoercibleInto<f64> for f64 {
    #[inline]
    fn coerce_into(self) -> f64 {
        self
    }
}

impl CoercibleInto<OptionNA<f64>> for f64 {
    #[inline]
    fn coerce_into(self) -> OptionNA<f64> {
        OptionNA::Some(self)
    }
}

impl CoercibleInto<OptionNA<i32>> for i32 {
    #[inline]
    fn coerce_into(self) -> OptionNA<i32> {
        OptionNA::Some(self)
    }
}

impl CoercibleInto<OptionNA<bool>> for bool {
    #[inline]
    fn coerce_into(self) -> OptionNA<bool> {
        OptionNA::Some(self)
    }
}

impl CoercibleInto<OptionNA<String>> for String {
    #[inline]
    fn coerce_into(self) -> OptionNA<String> {
        OptionNA::Some(self)
    }
}

impl CoercibleInto<f64> for bool {
    #[inline]
    fn coerce_into(self) -> f64 {
        self as i32 as f64
    }
}

impl CoercibleInto<f64> for i32 {
    #[inline]
    fn coerce_into(self) -> f64 {
        self as f64
    }
}

impl CoercibleInto<String> for String {
    #[inline]
    fn coerce_into(self) -> String {
        self
    }
}

impl CoercibleInto<String> for bool {
    #[inline]
    fn coerce_into(self) -> String {
        self.to_string()
    }
}

impl CoercibleInto<String> for i32 {
    #[inline]
    fn coerce_into(self) -> String {
        self.to_string()
    }
}

impl CoercibleInto<String> for f64 {
    #[inline]
    fn coerce_into(self) -> String {
        self.to_string()
    }
}

impl CoercibleInto<OptionNA<bool>> for OptionNA<bool> {
    #[inline]
    fn coerce_into(self) -> OptionNA<bool> {
        self
    }
}

impl CoercibleInto<OptionNA<f64>> for OptionNA<f64> {
    #[inline]
    fn coerce_into(self) -> OptionNA<f64> {
        self
    }
}

impl CoercibleInto<OptionNA<i32>> for OptionNA<i32> {
    #[inline]
    fn coerce_into(self) -> OptionNA<i32> {
        self
    }
}

impl CoercibleInto<OptionNA<i32>> for OptionNA<f64> {
    #[inline]
    fn coerce_into(self) -> OptionNA<i32> {
        self.map(|i| i as i32)
    }
}

impl CoercibleInto<OptionNA<i32>> for OptionNA<bool> {
    #[inline]
    fn coerce_into(self) -> OptionNA<i32> {
        self.map(|i| i as i32)
    }
}

impl CoercibleInto<OptionNA<f64>> for OptionNA<i32> {
    #[inline]
    fn coerce_into(self) -> OptionNA<f64> {
        self.map(|i| i as f64)
    }
}

impl CoercibleInto<OptionNA<f64>> for OptionNA<bool> {
    #[inline]
    fn coerce_into(self) -> OptionNA<f64> {
        self.map(|i| i as i32 as f64)
    }
}

impl CoercibleInto<bool> for f64 {
    #[inline]
    fn coerce_into(self) -> bool {
        !matches!(self.partial_cmp(&0.0), Some(std::cmp::Ordering::Equal))
    }
}

impl CoercibleInto<OptionNA<bool>> for OptionNA<f64> {
    #[inline]
    fn coerce_into(self) -> OptionNA<bool> {
        self.map(|i| i.coerce_into())
    }
}

impl CoercibleInto<bool> for i32 {
    #[inline]
    fn coerce_into(self) -> bool {
        self != 0
    }
}

impl CoercibleInto<OptionNA<bool>> for OptionNA<i32> {
    #[inline]
    fn coerce_into(self) -> OptionNA<bool> {
        self.map(|i| i.coerce_into())
    }
}

impl<T> CoercibleInto<OptionNA<T>> for OptionNA<String>
where
    T: FromStr,
{
    // this uses an extra `.to_lowercase` that is only necessary for
    // bools, could be removed is specialized
    fn coerce_into(self) -> OptionNA<T> {
        match self {
            OptionNA::Some(s) => s.parse().map_or(OptionNA::NA, |i| OptionNA::Some(i)),
            OptionNA::NA => OptionNA::NA,
        }
    }
}

impl CoercibleInto<OptionNA<String>> for OptionNA<bool> {
    fn coerce_into(self) -> OptionNA<String> {
        self.map(|i| format!("{}", i))
    }
}

impl CoercibleInto<OptionNA<String>> for OptionNA<i32> {
    fn coerce_into(self) -> OptionNA<String> {
        self.map(|i| format!("{}", i))
    }
}

impl CoercibleInto<OptionNA<String>> for OptionNA<f64> {
    fn coerce_into(self) -> OptionNA<String> {
        self.map(|i| format!("{}", i))
    }
}

pub trait MinimallyNumeric {
    type As;
}

impl<T, U> MinimallyNumeric for &T
where
    T: MinimallyNumeric<As = U>,
{
    type As = U;
}

impl MinimallyNumeric for bool {
    type As = i32;
}
impl MinimallyNumeric for i32 {
    type As = i32;
}
impl MinimallyNumeric for f64 {
    type As = f64;
}
impl<T, U> MinimallyNumeric for OptionNA<T>
where
    T: MinimallyNumeric<As = U>,
    OptionNA<T>: CoercibleInto<OptionNA<U>>,
{
    type As = OptionNA<U>;
}

pub trait IntoLogical<T> {
    fn as_logical(&mut self) -> T;
}

impl<T, U> IntoLogical<OptionNA<T>> for OptionNA<U>
where
    U: IntoLogical<T>,
{
    fn as_logical(&mut self) -> OptionNA<T> {
        use OptionNA::*;
        match self {
            Some(x) => Some(U::as_logical(x)),
            _ => NA,
        }
    }
}

impl IntoLogical<bool> for bool {
    fn as_logical(&mut self) -> bool {
        *self
    }
}

impl IntoLogical<bool> for i32 {
    fn as_logical(&mut self) -> bool {
        !matches!(self, 0)
    }
}

impl IntoLogical<bool> for i64 {
    fn as_logical(&mut self) -> bool {
        !matches!(self, 0)
    }
}

impl IntoLogical<bool> for i128 {
    fn as_logical(&mut self) -> bool {
        !matches!(self, 0)
    }
}

impl IntoLogical<bool> for f32 {
    fn as_logical(&mut self) -> bool {
        !matches!((*self).partial_cmp(&0.0), Some(std::cmp::Ordering::Equal))
    }
}

impl IntoLogical<bool> for f64 {
    fn as_logical(&mut self) -> bool {
        !matches!((*self).partial_cmp(&0.0), Some(std::cmp::Ordering::Equal))
    }
}

// Common representation for numeric calculations
pub trait CommonNum: Sized {
    type Common;
    fn into_common(self) -> (Self::Common, Self::Common);
}

// Common representation for comparison and logical calculations
pub trait CommonCmp: Sized {
    type Common;
    fn into_common(self) -> (Self::Common, Self::Common);
}

impl<T, U, V> CommonCmp for (OptionNA<U>, OptionNA<V>)
where
    (U, V): CommonCmp<Common = OptionNA<T>>,
{
    type Common = OptionNA<T>;
    fn into_common(self) -> (OptionNA<T>, OptionNA<T>) {
        use OptionNA::*;
        match self {
            (Some(l), Some(r)) => (l, r).into_common(),
            _ => (NA, NA),
        }
    }
}

#[macro_export]
macro_rules! register {
    (
      $trait:ident, ($lty:ty, $rty:ty) => $target:ty
    ) => {
        // register unification into RHS
        impl $trait for ($lty, $rty)
        where
            $lty: CoercibleInto<$target>,
            $rty: CoercibleInto<$target>,
        {
            type Common = $target;
            fn into_common(self) -> ($target, $target) {
                (
                    CoercibleInto::<$target>::coerce_into(self.0),
                    CoercibleInto::<$target>::coerce_into(self.1),
                )
            }
        }

        // register unification into RHS
        impl $trait for (OptionNA<$lty>, OptionNA<$rty>)
        where
            OptionNA<$lty>: CoercibleInto<OptionNA<$target>>,
            OptionNA<$rty>: CoercibleInto<OptionNA<$target>>,
        {
            type Common = OptionNA<$target>;
            fn into_common(self) -> (Self::Common, Self::Common) {
                (
                    CoercibleInto::<Self::Common>::coerce_into(self.0),
                    CoercibleInto::<Self::Common>::coerce_into(self.1),
                )
            }
        }

        // register unification into LHS
        impl $trait for ($rty, $lty)
        where
            $lty: CoercibleInto<$target>,
            $rty: CoercibleInto<$target>,
        {
            type Common = $target;
            fn into_common(self) -> ($target, $target) {
                (
                    CoercibleInto::<$target>::coerce_into(self.0),
                    CoercibleInto::<$target>::coerce_into(self.1),
                )
            }
        }

        // register unification into RHS
        impl $trait for (OptionNA<$rty>, OptionNA<$lty>)
        where
            OptionNA<$lty>: CoercibleInto<OptionNA<$target>>,
            OptionNA<$rty>: CoercibleInto<OptionNA<$target>>,
        {
            type Common = OptionNA<$target>;
            fn into_common(self) -> (Self::Common, Self::Common) {
                (
                    CoercibleInto::<Self::Common>::coerce_into(self.0),
                    CoercibleInto::<Self::Common>::coerce_into(self.1),
                )
            }
        }
    };

    (
      $trait:ident, $lty:ty => $target:ty
    ) => {
        // register unification into RHS
        impl $trait for ($lty, $lty)
        where
            $lty: CoercibleInto<$target>,
        {
            type Common = $target;
            fn into_common(self) -> ($target, $target) {
                (
                    CoercibleInto::<$target>::coerce_into(self.0),
                    CoercibleInto::<$target>::coerce_into(self.1),
                )
            }
        }

        // register unification into RHS
        impl $trait for (OptionNA<$lty>, OptionNA<$lty>)
        where
            OptionNA<$lty>: CoercibleInto<OptionNA<$target>>,
            OptionNA<$lty>: CoercibleInto<OptionNA<$target>>,
        {
            type Common = OptionNA<$target>;
            fn into_common(self) -> (Self::Common, Self::Common) {
                (
                    CoercibleInto::<Self::Common>::coerce_into(self.0),
                    CoercibleInto::<Self::Common>::coerce_into(self.1),
                )
            }
        }
    };
}

// register common numerics used for mathematical operators
register!(CommonNum, bool => i32);
register!(CommonNum, i32 => i32);
register!(CommonNum, f64 => f64);
register!(CommonNum, (bool, i32) => i32);
register!(CommonNum, (bool, f64) => f64);
register!(CommonNum, (i32 , f64) => f64);

register!(CommonCmp, bool => bool);
register!(CommonCmp, i32 => i32);
register!(CommonCmp, f64 => f64);
register!(CommonCmp, String => String);
register!(CommonCmp, (bool, i32) => i32);
register!(CommonCmp, (bool, f64) => f64);
register!(CommonCmp, (i32, f64) => f64);
register!(CommonCmp, (String, bool) => String);
register!(CommonCmp, (String, i32) => String);
register!(CommonCmp, (String, f64) => String);
