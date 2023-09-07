use super::{CoerceInto, OptionNa};

pub trait AsMinimallyNumeric: CoerceInto<Self::As> + Sized {
    type As;
    #[inline]
    fn as_minimally_numeric(self) -> Self::As {
        self.coerce()
    }
}

pub trait AsLogical: CoerceInto<Self::As> + Sized {
    type As;
    #[inline]
    fn as_logical(self) -> Self::As {
        self.coerce()
    }
}

pub trait AsNumeric: CoerceInto<Self::As> + Sized {
    type As;
    #[inline]
    fn as_numeric(self) -> Self::As {
        self.coerce()
    }
}

pub trait AsInteger: CoerceInto<Self::As> + Sized {
    type As;
    #[inline]
    fn as_integer(self) -> Self::As {
        self.coerce()
    }
}

pub trait Character: CoerceInto<Self::As> + Sized {
    type As;
    #[inline]
    fn as_character(self) -> Self::As {
        self.coerce()
    }
}

impl<T> AsLogical for T 
where
    T: CoerceInto<OptionNa<bool>>
{
    type As = OptionNa<bool>;
}

impl<T> AsInteger for T
where
    T: CoerceInto<OptionNa<i32>>
{
    type As = OptionNa<i32>;
}

impl<T> AsNumeric for T
where
    T: CoerceInto<f64>
{
    type As = f64;
}

impl<T> Character for T 
where
    T: CoerceInto<OptionNa<String>>
{
    type As = OptionNa<String>;
}

