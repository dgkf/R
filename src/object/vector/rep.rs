use std::cell::RefCell;
use std::fmt::{Debug, Display};
use std::rc::Rc;

use super::coercion::{AtomicMode, CoercibleInto, CommonCmp, CommonNum, MinimallyNumeric};
use super::iterators::{map_common_numeric, zip_recycle};
use super::subset::Subset;
use super::subsets::Subsets;
use super::types::*;
use super::{OptionNA, Pow, VecPartialCmp};

/// Vector
#[derive(Debug, Clone, PartialEq)]
pub enum Rep<T> {
    // Vector::Subset encompasses a "raw" vector (no subsetting)
    Subset(Rc<RefCell<Vec<T>>>, Subsets),
    // Iterator includes things like ranges 1:Inf, and lazily computed values
    // Iter(Box<dyn Iterator<Item = &T>>)
}

impl<T: AtomicMode + Clone + Default> Rep<T> {
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
        Rep::Subset(Rc::new(RefCell::new(Vec::new())), Subsets(Vec::new()))
    }

    /// Access the internal vector
    pub fn inner(&self) -> Rc<RefCell<Vec<T>>> {
        match self.materialize() {
            Rep::Subset(v, _) => v.clone(),
        }
    }

    /// Subsetting a Vector
    ///
    /// Introduce a new subset into the aggregate list of subset indices.
    ///
    pub fn subset(&self, subset: Subset) -> Self {
        match self {
            Rep::Subset(v, Subsets(subsets)) => {
                let mut subsets = subsets.clone();
                subsets.push(subset.into());
                Rep::Subset(v.clone(), Subsets(subsets))
            }
        }
    }

    pub fn len(&self) -> usize {
        match self {
            Rep::Subset(v, Subsets(s)) => match s.as_slice() {
                [] => v.clone().borrow().len(),
                [.., last] => std::cmp::min(v.clone().borrow().len(), last.len()),
            },
        }
    }

    /// Get a single element from a vector
    ///
    /// Access a single element without materializing a new vector
    ///
    pub fn get(&self, index: usize) -> Option<Rep<T>>
    where
        T: Clone,
    {
        match self {
            Rep::Subset(v, subsets) => {
                let vc = v.clone();
                let vb = vc.borrow();
                let index = subsets.get_index_at(index)?;
                let elem = vb.get(index)?;
                Some(Rep::Subset(
                    Rc::new(RefCell::new(vec![elem.clone()])),
                    Subsets::new(),
                ))
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
            (Rep::Subset(lv, ls), Rep::Subset(rv, rs)) => {
                let lvc = lv.clone();
                let mut lvb = lvc.borrow_mut();
                let rvc = rv.clone();
                let rvb = rvc.borrow();

                let lv_len = lvb.len().clone();
                let l_indices = ls.clone().into_iter().take_while(|(i, _)| i < &lv_len);
                let r_indices = rs.clone().into_iter().take_while(|(i, _)| i < &lv_len);

                for ((_, li), (_, ri)) in l_indices.zip(r_indices) {
                    match (li, ri) {
                        (Some(li), None) => lvb[li] = T::default(),
                        (Some(li), Some(ri)) => lvb[li] = rvb[ri % rvb.len()].clone(),
                        _ => (),
                    }
                }

                Rep::Subset(lvc.clone(), ls.clone())
            }
        }
    }

    /// Materialize a Vector
    ///
    /// Apply subsets and clone values into a new vector.
    ///
    pub fn materialize(&self) -> Self
    where
        T: Clone,
    {
        match self {
            Rep::Subset(v, subsets) => {
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

                Rep::Subset(Rc::new(RefCell::new(res)), Subsets(vec![]))
            }
        }
    }

    /// Test the mode of the internal vector type
    ///
    /// Internally, this is defined by the [r::vector::vectors::AtomicMode]
    /// implementation of the vector's element type.
    ///
    pub fn is_numeric(&self) -> bool {
        T::is_numeric()
    }

    /// See [Self::is_numeric] for more information
    pub fn is_logical(&self) -> bool {
        T::is_logical()
    }

    /// See [Self::is_numeric] for more information
    pub fn is_integer(&self) -> bool {
        T::is_integer()
    }

    /// See [Self::is_numeric] for more information
    pub fn is_character(&self) -> bool {
        T::is_character()
    }

    /// Convert a Vector into a vector of a specific class of internal type
    ///
    /// The internal type only needs to satisfy [CoerceInto] for the `Mode`,
    /// and for the `Mode` type to implement [Atomic]. Generally, this
    /// is used more directly via [Self::as_logical], [Self::as_integer],
    /// [Self::as_numeric] and [Self::as_character], which predefine the output
    /// type of the mode.
    ///
    /// ```
    /// use r::object::Vector;
    /// use r::object::OptionNA;
    ///
    /// let x = Vector::from(vec![false, true, true, false]);
    /// let n = x.as_numeric();
    ///
    /// assert_eq!(n, Vector::from(vec![
    ///    OptionNA::Some(0_f64),
    ///    OptionNA::Some(1_f64),
    ///    OptionNA::Some(1_f64),
    ///    OptionNA::Some(0_f64)
    /// ]))
    /// ```
    ///
    pub fn as_mode<Mode>(&self) -> Rep<Mode>
    where
        T: CoercibleInto<Mode>,
    {
        match self {
            Rep::Subset(v, subsets) => {
                let vc = v.clone();
                let vb = vc.borrow();

                let num_vec: Vec<Mode> = vb.iter().map(|i| (*i).clone().coerce_into()).collect();

                Rep::Subset(Rc::new(RefCell::new(num_vec)), subsets.clone())
            }
        }
    }

    /// See [Self::as_mode] for more information
    pub fn as_logical(&self) -> Rep<Logical>
    where
        T: CoercibleInto<Logical>,
    {
        self.as_mode::<Logical>()
    }

    /// See [Self::as_mode] for more information
    pub fn as_integer(&self) -> Rep<Integer>
    where
        T: CoercibleInto<Integer>,
    {
        self.as_mode::<Integer>()
    }

    /// See [Self::as_mode] for more information
    pub fn as_numeric(&self) -> Rep<Numeric>
    where
        T: CoercibleInto<Numeric>,
    {
        self.as_mode::<Numeric>()
    }

    /// See [Self::as_mode] for more information
    pub fn as_character(&self) -> Rep<Character>
    where
        T: CoercibleInto<Character>,
    {
        self.as_mode::<Character>()
    }

    /// Apply over the vector contents to produce a vector of [std::cmp::Ordering]
    ///
    /// This function is used primarily in support of the implementation of
    /// vectorized comparison operators and likely does not need to be used
    /// outside of that context.
    ///
    /// See [r::vecops::VecPartialCmp] for vectorized comparison operator
    /// implementations.
    ///
    pub fn vectorized_partial_cmp<R, C>(self, other: Rep<R>) -> Vec<Option<std::cmp::Ordering>>
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

    fn get_inner(&self, index: usize) -> Option<T> {
        match self {
            Rep::Subset(v, subsets) => {
                let vc = v.clone();
                let vb = vc.borrow();
                let index = subsets.get_index_at(index)?;
                match vb.get(index) {
                    Some(x) => Some(x.clone()),
                    None => None,
                }
            }
        }
    }
}

