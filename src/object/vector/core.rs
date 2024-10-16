use std::fmt::Debug;
use std::fmt::Display;

use crate::error::Error;
use crate::lang::EvalResult;
use crate::lang::Signal;
use crate::object::CowObj;
use crate::object::Obj;

use super::coercion::CoercibleInto;
use super::rep::Rep;
use super::reptype::IterableValues;
use super::reptype::RepType;
use super::subset::Subset;
use super::types::*;

#[derive(Default, Clone, PartialEq, Eq)]
pub enum OptionNA<T> {
    #[default]
    NA,
    Some(T),
}

impl<T> PartialOrd for OptionNA<T>
where
    T: PartialOrd,
{
    fn partial_cmp(&self, other: &Self) -> Option<std::cmp::Ordering> {
        match (self, other) {
            (OptionNA::Some(l), OptionNA::Some(r)) => l.partial_cmp(r),
            _ => None,
        }
    }
}

impl<T> OptionNA<T> {
    pub fn map<F, U>(self, f: F) -> OptionNA<U>
    where
        F: FnOnce(T) -> U,
    {
        match self {
            OptionNA::Some(x) => OptionNA::Some(f(x)),
            OptionNA::NA => OptionNA::NA,
        }
    }

    pub fn as_option(self) -> Option<T> {
        match self {
            OptionNA::Some(x) => Option::Some(x),
            OptionNA::NA => Option::None,
        }
    }
}

#[derive(Debug, PartialEq)]
pub enum Vector {
    Double(RepType<Double>),
    Integer(RepType<Integer>),
    Logical(RepType<Logical>),
    Character(RepType<Character>),
    // Complex(Complex),
    // Raw(Raw),
}

// TODO: Implement vector more like Rep<T>
// I.e. the conversion from and to objects should be handled via TryFrom/From
// and .into() calls
impl Clone for Vector {
    fn clone(&self) -> Self {
        match self {
            Vector::Double(v) => Vector::Double(v.clone()),
            Vector::Character(v) => Vector::Character(v.clone()),
            Vector::Integer(v) => Vector::Integer(v.clone()),
            Vector::Logical(v) => Vector::Logical(v.clone()),
        }
    }
}

// TODO: Ensure that Vector API does not go beyond Rep<Obj> unless it is really
// necessary.

/// See [`Rep`] for the documentation on the methods.
impl Vector {
    pub fn get(&self, index: usize) -> Option<Vector> {
        use Vector::*;
        match self {
            Double(x) => x.get(index).map(Double),
            Integer(x) => x.get(index).map(Integer),
            Logical(x) => x.get(index).map(Logical),
            Character(x) => x.get(index).map(Character),
        }
    }

    pub fn set_subset(&mut self, subset: Subset, value: Obj) -> Result<Self, Signal> {
        use Vector::*;
        match self {
            Double(x) => x
                .set_subset(subset, value.try_into()?)
                .map(|x| Double(RepType::from(vec![x]))),
            Integer(x) => x
                .set_subset(subset, value.try_into()?)
                .map(|x| Integer(RepType::from(vec![x]))),
            Character(x) => x
                .set_subset(subset, value.try_into()?)
                .map(|x| Character(RepType::from(vec![x]))),
            Logical(x) => x
                .set_subset(subset, value.try_into()?)
                .map(|x| Logical(RepType::from(vec![x]))),
        }
    }

    pub fn iter_names(&self) -> Option<IterableValues<Character>> {
        use Vector::*;
        match self {
            Double(x) => x.iter_names(),
            Integer(x) => x.iter_names(),
            Logical(x) => x.iter_names(),
            Character(x) => x.iter_names(),
        }
    }

    pub fn is_named(&self) -> bool {
        use Vector::*;
        match self {
            Double(x) => x.is_named(),
            Integer(x) => x.is_named(),
            Logical(x) => x.is_named(),
            Character(x) => x.is_named(),
        }
    }

    pub fn names(&self) -> Option<CowObj<Vec<Character>>> {
        use Vector::*;
        match self {
            Double(x) => x.names(),
            Integer(x) => x.names(),
            Logical(x) => x.names(),
            Character(x) => x.names(),
        }
    }

    pub fn set_names(&self, names: CowObj<Vec<Character>>) -> Self {
        use super::Vector::*;
        match self {
            Character(x) => Character(x.set_names(names)),
            Logical(x) => Logical(x.set_names(names)),
            Integer(x) => Integer(x.set_names(names)),
            Double(x) => Double(x.set_names(names)),
        }
    }

    pub fn try_get(&self, index: Obj) -> EvalResult {
        let err =
            Error::Other("Vector index cannot be coerced into a valid indexing type.".to_string());
        match (self, index.as_vector()?) {
            (Vector::Double(v), Obj::Vector(i)) => {
                Ok(Obj::Vector(Vector::from(v.subset(i.try_into()?))))
            }
            (Vector::Integer(v), Obj::Vector(i)) => {
                Ok(Obj::Vector(Vector::from(v.subset(i.try_into()?))))
            }
            (Vector::Logical(v), Obj::Vector(i)) => {
                Ok(Obj::Vector(Vector::from(v.subset(i.try_into()?))))
            }
            (Vector::Character(v), Obj::Vector(i)) => {
                Ok(Obj::Vector(Vector::from(v.subset(i.try_into()?))))
            }
            _ => Err(err.into()),
        }
    }

