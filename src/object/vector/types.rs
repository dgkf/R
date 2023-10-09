use super::coercion::AtomicMode;
use super::OptionNA;

pub type Numeric = OptionNA<f64>;
impl AtomicMode for Numeric {
    fn is_numeric() -> bool {
        true
    }
}

pub type Integer = OptionNA<i32>;
impl AtomicMode for Integer {
    fn is_integer() -> bool {
        true
    }
}

pub type Logical = OptionNA<bool>;
impl AtomicMode for Logical {
    fn is_logical() -> bool {
        true
    }
}

pub type Character = OptionNA<String>;
impl AtomicMode for Character {
    fn is_character() -> bool {
        true
    }
}
