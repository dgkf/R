use std::fmt::Debug;

use super::coercion::{AtomicMode, CoercibleInto, CommonCmp, CommonNum, MinimallyNumeric};
use super::iterators::{map_common_numeric, zip_recycle};
use super::subset::Subset;
use super::subsets::Subsets;
use super::types::*;
use super::{OptionNA, Pow, VecPartialCmp};
use crate::error::Error;
use crate::internal_err;
use crate::object::{CowObj, Obj, ViewMut};

/// Vector
#[derive(Debug, PartialEq)]
pub enum RepType<T: Clone> {
    // Vector::Subset encompasses a "raw" vector (no subsetting)
    Subset(CowObj<Vec<T>>, Subsets),
    // Iterator includes things like ranges 1:Inf, and lazily computed values
    // Iter(Box<dyn Iterator<Item = &T>>)
}

impl<T: Clone> Clone for RepType<T> {
    fn clone(&self) -> Self {
        match self {
            RepType::Subset(v, s) => RepType::Subset(v.view_mut(), s.clone()),
        }
    }
}

impl<T: Clone + Default> Default for RepType<T> {
    fn default() -> Self {
        Self::new()
    }
}

impl<T: Clone + Default + ViewMut> RepType<T> {
    /// Retrieve the internal data as a mutable view.
    pub fn get_inner_mut(&self, index: usize) -> Option<T> {
        match self {
            RepType::Subset(v, subsets) => {
                let vb = v.borrow();
                let index = subsets.get_index_at(index).unwrap();
                vb.get(index).map(|i| i.view_mut())
            }
        }
    }
}

impl<T> IntoIterator for RepType<T>
where
    T: Clone + Default,
{
    type Item = T;
    type IntoIter = RepTypeIter<T>;
    fn into_iter(self) -> Self::IntoIter {
        // FIXME: this might materialize
        let n = self.len();
        match self {
            RepType::Subset(..) => RepTypeIter::SubsetIter(self, 0, n),
        }
    }
}

pub enum RepTypeIter<T: Clone> {
    SubsetIter(RepType<T>, usize, usize),
}

impl<T: Clone + Default> Iterator for RepTypeIter<T> {
    type Item = T;
    fn next(&mut self) -> Option<Self::Item> {
        match self {
            RepTypeIter::SubsetIter(rep, i, len) => {
                if i < len {
                    let x = rep.get_inner(*i);
                    *i += 1;
                    x
                } else {
                    None
                }
            }
        }
    }
}

impl<T: Clone> ViewMut for RepType<T> {
    fn view_mut(&self) -> Self {
        match self {
            RepType::Subset(v, s) => RepType::Subset(v.view_mut(), s.clone()),
        }
    }
}

impl<T: Clone + Default> RepType<T> {
    /// Create an empty vector
    ///
    /// The primary use case for this function is to support testing, and there
    /// are few expected use cases outside. It is used for creating a vector
    /// of an explicit atomic type, likely to be tested with
    /// `SameType::is_same_type_as`.
    ///
    /// ```
    /// use r::utils::*;
    /// use r::object::Vector;
    /// use r::object::OptionNA;
    ///
    /// let result = Vector::from(vec![1, 2, 3]);
    /// let expect = Vector::from(Vec::<OptionNA<i32>>::new());
    ///
    /// assert!(result.is_same_type_as(&expect))
    /// ```
    ///
    pub fn new() -> Self {
        RepType::Subset(Vec::new().into(), Subsets(Vec::new()))
    }

    /// Access a lazy copy of the internal vector data
    pub fn inner(&self) -> CowObj<Vec<T>> {
        match self.materialize() {
            RepType::Subset(v, _) => v.clone(),
        }
    }

    /// Try to get mutable access to the internal vector through the passed closure.
    /// This requires the vector to be in materialized form, otherwise None is returned.
    /// None is returned if this is not the case.
    pub fn with_inner_mut<F, R>(&self, f: F) -> R
    where
        F: FnOnce(&mut Vec<T>) -> R,
    {
        match self {
            RepType::Subset(v, Subsets(s)) => v.with_inner_mut(f),
        }
    }