    pub fn subset(&self, subset: Subset) -> Self {
        match self {
            Vector::Double(x) => x.subset(subset).into(),
            Vector::Integer(x) => x.subset(subset).into(),
            Vector::Logical(x) => x.subset(subset).into(),
            Vector::Character(x) => x.subset(subset).into(),
        }
    }

    pub fn assign(&mut self, other: Obj) -> EvalResult {
        let err =
            Error::Other("Cannot assign to a vector from a different type".to_string()).into();
        match (self, other) {
            (Vector::Double(l), Obj::Vector(Vector::Double(r))) => {
                Ok(Obj::Vector(Vector::from(l.assign(r)?)))
            }
            (Vector::Integer(l), Obj::Vector(Vector::Integer(r))) => {
                Ok(Obj::Vector(Vector::from(l.assign(r)?)))
            }
            (Vector::Logical(l), Obj::Vector(Vector::Logical(r))) => {
                Ok(Obj::Vector(Vector::from(l.assign(r)?)))
            }
            (Vector::Character(l), Obj::Vector(Vector::Character(r))) => {
                Ok(Obj::Vector(Vector::from(l.assign(r)?)))
            }
            _ => Err(err),
        }
    }

    pub fn materialize(self) -> Self {
        match self {
            Vector::Double(x) => Vector::from(x.materialize()),
            Vector::Integer(x) => Vector::from(x.materialize()),
            Vector::Logical(x) => Vector::from(x.materialize()),
            Vector::Character(x) => Vector::from(x.materialize()),
        }
    }

    pub fn vec_coerce<T, U>(v: &[OptionNA<T>]) -> Vec<OptionNA<U>>
    where
        T: CoercibleInto<U> + Clone,
    {
        use OptionNA::*;
        v.iter()
            .map(|i| match i {
                Some(x) => Some(CoercibleInto::<U>::coerce_into((*x).clone())),
                NA => NA,
            })
            .collect()
    }

    pub fn vec_parse<U>(v: &[OptionNA<String>]) -> (bool, Vec<OptionNA<U>>)
    where
        U: std::str::FromStr,
    {
        use OptionNA::*;
        let mut any_new_nas = false;
        let result = v
            .iter()
            .map(|i| match i {
                Some(s) => match s.parse::<U>() {
                    Ok(value) => Some(value),
                    Err(_) => {
                        any_new_nas = true;
                        NA
                    }
                },
                NA => NA,
            })
            .collect();

        (any_new_nas, result)
    }

    pub fn as_integer(self) -> Vector {
        use Vector::*;
        match self {
            Double(v) => Integer(v.as_integer()),
            Integer(_) => self,
            Logical(v) => Integer(v.as_integer()),
            Character(v) => Integer(v.as_integer()),
        }
    }

    pub fn as_double(self) -> Vector {
        use Vector::*;
        match self {
            Double(_) => self,
            Integer(v) => Double(v.as_double()),
            Logical(v) => Double(v.as_double()),
            Character(v) => Double(v.as_double()),
        }
    }

    pub fn as_logical(self) -> Vector {
        use Vector::*;
        match self {
            Double(v) => Logical(v.as_logical()),
            Integer(v) => Logical(v.as_logical()),
            Logical(_) => self,
            Character(v) => Logical(v.as_logical()),
        }
    }

    pub fn as_character(self) -> Vector {
        use Vector::*;
        match self {
            Double(v) => Character(v.as_character()),
            Integer(v) => Character(v.as_character()),
            Logical(v) => Character(v.as_character()),
            Character(_) => self,
        }
    }

    pub fn len(&self) -> usize {
        use Vector::*;
        match self {
            Double(v) => v.len(),
            Integer(v) => v.len(),
            Logical(v) => v.len(),
            Character(v) => v.len(),
        }
    }

    #[must_use]
    pub fn is_empty(&self) -> bool {
        self.len() == 0
    }
}

impl TryInto<bool> for Vector {
    type Error = ();
    fn try_into(self) -> Result<bool, Self::Error> {
        use Vector::*;
        match self {
            Double(i) => i.try_into(),
            Integer(i) => i.try_into(),
            Logical(i) => i.try_into(),
            Character(i) => i.try_into(),
        }
    }
}

impl From<Vector> for Result<Vector, Signal> {
    fn from(x: Vector) -> Self {
        Ok(x)
    }
}

impl From<CowObj<Vec<Character>>> for Vector {
    fn from(x: CowObj<Vec<Character>>) -> Self {
        Vector::Character(x.into())
    }
}