impl<T> TryInto<bool> for Rep<OptionNA<T>>
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

impl From<Vec<OptionNA<f64>>> for Rep<Numeric> {
    fn from(value: Vec<OptionNA<f64>>) -> Self {
        let value: Vec<_> = value.into_iter().map(|i| i.coerce_into()).collect();
        Rep::Subset(Rc::new(RefCell::new(value)), Subsets(Vec::new()))
    }
}

impl From<Vec<f64>> for Rep<Numeric> {
    fn from(value: Vec<f64>) -> Self {
        let value: Vec<_> = value.into_iter().map(|i| i.coerce_into()).collect();
        Rep::Subset(Rc::new(RefCell::new(value)), Subsets(Vec::new()))
    }
}

impl From<Vec<OptionNA<i32>>> for Rep<Integer> {
    fn from(value: Vec<OptionNA<i32>>) -> Self {
        let value: Vec<_> = value.into_iter().map(|i| i.coerce_into()).collect();
        Rep::Subset(Rc::new(RefCell::new(value)), Subsets(Vec::new()))
    }
}

impl From<Vec<i32>> for Rep<Integer> {
    fn from(value: Vec<i32>) -> Self {
        let value: Vec<_> = value.into_iter().map(|i| i.coerce_into()).collect();
        Rep::Subset(Rc::new(RefCell::new(value)), Subsets(Vec::new()))
    }
}

