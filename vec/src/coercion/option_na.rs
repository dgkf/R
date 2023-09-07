use std::fmt::Display;

use crate::atomic::{Atomic, AtomicMode, IntoAtomic};
use super::{NaAble, AsMinimallyNumeric, CoerceInto};


/// OptionNa Types
///
/// OptionNa is used as a bespoke Option type that represents whether a value
/// is NA or not. Some types (f32, f64) have NaN values that can be used
/// in place of an NA, while others need to be wrapped in an OptionNa to 
/// provide this behavior.
///
#[derive(Debug, Clone, PartialEq)]
pub struct OptionNa<T>(pub Option<T>);

impl<T> OptionNa<T> {
    #[inline]
    pub fn inner(self) -> T 
    where
        T: Default,
    {
        let OptionNa(x) = self;
        match x {
            Some(s) => s,
            None => T::default(),
        }       
    }

    #[inline]
    pub fn map<U, F>(self, f: F) -> OptionNa<U>
    where
        F: FnOnce(T) -> U,
    {
        let OptionNa(inner) = self;
        OptionNa(inner.map(f))
    }

    #[inline]
    pub fn map2<U, F>(self, other: Self, f: F) -> OptionNa<U>
    where
        F: FnOnce(T, T) -> U,
    {
        let OptionNa(a) = self;
        let OptionNa(b) = other;
        if let (Some(a), Some(b)) = (a, b) {
            OptionNa(Some(f(a, b)))
        } else {
            OptionNa(None)
        }
    }
}

impl<T> Default for OptionNa<T> {
    fn default() -> Self {
        OptionNa(None)
    }
}

impl<T> Display for OptionNa<T>
where
    T: Display
{
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let OptionNa(x) = self;
        match x {
            Some(x) => write!(f, "{}", x),
            None => write!(f, "NA"),
        }
    }
}

impl<T, N, A> Atomic for OptionNa<T> 
where
    // As long as T is atomic (except for being Na-able), then we can wrap that
    // type in OptionNa to provide Na-ability.
    T: Clone + 
        AsMinimallyNumeric + 
        AtomicMode + 
        Default,
    
    // The remainder allow for minimizing NA-ifying overhead
    //
    // Given the example of OptionNa<bool> + f_32, we want an f_32 which 
    // is already NaAble using std::f32::NAN. We only want to wrap the result
    // back in a new NaAble wrapper if absolutely necessary.
    //
    T: AsMinimallyNumeric<As = N>,
    N: IntoAtomic<Atom = A>,
    OptionNa<T>: CoerceInto<A>,
{
}

impl<T> AtomicMode for OptionNa<T>
where
    T: AtomicMode
{
    fn is_numeric() -> bool { T::is_numeric() }
    fn is_logical() -> bool { T::is_logical() }
    fn is_integer() -> bool { T::is_integer() }
    fn is_character() -> bool { T::is_character() }
}

impl<T> NaAble for OptionNa<T> 
where
    T: Default
{
    type From = T;

    #[inline]
    fn na() -> Self {
        OptionNa(None)
    }

    #[inline]
    fn is_na(&self) -> bool {
        let OptionNa(x) = self;
        x.is_none()
    }

    #[inline]
    fn inner(self) -> Self::From {
        let OptionNa(x) = self;
        match x {
            Some(i) => i,
            None => T::default(),
        }
    }
}

impl<T, N, A> AsMinimallyNumeric for OptionNa<T> 
where
    T: AsMinimallyNumeric<As = N>,
    N: IntoAtomic<Atom = A>,
    OptionNa<T>: CoerceInto<A>,
{
    type As = A;
}

impl<T> std::ops::Neg for OptionNa<T>
where
    T: std::ops::Neg<Output = T>,
{
    type Output = OptionNa<T>;
    fn neg(self) -> Self::Output {
        let OptionNa(x) = self;
        match x {
            Some(x) => OptionNa(Some(x.neg())),
            _ => OptionNa(None),
        }
    }
}

impl<T> std::ops::Add for OptionNa<T>
where
    T: std::ops::Add<Output = T> + Default,
{
    type Output = OptionNa<T>;
    fn add(self, rhs: Self) -> Self::Output {
        self.map2(rhs, |l, r| l + r)
    }
}

impl<T> std::ops::Sub for OptionNa<T>
where
    T: std::ops::Sub<Output = T>,
{
    type Output = OptionNa<T>;
    fn sub(self, rhs: Self) -> Self::Output {
        self.map2(rhs, |l, r| l - r)
    }
}

impl<T> std::ops::Mul for OptionNa<T>
where
    T: std::ops::Mul<Output = T>,
{
    type Output = OptionNa<T>;
    fn mul(self, rhs: Self) -> Self::Output {
        self.map2(rhs, |l, r| l * r)
    }
}

impl<T> std::ops::Div for OptionNa<T>
where
    T: std::ops::Div<Output = T>,
{
    type Output = OptionNa<T>;
    fn div(self, rhs: Self) -> Self::Output {
        self.map2(rhs, |l, r| l / r)
    }
}

pub trait Pow {
    type Output;
    /// raise self to the rhs power
    fn power(self, rhs: Self) -> Self::Output;
}


impl<T> Pow for OptionNa<T>
where
    T: Pow,
{
    type Output = OptionNa<<T as Pow>::Output>;
    fn power(self, rhs: Self) -> Self::Output {
        let OptionNa(lhs) = self;
        let OptionNa(rhs) = rhs;
        match (lhs, rhs) {
            (Some(l), Some(r)) => OptionNa(Some(T::power(l, r))),
            _ => OptionNa(None),
        }
    }
}

impl<T> std::ops::Rem for OptionNa<T>
where
    T: std::ops::Rem,
{
    type Output = OptionNa<<T as std::ops::Rem>::Output>;
    fn rem(self, rhs: Self) -> Self::Output {
        let OptionNa(lhs) = self;
        let OptionNa(rhs) = rhs;
        match (lhs, rhs) {
            (Some(l), Some(r)) => OptionNa(Some(l.rem(r))),
            _ => OptionNa(None),
        }
    }
}

impl<T> std::ops::BitOr for OptionNa<T>
where
    T: std::ops::BitOr,
{
    type Output = OptionNa<<T as std::ops::BitOr>::Output>;
    fn bitor(self, rhs: Self) -> Self::Output {
        let OptionNa(lhs) = self;
        let OptionNa(rhs) = rhs;
        match (lhs, rhs) {
            (Some(l), Some(r)) => OptionNa(Some(l.bitor(r))),
            _ => OptionNa(None),
        }
    }
}

impl<T> std::ops::BitAnd for OptionNa<T>
where
    T: std::ops::BitAnd,
{
    type Output = OptionNa<<T as std::ops::BitAnd>::Output>;
    fn bitand(self, rhs: Self) -> Self::Output {
        let OptionNa(lhs) = self;
        let OptionNa(rhs) = rhs;
        match (lhs, rhs) {
            (Some(l), Some(r)) => OptionNa(Some(l.bitand(r))),
            _ => OptionNa(None),
        }
    }
}