impl From<RepType<Double>> for Vector {
    fn from(x: RepType<Double>) -> Self {
        Vector::Double(x.into())
    }
}

impl From<RepType<Integer>> for Vector {
    fn from(x: RepType<Integer>) -> Self {
        Vector::Integer(x.into())
    }
}

impl From<RepType<Logical>> for Vector {
    fn from(x: RepType<Logical>) -> Self {
        Vector::Logical(x.into())
    }
}

impl From<RepType<Character>> for Vector {
    fn from(x: RepType<Character>) -> Self {
        Vector::Character(x.into())
    }
}

// impl From<RepType<Double>> for Vector {
//     fn from(x: Rep<Double>) -> Self {
//         Vector::Double(x)
//     }
// }

// impl From<Rep<Integer>> for Vector {
//     fn from(x: Rep<Integer>) -> Self {
//         Vector::Integer(x)
//     }
// }

// impl From<Rep<Logical>> for Vector {
//     fn from(x: Rep<Logical>) -> Self {
//         Vector::Logical(x)
//     }
// }

// impl From<Rep<Character>> for Vector {
//     fn from(x: Rep<Character>) -> Self {
//         Vector::Character(x)
//     }
// }

impl From<Vec<f64>> for Vector {
    fn from(x: Vec<f64>) -> Self {
        Vector::Double(x.into())
    }
}

impl From<Vec<OptionNA<f64>>> for Vector {
    fn from(x: Vec<OptionNA<f64>>) -> Self {
        Vector::Double(x.into())
    }
}

impl From<Vec<i32>> for Vector {
    fn from(x: Vec<i32>) -> Self {
        Vector::Integer(x.into())
    }
}

impl From<Vec<OptionNA<i32>>> for Vector {
    fn from(x: Vec<OptionNA<i32>>) -> Self {
        Vector::Integer(x.into())
    }
}

impl From<Vec<bool>> for Vector {
    fn from(x: Vec<bool>) -> Self {
        Vector::Logical(x.into())
    }
}

impl From<bool> for Vector {
    fn from(x: bool) -> Self {
        Vector::Logical(vec![x].into())
    }
}

impl From<Vec<OptionNA<bool>>> for Vector {
    fn from(x: Vec<OptionNA<bool>>) -> Self {
        Vector::Logical(x.into())
    }
}

impl From<Vec<String>> for Vector {
    fn from(x: Vec<String>) -> Self {
        Vector::Character(x.into())
    }
}

impl From<Vector> for String {
    fn from(val: Vector) -> Self {
        match val.as_character() {
            Vector::Character(v) => match v.inner().clone().borrow().first() {
                Some(OptionNA::Some(s)) => s.clone(),
                Some(OptionNA::NA) => "NA".to_string(),
                None => "".to_string(),
            },
            _ => unreachable!(),
        }
    }
}

impl From<Vector> for Vec<String> {
    fn from(val: Vector) -> Self {
        match val.as_character() {
            Vector::Character(v) => v
                .inner()
                .clone()
                .borrow()
                .iter()
                .map(|x| format!("{}", x))
                .collect(),
            _ => unreachable!(),
        }
    }
}

impl From<Vec<OptionNA<String>>> for Vector {
    fn from(x: Vec<OptionNA<String>>) -> Self {
        Vector::Character(x.into())
    }
}

pub trait DefaultDebug {}
impl DefaultDebug for bool {}
impl DefaultDebug for i32 {}
impl DefaultDebug for f64 {}

impl<T> Debug for OptionNA<T>
where
    T: DefaultDebug + Display,
{
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            OptionNA::Some(x) => write!(f, "{}", x),
            OptionNA::NA => write!(f, "NA"),
        }
    }
}

impl Debug for OptionNA<String> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            OptionNA::Some(x) => write!(f, "\"{}\"", x),
            OptionNA::NA => write!(f, "NA"),
        }
    }
}

impl<T> Display for OptionNA<T>
where
    T: Display,
{
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            OptionNA::Some(x) => write!(f, "{}", x),
            OptionNA::NA => write!(f, "NA"),
        }
    }
}

impl Display for Vector {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Vector::Double(x) => std::fmt::Display::fmt(&x, f),
            Vector::Integer(x) => std::fmt::Display::fmt(&x, f),
            Vector::Logical(x) => std::fmt::Display::fmt(&x, f),
            Vector::Character(x) => std::fmt::Display::fmt(&x, f),
        }
    }
}

impl<T, U, O> std::ops::Add<OptionNA<U>> for OptionNA<T>
where
    T: std::ops::Add<U, Output = O>,
{
    type Output = OptionNA<O>;
    fn add(self, rhs: OptionNA<U>) -> Self::Output {
        use OptionNA::*;
        match (self, rhs) {
            (Some(l), Some(r)) => Some(l + r),
            _ => NA,
        }
    }
}

impl<T> std::ops::Sub for OptionNA<T>
where
    T: std::ops::Sub<Output = T>,
{
    type Output = OptionNA<T>;
    fn sub(self, rhs: Self) -> Self::Output {
        use OptionNA::*;
        match (self, rhs) {
            (Some(l), Some(r)) => Some(l - r),
            _ => NA,
        }
    }
}