impl From<Vec<OptionNA<bool>>> for Rep<Logical> {
    fn from(value: Vec<OptionNA<bool>>) -> Self {
        let value: Vec<_> = value.into_iter().map(|i| i.coerce_into()).collect();
        Rep::Subset(Rc::new(RefCell::new(value)), Subsets(Vec::new()))
    }
}

impl From<Vec<bool>> for Rep<Logical> {
    fn from(value: Vec<bool>) -> Self {
        let value: Vec<_> = value.into_iter().map(|i| i.coerce_into()).collect();
        Rep::Subset(Rc::new(RefCell::new(value)), Subsets(Vec::new()))
    }
}

impl From<Vec<OptionNA<String>>> for Rep<Character> {
    fn from(value: Vec<OptionNA<String>>) -> Self {
        let value: Vec<_> = value.into_iter().map(|i| i.coerce_into()).collect();
        Rep::Subset(Rc::new(RefCell::new(value)), Subsets(Vec::new()))
    }
}

impl From<Vec<String>> for Rep<Character> {
    fn from(value: Vec<String>) -> Self {
        let value: Vec<_> = value.into_iter().map(|i| i.coerce_into()).collect();
        Rep::Subset(Rc::new(RefCell::new(value)), Subsets(Vec::new()))
    }
}

impl<F, T> From<(Vec<F>, Subsets)> for Rep<T>
where
    Rep<T>: From<Vec<F>>,
{
    fn from(value: (Vec<F>, Subsets)) -> Self {
        match Self::from(value.0) {
            Rep::Subset(v, _) => Rep::Subset(v, value.1),
        }
    }
}

impl<T> Display for Rep<T>
where
    T: AtomicMode + Debug + Default + Clone,
{
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let n = self.len();
        if n == 0 {
            if self.is_numeric() {
                return write!(f, "numeric(0)");
            }
            if self.is_integer() {
                return write!(f, "integer(0)");
            }
            if self.is_logical() {
                return write!(f, "logical(0)");
            }
            if self.is_character() {
                return write!(f, "character(0)");
            }
        }

        let nlen = format!("{}", n).len();
        // TODO: iteratively calculate when we hit max print so our
        // max_len isn't inflated by a value that is omitted

        let xc = self.inner().clone();
        let xb = xc.borrow();

        let x_strs = xb.iter().map(|xi| format!("{:?}", xi));
        let max_len = x_strs
            .clone()
            .fold(0, |max_len, xi| std::cmp::max(max_len, xi.len()));

        let mut col = 0;
        let gutterlen = 2 + nlen + 1;

        // hard coded max print & console width
        let maxprint = 20 * ((80 - gutterlen) / max_len);

        x_strs
            .take(maxprint)
            .enumerate()
            .try_for_each(|(i, x_str)| {
                if i == 0 {
                    col = gutterlen + max_len;
                    write!(f, "{:>3$}[{}] {:>4$}", "", i + 1, x_str, nlen - 1, max_len)
                } else if col + 1 + max_len > 80 {
                    col = gutterlen + max_len;
                    let i_str = format!("{}", i + 1);
                    let gutter = nlen - i_str.len();
                    write!(f, "\n{:>3$}[{}] {:>4$}", "", i_str, x_str, gutter, max_len)
                } else {
                    col += 1 + max_len;
                    write!(f, " {:>1$}", x_str, max_len)
                }
            })?;

        if n > maxprint {
            write!(f, "\n[ omitting {} entries ]", n - maxprint)?;
        }

        Ok(())
    }
}

