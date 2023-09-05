pub trait CoerceInto<T> {
    fn coerce(self) -> T;
}

impl<T> CoerceInto<T> for T {
    fn coerce(self) -> T {
        self
    }
}

impl CoerceInto<i8> for bool {
    fn coerce(self) -> i8 {
        self as i8
    }
}

impl CoerceInto<i16> for bool {
    fn coerce(self) -> i16 {
        self as i16
    }
}

impl CoerceInto<i32> for bool {
    fn coerce(self) -> i32 {
        self as i32
    }
}

impl CoerceInto<i64> for bool {
    fn coerce(self) -> i64 {
        self as i64
    }
}

impl CoerceInto<i128> for bool {
    fn coerce(self) -> i128 {
        self as i128
    }
}

impl CoerceInto<f32> for bool {
    fn coerce(self) -> f32 {
        self as i32 as f32
    }
}

impl CoerceInto<f64> for bool {
    fn coerce(self) -> f64 {
        self as i32 as f64
    }
}

impl CoerceInto<String> for bool {
    fn coerce(self) -> String {
        self.to_string()
    }
}

impl CoerceInto<f32> for i32 {
    fn coerce(self) -> f32 {
        self as f32
    }
}

impl CoerceInto<i32> for f64 {
    fn coerce(self) -> i32 {
        self as i32
    }
}

impl CoerceInto<f64> for i32 {
    fn coerce(self) -> f64 {
        self as f64
    }
}

impl CoerceInto<String> for i32 {
    fn coerce(self) -> String {
        self.to_string()
    }
}

impl CoerceInto<String> for f64 {
    fn coerce(self) -> String {
        self.to_string()
    }
}
