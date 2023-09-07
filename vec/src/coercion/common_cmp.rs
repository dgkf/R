/// Coerce Two Values to Commonly Operable Data
///
/// Provided a tuple of (lhs, rhs), coerce both into data that can be added
/// together, presumably into the "lowest common denominator" of data
///
pub trait CommonCmp {
    type Common;
    fn as_common(self) -> (Self::Common, Self::Common);
}

impl<T> CommonCmp for (T, T) {
    type Common = T;
    fn as_common(self) -> (Self::Common, Self::Common) {
        self
    }
}

use crate::register;
register!(CommonCmp: (bool, i8) => i8);
register!(CommonCmp: (bool, i16) => i16);
register!(CommonCmp: (bool, i32) => i32);
register!(CommonCmp: (bool, i64) => i64);
register!(CommonCmp: (bool, i128) => i128);
register!(CommonCmp: (bool, f32) => f32);
register!(CommonCmp: (bool, f64) => f64);
register!(CommonCmp: (i8, f32) => f32);
register!(CommonCmp: (i16, f32) => f32);
register!(CommonCmp: (i32, f32) => f32);
register!(CommonCmp: (i8, f64) => f64);
register!(CommonCmp: (i16, f64) => f64);
register!(CommonCmp: (i32, f64) => f64);
register!(CommonCmp: (i64, f64) => f64);
register!(CommonCmp: (i128, f64) => f64);
register!(CommonCmp: (String, bool) => String);
register!(CommonCmp: (String, i8) => String);
register!(CommonCmp: (String, i16) => String);
register!(CommonCmp: (String, i32) => String);
register!(CommonCmp: (String, i64) => String);
register!(CommonCmp: (String, i128) => String);
register!(CommonCmp: (String, f32) => String);
register!(CommonCmp: (String, f64) => String);