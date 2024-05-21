use super::coercion::Atomic;
use super::OptionNA;

pub type Double = OptionNA<f64>;
impl Atomic for Double {
    fn is_double() -> bool {
        true
    }
}

impl Atomic for f64 {
    fn is_double() -> bool {
        true
    }
}

pub type Integer = OptionNA<i32>;
impl Atomic for Integer {
    fn is_integer() -> bool {
        true
    }
}

impl Atomic for i32 {
    fn is_integer() -> bool {
        true
    }
}

pub type Logical = OptionNA<bool>;
impl Atomic for Logical {
    fn is_logical() -> bool {
        true
    }
}

impl Atomic for bool {
    fn is_logical() -> bool {
        true
    }
}

pub type Character = OptionNA<String>;
impl Atomic for Character {
    fn is_character() -> bool {
        true
    }
}
impl Atomic for String {
    fn is_character() -> bool {
        true
    }
}
