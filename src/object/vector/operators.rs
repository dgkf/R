use std::cell::RefCell;
use std::iter::repeat;

use crate::error::Error;
use crate::lang::Signal;
use crate::object::coercion::{AtomicMode, CoercibleInto, CommonNum, MinimallyNumeric};
use crate::object::rep::Rep;
use crate::object::reptype::RepType;
use crate::object::Vector;

pub trait TryAdd<Rhs = Self> {
    type Output;
    fn try_add(self, rhs: Rhs) -> Result<Self::Output, Signal>;
}

pub trait TrySub<Rhs = Self> {
    type Output;
    fn try_sub(self, rhs: Rhs) -> Result<Self::Output, Signal>;
}

pub trait TryMul<Rhs = Self> {
    type Output;
    fn try_mul(self, rhs: Rhs) -> Result<Self::Output, Signal>;
}

pub trait TryDiv<Rhs = Self> {
    type Output;
    fn try_div(self, rhs: Rhs) -> Result<Self::Output, Signal>;
}

pub trait TryRem<Rhs = Self> {
    type Output;
    fn try_rem(self, rhs: Rhs) -> Result<Self::Output, Signal>;
}

pub trait TryPow<Rhs = Self> {
    type Output;
    fn try_pow(self, rhs: Rhs) -> Result<Self::Output, Signal>;
}

pub trait TryBitOr<Rhs = Self> {
    type Output;
    fn try_bitor(self, rhs: Rhs) -> Result<Self::Output, Signal>;
}

pub trait TryBitAnd<Rhs = Self> {
    type Output;
    fn try_bitand(self, rhs: Rhs) -> Result<Self::Output, Signal>;
}

pub trait TryVecPartialCmp<Rhs> {
    type Output;
    fn try_vec_gt(self, rhs: Rhs) -> Result<Self::Output, Signal>;
    fn try_vec_gte(self, rhs: Rhs) -> Result<Self::Output, Signal>;
    fn try_vec_lt(self, rhs: Rhs) -> Result<Self::Output, Signal>;
    fn try_vec_lte(self, rhs: Rhs) -> Result<Self::Output, Signal>;
    fn try_vec_eq(self, rhs: Rhs) -> Result<Self::Output, Signal>;
    fn try_vec_neq(self, rhs: Rhs) -> Result<Self::Output, Signal>;
}
