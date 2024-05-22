use std::fmt::Display;

use crate::{
    lang::{EvalResult, Signal},
    object::vector::rep::Rep,
    object::{coercion::CoercibleInto, Obj, Vector},
};

use super::{
    coercion::{Atomic, CommonCmp, CommonNum, MinimallyNumeric},
    VecPartialCmp,
};

#[derive(Debug, Clone, PartialEq)]
pub struct ScalarType<T>(pub T);

impl<T: Atomic + Display> Display for ScalarType<T> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            ScalarType(inner) => write!(f, "{inner}"),
        }
    }
}

#[derive(Debug, Clone, PartialEq)]
pub enum Scalar {
    Bool(ScalarType<bool>),
    I32(ScalarType<i32>),
    F64(ScalarType<f64>),
    String(ScalarType<String>),
    NA,
    NEGINF,
    INF,
}

impl Display for Scalar {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        use Scalar::*;
        match self {
            Bool(x) => write!(f, "{x}"),
            I32(x) => write!(f, "{x}"),
            F64(x) => write!(f, "{x}"),
            String(x) => write!(f, "{x}"),
            NA => write!(f, "NA"),
            INF => write!(f, "inf"),
            NEGINF => write!(f, "-inf"),
        }
    }
}

impl Default for Scalar {
    fn default() -> Self {
        Self::Bool(ScalarType(true))
    }
}

impl<T> ScalarType<T> {
    pub fn as_integer(self) -> EvalResult
    where
        T: CoercibleInto<i32>,
    {
        match self {
            ScalarType(inner) => Ok(Obj::Scalar(Scalar::I32(ScalarType(inner.coerce_into())))),
        }
    }

    pub fn as_double(self) -> EvalResult
    where
        T: CoercibleInto<f64>,
    {
        match self {
            ScalarType(inner) => Ok(Obj::Scalar(Scalar::F64(ScalarType(inner.coerce_into())))),
        }
    }

    pub fn as_logical(self) -> EvalResult
    where
        T: CoercibleInto<bool>,
    {
        match self {
            ScalarType(inner) => Ok(Obj::Scalar(Scalar::Bool(ScalarType(inner.coerce_into())))),
        }
    }

    pub fn as_character(self) -> EvalResult
    where
        T: CoercibleInto<String>,
    {
        match self {
            ScalarType(inner) => Ok(Obj::Scalar(Scalar::String(ScalarType(inner.coerce_into())))),
        }
    }
}

impl Scalar {
    pub fn as_vector(self) -> EvalResult {
        use Scalar::*;
        match self {
            Bool(ScalarType(x)) => Ok(Obj::Vector(Vector::from(vec![x]))),
            I32(ScalarType(x)) => Ok(Obj::Vector(Vector::from(vec![x]))),
            F64(ScalarType(x)) => Ok(Obj::Vector(Vector::from(vec![x]))),
            String(ScalarType(x)) => Ok(Obj::Vector(Vector::from(vec![x]))),
            NA => todo!(),
            INF => todo!(),
            NEGINF => todo!(),
        }
    }

    pub fn as_integer(self) -> EvalResult {
        use crate::error::Error::*;
        use Scalar::*;
        match self {
            Bool(x) => x.as_integer(),
            I32(_) => Ok(Obj::Scalar(self)),
            F64(x) => x.as_integer(),
            String(_) => CannotBeCoercedToInteger.into(),
            NA => todo!(),
            INF => todo!(),
            NEGINF => todo!(),
        }
    }

    pub fn as_double(self) -> EvalResult {
        use crate::error::Error::*;
        use Scalar::*;
        match self {
            Bool(x) => x.as_double(),
            I32(x) => x.as_double(),
            F64(_) => Ok(Obj::Scalar(self)),
            String(_) => CannotBeCoercedToDouble.into(),
            NA => todo!(),
            INF => todo!(),
            NEGINF => todo!(),
        }
    }

    pub fn as_logical(self) -> EvalResult {
        use crate::error::Error::*;
        use Scalar::*;
        match self {
            Bool(_) => Ok(Obj::Scalar(self)),
            I32(x) => x.as_logical(),
            F64(x) => x.as_logical(),
            String(_) => CannotBeCoercedToLogical.into(),
            NA => todo!(),
            INF => todo!(),
            NEGINF => todo!(),
        }
    }

    pub fn as_character(self) -> EvalResult {
        use Scalar::*;
        match self {
            Bool(x) => x.as_character(),
            I32(x) => x.as_character(),
            F64(x) => x.as_character(),
            String(_) => Ok(Obj::Scalar(self)),
            NA => todo!(),
            INF => todo!(),
            NEGINF => todo!(),
        }
    }
}