impl<T> std::ops::Neg for OptionNA<T>
where
    T: std::ops::Neg<Output = T>,
{
    type Output = OptionNA<T>;
    fn neg(self) -> Self::Output {
        use OptionNA::*;
        match self {
            Some(x) => Some(x.neg()),
            _ => NA,
        }
    }
}

impl<T> std::ops::Mul for OptionNA<T>
where
    T: std::ops::Mul<Output = T>,
{
    type Output = OptionNA<T>;
    fn mul(self, rhs: Self) -> Self::Output {
        use OptionNA::*;
        match (self, rhs) {
            (Some(l), Some(r)) => Some(l * r),
            _ => NA,
        }
    }
}

impl<T> std::ops::Div for OptionNA<T>
where
    T: std::ops::Div<Output = T>,
{
    type Output = OptionNA<T>;
    fn div(self, rhs: Self) -> Self::Output {
        use OptionNA::*;
        match (self, rhs) {
            (Some(l), Some(r)) => Some(l / r),
            _ => NA,
        }
    }
}

pub trait Pow<Rhs> {
    type Output;
    /// raise self to the rhs power
    fn power(self, rhs: Rhs) -> Self::Output;
}

impl Pow<i32> for i32 {
    type Output = i32;
    fn power(self, rhs: Self) -> Self::Output {
        i32::pow(self, rhs as u32)
    }
}

impl Pow<f64> for i32 {
    type Output = f64;
    fn power(self, rhs: f64) -> Self::Output {
        f64::powf(self as f64, rhs)
    }
}

impl<T> Pow<T> for f64
where
    f64: From<T>,
{
    type Output = f64;
    fn power(self, rhs: T) -> Self::Output {
        f64::powf(self, rhs.into())
    }
}

impl<T, U, O> Pow<OptionNA<U>> for OptionNA<T>
where
    T: Pow<U, Output = O>,
{
    type Output = OptionNA<O>;
    fn power(self, rhs: OptionNA<U>) -> Self::Output {
        use OptionNA::*;
        match (self, rhs) {
            (Some(l), Some(r)) => Some(T::power(l, r)),
            _ => NA,
        }
    }
}

impl<T> std::ops::Rem for OptionNA<T>
where
    T: std::ops::Rem,
{
    type Output = OptionNA<<T as std::ops::Rem>::Output>;
    fn rem(self, rhs: Self) -> Self::Output {
        use OptionNA::*;
        match (self, rhs) {
            (Some(l), Some(r)) => Some(l.rem(r)),
            _ => NA,
        }
    }
}

impl std::ops::BitOr<Logical> for Logical {
    type Output = Logical;
    fn bitor(self, rhs: Logical) -> Self::Output {
        use OptionNA::*;
        match (self, rhs) {
            (Some(l), Some(r)) => Some(l || r),
            _ => NA,
        }
    }
}

impl std::ops::BitAnd<Logical> for Logical {
    type Output = Logical;
    fn bitand(self, rhs: Logical) -> Self::Output {
        use OptionNA::*;
        match (self, rhs) {
            (Some(l), Some(r)) => Some(l && r),
            _ => NA,
        }
    }
}

impl std::ops::Not for Logical {
    type Output = Logical;
    fn not(self) -> Self::Output {
        self.map(|x| !x)
    }
}

impl std::ops::Neg for Vector {
    type Output = Result<Vector, Signal>;
    fn neg(self) -> Self::Output {
        use Vector::*;
        match self {
            Double(x) => x.neg().map(|x| x.into()),
            Integer(x) => x.neg().map(|x| x.into()),
            Logical(x) => x.neg().map(|x| x.into()),
            _ => todo!(),
        }
    }
}

impl std::ops::Not for Vector {
    type Output = Result<Vector, Signal>;
    fn not(self) -> Self::Output {
        use Vector::*;
        match self {
            Logical(x) => (!x).map(|x| x.into()),
            _ => todo!(),
        }
    }
}

impl std::ops::Add for Vector {
    type Output = Result<Self, Signal>;
    fn add(self, other: Self) -> Self::Output {
        use Vector::*;
        match (self, other) {
            (Double(l), Double(r)) => (l + r).map(|x| x.into()),
            (Double(l), Integer(r)) => (l + r).map(|x| x.into()),
            (Double(l), Logical(r)) => (l + r).map(|x| x.into()),
            (Integer(l), Double(r)) => (l + r).map(|x| x.into()),
            (Integer(l), Integer(r)) => (l + r).map(|x| x.into()),
            (Integer(l), Logical(r)) => (l + r).map(|x| x.into()),
            (Logical(l), Double(r)) => (l + r).map(|x| x.into()),
            (Logical(l), Integer(r)) => (l + r).map(|x| x.into()),
            (Logical(l), Logical(r)) => (l + r).map(|x| x.into()),
            // Add more combinations if necessary
            _ => todo!(),
        }
    }
}