impl<L, LNum, O> std::ops::Neg for Rep<L>
where
    L: AtomicMode + Default + Clone + MinimallyNumeric<As = LNum> + CoercibleInto<LNum>,
    LNum: std::ops::Neg<Output = O>,
    Rep<O>: From<Vec<O>>,
{
    type Output = Rep<O>;
    fn neg(self) -> Self::Output {
        Rep::from(
            self.inner()
                .clone()
                .borrow()
                .iter()
                .map(|l| CoercibleInto::<LNum>::coerce_into(l.clone()).neg())
                .collect::<Vec<O>>(),
        )
    }
}

impl<L, R, C, O, LNum, RNum> std::ops::Add<Rep<R>> for Rep<L>
where
    L: AtomicMode + Default + Clone + MinimallyNumeric<As = LNum> + CoercibleInto<LNum>,
    R: AtomicMode + Default + Clone + MinimallyNumeric<As = RNum> + CoercibleInto<RNum>,
    (LNum, RNum): CommonNum<Common = C>,
    C: Clone + std::ops::Add<Output = O>,
    Rep<C>: From<Vec<O>>,
{
    type Output = Rep<C>;
    fn add(self, rhs: Rep<R>) -> Self::Output {
        let lc = self.inner().clone();
        let lb = lc.borrow();
        let lhs = lb.iter();

        let rc = rhs.inner().clone();
        let rb = rc.borrow();
        let rhs = rb.iter();

        Rep::from(
            map_common_numeric(zip_recycle(lhs, rhs))
                .map(|(l, r)| l + r)
                .collect::<Vec<O>>(),
        )
    }
}

impl<L, R, C, O, LNum, RNum> std::ops::Sub<Rep<R>> for Rep<L>
where
    L: AtomicMode + Default + Clone + MinimallyNumeric<As = LNum> + CoercibleInto<LNum>,
    R: AtomicMode + Default + Clone + MinimallyNumeric<As = RNum> + CoercibleInto<RNum>,
    (LNum, RNum): CommonNum<Common = C>,
    C: std::ops::Sub<Output = O>,
    Rep<C>: From<Vec<O>>,
{
    type Output = Rep<C>;
    fn sub(self, rhs: Rep<R>) -> Self::Output {
        let lc = self.inner().clone();
        let lb = lc.borrow();
        let lhs = lb.iter();

        let rc = rhs.inner().clone();
        let rb = rc.borrow();
        let rhs = rb.iter();

        Rep::from(
            map_common_numeric(zip_recycle(lhs, rhs))
                .map(|(l, r)| l - r)
                .collect::<Vec<O>>(),
        )
    }
}

impl<L, R, C, O, LNum, RNum> std::ops::Mul<Rep<R>> for Rep<L>
where
    L: AtomicMode + Default + Clone + MinimallyNumeric<As = LNum> + CoercibleInto<LNum>,
    R: AtomicMode + Default + Clone + MinimallyNumeric<As = RNum> + CoercibleInto<RNum>,
    (LNum, RNum): CommonNum<Common = C>,
    C: std::ops::Mul<Output = O>,
    Rep<C>: From<Vec<O>>,
{
    type Output = Rep<C>;
    fn mul(self, rhs: Rep<R>) -> Self::Output {
        let lc = self.inner().clone();
        let lb = lc.borrow();
        let lhs = lb.iter();

        let rc = rhs.inner().clone();
        let rb = rc.borrow();
        let rhs = rb.iter();

        Rep::from(
            map_common_numeric(zip_recycle(lhs, rhs))
                .map(|(l, r)| l * r)
                .collect::<Vec<O>>(),
        )
    }
}

