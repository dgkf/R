use crate::register;

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

register!(CommonNum: (bool, i8) => i8);
register!(CommonNum: (bool, i16) => i16);
register!(CommonNum: (bool, i32) => i32);
register!(CommonNum: (bool, i64) => i64);
register!(CommonNum: (bool, i128) => i128);
register!(CommonNum: (bool, f32) => f32);
register!(CommonNum: (bool, f64) => f64);

register!(CommonNum: (i8, i16) => i16);
register!(CommonNum: (i8, i32) => i32);
register!(CommonNum: (i8, i64) => i64);
register!(CommonNum: (i8, i128) => i128);
register!(CommonNum: (i8, f32) => f32);
register!(CommonNum: (i8, f64) => f64);

register!(CommonNum: (i16, i32) => i32);
register!(CommonNum: (i16, i64) => i64);
register!(CommonNum: (i16, i128) => i128);
register!(CommonNum: (i16, f32) => f32);
register!(CommonNum: (i16, f64) => f64);

register!(CommonNum: (i32, i64) => i64);
register!(CommonNum: (i32, i128) => i128);
register!(CommonNum: (i32, f32) => f32);
register!(CommonNum: (i32, f64) => f64);

register!(CommonNum: (i64, i128) => i128);
register!(CommonNum: (i64, f32) => f32);
register!(CommonNum: (i64, f64) => f64);

register!(CommonNum: (i128, f32) => f32);
register!(CommonNum: (i128, f64) => f64);

register!(CommonNum: (f32, f64) => f64);