impl std::ops::Sub for Vector {
    type Output = Result<Self, Signal>;
    fn sub(self, rhs: Self) -> Self::Output {
        use Vector::*;
        match (self, rhs) {
            (Double(l), Double(r)) => (l - r).map(|x| x.into()),
            (Double(l), Integer(r)) => (l - r).map(|x| x.into()),
            (Double(l), Logical(r)) => (l - r).map(|x| x.into()),
            (Integer(l), Double(r)) => (l - r).map(|x| x.into()),
            (Integer(l), Integer(r)) => (l - r).map(|x| x.into()),
            (Integer(l), Logical(r)) => (l - r).map(|x| x.into()),
            (Logical(l), Double(r)) => (l - r).map(|x| x.into()),
            (Logical(l), Integer(r)) => (l - r).map(|x| x.into()),
            (Logical(l), Logical(r)) => (l - r).map(|x| x.into()),
            _ => todo!(),
        }
    }
}

impl std::ops::Mul for Vector {
    type Output = Result<Vector, Signal>;
    fn mul(self, rhs: Self) -> Self::Output {
        use Vector::*;
        match (self, rhs) {
            (Double(l), Double(r)) => (l * r).map(|x| x.into()),
            (Double(l), Integer(r)) => (l * r).map(|x| x.into()),
            (Double(l), Logical(r)) => (l * r).map(|x| x.into()),
            (Integer(l), Double(r)) => (l * r).map(|x| x.into()),
            (Integer(l), Integer(r)) => (l * r).map(|x| x.into()),
            (Integer(l), Logical(r)) => (l * r).map(|x| x.into()),
            (Logical(l), Double(r)) => (l * r).map(|x| x.into()),
            (Logical(l), Integer(r)) => (l * r).map(|x| x.into()),
            (Logical(l), Logical(r)) => (l * r).map(|x| x.into()),
            _ => todo!(),
        }
    }
}

impl std::ops::Div for Vector {
    type Output = Result<Vector, Signal>;
    fn div(self, rhs: Self) -> Self::Output {
        use Vector::*;
        match (self, rhs) {
            (Double(l), Double(r)) => (l / r).map(|x| x.into()),
            (Double(l), Integer(r)) => (l / r).map(|x| x.into()),
            (Double(l), Logical(r)) => (l / r).map(|x| x.into()),
            (Integer(l), Double(r)) => (l / r).map(|x| x.into()),
            (Integer(l), Integer(r)) => (l / r).map(|x| x.into()),
            (Integer(l), Logical(r)) => (l / r).map(|x| x.into()),
            (Logical(l), Double(r)) => (l / r).map(|x| x.into()),
            (Logical(l), Integer(r)) => (l / r).map(|x| x.into()),
            (Logical(l), Logical(r)) => (l / r).map(|x| x.into()),
            _ => todo!(),
        }
    }
}

impl Pow<Vector> for Vector {
    type Output = Result<Vector, Signal>;
    fn power(self, rhs: Self) -> Self::Output {
        use Vector::*;
        match (self, rhs) {
            (Double(l), Double(r)) => l.power(r).map(|x| x.into()),
            (Double(l), Integer(r)) => l.power(r).map(|x| x.into()),
            (Double(l), Logical(r)) => l.power(r).map(|x| x.into()),
            (Integer(l), Double(r)) => l.power(r).map(|x| x.into()),
            (Integer(l), Integer(r)) => l.power(r).map(|x| x.into()),
            (Integer(l), Logical(r)) => l.power(r).map(|x| x.into()),
            (Logical(l), Double(r)) => l.power(r).map(|x| x.into()),
            (Logical(l), Integer(r)) => l.power(r).map(|x| x.into()),
            (Logical(l), Logical(r)) => l.power(r).map(|x| x.into()),
            _ => todo!(),
        }
    }
}

pub trait VecPartialCmp<Rhs> {
    type Output;
    fn vec_gt(self, rhs: Rhs) -> Self::Output;
    fn vec_gte(self, rhs: Rhs) -> Self::Output;
    fn vec_lt(self, rhs: Rhs) -> Self::Output;
    fn vec_lte(self, rhs: Rhs) -> Self::Output;
    fn vec_eq(self, rhs: Rhs) -> Self::Output;
    fn vec_neq(self, rhs: Rhs) -> Self::Output;
}