impl<T> TryInto<Scalar> for Rep<T> {
    type Error = Signal;
    fn try_into(self) -> Result<Scalar, Self::Error> {
        match self {
            Rep::Subset(_, _) => todo!(),
        }
    }
}

impl From<Scalar> for bool {
    fn from(value: Scalar) -> Self {
        match value {
            Scalar::Bool(ScalarType(x)) => x,
            Scalar::I32(ScalarType(x)) => x != 0,
            Scalar::F64(ScalarType(x)) => x.round() as i32 != 0,
            Scalar::String(ScalarType(_)) => todo!(),
            Scalar::NA => todo!(),
            Scalar::NEGINF => todo!(),
            Scalar::INF => todo!(),
        }
    }
}

impl From<bool> for Scalar {
    fn from(val: bool) -> Self {
        Scalar::Bool(ScalarType(val))
    }
}

impl From<i32> for Scalar {
    fn from(val: i32) -> Self {
        Scalar::I32(ScalarType(val))
    }
}

impl From<f64> for Scalar {
    fn from(val: f64) -> Self {
        Scalar::F64(ScalarType(val))
    }
}

impl From<String> for Scalar {
    fn from(val: String) -> Self {
        Scalar::String(ScalarType(val))
    }
}

impl std::ops::Neg for Scalar {
    type Output = Scalar;
    fn neg(self) -> Self::Output {
        match self {
            Scalar::Bool(ScalarType(x)) => {
                Scalar::I32(ScalarType(-CoercibleInto::<i32>::coerce_into(x)))
            }
            Scalar::I32(ScalarType(x)) => Scalar::I32(ScalarType(-x)),
            Scalar::F64(ScalarType(x)) => Scalar::F64(ScalarType(-x)),
            Scalar::NA => Scalar::NA,
            Scalar::INF => Scalar::NEGINF,
            Scalar::NEGINF => Scalar::INF,
            _ => todo!(),
        }
    }
}

impl std::ops::Add for Scalar {
    type Output = Scalar;
    fn add(self, rhs: Self) -> Self::Output {
        match (self, rhs) {
            (Scalar::Bool(l), Scalar::Bool(r)) => (l + r).into(),
            (Scalar::Bool(l), Scalar::I32(r)) => (l + r).into(),
            (Scalar::Bool(l), Scalar::F64(r)) => (l + r).into(),
            (Scalar::I32(l), Scalar::Bool(r)) => (l + r).into(),
            (Scalar::I32(l), Scalar::I32(r)) => (l + r).into(),
            (Scalar::I32(l), Scalar::F64(r)) => (l + r).into(),
            (Scalar::F64(l), Scalar::Bool(r)) => (l + r).into(),
            (Scalar::F64(l), Scalar::I32(r)) => (l + r).into(),
            (Scalar::F64(l), Scalar::F64(r)) => (l + r).into(),
            (Scalar::NA, _) => Scalar::NA,
            (_, Scalar::NA) => Scalar::NA,
            (Scalar::INF, _) => Scalar::INF,
            (_, Scalar::INF) => Scalar::INF,
            _ => todo!(),
        }
    }
}

impl<L, R, C, O, LNum, RNum> std::ops::Add<ScalarType<R>> for ScalarType<L>
where
    L: Atomic + MinimallyNumeric<As = LNum> + CoercibleInto<LNum>,
    R: Atomic + MinimallyNumeric<As = RNum> + CoercibleInto<RNum>,
    (LNum, RNum): CommonNum<Common = C>,
    C: std::ops::Add<Output = O>,
{
    type Output = O;
    fn add(self, rhs: ScalarType<R>) -> Self::Output {
        let ScalarType(l_inner) = self;
        let ScalarType(r_inner) = rhs;
        let (l_common, r_common) = (
            CoercibleInto::<LNum>::coerce_into(l_inner),
            CoercibleInto::<RNum>::coerce_into(r_inner),
        )
            .into_common();

        l_common + r_common
    }
}

