use crate::vector::types::{OptionNa, NaAble};

/// CoerceInto
///
/// A coercion trait often equivalent to [Into]. This allows for slight 
/// deviations from the way basic rust internals are implemented.
///
pub trait CoerceInto<T> {
    fn coerce(self) -> T;
}

impl<T, U> CoerceInto<OptionNa<U>> for OptionNa<T> 
where
    T: CoerceInto<U>,
    U: Default,
{
    #[inline]
    fn coerce(self) -> OptionNa<U> {
        let OptionNa(x) = self;
        match x {
            Some(s) => OptionNa(Some(s.coerce())),
            None => OptionNa(None),
        }
    }
}

// bool
impl CoerceInto<bool> for bool { fn coerce(self) -> bool { self } }
impl CoerceInto<i8> for bool { fn coerce(self) -> i8 { self as i8 } }
impl CoerceInto<i16> for bool { fn coerce(self) -> i16 { self as i16 } }
impl CoerceInto<i32> for bool { fn coerce(self) -> i32 { self as i32 } }
impl CoerceInto<i64> for bool { fn coerce(self) -> i64 { self as i64 } }
impl CoerceInto<i128> for bool { fn coerce(self) -> i128 { self as i128 } }
impl CoerceInto<f32> for bool { fn coerce(self) -> f32 { self as i32 as f32 } }
impl CoerceInto<f64> for bool { fn coerce(self) -> f64 { self as i32 as f64 } }
impl CoerceInto<String> for bool { fn coerce(self) -> String { self.to_string() } }

impl<T> CoerceInto<OptionNa<T>> for bool 
where
    Self: CoerceInto<T> 
{
    fn coerce(self) -> OptionNa<T> {
        OptionNa(Some(self.coerce()))
    }
}

// i8
impl CoerceInto<bool> for i8 { fn coerce(self) -> bool { self != 0 } }
impl CoerceInto<i8> for i8 { fn coerce(self) -> i8 { self } }
impl CoerceInto<i16> for i8 { fn coerce(self) -> i16 { self as i16 } }
impl CoerceInto<i32> for i8 { fn coerce(self) -> i32 { self as i32 } }
impl CoerceInto<i64> for i8 { fn coerce(self) -> i64 { self as i64 } }
impl CoerceInto<i128> for i8 { fn coerce(self) -> i128 { self as i128 } }
impl CoerceInto<f32> for i8 { fn coerce(self) -> f32 { self as f32 } }
impl CoerceInto<f64> for i8 { fn coerce(self) -> f64 { self as f64 } }
impl CoerceInto<String> for i8 { fn coerce(self) -> String { self.to_string() } }

impl<T> CoerceInto<OptionNa<T>> for i8
where
    Self: CoerceInto<T> 
{
    fn coerce(self) -> OptionNa<T> {
        OptionNa(Some(self.coerce()))
    }
}

// i16
impl CoerceInto<bool> for i16 { fn coerce(self) -> bool { self != 0 } }
impl CoerceInto<i8> for i16 { fn coerce(self) -> i8 { self as i8 } }
impl CoerceInto<i16> for i16 { fn coerce(self) -> i16 { self } }
impl CoerceInto<i32> for i16 { fn coerce(self) -> i32 { self as i32 } }
impl CoerceInto<i64> for i16 { fn coerce(self) -> i64 { self as i64 } }
impl CoerceInto<i128> for i16 { fn coerce(self) -> i128 { self as i128 } }
impl CoerceInto<f32> for i16 { fn coerce(self) -> f32 { self as f32 } }
impl CoerceInto<f64> for i16 { fn coerce(self) -> f64 { self as f64 } }
impl CoerceInto<String> for i16 { fn coerce(self) -> String { self.to_string() } }

impl<T> CoerceInto<OptionNa<T>> for i16
where
    Self: CoerceInto<T> 
{
    fn coerce(self) -> OptionNa<T> {
        OptionNa(Some(self.coerce()))
    }
}

