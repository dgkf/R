/// Coerce Two Values to Commonly Operable Data
///
/// Provided a tuple of (lhs, rhs), coerce both into data that can be added
/// together, presumably into the "lowest common denominator" of data
///
pub trait CommonNum {
    type Common;
    fn as_common(self) -> (Self::Common, Self::Common);
}

impl<T> CommonNum for (T, T) {
    type Common = T;
    fn as_common(self) -> (Self::Common, Self::Common) {
        self
    }
}

use crate::register_common_num;
register_common_num!((bool, i8) => i8);
register_common_num!((bool, i16) => i16);
register_common_num!((bool, i32) => i32);
register_common_num!((bool, i64) => i64);
register_common_num!((bool, i128) => i128);
register_common_num!((bool, f32) => f32);
register_common_num!((bool, f64) => f64);
register_common_num!((i32, f32) => f32);
register_common_num!((i32, f64) => f64);