impl std::ops::Sub for Scalar {
    type Output = Scalar;
    fn sub(self, rhs: Self) -> Self::Output {
        match (self, rhs) {
            (Scalar::Bool(l), Scalar::Bool(r)) => (l - r).into(),
            (Scalar::Bool(l), Scalar::I32(r)) => (l - r).into(),
            (Scalar::Bool(l), Scalar::F64(r)) => (l - r).into(),
            (Scalar::I32(l), Scalar::Bool(r)) => (l - r).into(),
            (Scalar::I32(l), Scalar::I32(r)) => (l - r).into(),
            (Scalar::I32(l), Scalar::F64(r)) => (l - r).into(),
            (Scalar::F64(l), Scalar::Bool(r)) => (l - r).into(),
            (Scalar::F64(l), Scalar::I32(r)) => (l - r).into(),
            (Scalar::F64(l), Scalar::F64(r)) => (l - r).into(),
            (Scalar::NA, _) => Scalar::NA,
            (_, Scalar::NA) => Scalar::NA,
            (Scalar::INF, _) => Scalar::INF,
            (_, Scalar::INF) => Scalar::NEGINF,
            (Scalar::NEGINF, _) => Scalar::NEGINF,
            (_, Scalar::NEGINF) => Scalar::INF,
            _ => todo!(),
        }
    }
}

impl<L, R, C, O, LNum, RNum> std::ops::Sub<ScalarType<R>> for ScalarType<L>
where
    L: Atomic + MinimallyNumeric<As = LNum> + CoercibleInto<LNum>,
    R: Atomic + MinimallyNumeric<As = RNum> + CoercibleInto<RNum>,
    (LNum, RNum): CommonNum<Common = C>,
    C: std::ops::Sub<Output = O>,
{
    type Output = O;
    fn sub(self, rhs: ScalarType<R>) -> Self::Output {
        let ScalarType(l_inner) = self;
        let ScalarType(r_inner) = rhs;
        let (l_common, r_common) = (
            CoercibleInto::<LNum>::coerce_into(l_inner),
            CoercibleInto::<RNum>::coerce_into(r_inner),
        )
            .into_common();

        l_common - r_common
    }
}
impl std::ops::Mul for Scalar {
    type Output = Scalar;
    fn mul(self, rhs: Self) -> Self::Output {
        match (self, rhs) {
            (Scalar::Bool(l), Scalar::Bool(r)) => (l * r).into(),
            (Scalar::Bool(l), Scalar::I32(r)) => (l * r).into(),
            (Scalar::Bool(l), Scalar::F64(r)) => (l * r).into(),
            (Scalar::I32(l), Scalar::Bool(r)) => (l * r).into(),
            (Scalar::I32(l), Scalar::I32(r)) => (l * r).into(),
            (Scalar::I32(l), Scalar::F64(r)) => (l * r).into(),
            (Scalar::F64(l), Scalar::Bool(r)) => (l * r).into(),
            (Scalar::F64(l), Scalar::I32(r)) => (l * r).into(),
            (Scalar::F64(l), Scalar::F64(r)) => (l * r).into(),
            (Scalar::NA, _) => Scalar::NA,
            (_, Scalar::NA) => Scalar::NA,
            (Scalar::INF, _) => Scalar::INF,
            (_, Scalar::INF) => Scalar::INF,
            (Scalar::NEGINF, _) => Scalar::NEGINF,
            (_, Scalar::NEGINF) => Scalar::NEGINF,
            _ => todo!(),
        }
    }
}

impl<L, R, C, O, LNum, RNum> std::ops::Mul<ScalarType<R>> for ScalarType<L>
where
    L: Atomic + MinimallyNumeric<As = LNum> + CoercibleInto<LNum>,
    R: Atomic + MinimallyNumeric<As = RNum> + CoercibleInto<RNum>,
    (LNum, RNum): CommonNum<Common = C>,
    C: std::ops::Mul<Output = O>,
{
    type Output = O;
    fn mul(self, rhs: ScalarType<R>) -> Self::Output {
        let ScalarType(l_inner) = self;
        let ScalarType(r_inner) = rhs;
        let (l_common, r_common) = (
            CoercibleInto::<LNum>::coerce_into(l_inner),
            CoercibleInto::<RNum>::coerce_into(r_inner),
        )
            .into_common();

        l_common * r_common
    }
}