// i32
impl CoerceInto<bool> for i32 { fn coerce(self) -> bool { self != 0 } }
impl CoerceInto<i8> for i32 { fn coerce(self) -> i8 { self as i8 } }
impl CoerceInto<i16> for i32 { fn coerce(self) -> i16 { self as i16 } }
impl CoerceInto<i32> for i32 { fn coerce(self) -> i32 { self } }
impl CoerceInto<i64> for i32 { fn coerce(self) -> i64 { self as i64 } }
impl CoerceInto<i128> for i32 { fn coerce(self) -> i128 { self as i128 } }
impl CoerceInto<f32> for i32 { fn coerce(self) -> f32 { self as f32 } }
impl CoerceInto<f64> for i32 { fn coerce(self) -> f64 { self as f64 } }
impl CoerceInto<String> for i32 { fn coerce(self) -> String { self.to_string() } }

impl<T> CoerceInto<OptionNa<T>> for i32
where
    Self: CoerceInto<T> 
{
    fn coerce(self) -> OptionNa<T> {
        OptionNa(Some(self.coerce()))
    }
}

// i64
impl CoerceInto<bool> for i64 { fn coerce(self) -> bool { self != 0 } }
impl CoerceInto<i8> for i64 { fn coerce(self) -> i8 { self as i8 } }
impl CoerceInto<i16> for i64 { fn coerce(self) -> i16 { self as i16 } }
impl CoerceInto<i32> for i64 { fn coerce(self) -> i32 { self as i32 } }
impl CoerceInto<i64> for i64 { fn coerce(self) -> i64 { self } }
impl CoerceInto<i128> for i64 { fn coerce(self) -> i128 { self as i128 } }
impl CoerceInto<f32> for i64 { fn coerce(self) -> f32 { self as f32 } }
impl CoerceInto<f64> for i64 { fn coerce(self) -> f64 { self as f64 } }
impl CoerceInto<String> for i64 { fn coerce(self) -> String { self.to_string() } }

impl<T> CoerceInto<OptionNa<T>> for i64
where
    Self: CoerceInto<T> 
{
    fn coerce(self) -> OptionNa<T> {
        OptionNa(Some(self.coerce()))
    }
}

// i128
impl CoerceInto<bool> for i128 { fn coerce(self) -> bool { self != 0 } }
impl CoerceInto<i8> for i128 { fn coerce(self) -> i8 { self as i8 } }
impl CoerceInto<i16> for i128 { fn coerce(self) -> i16 { self as i16 } }
impl CoerceInto<i32> for i128 { fn coerce(self) -> i32 { self as i32 } }
impl CoerceInto<i64> for i128 { fn coerce(self) -> i64 { self as i64 } }
impl CoerceInto<i128> for i128 { fn coerce(self) -> i128 { self } }
impl CoerceInto<f32> for i128 { fn coerce(self) -> f32 { self as f32 } }
impl CoerceInto<f64> for i128 { fn coerce(self) -> f64 { self as f64 } }
impl CoerceInto<String> for i128 { fn coerce(self) -> String { self.to_string() } }

impl<T> CoerceInto<OptionNa<T>> for i128
where
    Self: CoerceInto<T> 
{
    fn coerce(self) -> OptionNa<T> {
        OptionNa(Some(self.coerce()))
    }
}

// f32
impl CoerceInto<bool> for f32 {
    fn coerce(self) -> bool {
        match self.partial_cmp(&0.0) {
            Some(std::cmp::Ordering::Equal) => false,
            _ => true,
        }
    }
}

impl CoerceInto<i8> for f32 { fn coerce(self) -> i8 { self as i8 } }
impl CoerceInto<i16> for f32 { fn coerce(self) -> i16 { self as i16 } }
impl CoerceInto<i32> for f32 { fn coerce(self) -> i32 { self as i32 } }
impl CoerceInto<i64> for f32 { fn coerce(self) -> i64 { self as i64 } }
impl CoerceInto<i128> for f32 { fn coerce(self) -> i128 { self as i128 } }
impl CoerceInto<f32> for f32 { fn coerce(self) -> f32 { self } }
impl CoerceInto<f64> for f32 { fn coerce(self) -> f64 { self as f64 } }
impl CoerceInto<String> for f32 { fn coerce(self) -> String { self.to_string() } }

impl<T> CoerceInto<OptionNa<T>> for f32
where
    Self: CoerceInto<T>,
    f32: NaAble,
{
    fn coerce(self) -> OptionNa<T> {
        if self.is_na() { OptionNa(None) } else { OptionNa(Some(self.coerce())) }
    }
}