    /// Subsetting a Vector
    ///
    /// Introduce a new subset into the aggregate list of subset indices.
    ///
    pub fn subset(&self, subset: Subset) -> Self {
        match self {
            RepType::Subset(v, Subsets(subsets)) => {
                let mut subsets = subsets.clone();
                subsets.push(subset);
                RepType::Subset(v.view_mut(), Subsets(subsets))
            }
        }
    }

    pub fn len(&self) -> usize {
        match self {
            RepType::Subset(v, Subsets(s)) => match s.as_slice() {
                [] => v.borrow().len(),
                _ => unimplemented!(),
            },
        }
    }
    #[must_use]
    pub fn is_empty(&self) -> bool {
        self.len() == 0
    }

    /// Get a single element from a vector
    ///
    /// Access a single element without materializing a new vector
    ///
    pub fn get(&self, index: usize) -> Option<RepType<T>>
    where
        T: Clone,
    {
        match self {
            RepType::Subset(v, subsets) => {
                let vb = v.borrow();
                let index = subsets.get_index_at(index)?;
                let elem = vb.get(index)?;
                Some(RepType::Subset(vec![elem.clone()].into(), Subsets::new()))
            }
        }
    }

    /// Assignment to Subset Indices
    ///
    /// Assignment to a vector from another. The aggregate subsetted indices
    /// are iterated over while performing the assignment.
    ///
    pub fn assign(&mut self, value: Self) -> Self
    where
        T: Clone + Default,
    {
        match (self, value) {
            (RepType::Subset(lv, ls), RepType::Subset(rv, rs)) => {
                lv.with_inner_mut(|lvb| {
                    let rvc = rv.clone();
                    let rvb = rvc.borrow();

                    let lv_len = lvb.len();
                    let l_indices = ls.clone().into_iter().take_while(|(i, _)| i < &lv_len);
                    let r_indices = rs.clone().into_iter().take_while(|(i, _)| i < &lv_len);

                    for ((_, li), (_, ri)) in l_indices.zip(r_indices) {
                        match (li, ri) {
                            (Some(li), None) => lvb[li] = T::default(),
                            (Some(li), Some(ri)) => lvb[li] = rvb[ri % rvb.len()].clone(),
                            _ => (),
                        }
                    }
                });

                RepType::Subset(lv.clone(), ls.clone())
            }
        }
    }

    /// Materialize a Vector
    ///
    /// Apply subsets and clone values into a new vector.
    pub fn materialize(&self) -> Self
    where
        T: Clone,
    {
        match self {
            RepType::Subset(v, subsets) => {
                // early exit when there is nothing to do
                match subsets {
                    Subsets(s) => match s.as_slice() {
                        [] => return self.clone(),
                        _ => (),
                    },
                }

                let vc = v.clone();
                let vb = vc.borrow();
                let mut res: Vec<T> = vec![];
                let vb_len = vb.len();

                let iter = subsets.clone().into_iter().take_while(|(i, _)| i < &vb_len);
                for (_, i) in iter {
                    match i {
                        Some(i) => res.push(vb[i].clone()),
                        None => res.push(T::default()),
                    }
                }

                RepType::Subset(res.into(), Subsets(vec![]))
            }
        }
    }

    pub fn is_double(&self) -> bool
    where
        T: AtomicMode,
    {
        T::is_double()
    }

    pub fn is_logical(&self) -> bool
    where
        T: AtomicMode,
    {
        T::is_logical()
    }

    pub fn is_integer(&self) -> bool
    where
        T: AtomicMode,
    {
        T::is_integer()
    }

    pub fn is_character(&self) -> bool
    where
        T: AtomicMode,
    {
        T::is_character()
    }

    pub fn as_mode<Mode>(&self) -> RepType<Mode>
    where
        T: CoercibleInto<Mode>,
        Mode: Clone,
    {
        match self {
            RepType::Subset(v, subsets) => {
                let vc = v.clone();
                let vb = vc.borrow();

                let num_vec: Vec<Mode> = vb.iter().map(|i| (*i).clone().coerce_into()).collect();

                RepType::Subset(num_vec.into(), subsets.clone())
            }
        }
    }

