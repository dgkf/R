/// Coerce Two Values to Commonly Operable Data
///
/// Provided a tuple of (lhs, rhs), coerce both into data that can be added
/// together, presumably into the "lowest common denominator" of data
///
pub trait CommonLog {
    type Common;
    fn as_common(self) -> (Self::Common, Self::Common);
}

impl<T> CommonLog for (T, T) {
    type Common = T;
    fn as_common(self) -> (Self::Common, Self::Common) {
        self
    }
}

use crate::register;
register!(CommonLog: (bool, i8) => i8);
register!(CommonLog: (bool, i16) => i16);
register!(CommonLog: (bool, i32) => i32);
register!(CommonLog: (bool, i64) => i64);
register!(CommonLog: (bool, i128) => i128);
register!(CommonLog: (bool, f32) => f32);
register!(CommonLog: (bool, f64) => f64);
register!(CommonLog: (i8, f32) => f32);
register!(CommonLog: (i16, f32) => f32);
register!(CommonLog: (i32, f32) => f32);
register!(CommonLog: (i8, f64) => f64);
register!(CommonLog: (i16, f64) => f64);
register!(CommonLog: (i32, f64) => f64);
register!(CommonLog: (i64, f64) => f64);
register!(CommonLog: (i128, f64) => f64);
register!(CommonLog: (String, bool) => String);
register!(CommonLog: (String, i8) => String);
register!(CommonLog: (String, i16) => String);
register!(CommonLog: (String, i32) => String);
register!(CommonLog: (String, i64) => String);
register!(CommonLog: (String, i128) => String);
register!(CommonLog: (String, f32) => String);
register!(CommonLog: (String, f64) => String);