impl std::ops::Div for Scalar {
    type Output = Scalar;
    fn div(self, rhs: Self) -> Self::Output {
        match (self, rhs) {
            (Scalar::Bool(l), Scalar::Bool(r)) => (l / r).into(),
            (Scalar::Bool(l), Scalar::I32(r)) => (l / r).into(),
            (Scalar::Bool(l), Scalar::F64(r)) => (l / r).into(),
            (Scalar::I32(l), Scalar::Bool(r)) => (l / r).into(),
            (Scalar::I32(l), Scalar::I32(r)) => (l / r).into(),
            (Scalar::I32(l), Scalar::F64(r)) => (l / r).into(),
            (Scalar::F64(l), Scalar::Bool(r)) => (l / r).into(),
            (Scalar::F64(l), Scalar::I32(r)) => (l / r).into(),
            (Scalar::F64(l), Scalar::F64(r)) => (l / r).into(),
            (Scalar::NA, _) => Scalar::NA,
            (_, Scalar::NA) => Scalar::NA,
            (Scalar::INF, _) => Scalar::INF,
            (_, Scalar::INF) => Scalar::F64(ScalarType(0.0)),
            (Scalar::NEGINF, _) => Scalar::NEGINF,
            (_, Scalar::NEGINF) => Scalar::F64(ScalarType(-0.0)),
            _ => todo!(),
        }
    }
}

impl<L, R, C, O, LNum, RNum> std::ops::Div<ScalarType<R>> for ScalarType<L>
where
    L: Atomic + MinimallyNumeric<As = LNum> + CoercibleInto<LNum>,
    R: Atomic + MinimallyNumeric<As = RNum> + CoercibleInto<RNum>,
    (LNum, RNum): CommonNum<Common = C>,
    C: std::ops::Div<Output = O>,
{
    type Output = O;
    fn div(self, rhs: ScalarType<R>) -> Self::Output {
        let ScalarType(l_inner) = self;
        let ScalarType(r_inner) = rhs;
        let (l_common, r_common) = (
            CoercibleInto::<LNum>::coerce_into(l_inner),
            CoercibleInto::<RNum>::coerce_into(r_inner),
        )
            .into_common();

        l_common / r_common
    }
}

impl super::Pow<Scalar> for Scalar {
    type Output = Scalar;
    fn power(self, rhs: Self) -> Self::Output {
        match (self, rhs) {
            (Scalar::Bool(l), Scalar::Bool(r)) => (l.power(r)).into(),
            (Scalar::Bool(l), Scalar::I32(r)) => (l.power(r)).into(),
            (Scalar::Bool(l), Scalar::F64(r)) => (l.power(r)).into(),
            (Scalar::I32(l), Scalar::Bool(r)) => (l.power(r)).into(),
            (Scalar::I32(l), Scalar::I32(r)) => (l.power(r)).into(),
            (Scalar::I32(l), Scalar::F64(r)) => (l.power(r)).into(),
            (Scalar::F64(l), Scalar::Bool(r)) => (l.power(r)).into(),
            (Scalar::F64(l), Scalar::I32(r)) => (l.power(r)).into(),
            (Scalar::F64(l), Scalar::F64(r)) => (l.power(r)).into(),
            (Scalar::NA, _) => Scalar::NA,
            (_, Scalar::NA) => Scalar::NA,
            (Scalar::INF, _) => Scalar::INF,
            (_, Scalar::INF) => Scalar::F64(ScalarType(0.0)),
            (Scalar::NEGINF, _) => Scalar::NEGINF,
            (_, Scalar::NEGINF) => Scalar::F64(ScalarType(-0.0)),
            _ => todo!(),
        }
    }
}

impl<L, R, C, O, LNum, RNum> super::Pow<ScalarType<R>> for ScalarType<L>
where
    L: Atomic + MinimallyNumeric<As = LNum> + CoercibleInto<LNum>,
    R: Atomic + MinimallyNumeric<As = RNum> + CoercibleInto<RNum>,
    (LNum, RNum): CommonNum<Common = C>,
    C: super::Pow<C, Output = O>,
{
    type Output = O;
    fn power(self, rhs: ScalarType<R>) -> Self::Output {
        let ScalarType(l_inner) = self;
        let ScalarType(r_inner) = rhs;
        let (l_common, r_common) = (
            CoercibleInto::<LNum>::coerce_into(l_inner),
            CoercibleInto::<RNum>::coerce_into(r_inner),
        )
            .into_common();

        l_common.power(r_common)
    }
}