impl VecPartialCmp<Vector> for Vector {
    type Output = Result<Vector, Signal>;
    fn vec_gt(self, rhs: Self) -> Self::Output {
        use Vector::*;
        match (self, rhs) {
            (Double(l), Double(r)) => l.vec_gt(r).map(|x| x.into()),
            (Double(l), Integer(r)) => l.vec_gt(r).map(|x| x.into()),
            (Double(l), Logical(r)) => l.vec_gt(r).map(|x| x.into()),
            (Double(l), Character(r)) => l.vec_gt(r).map(|x| x.into()),
            (Integer(l), Double(r)) => l.vec_gt(r).map(|x| x.into()),
            (Integer(l), Integer(r)) => l.vec_gt(r).map(|x| x.into()),
            (Integer(l), Logical(r)) => l.vec_gt(r).map(|x| x.into()),
            (Integer(l), Character(r)) => l.vec_gt(r).map(|x| x.into()),
            (Logical(l), Double(r)) => l.vec_gt(r).map(|x| x.into()),
            (Logical(l), Integer(r)) => l.vec_gt(r).map(|x| x.into()),
            (Logical(l), Logical(r)) => l.vec_gt(r).map(|x| x.into()),
            (Logical(l), Character(r)) => l.vec_gt(r).map(|x| x.into()),
            (Character(l), Double(r)) => l.vec_gt(r).map(|x| x.into()),
            (Character(l), Integer(r)) => l.vec_gt(r).map(|x| x.into()),
            (Character(l), Logical(r)) => l.vec_gt(r).map(|x| x.into()),
            (Character(l), Character(r)) => l.vec_gt(r).map(|x| x.into()),
        }
    }

    fn vec_gte(self, rhs: Self) -> Self::Output {
        use Vector::*;
        match (self, rhs) {
            (Double(l), Double(r)) => l.vec_gte(r).map(|x| x.into()),
            (Double(l), Integer(r)) => l.vec_gte(r).map(|x| x.into()),
            (Double(l), Logical(r)) => l.vec_gte(r).map(|x| x.into()),
            (Double(l), Character(r)) => l.vec_gte(r).map(|x| x.into()),
            (Integer(l), Double(r)) => l.vec_gte(r).map(|x| x.into()),
            (Integer(l), Integer(r)) => l.vec_gte(r).map(|x| x.into()),
            (Integer(l), Logical(r)) => l.vec_gte(r).map(|x| x.into()),
            (Integer(l), Character(r)) => l.vec_gte(r).map(|x| x.into()),
            (Logical(l), Double(r)) => l.vec_gte(r).map(|x| x.into()),
            (Logical(l), Integer(r)) => l.vec_gte(r).map(|x| x.into()),
            (Logical(l), Logical(r)) => l.vec_gte(r).map(|x| x.into()),
            (Logical(l), Character(r)) => l.vec_gte(r).map(|x| x.into()),
            (Character(l), Double(r)) => l.vec_gte(r).map(|x| x.into()),
            (Character(l), Integer(r)) => l.vec_gte(r).map(|x| x.into()),
            (Character(l), Logical(r)) => l.vec_gte(r).map(|x| x.into()),
            (Character(l), Character(r)) => l.vec_gte(r).map(|x| x.into()),
        }
    }

    fn vec_lt(self, rhs: Self) -> Self::Output {
        use Vector::*;
        match (self, rhs) {
            (Double(l), Double(r)) => l.vec_lt(r).map(|x| x.into()),
            (Double(l), Integer(r)) => l.vec_lt(r).map(|x| x.into()),
            (Double(l), Logical(r)) => l.vec_lt(r).map(|x| x.into()),
            (Double(l), Character(r)) => l.vec_lt(r).map(|x| x.into()),
            (Integer(l), Double(r)) => l.vec_lt(r).map(|x| x.into()),
            (Integer(l), Integer(r)) => l.vec_lt(r).map(|x| x.into()),
            (Integer(l), Logical(r)) => l.vec_lt(r).map(|x| x.into()),
            (Integer(l), Character(r)) => l.vec_lt(r).map(|x| x.into()),
            (Logical(l), Double(r)) => l.vec_lt(r).map(|x| x.into()),
            (Logical(l), Integer(r)) => l.vec_lt(r).map(|x| x.into()),
            (Logical(l), Logical(r)) => l.vec_lt(r).map(|x| x.into()),
            (Logical(l), Character(r)) => l.vec_lt(r).map(|x| x.into()),
            (Character(l), Double(r)) => l.vec_lt(r).map(|x| x.into()),
            (Character(l), Integer(r)) => l.vec_lt(r).map(|x| x.into()),
            (Character(l), Logical(r)) => l.vec_lt(r).map(|x| x.into()),
            (Character(l), Character(r)) => l.vec_lt(r).map(|x| x.into()),
        }
    }