impl<T> CoerceInto<f32> for OptionNa<T> 
where
    T: CoerceInto<f32>,
    f32: NaAble,
{
    #[inline]
    fn coerce(self) -> f32 {
        let OptionNa(x) = self;
        match x {
            Some(x) => x.coerce(),
            None => self.na(),
        }

    }
}


// f64
impl CoerceInto<bool> for f64 {
    fn coerce(self) -> bool {
        match self.partial_cmp(&0.0) {
            Some(std::cmp::Ordering::Equal) => false,
            _ => true,
        }
    }
}

impl CoerceInto<i8> for f64 { fn coerce(self) -> i8 { self as i8 } }
impl CoerceInto<i16> for f64 { fn coerce(self) -> i16 { self as i16 } }
impl CoerceInto<i32> for f64 { fn coerce(self) -> i32 { self as i32 } }
impl CoerceInto<i64> for f64 { fn coerce(self) -> i64 { self as i64 } }
impl CoerceInto<i128> for f64 { fn coerce(self) -> i128 { self as i128 } }
impl CoerceInto<f32> for f64 { fn coerce(self) -> f32 { self as f32 } }
impl CoerceInto<f64> for f64 { fn coerce(self) -> f64 { self as f64 } }
impl CoerceInto<String> for f64 { fn coerce(self) -> String { self.to_string() } }

impl<T> CoerceInto<OptionNa<T>> for f64
where
    Self: CoerceInto<T>,
    f64: NaAble,
{
    fn coerce(self) -> OptionNa<T> {
        if self.is_na() { OptionNa(None) } else { OptionNa(Some(self.coerce())) }
    }
}

impl<T> CoerceInto<f64> for OptionNa<T> 
where
    T: CoerceInto<f64>,
    f64: NaAble,
{
    #[inline]
    fn coerce(self) -> f64 {
        let OptionNa(x) = self;
        match x {
            Some(x) => x.coerce(),
            None => std::f64::NAN,
        }

    }
}

// String
impl CoerceInto<OptionNa<bool>> for String { 
    fn coerce(self) -> OptionNa<bool> { 
        self.to_lowercase()
            .parse::<bool>()
            .map_or(OptionNa::default(), |i| OptionNa(Some(i)))
    } 
}

impl CoerceInto<OptionNa<i8>> for String {
    fn coerce(self) -> OptionNa<i8> {
        self.to_lowercase()
            .parse::<i8>()
            .map_or(OptionNa::default(), |i| OptionNa(Some(i)))
    } 
}

impl CoerceInto<OptionNa<i16>> for String {
    fn coerce(self) -> OptionNa<i16> {
        self.to_lowercase()
            .parse::<i16>()
            .map_or(OptionNa::default(), |i| OptionNa(Some(i)))
    } 
}

impl CoerceInto<OptionNa<i32>> for String {
    fn coerce(self) -> OptionNa<i32> {
        self.to_lowercase()
            .parse::<i32>()
            .map_or(OptionNa::default(), |i| OptionNa(Some(i)))
    } 
}

impl CoerceInto<OptionNa<i64>> for String {
    fn coerce(self) -> OptionNa<i64> {
        self.to_lowercase()
            .parse::<i64>()
            .map_or(OptionNa::default(), |i| OptionNa(Some(i)))
    } 
}

impl CoerceInto<OptionNa<i128>> for String {
    fn coerce(self) -> OptionNa<i128> {
        self.to_lowercase()
            .parse::<i128>()
            .map_or(OptionNa::default(), |i| OptionNa(Some(i)))
    } 
}

impl CoerceInto<f32> for String {
    fn coerce(self) -> f32 {
        self.parse::<f32>().unwrap_or(std::f32::NAN)
    } 
}

impl CoerceInto<f64> for String {
    fn coerce(self) -> f64 {
        self.parse::<f64>().unwrap_or(std::f64::NAN)
    } 
}

impl CoerceInto<String> for String {
    fn coerce(self) -> String {
        self
    } 
}

impl<T> CoerceInto<OptionNa<T>> for String 
where
    Self: CoerceInto<T> 
{
    fn coerce(self) -> OptionNa<T> {
        OptionNa(Some(self.coerce()))
    }
}
