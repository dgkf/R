use super::vectors::OptionNA;

pub trait CoercibleInto<T> {
    fn coerce_into(self) -> T;
}

impl<T> CoercibleInto<T> for T {
    fn coerce_into(self) -> T {
        self
    }
}

impl CoercibleInto<i32> for bool {
    fn coerce_into(self) -> i32 {
        self as i32
    }
}

impl CoercibleInto<i32> for f64 {
    fn coerce_into(self) -> i32 {
        self as i32
    }
}

impl CoercibleInto<f64> for bool {
    fn coerce_into(self) -> f64 {
        self as i32 as f64
    }
}

impl CoercibleInto<f64> for i32 {
    fn coerce_into(self) -> f64 {
        self as f64
    }
}

impl CoercibleInto<String> for bool {
    fn coerce_into(self) -> String {
        self.to_string()
    }
}

impl CoercibleInto<String> for i32 {
    fn coerce_into(self) -> String {
        self.to_string()
    }
}

impl CoercibleInto<String> for f64 {
    fn coerce_into(self) -> String {
        self.to_string()
    }
}

pub trait IntoNumeric<T> {
    fn as_numeric(self) -> T;
}

impl<T, U> IntoNumeric<OptionNA<T>> for OptionNA<U>
where
    U: IntoNumeric<T>,
{
    fn as_numeric(self) -> OptionNA<T> {
        use OptionNA::*;
        match self {
            Some(x) => Some(U::as_numeric(x)),
            _ => NA,
        }
    }
}

impl IntoNumeric<i32> for bool {
    fn as_numeric(self) -> i32 {
        self as i32
    }
}

impl IntoNumeric<i32> for i32 {
    fn as_numeric(self) -> i32 {
        self
    }
}

impl IntoNumeric<i64> for i64 {
    fn as_numeric(self) -> i64 {
        self
    }
}

impl IntoNumeric<i128> for i128 {
    fn as_numeric(self) -> i128 {
        self
    }
}

impl IntoNumeric<f32> for f32 {
    fn as_numeric(self) -> f32 {
        self
    }
}

impl IntoNumeric<f64> for f64 {
    fn as_numeric(self) -> f64 {
        self
    }
}

pub trait IntoLogical<T> {
    fn as_logical(self) -> T;
}

impl<T, U> IntoLogical<OptionNA<T>> for OptionNA<U>
where
    U: IntoLogical<T>,
{
    fn as_logical(self) -> OptionNA<T> {
        use OptionNA::*;
        match self {
            Some(x) => Some(U::as_logical(x)),
            _ => NA,
        }
    }
}

impl IntoLogical<bool> for bool {
    fn as_logical(self) -> bool {
        self
    }
}

impl IntoLogical<bool> for i32 {
    fn as_logical(self) -> bool {
        match self {
            0 => false,
            _ => true,
        }
    }
}

impl IntoLogical<bool> for i64 {
    fn as_logical(self) -> bool {
        match self {
            0 => false,
            _ => true,
        }
    }
}

impl IntoLogical<bool> for i128 {
    fn as_logical(self) -> bool {
        match self {
            0 => false,
            _ => true,
        }
    }
}

impl IntoLogical<bool> for f32 {
    fn as_logical(self) -> bool {
        match self.partial_cmp(&0.0) {
            Some(std::cmp::Ordering::Equal) => false,
            _ => true,
        }
    }
}

impl IntoLogical<bool> for f64 {
    fn as_logical(self) -> bool {
        match self.partial_cmp(&0.0) {
            Some(std::cmp::Ordering::Equal) => false,
            _ => true,
        }
    }
}

pub trait CommonNum<T>: Sized {
    fn as_common(self) -> (T, T);
}

impl<T, U, V> CommonNum<OptionNA<T>> for (OptionNA<U>, OptionNA<V>)
where
    (U, V): CommonNum<T>,
{
    fn as_common(self) -> (OptionNA<T>, OptionNA<T>) {
        use OptionNA::*;
        match self {
            (Some(l), Some(r)) => {
                let (l, r) = (l, r).as_common();
                (Some(l), Some(r))
            }
            _ => (NA, NA),
        }
    }
}

pub trait CommonMul<T>: Sized {
    fn as_common(self) -> (T, T);
}

pub trait CommonCmp<T>: Sized {
    fn as_common(self) -> (T, T);
}

impl<T, U, V> CommonCmp<OptionNA<T>> for (OptionNA<U>, OptionNA<V>)
where
    (U, V): CommonCmp<T>,
{
    fn as_common(self) -> (OptionNA<T>, OptionNA<T>) {
        use OptionNA::*;
        match self {
            (Some(l), Some(r)) => {
                let (l, r) = (l, r).as_common();
                (Some(l), Some(r))
            }
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
        impl $trait<$target> for ($lty, $rty)
        where
            $lty: CoercibleInto<$target>,
            $rty: CoercibleInto<$target>,
        {
            fn as_common(self) -> ($target, $target) {
                (
                    CoercibleInto::<$target>::coerce_into(self.0),
                    CoercibleInto::<$target>::coerce_into(self.1),
                )
            }
        }

        // register unification into LHS
        impl $trait<$target> for ($rty, $lty)
        where
            $lty: CoercibleInto<$target>,
            $rty: CoercibleInto<$target>,
        {
            fn as_common(self) -> ($target, $target) {
                (
                    CoercibleInto::<$target>::coerce_into(self.0),
                    CoercibleInto::<$target>::coerce_into(self.1),
                )
            }
        }
    };

    (
      $trait:ident, $lty:ty => $target:ty
    ) => {
        // register unification into RHS
        impl $trait<$target> for ($lty, $lty)
        where
            $lty: CoercibleInto<$target>,
        {
            fn as_common(self) -> ($target, $target) {
                (
                    CoercibleInto::<$target>::coerce_into(self.0),
                    CoercibleInto::<$target>::coerce_into(self.1),
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