    pub fn as_logical(&self) -> RepType<Logical>
    where
        T: CoercibleInto<Logical>,
    {
        self.as_mode::<Logical>()
    }

    pub fn as_integer(&self) -> RepType<Integer>
    where
        T: CoercibleInto<Integer>,
    {
        self.as_mode::<Integer>()
    }

    pub fn as_double(&self) -> RepType<Double>
    where
        T: CoercibleInto<Double>,
    {
        self.as_mode::<Double>()
    }

    pub fn as_character(&self) -> RepType<Character>
    where
        T: CoercibleInto<Character>,
    {
        self.as_mode::<Character>()
    }

    pub fn vectorized_partial_cmp<R, C>(self, other: RepType<R>) -> Vec<Option<std::cmp::Ordering>>
    where
        T: AtomicMode + Default + Clone + CoercibleInto<C>,
        R: AtomicMode + Default + Clone + CoercibleInto<C>,
        (T, R): CommonCmp<Common = C>,
        C: PartialOrd,
    {
        let lc = self.inner().clone();
        let lb = lc.borrow();
        let lhs = lb.iter();

        let rc = other.inner().clone();
        let rb = rc.borrow();
        let rhs = rb.iter();

        zip_recycle(lhs, rhs)
            .map(|(l, r)| {
                let lc = CoercibleInto::<C>::coerce_into(l.clone());
                let rc = CoercibleInto::<C>::coerce_into(r.clone());
                lc.partial_cmp(&rc)
            })
            .collect()
    }

    pub fn get_inner(&self, index: usize) -> Option<T> {
        match self {
            RepType::Subset(v, subsets) => {
                let vb = v.borrow();
                let index = subsets.get_index_at(index)?;
                vb.get(index).cloned()
            }
        }
    }
}

impl<T> TryInto<bool> for RepType<OptionNA<T>>
where
    OptionNA<T>: AtomicMode + Clone + CoercibleInto<OptionNA<bool>>,
{
    type Error = ();
    fn try_into(self) -> Result<bool, Self::Error> {
        self.get_inner(0).map_or(
            Err(()),
            |i| match CoercibleInto::<OptionNA<bool>>::coerce_into(i) {
                OptionNA::Some(x) => Ok(x),
                OptionNA::NA => Err(()),
            },
        )
    }
}

impl From<Vec<(Option<String>, Obj)>> for RepType<(Option<String>, Obj)> {
    fn from(value: Vec<(Option<String>, Obj)>) -> Self {
        RepType::Subset(value.into(), Subsets::default())
    }
}

impl From<Vec<OptionNA<f64>>> for RepType<Double> {
    fn from(value: Vec<OptionNA<f64>>) -> Self {
        let value: Vec<_> = value.into_iter().map(|i| i.coerce_into()).collect();
        RepType::Subset(value.into(), Subsets(Vec::new()))
    }
}

impl From<Vec<f64>> for RepType<Double> {
    fn from(value: Vec<f64>) -> Self {
        let value: Vec<_> = value.into_iter().map(|i| i.coerce_into()).collect();
        RepType::Subset(value.into(), Subsets(Vec::new()))
    }
}

impl From<Vec<OptionNA<i32>>> for RepType<Integer> {
    fn from(value: Vec<OptionNA<i32>>) -> Self {
        let value: Vec<_> = value.into_iter().map(|i| i.coerce_into()).collect();
        RepType::Subset(value.into(), Subsets(Vec::new()))
    }
}

impl From<Vec<i32>> for RepType<Integer> {
    fn from(value: Vec<i32>) -> Self {
        let value: Vec<_> = value.into_iter().map(|i| i.coerce_into()).collect();
        RepType::Subset(value.into(), Subsets(Vec::new()))
    }
}

impl From<Vec<OptionNA<bool>>> for RepType<Logical> {
    fn from(value: Vec<OptionNA<bool>>) -> Self {
        let value: Vec<_> = value.into_iter().map(|i| i.coerce_into()).collect();
        RepType::Subset(value.into(), Subsets(Vec::new()))
    }
}

impl From<Vec<bool>> for RepType<Logical> {
    fn from(value: Vec<bool>) -> Self {
        let value: Vec<_> = value.into_iter().map(|i| i.coerce_into()).collect();
        RepType::Subset(value.into(), Subsets(Vec::new()))
    }
}

