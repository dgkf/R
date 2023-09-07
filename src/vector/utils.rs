use crate::vector::types::OptionNa;

pub fn some<T>(x: T) -> OptionNa<T> {
    OptionNa(Some(x))    
}

pub fn na<T>() -> OptionNa<T> {
    OptionNa(None)    
}

pub trait SameType<T>: Sized {
    fn is_same_type_as(&self, _other: &T) -> bool {
        false
    }
}

impl<T, U> SameType<T> for U {
    fn is_same_type_as(&self, _other: &T) -> bool {
        true
    }
}