impl std::ops::Rem for Scalar {
    type Output = Scalar;
    fn rem(self, rhs: Self) -> Self::Output {
        match (self, rhs) {
            (Scalar::Bool(l), Scalar::Bool(r)) => (l % r).into(),
            (Scalar::Bool(l), Scalar::I32(r)) => (l % r).into(),
            (Scalar::Bool(l), Scalar::F64(r)) => (l % r).into(),
            (Scalar::I32(l), Scalar::Bool(r)) => (l % r).into(),
            (Scalar::I32(l), Scalar::I32(r)) => (l % r).into(),
            (Scalar::I32(l), Scalar::F64(r)) => (l % r).into(),
            (Scalar::F64(l), Scalar::Bool(r)) => (l % r).into(),
            (Scalar::F64(l), Scalar::I32(r)) => (l % r).into(),
            (Scalar::F64(l), Scalar::F64(r)) => (l % r).into(),
            (Scalar::NA, _) => Scalar::NA,
            (_, Scalar::NA) => Scalar::NA,
            (Scalar::INF, _) => Scalar::INF,
            (_, Scalar::INF) => Scalar::F64(ScalarType(0.0)),
            (Scalar::NEGINF, _) => Scalar::NEGINF,
            (_, Scalar::NEGINF) => Scalar::F64(ScalarType(-0.0)),
            _ => todo!(),
        }
    }
}

impl<L, R, C, O, LNum, RNum> std::ops::Rem<ScalarType<R>> for ScalarType<L>
where
    L: Atomic + MinimallyNumeric<As = LNum> + CoercibleInto<LNum>,
    R: Atomic + MinimallyNumeric<As = RNum> + CoercibleInto<RNum>,
    (LNum, RNum): CommonNum<Common = C>,
    C: std::ops::Rem<Output = O>,
{
    type Output = O;
    fn rem(self, rhs: ScalarType<R>) -> Self::Output {
        let ScalarType(l_inner) = self;
        let ScalarType(r_inner) = rhs;
        let (l_common, r_common) = (
            CoercibleInto::<LNum>::coerce_into(l_inner),
            CoercibleInto::<RNum>::coerce_into(r_inner),
        )
            .into_common();

        l_common % r_common
    }
}

impl std::ops::BitOr for Scalar {
    type Output = Scalar;
    fn bitor(self, rhs: Self) -> Self::Output {
        match (self, rhs) {
            (Scalar::Bool(l), Scalar::Bool(r)) => (l | r).into(),
            (Scalar::Bool(l), Scalar::I32(r)) => (l | r).into(),
            (Scalar::Bool(l), Scalar::F64(r)) => (l | r).into(),
            (Scalar::I32(l), Scalar::Bool(r)) => (l | r).into(),
            (Scalar::I32(l), Scalar::I32(r)) => (l | r).into(),
            (Scalar::I32(l), Scalar::F64(r)) => (l | r).into(),
            (Scalar::F64(l), Scalar::Bool(r)) => (l | r).into(),
            (Scalar::F64(l), Scalar::I32(r)) => (l | r).into(),
            (Scalar::F64(l), Scalar::F64(r)) => (l | r).into(),
            (Scalar::NA, _) => Scalar::NA,
            (_, Scalar::NA) => Scalar::NA,
            (Scalar::INF, _) => Scalar::INF,
            (_, Scalar::INF) => Scalar::F64(ScalarType(0.0)),
            (Scalar::NEGINF, _) => Scalar::NEGINF,
            (_, Scalar::NEGINF) => Scalar::F64(ScalarType(-0.0)),
            _ => todo!(),
        }
    }
}

impl<L, R> std::ops::BitOr<ScalarType<R>> for ScalarType<L>
where
    L: Atomic + CoercibleInto<bool>,
    R: Atomic + CoercibleInto<bool>,
    bool: std::ops::BitOr<bool, Output = bool>,
{
    type Output = bool;
    fn bitor(self, rhs: ScalarType<R>) -> Self::Output {
        let ScalarType(l_inner) = self;
        let ScalarType(r_inner) = rhs;
        CoercibleInto::<bool>::coerce_into(l_inner) | CoercibleInto::<bool>::coerce_into(r_inner)
    }
}

impl std::ops::BitAnd for Scalar {
    type Output = Scalar;
    fn bitand(self, rhs: Self) -> Self::Output {
        match (self, rhs) {
            (Scalar::Bool(l), Scalar::Bool(r)) => (l & r).into(),
            (Scalar::Bool(l), Scalar::I32(r)) => (l & r).into(),
            (Scalar::Bool(l), Scalar::F64(r)) => (l & r).into(),
            (Scalar::I32(l), Scalar::Bool(r)) => (l & r).into(),
            (Scalar::I32(l), Scalar::I32(r)) => (l & r).into(),
            (Scalar::I32(l), Scalar::F64(r)) => (l & r).into(),
            (Scalar::F64(l), Scalar::Bool(r)) => (l & r).into(),
            (Scalar::F64(l), Scalar::I32(r)) => (l & r).into(),
            (Scalar::F64(l), Scalar::F64(r)) => (l & r).into(),
            (Scalar::NA, _) => Scalar::NA,
            (_, Scalar::NA) => Scalar::NA,
            (Scalar::INF, _) => Scalar::INF,
            (_, Scalar::INF) => Scalar::F64(ScalarType(0.0)),
            (Scalar::NEGINF, _) => Scalar::NEGINF,
            (_, Scalar::NEGINF) => Scalar::F64(ScalarType(-0.0)),
            _ => todo!(),
        }
    }
}