impl<L, R, C, O, LNum, RNum> std::ops::Div<Rep<R>> for Rep<L>
where
    L: AtomicMode + Default + Clone + MinimallyNumeric<As = LNum> + CoercibleInto<LNum>,
    R: AtomicMode + Default + Clone + MinimallyNumeric<As = RNum> + CoercibleInto<RNum>,
    (LNum, RNum): CommonNum<Common = C>,
    C: std::ops::Div<Output = O>,
    Rep<C>: From<Vec<O>>,
{
    type Output = Rep<C>;
    fn div(self, rhs: Rep<R>) -> Self::Output {
        let lc = self.inner().clone();
        let lb = lc.borrow();
        let lhs = lb.iter();

        let rc = rhs.inner().clone();
        let rb = rc.borrow();
        let rhs = rb.iter();

        Rep::from(
            map_common_numeric(zip_recycle(lhs, rhs))
                .map(|(l, r)| l / r)
                .collect::<Vec<O>>(),
        )
    }
}

impl<L, R, C, O, LNum, RNum> std::ops::Rem<Rep<R>> for Rep<L>
where
    L: AtomicMode + Default + Clone + MinimallyNumeric<As = LNum> + CoercibleInto<LNum>,
    R: AtomicMode + Default + Clone + MinimallyNumeric<As = RNum> + CoercibleInto<RNum>,
    (LNum, RNum): CommonNum<Common = C>,
    C: std::ops::Rem<Output = O>,
    Rep<C>: From<Vec<O>>,
{
    type Output = Rep<C>;
    fn rem(self, rhs: Rep<R>) -> Self::Output {
        let lc = self.inner().clone();
        let lb = lc.borrow();
        let lhs = lb.iter();

        let rc = rhs.inner().clone();
        let rb = rc.borrow();
        let rhs = rb.iter();

        Rep::from(
            map_common_numeric(zip_recycle(lhs.into_iter(), rhs.into_iter()))
                .map(|(l, r)| l.rem(r))
                .collect::<Vec<O>>(),
        )
    }
}

impl<L, R, O, LNum, RNum> Pow<Rep<R>> for Rep<L>
where
    L: AtomicMode + Default + Clone + MinimallyNumeric<As = LNum> + CoercibleInto<LNum>,
    R: AtomicMode + Default + Clone + MinimallyNumeric<As = RNum> + CoercibleInto<RNum>,
    LNum: Pow<RNum, Output = O>,
    Rep<O>: From<Vec<O>>,
{
    type Output = Rep<O>;
    fn power(self, rhs: Rep<R>) -> Self::Output {
        let lc = self.inner().clone();
        let lb = lc.borrow();
        let lhs = lb.iter();

        let rc = rhs.inner().clone();
        let rb = rc.borrow();
        let rhs = rb.iter();

        Rep::from(
            zip_recycle(lhs, rhs)
                .map(|(l, r)| l.clone().coerce_into().power(r.clone().coerce_into()))
                .collect::<Vec<O>>(),
        )
    }
}

impl<L, R, O> std::ops::BitOr<Rep<R>> for Rep<L>
where
    L: AtomicMode + Default + Clone + CoercibleInto<Logical>,
    R: AtomicMode + Default + Clone + CoercibleInto<Logical>,
    Logical: std::ops::BitOr<Logical, Output = O>,
    Rep<O>: From<Vec<O>>,
{
    type Output = Rep<O>;
    fn bitor(self, rhs: Rep<R>) -> Self::Output {
        let lc = self.inner().clone();
        let lb = lc.borrow();
        let lhs = lb.iter();

        let rc = rhs.inner().clone();
        let rb = rc.borrow();
        let rhs = rb.iter();

        Rep::from(
            zip_recycle(lhs, rhs)
                .map(|(l, r)| l.clone().coerce_into().bitor(r.clone().coerce_into()))
                .collect::<Vec<O>>(),
        )
    }
}

impl<L, R, O> std::ops::BitAnd<Rep<R>> for Rep<L>
where
    L: AtomicMode + Default + Clone + CoercibleInto<Logical>,
    R: AtomicMode + Default + Clone + CoercibleInto<Logical>,
    Logical: std::ops::BitAnd<Logical, Output = O>,
    Rep<O>: From<Vec<O>>,
{
    type Output = Rep<O>;
    fn bitand(self, rhs: Rep<R>) -> Self::Output {
        let lc = self.inner().clone();
        let lb = lc.borrow();
        let lhs = lb.iter();

        let rc = rhs.inner().clone();
        let rb = rc.borrow();
        let rhs = rb.iter();

        Rep::from(
            zip_recycle(lhs, rhs)
                .map(|(l, r)| l.clone().coerce_into().bitand(r.clone().coerce_into()))
                .collect::<Vec<O>>(),
        )
    }
}