impl From<Vec<OptionNA<String>>> for RepType<Character> {
    fn from(value: Vec<OptionNA<String>>) -> Self {
        let value: Vec<_> = value.into_iter().map(|i| i.coerce_into()).collect();
        RepType::Subset(value.into(), Subsets(Vec::new()))
    }
}

impl From<Vec<String>> for RepType<Character> {
    fn from(value: Vec<String>) -> Self {
        let value: Vec<_> = value.into_iter().map(|i| i.coerce_into()).collect();
        RepType::Subset(value.into(), Subsets(Vec::new()))
    }
}

impl<F, T> From<(Vec<F>, Subsets)> for RepType<T>
where
    RepType<T>: From<Vec<F>>,
    T: Clone,
{
    fn from(value: (Vec<F>, Subsets)) -> Self {
        match Self::from(value.0) {
            RepType::Subset(v, _) => RepType::Subset(v, value.1),
        }
    }
}

impl<L, LNum, O> std::ops::Neg for RepType<L>
where
    L: AtomicMode + Default + Clone + MinimallyNumeric<As = LNum> + CoercibleInto<LNum>,
    LNum: std::ops::Neg<Output = O>,
    RepType<O>: From<Vec<O>>,
    O: Clone,
{
    type Output = RepType<O>;
    fn neg(self) -> Self::Output {
        RepType::from(
            self.inner()
                .iter()
                .map(|l| CoercibleInto::<LNum>::coerce_into(l.clone()).neg())
                .collect::<Vec<O>>(),
        )
    }
}

impl<L, R, C, O, LNum, RNum> std::ops::Add<RepType<R>> for RepType<L>
where
    L: AtomicMode + Default + Clone + MinimallyNumeric<As = LNum> + CoercibleInto<LNum>,
    R: AtomicMode + Default + Clone + MinimallyNumeric<As = RNum> + CoercibleInto<RNum>,
    (LNum, RNum): CommonNum<Common = C>,
    C: Clone + std::ops::Add<Output = O>,
    RepType<C>: From<Vec<O>>,
{
    type Output = RepType<C>;
    fn add(self, rhs: RepType<R>) -> Self::Output {
        let lc = self.inner().clone();
        let lb = lc.borrow();
        let lhs = lb.iter();

        let rc = rhs.inner().clone();
        let rb = rc.borrow();
        let rhs = rb.iter();

        RepType::from(
            map_common_numeric(zip_recycle(lhs, rhs))
                .map(|(l, r)| l + r)
                .collect::<Vec<O>>(),
        )
    }
}

impl<L, R, C, O, LNum, RNum> std::ops::Sub<RepType<R>> for RepType<L>
where
    L: AtomicMode + Default + Clone + MinimallyNumeric<As = LNum> + CoercibleInto<LNum>,
    R: AtomicMode + Default + Clone + MinimallyNumeric<As = RNum> + CoercibleInto<RNum>,
    (LNum, RNum): CommonNum<Common = C>,
    C: std::ops::Sub<Output = O> + Clone,
    RepType<C>: From<Vec<O>>,
    O: Clone,
{
    type Output = RepType<C>;
    fn sub(self, rhs: RepType<R>) -> Self::Output {
        let lc = self.inner().clone();
        let lb = lc.borrow();
        let lhs = lb.iter();

        let rc = rhs.inner().clone();
        let rb = rc.borrow();
        let rhs = rb.iter();

        RepType::from(
            map_common_numeric(zip_recycle(lhs, rhs))
                .map(|(l, r)| l - r)
                .collect::<Vec<O>>(),
        )
    }
}