    fn vec_lte(self, rhs: Self) -> Self::Output {
        use Vector::*;
        match (self, rhs) {
            (Double(l), Double(r)) => l.vec_lte(r).map(|x| x.into()),
            (Double(l), Integer(r)) => l.vec_lte(r).map(|x| x.into()),
            (Double(l), Logical(r)) => l.vec_lte(r).map(|x| x.into()),
            (Double(l), Character(r)) => l.vec_lte(r).map(|x| x.into()),
            (Integer(l), Double(r)) => l.vec_lte(r).map(|x| x.into()),
            (Integer(l), Integer(r)) => l.vec_lte(r).map(|x| x.into()),
            (Integer(l), Logical(r)) => l.vec_lte(r).map(|x| x.into()),
            (Integer(l), Character(r)) => l.vec_lte(r).map(|x| x.into()),
            (Logical(l), Double(r)) => l.vec_lte(r).map(|x| x.into()),
            (Logical(l), Integer(r)) => l.vec_lte(r).map(|x| x.into()),
            (Logical(l), Logical(r)) => l.vec_lte(r).map(|x| x.into()),
            (Logical(l), Character(r)) => l.vec_lte(r).map(|x| x.into()),
            (Character(l), Double(r)) => l.vec_lte(r).map(|x| x.into()),
            (Character(l), Integer(r)) => l.vec_lte(r).map(|x| x.into()),
            (Character(l), Logical(r)) => l.vec_lte(r).map(|x| x.into()),
            (Character(l), Character(r)) => l.vec_lte(r).map(|x| x.into()),
        }
    }

    fn vec_eq(self, rhs: Self) -> Self::Output {
        use Vector::*;
        match (self, rhs) {
            (Double(l), Double(r)) => l.vec_eq(r).map(|x| x.into()),
            (Double(l), Integer(r)) => l.vec_eq(r).map(|x| x.into()),
            (Double(l), Logical(r)) => l.vec_eq(r).map(|x| x.into()),
            (Double(l), Character(r)) => l.vec_eq(r).map(|x| x.into()),
            (Integer(l), Double(r)) => l.vec_eq(r).map(|x| x.into()),
            (Integer(l), Integer(r)) => l.vec_eq(r).map(|x| x.into()),
            (Integer(l), Logical(r)) => l.vec_eq(r).map(|x| x.into()),
            (Integer(l), Character(r)) => l.vec_eq(r).map(|x| x.into()),
            (Logical(l), Double(r)) => l.vec_eq(r).map(|x| x.into()),
            (Logical(l), Integer(r)) => l.vec_eq(r).map(|x| x.into()),
            (Logical(l), Logical(r)) => l.vec_eq(r).map(|x| x.into()),
            (Logical(l), Character(r)) => l.vec_eq(r).map(|x| x.into()),
            (Character(l), Double(r)) => l.vec_eq(r).map(|x| x.into()),
            (Character(l), Integer(r)) => l.vec_eq(r).map(|x| x.into()),
            (Character(l), Logical(r)) => l.vec_eq(r).map(|x| x.into()),
            (Character(l), Character(r)) => l.vec_eq(r).map(|x| x.into()),
        }
    }

    fn vec_neq(self, rhs: Self) -> Self::Output {
        use Vector::*;
        match (self, rhs) {
            (Double(l), Double(r)) => l.vec_neq(r).map(|x| x.into()),
            (Double(l), Integer(r)) => l.vec_neq(r).map(|x| x.into()),
            (Double(l), Logical(r)) => l.vec_neq(r).map(|x| x.into()),
            (Double(l), Character(r)) => l.vec_neq(r).map(|x| x.into()),
            (Integer(l), Double(r)) => l.vec_neq(r).map(|x| x.into()),
            (Integer(l), Integer(r)) => l.vec_neq(r).map(|x| x.into()),
            (Integer(l), Logical(r)) => l.vec_neq(r).map(|x| x.into()),
            (Integer(l), Character(r)) => l.vec_neq(r).map(|x| x.into()),
            (Logical(l), Double(r)) => l.vec_neq(r).map(|x| x.into()),
            (Logical(l), Integer(r)) => l.vec_neq(r).map(|x| x.into()),
            (Logical(l), Logical(r)) => l.vec_neq(r).map(|x| x.into()),
            (Logical(l), Character(r)) => l.vec_neq(r).map(|x| x.into()),
            (Character(l), Double(r)) => l.vec_neq(r).map(|x| x.into()),
            (Character(l), Integer(r)) => l.vec_neq(r).map(|x| x.into()),
            (Character(l), Logical(r)) => l.vec_neq(r).map(|x| x.into()),
            (Character(l), Character(r)) => l.vec_neq(r).map(|x| x.into()),
        }
    }
}

impl std::ops::Rem for Vector {
    type Output = Result<Vector, Signal>;
    fn rem(self, rhs: Self) -> Self::Output {
        use Vector::*;
        match (self, rhs) {
            (Double(l), Double(r)) => l.rem(r).map(|x| x.into()),
            (Double(l), Integer(r)) => l.rem(r).map(|x| x.into()),
            (Double(l), Logical(r)) => l.rem(r).map(|x| x.into()),
            (Integer(l), Double(r)) => l.rem(r).map(|x| x.into()),
            (Integer(l), Integer(r)) => l.rem(r).map(|x| x.into()),
            (Integer(l), Logical(r)) => l.rem(r).map(|x| x.into()),
            (Logical(l), Double(r)) => l.rem(r).map(|x| x.into()),
            (Logical(l), Integer(r)) => l.rem(r).map(|x| x.into()),
            (Logical(l), Logical(r)) => l.rem(r).map(|x| x.into()),
            _ => todo!(),
        }
    }
}