impl<L, R, C> VecPartialCmp<Rep<R>> for Rep<L>
where
    L: AtomicMode + Default + Clone + CoercibleInto<C>,
    R: AtomicMode + Default + Clone + CoercibleInto<C>,
    (L, R): CommonCmp<Common = C>,
    C: PartialOrd,
{
    type Output = Rep<Logical>;

    fn vec_gt(self, rhs: Rep<R>) -> Self::Output {
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

    fn vec_gte(self, rhs: Rep<R>) -> Self::Output {
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

    fn vec_lt(self, rhs: Rep<R>) -> Self::Output {
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

    fn vec_lte(self, rhs: Rep<R>) -> Self::Output {
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

    fn vec_eq(self, rhs: Rep<R>) -> Self::Output {
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

    fn vec_neq(self, rhs: Rep<R>) -> Self::Output {
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
    use crate::utils::SameType;
    use crate::object::{types::*, VecPartialCmp};
    use crate::object::rep::Rep;

    #[test]
    fn vector_add() {
        let x = Rep::from((1..=10).into_iter().collect::<Vec<_>>());
        let y = Rep::from(vec![2, 5, 6, 2, 3]);

        let z = x + y;
        assert_eq!(z, Rep::from(vec![3, 7, 9, 6, 8, 8, 12, 14, 11, 13]));

        let expected_type = Rep::<Integer>::new();
        assert!(z.is_same_type_as(&expected_type));
        assert!(z.is_integer());
    }

    #[test]
    fn vector_mul() {
        let x = Rep::from((1..=10).into_iter().collect::<Vec<_>>());
        let y = Rep::from(vec![Some(2), NA, Some(6), NA, Some(3)]);

        let z = x * y;
        assert_eq!(
            z,
            Rep::from(vec![
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

        let expected_type = Rep::<Integer>::new();
        assert!(z.is_same_type_as(&expected_type));
        assert!(z.is_integer());
    }

    #[test]
    fn vector_common_mul_f32_na() {
        // expect that f32's do not get coerced into an OptionNA:: instead
        // using std::f32::NAN as NA representation.

        let x = Rep::from(vec![Some(0_f64), NA, Some(10_f64)]);
        let y = Rep::from(vec![100, 10]);

        let z = x * y;
        // assert_eq!(z, Vector::from(vec![0_f32, std::f32::NAN, 1_000_f32]));
        // comparing floats is error prone

        let expected_type = Rep::<Numeric>::new();
        assert!(z.is_same_type_as(&expected_type));
        assert!(z.is_numeric());
    }

    #[test]
    fn vector_and() {
        // expect that f32's do not get coerced into an OptionNA:: instead
        // using std::f32::NAN as NA representation.

        let x = Rep::from(vec![Some(0_f64), NA, Some(10_f64)]);
        let y = Rep::from(vec![100, 10]);

        let z = x & y;
        assert_eq!(z, Rep::from(vec![Some(false), NA, Some(true)]));

        let expected_type = Rep::<Logical>::new();
        assert!(z.is_same_type_as(&expected_type));
        assert!(z.is_logical());
    }

    #[test]
    fn vector_gt() {
        // expect that f32's do not get coerced into an  instead
        // using std::f32::NAN as NA representation.

        let x = Rep::from(vec![Some(0_f64), NA, Some(10000_f64)]);
        let y = Rep::from(vec![100, 10]);

        let z = x.vec_gt(y);
        assert_eq!(z, Rep::from(vec![Some(false), NA, Some(true)]));

        let expected_type = Rep::<Logical>::new();
        assert!(z.is_same_type_as(&expected_type));
        assert!(z.is_logical());
    }
}