impl<L, R, C, O, LNum, RNum> std::ops::Mul<RepType<R>> for RepType<L>
where
    L: AtomicMode + Default + Clone + MinimallyNumeric<As = LNum> + CoercibleInto<LNum>,
    R: AtomicMode + Default + Clone + MinimallyNumeric<As = RNum> + CoercibleInto<RNum>,
    (LNum, RNum): CommonNum<Common = C>,
    C: std::ops::Mul<Output = O> + Clone,
    RepType<C>: From<Vec<O>>,
{
    type Output = RepType<C>;
    fn mul(self, rhs: RepType<R>) -> Self::Output {
        let lc = self.inner().clone();
        let lb = lc.borrow();
        let lhs = lb.iter();

        let rc = rhs.inner().clone();
        let rb = rc.borrow();
        let rhs = rb.iter();

        RepType::from(
            map_common_numeric(zip_recycle(lhs, rhs))
                .map(|(l, r)| l * r)
                .collect::<Vec<O>>(),
        )
    }
}

impl<L, R, C, O, LNum, RNum> std::ops::Div<RepType<R>> for RepType<L>
where
    L: AtomicMode + Default + Clone + MinimallyNumeric<As = LNum> + CoercibleInto<LNum>,
    R: AtomicMode + Default + Clone + MinimallyNumeric<As = RNum> + CoercibleInto<RNum>,
    (LNum, RNum): CommonNum<Common = C>,
    C: std::ops::Div<Output = O> + Clone,
    RepType<C>: From<Vec<O>>,
    O: Clone,
{
    type Output = RepType<C>;
    fn div(self, rhs: RepType<R>) -> Self::Output {
        let lc = self.inner().clone();
        let lb = lc.borrow();
        let lhs = lb.iter();

        let rc = rhs.inner().clone();
        let rb = rc.borrow();
        let rhs = rb.iter();

        RepType::from(
            map_common_numeric(zip_recycle(lhs, rhs))
                .map(|(l, r)| l / r)
                .collect::<Vec<O>>(),
        )
    }
}

impl<L, R, C, O, LNum, RNum> std::ops::Rem<RepType<R>> for RepType<L>
where
    L: AtomicMode + Default + Clone + MinimallyNumeric<As = LNum> + CoercibleInto<LNum>,
    R: AtomicMode + Default + Clone + MinimallyNumeric<As = RNum> + CoercibleInto<RNum>,
    (LNum, RNum): CommonNum<Common = C>,
    C: std::ops::Rem<Output = O> + Clone,
    O: Clone,
    RepType<C>: From<Vec<O>>,
{
    type Output = RepType<C>;
    fn rem(self, rhs: RepType<R>) -> Self::Output {
        let lc = self.inner().clone();
        let lb = lc.borrow();
        let lhs = lb.iter();

        let rc = rhs.inner().clone();
        let rb = rc.borrow();
        let rhs = rb.iter();

        RepType::from(
            map_common_numeric(zip_recycle(lhs.into_iter(), rhs.into_iter()))
                .map(|(l, r)| l.rem(r))
                .collect::<Vec<O>>(),
        )
    }
}

impl<L, R, O, LNum, RNum> Pow<RepType<R>> for RepType<L>
where
    L: AtomicMode + Default + Clone + MinimallyNumeric<As = LNum> + CoercibleInto<LNum>,
    R: AtomicMode + Default + Clone + MinimallyNumeric<As = RNum> + CoercibleInto<RNum>,
    LNum: Pow<RNum, Output = O>,
    RepType<O>: From<Vec<O>>,
    O: Clone,
{
    type Output = RepType<O>;
    fn power(self, rhs: RepType<R>) -> Self::Output {
        let lc = self.inner().clone();
        let lb = lc.borrow();
        let lhs = lb.iter();

        let rc = rhs.inner().clone();
        let rb = rc.borrow();
        let rhs = rb.iter();

        RepType::from(
            zip_recycle(lhs, rhs)
                .map(|(l, r)| l.clone().coerce_into().power(r.clone().coerce_into()))
                .collect::<Vec<O>>(),
        )
    }
}

impl<L, R, O> std::ops::BitOr<RepType<R>> for RepType<L>
where
    L: AtomicMode + Default + Clone + CoercibleInto<Logical>,
    R: AtomicMode + Default + Clone + CoercibleInto<Logical>,
    Logical: std::ops::BitOr<Logical, Output = O>,
    RepType<O>: From<Vec<O>>,
    O: Clone,
{
    type Output = RepType<O>;
    fn bitor(self, rhs: RepType<R>) -> Self::Output {
        let lc = self.inner().clone();
        let lb = lc.borrow();
        let lhs = lb.iter();

        let rc = rhs.inner().clone();
        let rb = rc.borrow();
        let rhs = rb.iter();

        RepType::from(
            zip_recycle(lhs, rhs)
                .map(|(l, r)| l.clone().coerce_into().bitor(r.clone().coerce_into()))
                .collect::<Vec<O>>(),
        )
    }
}