impl std::ops::BitOr for Vector {
    type Output = Result<Vector, Signal>;
    fn bitor(self, rhs: Self) -> Self::Output {
        use Vector::*;
        match (self, rhs) {
            (Double(l), Double(r)) => l.bitor(r).map(|x| x.into()),
            (Double(l), Integer(r)) => l.bitor(r).map(|x| x.into()),
            (Double(l), Logical(r)) => l.bitor(r).map(|x| x.into()),
            (Double(l), Character(r)) => l.bitor(r).map(|x| x.into()),
            (Integer(l), Double(r)) => l.bitor(r).map(|x| x.into()),
            (Integer(l), Integer(r)) => l.bitor(r).map(|x| x.into()),
            (Integer(l), Logical(r)) => l.bitor(r).map(|x| x.into()),
            (Integer(l), Character(r)) => l.bitor(r).map(|x| x.into()),
            (Logical(l), Double(r)) => l.bitor(r).map(|x| x.into()),
            (Logical(l), Integer(r)) => l.bitor(r).map(|x| x.into()),
            (Logical(l), Logical(r)) => l.bitor(r).map(|x| x.into()),
            (Logical(l), Character(r)) => l.bitor(r).map(|x| x.into()),
            (Character(l), Double(r)) => l.bitor(r).map(|x| x.into()),
            (Character(l), Integer(r)) => l.bitor(r).map(|x| x.into()),
            (Character(l), Logical(r)) => l.bitor(r).map(|x| x.into()),
            (Character(l), Character(r)) => l.bitor(r).map(|x| x.into()),
        }
    }
}

impl std::ops::BitAnd for Vector {
    type Output = Result<Vector, Signal>;
    fn bitand(self, rhs: Self) -> Self::Output {
        use Vector::*;
        match (self, rhs) {
            (Double(l), Double(r)) => l.bitand(r).map(|x| x.into()),
            (Double(l), Integer(r)) => l.bitand(r).map(|x| x.into()),
            (Double(l), Logical(r)) => l.bitand(r).map(|x| x.into()),
            (Double(l), Character(r)) => l.bitand(r).map(|x| x.into()),
            (Integer(l), Double(r)) => l.bitand(r).map(|x| x.into()),
            (Integer(l), Integer(r)) => l.bitand(r).map(|x| x.into()),
            (Integer(l), Logical(r)) => l.bitand(r).map(|x| x.into()),
            (Integer(l), Character(r)) => l.bitand(r).map(|x| x.into()),
            (Logical(l), Double(r)) => l.bitand(r).map(|x| x.into()),
            (Logical(l), Integer(r)) => l.bitand(r).map(|x| x.into()),
            (Logical(l), Logical(r)) => l.bitand(r).map(|x| x.into()),
            (Logical(l), Character(r)) => l.bitand(r).map(|x| x.into()),
            (Character(l), Double(r)) => l.bitand(r).map(|x| x.into()),
            (Character(l), Integer(r)) => l.bitand(r).map(|x| x.into()),
            (Character(l), Logical(r)) => l.bitand(r).map(|x| x.into()),
            (Character(l), Character(r)) => l.bitand(r).map(|x| x.into()),
        }
    }
}

#[cfg(test)]

mod tests {
    use crate::{r, r_expect};

    #[test]
    fn double_brackets_assign() {
        r_expect! {{"
            x = c(1, 2)
            x[[1]] = 10
            x[[1]] == 10 & x[[2]] == 2
        "}}
    }
    #[test]
    fn comparison_conversion() {
        // double
        r_expect!(2 == 2);
        r_expect!(2 == 2L);
        r_expect!(1 == true);
        r_expect!(2 != false);
        r_expect!(2 != true);
        r_expect!(0 == false);
        // integer
        r_expect!(2L == 2L);
        r_expect!(2L == 2);
        r_expect!(2L != true);
        r_expect!(0L == false);
        r_expect!(1L == true);
        // logical
        r_expect!(true == 1);
        r_expect!(true == 1L);
        r_expect!(false == 0L);
        r_expect!(false == 0);
        r_expect! {{r#""true" == true"#}}
        // character
        r_expect!("a" == "a");
        r_expect!("a" != "b");
        r_expect!("1" == 1);
        r_expect!("1" == 1L);
        r_expect!("1" != 2L);
        r_expect!("1" != 2);

        // metaprogramming objects
        r_expect!(environment() == environment());
        r_expect!(quote(1) == quote(1));

        // length > 1 also works
        r_expect! {{"
            x = [1L, 2L]
            y = [1L, 2L]
            z = x == y
            z[1] && z[2]
        "}}
    }
    #[test]
    // test that types are as expected
    fn type_stability_num_ops() {
        r_expect! {{r#"
            typeof(1L + 1L) == "integer"
            typeof(1 + 1) == "double"
            typeof(1L + 1) == "double"
            typeof(1 + 1L) == "double"
            typeof(true + true) == "integer"
            typeof(true + false) == "integer"
        "#}}
    }
}