impl<L, R> std::ops::BitAnd<ScalarType<R>> for ScalarType<L>
where
    L: Atomic + CoercibleInto<bool>,
    R: Atomic + CoercibleInto<bool>,
    bool: std::ops::BitAnd<bool, Output = bool>,
{
    type Output = bool;
    fn bitand(self, rhs: ScalarType<R>) -> Self::Output {
        let ScalarType(l_inner) = self;
        let ScalarType(r_inner) = rhs;
        CoercibleInto::<bool>::coerce_into(l_inner) & CoercibleInto::<bool>::coerce_into(r_inner)
    }
}

impl VecPartialCmp<Scalar> for Scalar {
    type Output = Scalar;
    fn vec_gt(self, rhs: Scalar) -> Self::Output {
        match (self, rhs) {
            (Scalar::Bool(l), Scalar::Bool(r)) => Scalar::Bool(l.vec_gt(r)),
            (Scalar::Bool(l), Scalar::I32(r)) => Scalar::Bool(l.vec_gt(r)),
            (Scalar::Bool(l), Scalar::F64(r)) => Scalar::Bool(l.vec_gt(r)),
            (Scalar::I32(l), Scalar::Bool(r)) => Scalar::Bool(l.vec_gt(r)),
            (Scalar::I32(l), Scalar::I32(r)) => Scalar::Bool(l.vec_gt(r)),
            (Scalar::I32(l), Scalar::F64(r)) => Scalar::Bool(l.vec_gt(r)),
            (Scalar::F64(l), Scalar::Bool(r)) => Scalar::Bool(l.vec_gt(r)),
            (Scalar::F64(l), Scalar::I32(r)) => Scalar::Bool(l.vec_gt(r)),
            (Scalar::F64(l), Scalar::F64(r)) => Scalar::Bool(l.vec_gt(r)),
            (Scalar::NA, _) => Scalar::NA,
            (_, Scalar::NA) => Scalar::NA,
            (Scalar::INF, _) => Scalar::Bool(ScalarType(true)),
            (_, Scalar::INF) => Scalar::Bool(ScalarType(false)),
            (Scalar::NEGINF, _) => Scalar::Bool(ScalarType(false)),
            (_, Scalar::NEGINF) => Scalar::Bool(ScalarType(true)),
            _ => todo!(),
        }
    }

    fn vec_gte(self, rhs: Scalar) -> Self::Output {
        match (self, rhs) {
            (Scalar::Bool(l), Scalar::Bool(r)) => Scalar::Bool(l.vec_gte(r)),
            (Scalar::Bool(l), Scalar::I32(r)) => Scalar::Bool(l.vec_gte(r)),
            (Scalar::Bool(l), Scalar::F64(r)) => Scalar::Bool(l.vec_gte(r)),
            (Scalar::I32(l), Scalar::Bool(r)) => Scalar::Bool(l.vec_gte(r)),
            (Scalar::I32(l), Scalar::I32(r)) => Scalar::Bool(l.vec_gte(r)),
            (Scalar::I32(l), Scalar::F64(r)) => Scalar::Bool(l.vec_gte(r)),
            (Scalar::F64(l), Scalar::Bool(r)) => Scalar::Bool(l.vec_gte(r)),
            (Scalar::F64(l), Scalar::I32(r)) => Scalar::Bool(l.vec_gte(r)),
            (Scalar::F64(l), Scalar::F64(r)) => Scalar::Bool(l.vec_gte(r)),
            (Scalar::NA, _) => Scalar::NA,
            (_, Scalar::NA) => Scalar::NA,
            (Scalar::INF, _) => Scalar::Bool(ScalarType(true)),
            (_, Scalar::INF) => Scalar::Bool(ScalarType(false)),
            (Scalar::NEGINF, _) => Scalar::Bool(ScalarType(false)),
            (_, Scalar::NEGINF) => Scalar::Bool(ScalarType(true)),
            _ => todo!(),
        }
    }

