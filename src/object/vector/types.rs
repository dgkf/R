use super::coercion::AtomicMode;
use super::OptionNA;
use crate::error::Error;
use crate::object::Obj;
use crate::object::Vector;

pub type Double = OptionNA<f64>;
impl AtomicMode for Double {
    fn is_double() -> bool {
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

impl From<Option<i32>> for OptionNA<i32> {
    fn from(value: Option<i32>) -> Self {
        match value {
            None => Self::NA,
            Some(x) => Self::Some(x),
        }
    }
}

impl From<Option<bool>> for OptionNA<bool> {
    fn from(value: Option<bool>) -> Self {
        match value {
            None => Self::NA,
            Some(x) => Self::Some(x),
        }
    }
}

impl From<Option<String>> for OptionNA<String> {
    fn from(value: Option<String>) -> Self {
        match value {
            None => Self::NA,
            Some(x) => Self::Some(x),
        }
    }
}

impl From<Option<f64>> for OptionNA<f64> {
    fn from(value: Option<f64>) -> Self {
        match value {
            None => Self::NA,
            Some(x) => Self::Some(x),
        }
    }
}

impl<T> OptionNA<T> {
    pub fn is_na(&self) -> bool {
        matches!(self, OptionNA::NA)
    }
}

impl TryFrom<Obj> for Double {
    type Error = Error;
    fn try_from(value: Obj) -> Result<Self, Self::Error> {
        let err = Err(Error::Other(
            "Cannot convert object to scalar double.".to_string(),
        ));
        if let Obj::Vector(Vector::Double(v)) = value {
            if v.len() == 1 {
                Ok(v.iter_values().next().expect("length is one"))
            } else {
                err
            }
        } else {
            err
        }
    }
}

impl TryFrom<Obj> for Integer {
    type Error = Error;
    fn try_from(value: Obj) -> Result<Self, Self::Error> {
        let err = Err(Error::Other(
            "Cannot convert object to scalar integer.".to_string(),
        ));
        if let Obj::Vector(Vector::Integer(v)) = value {
            if v.len() == 1 {
                Ok(v.iter_values().next().expect("length is one"))
            } else {
                err
            }
        } else {
            err
        }
    }
}

impl TryFrom<Obj> for Character {
    type Error = Error;
    fn try_from(value: Obj) -> Result<Self, Self::Error> {
        let err = Err(Error::Other(
            "Cannot convert object to scalar character.".to_string(),
        ));
        if let Obj::Vector(Vector::Character(v)) = value {
            if v.len() == 1 {
                Ok(v.iter_values().next().expect("length is one"))
            } else {
                err
            }
        } else {
            err
        }
    }
}

impl TryFrom<Obj> for Logical {
    type Error = Error;
    fn try_from(value: Obj) -> Result<Self, Self::Error> {
        let err = Err(Error::Other(
            "Cannot convert object to scalar logical.".to_string(),
        ));
        if let Obj::Vector(Vector::Logical(v)) = value {
            if v.len() == 1 {
                Ok(v.iter_values().next().expect("length is one"))
            } else {
                err
            }
        } else {
            err
        }
    }
}

// TODO: Because From<T: Into<Vector>> for Obj is implement this means that
// when converting Character -> Obj we get a character vector.
// Change this once we have scalars.
impl From<Character> for Vector {
    fn from(value: Character) -> Self {
        Vector::Character(vec![value].into())
    }
}

impl From<Logical> for Vector {
    fn from(value: Logical) -> Self {
        Vector::Logical(vec![value].into())
    }
}

impl From<Double> for Vector {
    fn from(value: Double) -> Self {
        Vector::Double(vec![value].into())
    }
}

impl From<Integer> for Vector {
    fn from(value: Integer) -> Self {
        Vector::Integer(vec![value].into())
    }
}