impl<L, R, O> std::ops::BitAnd<RepType<R>> for RepType<L>
where
    L: AtomicMode + Default + Clone + CoercibleInto<Logical>,
    R: AtomicMode + Default + Clone + CoercibleInto<Logical>,
    Logical: std::ops::BitAnd<Logical, Output = O>,
    RepType<O>: From<Vec<O>>,
    O: Clone,
{
    type Output = RepType<O>;
    fn bitand(self, rhs: RepType<R>) -> Self::Output {
        let lc = self.inner().clone();
        let lb = lc.borrow();
        let lhs = lb.iter();

        let rc = rhs.inner().clone();
        let rb = rc.borrow();
        let rhs = rb.iter();

        RepType::from(
            zip_recycle(lhs, rhs)
                .map(|(l, r)| l.clone().coerce_into().bitand(r.clone().coerce_into()))
                .collect::<Vec<O>>(),
        )
    }
}

impl<L, O> std::ops::Not for RepType<L>
where
    L: AtomicMode + Default + Clone + CoercibleInto<Logical>,
    Logical: std::ops::Not<Output = O>,
    RepType<O>: From<Vec<O>>,
    O: Clone,
{
    type Output = RepType<O>;
    fn not(self) -> Self::Output {
        RepType::from(
            self.inner()
                .iter()
                .map(|l| CoercibleInto::<Logical>::coerce_into(l.clone()).not())
                .collect::<Vec<O>>(),
        )
    }
}

impl<L, R, C> VecPartialCmp<RepType<R>> for RepType<L>
where
    L: AtomicMode + Default + Clone + CoercibleInto<C>,
    R: AtomicMode + Default + Clone + CoercibleInto<C>,
    (L, R): CommonCmp<Common = C>,
    C: PartialOrd,
{
    type Output = RepType<Logical>;

    fn vec_gt(self, rhs: RepType<R>) -> Self::Output {
        use std::cmp::Ordering::*;
        self.vectorized_partial_cmp(rhs)
            .into_iter()
            .map(|i| match i {
                Some(Greater) => OptionNA::Some(true),
                Some(_) => OptionNA::Some(false),
                None => OptionNA::NA,
            })
            .collect::<Vec<Logical>>()
            .into()
    }

    fn vec_gte(self, rhs: RepType<R>) -> Self::Output {
        use std::cmp::Ordering::*;
        self.vectorized_partial_cmp(rhs)
            .into_iter()
            .map(|i| match i {
                Some(Greater | Equal) => OptionNA::Some(true),
                Some(_) => OptionNA::Some(false),
                None => OptionNA::NA,
            })
            .collect::<Vec<Logical>>()
            .into()
    }

    fn vec_lt(self, rhs: RepType<R>) -> Self::Output {
        use std::cmp::Ordering::*;
        self.vectorized_partial_cmp(rhs)
            .into_iter()
            .map(|i| match i {
                Some(Less) => OptionNA::Some(true),
                Some(_) => OptionNA::Some(false),
                None => OptionNA::NA,
            })
            .collect::<Vec<Logical>>()
            .into()
    }

    fn vec_lte(self, rhs: RepType<R>) -> Self::Output {
        use std::cmp::Ordering::*;
        self.vectorized_partial_cmp(rhs)
            .into_iter()
            .map(|i| match i {
                Some(Less | Equal) => OptionNA::Some(true),
                Some(_) => OptionNA::Some(false),
                None => OptionNA::NA,
            })
            .collect::<Vec<Logical>>()
            .into()
    }

    fn vec_eq(self, rhs: RepType<R>) -> Self::Output {
        use std::cmp::Ordering::*;
        self.vectorized_partial_cmp(rhs)
            .into_iter()
            .map(|i| match i {
                Some(Equal) => OptionNA::Some(true),
                Some(_) => OptionNA::Some(false),
                None => OptionNA::NA,
            })
            .collect::<Vec<Logical>>()
            .into()
    }

    fn vec_neq(self, rhs: RepType<R>) -> Self::Output {
        use std::cmp::Ordering::*;
        self.vectorized_partial_cmp(rhs)
            .into_iter()
            .map(|i| match i {
                Some(Equal) => OptionNA::Some(false),
                Some(_) => OptionNA::Some(true),
                None => OptionNA::NA,
            })
            .collect::<Vec<Logical>>()
            .into()
    }
}