    fn vec_lt(self, rhs: Scalar) -> Self::Output {
        match (self, rhs) {
            (Scalar::Bool(l), Scalar::Bool(r)) => Scalar::Bool(l.vec_lt(r)),
            (Scalar::Bool(l), Scalar::I32(r)) => Scalar::Bool(l.vec_lt(r)),
            (Scalar::Bool(l), Scalar::F64(r)) => Scalar::Bool(l.vec_lt(r)),
            (Scalar::I32(l), Scalar::Bool(r)) => Scalar::Bool(l.vec_lt(r)),
            (Scalar::I32(l), Scalar::I32(r)) => Scalar::Bool(l.vec_lt(r)),
            (Scalar::I32(l), Scalar::F64(r)) => Scalar::Bool(l.vec_lt(r)),
            (Scalar::F64(l), Scalar::Bool(r)) => Scalar::Bool(l.vec_lt(r)),
            (Scalar::F64(l), Scalar::I32(r)) => Scalar::Bool(l.vec_lt(r)),
            (Scalar::F64(l), Scalar::F64(r)) => Scalar::Bool(l.vec_lt(r)),
            (Scalar::NA, _) => Scalar::NA,
            (_, Scalar::NA) => Scalar::NA,
            (Scalar::INF, _) => Scalar::Bool(ScalarType(false)),
            (_, Scalar::INF) => Scalar::Bool(ScalarType(true)),
            (Scalar::NEGINF, _) => Scalar::Bool(ScalarType(true)),
            (_, Scalar::NEGINF) => Scalar::Bool(ScalarType(false)),
            _ => todo!(),
        }
    }

    fn vec_lte(self, rhs: Scalar) -> Self::Output {
        match (self, rhs) {
            (Scalar::Bool(l), Scalar::Bool(r)) => Scalar::Bool(l.vec_lte(r)),
            (Scalar::Bool(l), Scalar::I32(r)) => Scalar::Bool(l.vec_lte(r)),
            (Scalar::Bool(l), Scalar::F64(r)) => Scalar::Bool(l.vec_lte(r)),
            (Scalar::I32(l), Scalar::Bool(r)) => Scalar::Bool(l.vec_lte(r)),
            (Scalar::I32(l), Scalar::I32(r)) => Scalar::Bool(l.vec_lte(r)),
            (Scalar::I32(l), Scalar::F64(r)) => Scalar::Bool(l.vec_lte(r)),
            (Scalar::F64(l), Scalar::Bool(r)) => Scalar::Bool(l.vec_lte(r)),
            (Scalar::F64(l), Scalar::I32(r)) => Scalar::Bool(l.vec_lte(r)),
            (Scalar::F64(l), Scalar::F64(r)) => Scalar::Bool(l.vec_lte(r)),
            (Scalar::NA, _) => Scalar::NA,
            (_, Scalar::NA) => Scalar::NA,
            (Scalar::INF, _) => Scalar::Bool(ScalarType(false)),
            (_, Scalar::INF) => Scalar::Bool(ScalarType(true)),
            (Scalar::NEGINF, _) => Scalar::Bool(ScalarType(true)),
            (_, Scalar::NEGINF) => Scalar::Bool(ScalarType(false)),
            _ => todo!(),
        }
    }

    fn vec_eq(self, rhs: Scalar) -> Self::Output {
        match (self, rhs) {
            (Scalar::Bool(l), Scalar::Bool(r)) => Scalar::Bool(l.vec_eq(r)),
            (Scalar::Bool(l), Scalar::I32(r)) => Scalar::Bool(l.vec_eq(r)),
            (Scalar::Bool(l), Scalar::F64(r)) => Scalar::Bool(l.vec_eq(r)),
            (Scalar::I32(l), Scalar::Bool(r)) => Scalar::Bool(l.vec_eq(r)),
            (Scalar::I32(l), Scalar::I32(r)) => Scalar::Bool(l.vec_eq(r)),
            (Scalar::I32(l), Scalar::F64(r)) => Scalar::Bool(l.vec_eq(r)),
            (Scalar::F64(l), Scalar::Bool(r)) => Scalar::Bool(l.vec_eq(r)),
            (Scalar::F64(l), Scalar::I32(r)) => Scalar::Bool(l.vec_eq(r)),
            (Scalar::F64(l), Scalar::F64(r)) => Scalar::Bool(l.vec_eq(r)),
            (Scalar::NA, _) => Scalar::NA,
            (_, Scalar::NA) => Scalar::NA,
            (Scalar::INF, Scalar::INF) => Scalar::Bool(ScalarType(true)),
            (Scalar::INF, _) => Scalar::Bool(ScalarType(false)),
            (_, Scalar::INF) => Scalar::Bool(ScalarType(false)),
            (Scalar::NEGINF, Scalar::NEGINF) => Scalar::Bool(ScalarType(true)),
            (Scalar::NEGINF, _) => Scalar::Bool(ScalarType(false)),
            (_, Scalar::NEGINF) => Scalar::Bool(ScalarType(false)),
            _ => todo!(),
        }
    }