#[cfg(test)]
mod test {
    use super::OptionNA::*;
    use crate::object::reptype::RepType;
    use crate::object::{types::*, OptionNA, VecPartialCmp};
    use crate::utils::SameType;

    #[test]
    fn vector_add() {
        let x = RepType::from((1..=10).collect::<Vec<_>>());
        let y = RepType::from(vec![2, 5, 6, 2, 3]);

        let z = x + y;
        assert_eq!(z, RepType::from(vec![3, 7, 9, 6, 8, 8, 12, 14, 11, 13]));

        let expected_type = RepType::<Integer>::new();
        assert!(z.is_same_type_as(&expected_type));
        assert!(z.is_integer());
    }

    #[test]
    fn vector_mul() {
        let x = RepType::from((1..=10).collect::<Vec<_>>());
        let y = RepType::from(vec![Some(2), NA, Some(6), NA, Some(3)]);

        let z = x * y;
        assert_eq!(
            z,
            RepType::from(vec![
                Some(2),
                NA,
                Some(18),
                NA,
                Some(15),
                Some(12),
                NA,
                Some(48),
                NA,
                Some(30)
            ])
        );

        let expected_type = RepType::<Integer>::new();
        assert!(z.is_same_type_as(&expected_type));
        assert!(z.is_integer());
    }

    #[test]
    fn vector_common_mul_f32_na() {
        // expect that f32's do not get coerced into an OptionNA:: instead
        // using std::f32::NAN as NA representation.

        let x = RepType::from(vec![Some(0_f64), NA, Some(10_f64)]);
        let y = RepType::from(vec![100, 10]);

        let z = x * y;
        // assert_eq!(z, Vector::from(vec![0_f32, std::f32::NAN, 1_000_f32]));
        // comparing floats is error prone

        let expected_type = RepType::<Double>::new();
        assert!(z.is_same_type_as(&expected_type));
        assert!(z.is_double());
    }

    #[test]
    fn vector_and() {
        // expect that f32's do not get coerced into an OptionNA:: instead
        // using std::f32::NAN as NA representation.

        let x = RepType::from(vec![Some(0_f64), NA, Some(10_f64)]);
        let y = RepType::from(vec![100, 10]);

        let z = x & y;
        assert_eq!(z, RepType::from(vec![Some(false), NA, Some(true)]));

        let expected_type = RepType::<Logical>::new();
        assert!(z.is_same_type_as(&expected_type));
        assert!(z.is_logical());
    }

    #[test]
    fn vector_gt() {
        // expect that f32's do not get coerced into an  instead
        // using std::f32::NAN as NA representation.

        let x = RepType::from(vec![Some(0_f64), NA, Some(10000_f64)]);
        let y = RepType::from(vec![100, 10]);

        let z = x.vec_gt(y);
        assert_eq!(z, RepType::from(vec![Some(false), NA, Some(true)]));

        let expected_type = RepType::<Logical>::new();
        assert!(z.is_same_type_as(&expected_type));
        assert!(z.is_logical());
    }

    #[test]
    fn iter() {
        let x = RepType::from(vec![Some(1), Some(2)]);
        let mut xi = x.into_iter();
        assert_eq!(xi.next(), Option::Some(OptionNA::Some(1)));
        assert_eq!(xi.next(), Option::Some(OptionNA::Some(2)));
        assert_eq!(xi.next(), Option::None);
        let xs = RepType::from(vec![Some("a".to_string())]);
        let mut xsi = xs.into_iter();
        assert_eq!(xsi.next(), Option::Some(OptionNA::Some("a".to_string())));
        assert_eq!(xsi.next(), Option::None);
    }
}