    fn vec_neq(self, rhs: Scalar) -> Self::Output {
        match (self, rhs) {
            (Scalar::Bool(l), Scalar::Bool(r)) => Scalar::Bool(l.vec_neq(r)),
            (Scalar::Bool(l), Scalar::I32(r)) => Scalar::Bool(l.vec_neq(r)),
            (Scalar::Bool(l), Scalar::F64(r)) => Scalar::Bool(l.vec_neq(r)),
            (Scalar::I32(l), Scalar::Bool(r)) => Scalar::Bool(l.vec_neq(r)),
            (Scalar::I32(l), Scalar::I32(r)) => Scalar::Bool(l.vec_neq(r)),
            (Scalar::I32(l), Scalar::F64(r)) => Scalar::Bool(l.vec_neq(r)),
            (Scalar::F64(l), Scalar::Bool(r)) => Scalar::Bool(l.vec_neq(r)),
            (Scalar::F64(l), Scalar::I32(r)) => Scalar::Bool(l.vec_neq(r)),
            (Scalar::F64(l), Scalar::F64(r)) => Scalar::Bool(l.vec_neq(r)),
            (Scalar::NA, _) => Scalar::NA,
            (_, Scalar::NA) => Scalar::NA,
            (Scalar::INF, Scalar::INF) => Scalar::Bool(ScalarType(false)),
            (Scalar::INF, _) => Scalar::Bool(ScalarType(true)),
            (_, Scalar::INF) => Scalar::Bool(ScalarType(true)),
            (Scalar::NEGINF, Scalar::NEGINF) => Scalar::Bool(ScalarType(false)),
            (Scalar::NEGINF, _) => Scalar::Bool(ScalarType(true)),
            (_, Scalar::NEGINF) => Scalar::Bool(ScalarType(true)),
            _ => todo!(),
        }
    }
}

impl<L, R, C> VecPartialCmp<ScalarType<R>> for ScalarType<L>
where
    L: Atomic + CoercibleInto<C>,
    R: Atomic + CoercibleInto<C>,
    (L, R): CommonCmp<Common = C>,
    C: PartialOrd,
{
    type Output = ScalarType<bool>;

    fn vec_gt(self, rhs: ScalarType<R>) -> Self::Output {
        let ScalarType(l_inner) = self;
        let ScalarType(r_inner) = rhs;
        ScalarType(
            CoercibleInto::<C>::coerce_into(l_inner) > CoercibleInto::<C>::coerce_into(r_inner),
        )
    }

    fn vec_gte(self, rhs: ScalarType<R>) -> Self::Output {
        let ScalarType(l_inner) = self;
        let ScalarType(r_inner) = rhs;
        ScalarType(
            CoercibleInto::<C>::coerce_into(l_inner) >= CoercibleInto::<C>::coerce_into(r_inner),
        )
    }

    fn vec_lt(self, rhs: ScalarType<R>) -> Self::Output {
        let ScalarType(l_inner) = self;
        let ScalarType(r_inner) = rhs;
        ScalarType(
            CoercibleInto::<C>::coerce_into(l_inner) < CoercibleInto::<C>::coerce_into(r_inner),
        )
    }

    fn vec_lte(self, rhs: ScalarType<R>) -> Self::Output {
        let ScalarType(l_inner) = self;
        let ScalarType(r_inner) = rhs;
        ScalarType(
            CoercibleInto::<C>::coerce_into(l_inner) <= CoercibleInto::<C>::coerce_into(r_inner),
        )
    }

    fn vec_eq(self, rhs: ScalarType<R>) -> Self::Output {
        let ScalarType(l_inner) = self;
        let ScalarType(r_inner) = rhs;
        ScalarType(
            CoercibleInto::<C>::coerce_into(l_inner) == CoercibleInto::<C>::coerce_into(r_inner),
        )
    }

    fn vec_neq(self, rhs: ScalarType<R>) -> Self::Output {
        let ScalarType(l_inner) = self;
        let ScalarType(r_inner) = rhs;
        ScalarType(
            CoercibleInto::<C>::coerce_into(l_inner) != CoercibleInto::<C>::coerce_into(r_inner),
        )
    }
}
