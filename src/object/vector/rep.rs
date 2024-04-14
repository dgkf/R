use std::cell::{Ref, RefCell};
use std::fmt::{Debug, Display};
use std::rc::Rc;

use super::coercion::{AtomicMode, CoercibleInto, CommonCmp, CommonNum, MinimallyNumeric};
use super::iterators::{map_common_numeric, zip_recycle};
use super::reptype::RepType;
use super::subset::Subset;
use super::types::*;
use super::{OptionNA, Pow, VecPartialCmp};

/// Variable Representation
///
/// This is a variable representation of a vector.
/// It is variable, because the internal vector representation might be transformed when necessary, thereby exchanging
/// one internal representation for another, usually a computational graph into a materialized
/// vector.
#[derive(Debug, Clone, PartialEq)]
pub struct VarRep<T>(RefCell<RepType<T>>);

impl<T> VarRep<T>
where
    T: AtomicMode + Clone + Default,
{
    // the underlying Rep<T> should not be exposed through the public API
    fn borrow(&self) -> Ref<RepType<T>> {
        self.0.borrow()
    }
    // this is the central
    fn materialize_inplace(&self) -> &Self {
        // FIXME: Rewrite this to avoid copying unnecessarily
        let new_repr = { self.borrow().materialize() };
        self.0.replace(new_repr);

        self
    }

    // FIXME: Not public
    pub fn materialize(&self) -> Self {
        self.borrow().materialize().into()
    }

    pub fn new() -> Self {
        RepType::new().into()
    }

    // FIXME: This should be refactored
    pub fn inner(&self) -> Rc<RefCell<Vec<T>>> {
        // does this make sense here?
        // sef
        // FIXME: Does this make sense here?
        self.borrow().inner()
    }

    pub fn len(&self) -> usize {
        // FIXME: Only materialize when necessary
        self.materialize_inplace();
        self.borrow().len()
    }

    pub fn subset(&self, subset: Subset) -> Self {
        (*self.borrow()).subset(subset).into()
    }

    #[must_use]
    pub fn is_empty(&self) -> bool {
        self.len() == 0
    }

    pub fn get(&self, index: usize) -> Option<Self> {
        let x = self.borrow().get(index);
        match x {
            Some(x) => Some(x.into()),
            None => None,
        }
    }

    pub fn assign(&mut self, value: Self) -> Self {
        self.0.borrow_mut().assign(value.0.into_inner()).into()
    }
    pub fn is_double(&self) -> bool {
        T::is_double()
    }
    pub fn is_logical(&self) -> bool {
        T::is_logical()
    }
    pub fn is_integer(&self) -> bool {
        T::is_integer()
    }
    pub fn is_character(&self) -> bool {
        T::is_character()
    }

    pub fn as_mode<Mode>(&self) -> VarRep<Mode>
    where
        T: CoercibleInto<Mode>,
    {
        VarRep(RefCell::new(self.borrow().as_mode()))
    }

    /// See [Self::as_mode] for more information
    pub fn as_logical(&self) -> VarRep<Logical>
    where
        T: CoercibleInto<Logical>,
    {
        self.as_mode::<Logical>()
    }

    /// See [Self::as_mode] for more information
    pub fn as_integer(&self) -> VarRep<Integer>
    where
        T: CoercibleInto<Integer>,
    {
        self.as_mode::<Integer>()
    }

    /// See [Self::as_mode] for more information
    pub fn as_double(&self) -> VarRep<Double>
    where
        T: CoercibleInto<Double>,
    {
        self.as_mode::<Double>()
    }

    /// See [Self::as_mode] for more information
    pub fn as_character(&self) -> VarRep<Character>
    where
        T: CoercibleInto<Character>,
    {
        self.as_mode::<Character>()
    }

    pub fn vectorized_partial_cmp<R, C>(self, other: VarRep<R>) -> Vec<Option<std::cmp::Ordering>>
    where
        T: AtomicMode + Default + Clone + CoercibleInto<C>,
        R: AtomicMode + Default + Clone + CoercibleInto<C>,
        (T, R): CommonCmp<Common = C>,
        C: PartialOrd,
    {
        self.0
            .into_inner()
            .vectorized_partial_cmp(other.0.into_inner())
    }

    fn get_inner(&self, index: usize) -> Option<T> {
        self.borrow().get_inner(index)
    }
}

impl<T> Default for VarRep<T>
where
    T: AtomicMode + Clone + Default,
{
    fn default() -> Self {
        VarRep(RefCell::new(RepType::default()))
    }
}

impl<T> From<RepType<T>> for VarRep<T>
where
    T: AtomicMode + Clone + Default,
{
    fn from(rep: RepType<T>) -> Self {
        VarRep(RefCell::new(rep))
    }
}

impl<T> TryInto<bool> for VarRep<OptionNA<T>>
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

impl From<Vec<OptionNA<f64>>> for VarRep<Double> {
    fn from(value: Vec<OptionNA<f64>>) -> Self {
        VarRep(RefCell::new(value.into()))
    }
}

impl From<Vec<f64>> for VarRep<Double> {
    fn from(value: Vec<f64>) -> Self {
        VarRep(RefCell::new(value.into()))
    }
}

impl From<Vec<OptionNA<i32>>> for VarRep<Integer> {
    fn from(value: Vec<OptionNA<i32>>) -> Self {
        VarRep(RefCell::new(value.into()))
    }
}

impl From<Vec<i32>> for VarRep<Integer> {
    fn from(value: Vec<i32>) -> Self {
        VarRep(RefCell::new(value.into()))
    }
}

impl From<Vec<OptionNA<bool>>> for VarRep<Logical> {
    fn from(value: Vec<OptionNA<bool>>) -> Self {
        VarRep(RefCell::new(value.into()))
    }
}

impl From<Vec<bool>> for VarRep<Logical> {
    fn from(value: Vec<bool>) -> Self {
        VarRep(RefCell::new(value.into()))
    }
}

impl From<Vec<OptionNA<String>>> for VarRep<Character> {
    fn from(value: Vec<OptionNA<String>>) -> Self {
        VarRep(RefCell::new(value.into()))
    }
}

impl From<Vec<String>> for VarRep<Character> {
    fn from(value: Vec<String>) -> Self {
        VarRep(RefCell::new(value.into()))
    }
}

impl<T> Display for VarRep<T>
where
    T: AtomicMode + Debug + Default + Clone,
{
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let n = self.len();
        if n == 0 {
            if self.is_double() {
                return write!(f, "double(0)");
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

impl<L, LNum, O> std::ops::Neg for VarRep<L>
where
    L: AtomicMode + Default + Clone + MinimallyNumeric<As = LNum> + CoercibleInto<LNum>,
    LNum: std::ops::Neg<Output = O>,
    RepType<O>: From<Vec<O>>,
{
    type Output = VarRep<O>;
    fn neg(self) -> Self::Output {
        let result = -(self.0.into_inner());
        VarRep(RefCell::new(result))
    }
}

impl<L, R, C, O, LNum, RNum> std::ops::Add<VarRep<R>> for VarRep<L>
where
    L: AtomicMode + Default + Clone + MinimallyNumeric<As = LNum> + CoercibleInto<LNum>,
    R: AtomicMode + Default + Clone + MinimallyNumeric<As = RNum> + CoercibleInto<RNum>,
    (LNum, RNum): CommonNum<Common = C>,
    C: Clone + std::ops::Add<Output = O>,
    RepType<C>: From<Vec<O>>,
{
    type Output = VarRep<C>;
    fn add(self, rhs: VarRep<R>) -> Self::Output {
        let lc = self.inner().clone();
        let lb = lc.borrow();
        let lhs = lb.iter();

        let rc = rhs.inner().clone();
        let rb = rc.borrow();
        let rhs = rb.iter();

        let result = RepType::from(
            map_common_numeric(zip_recycle(lhs, rhs))
                .map(|(l, r)| l + r)
                .collect::<Vec<O>>(),
        );

        VarRep(RefCell::new(result))
    }
}

impl<L, R, C, O, LNum, RNum> std::ops::Sub<VarRep<R>> for VarRep<L>
where
    L: AtomicMode + Default + Clone + MinimallyNumeric<As = LNum> + CoercibleInto<LNum>,
    R: AtomicMode + Default + Clone + MinimallyNumeric<As = RNum> + CoercibleInto<RNum>,
    (LNum, RNum): CommonNum<Common = C>,
    C: std::ops::Sub<Output = O>,
    RepType<C>: From<Vec<O>>,
{
    type Output = VarRep<C>;
    fn sub(self, rhs: VarRep<R>) -> Self::Output {
        let result = (self.0.into_inner()) - (rhs.0.into_inner()).into();
        VarRep(RefCell::new(result))
    }
}

impl<L, R, C, O, LNum, RNum> std::ops::Mul<VarRep<R>> for VarRep<L>
where
    L: AtomicMode + Default + Clone + MinimallyNumeric<As = LNum> + CoercibleInto<LNum>,
    R: AtomicMode + Default + Clone + MinimallyNumeric<As = RNum> + CoercibleInto<RNum>,
    (LNum, RNum): CommonNum<Common = C>,
    C: std::ops::Mul<Output = O>,
    RepType<C>: From<Vec<O>>,
{
    type Output = VarRep<C>;
    fn mul(self, rhs: VarRep<R>) -> Self::Output {
        use std::ops::Mul;
        let result = Mul::mul(self.0.into_inner(), rhs.0.into_inner());

        VarRep(RefCell::new(result))
    }
}

impl<L, R, C, O, LNum, RNum> std::ops::Div<VarRep<R>> for VarRep<L>
where
    L: AtomicMode + Default + Clone + MinimallyNumeric<As = LNum> + CoercibleInto<LNum>,
    R: AtomicMode + Default + Clone + MinimallyNumeric<As = RNum> + CoercibleInto<RNum>,
    (LNum, RNum): CommonNum<Common = C>,
    C: std::ops::Div<Output = O>,
    RepType<C>: From<Vec<O>>,
{
    type Output = VarRep<C>;
    fn div(self, rhs: VarRep<R>) -> Self::Output {
        let result = (self.0.into_inner()) / (rhs.0.into_inner());
        VarRep(RefCell::new(result))
    }
}

impl<L, R, C, O, LNum, RNum> std::ops::Rem<VarRep<R>> for VarRep<L>
where
    L: AtomicMode + Default + Clone + MinimallyNumeric<As = LNum> + CoercibleInto<LNum>,
    R: AtomicMode + Default + Clone + MinimallyNumeric<As = RNum> + CoercibleInto<RNum>,
    (LNum, RNum): CommonNum<Common = C>,
    C: std::ops::Rem<Output = O>,
    RepType<C>: From<Vec<O>>,
{
    type Output = VarRep<C>;
    fn rem(self, rhs: VarRep<R>) -> Self::Output {
        pub use std::ops::Rem;
        let result = Rem::rem(self.0.into_inner(), rhs.0.into_inner());
        VarRep(RefCell::new(result))
    }
}

impl<L, R, O, LNum, RNum> Pow<VarRep<R>> for VarRep<L>
where
    L: AtomicMode + Default + Clone + MinimallyNumeric<As = LNum> + CoercibleInto<LNum>,
    R: AtomicMode + Default + Clone + MinimallyNumeric<As = RNum> + CoercibleInto<RNum>,
    LNum: Pow<RNum, Output = O>,
    RepType<O>: From<Vec<O>>,
{
    type Output = VarRep<O>;
    fn power(self, rhs: VarRep<R>) -> Self::Output {
        let result = Pow::power(self.0.into_inner(), rhs.0.into_inner());
        VarRep(RefCell::new(result))
    }
}

impl<L, R, O> std::ops::BitOr<VarRep<R>> for VarRep<L>
where
    L: AtomicMode + Default + Clone + CoercibleInto<Logical>,
    R: AtomicMode + Default + Clone + CoercibleInto<Logical>,
    Logical: std::ops::BitOr<Logical, Output = O>,
    RepType<O>: From<Vec<O>>,
{
    type Output = VarRep<O>;
    fn bitor(self, rhs: VarRep<R>) -> Self::Output {
        let result: RepType<O> = (self.0.into_inner()) | (rhs.0.into_inner());
        VarRep(RefCell::new(result))
    }
}

impl<L, R, O> std::ops::BitAnd<VarRep<R>> for VarRep<L>
where
    L: AtomicMode + Default + Clone + CoercibleInto<Logical>,
    R: AtomicMode + Default + Clone + CoercibleInto<Logical>,
    Logical: std::ops::BitAnd<Logical, Output = O>,
    RepType<O>: From<Vec<O>>,
{
    type Output = VarRep<O>;
    fn bitand(self, rhs: VarRep<R>) -> Self::Output {
        let result: RepType<O> = (self.0.into_inner()) & (rhs.0.into_inner());
        VarRep(RefCell::new(result))
    }
}

impl<L, R, C> VecPartialCmp<VarRep<R>> for VarRep<L>
where
    L: AtomicMode + Default + Clone + CoercibleInto<C>,
    R: AtomicMode + Default + Clone + CoercibleInto<C>,
    (L, R): CommonCmp<Common = C>,
    C: PartialOrd,
{
    type Output = VarRep<Logical>;

    fn vec_gt(self, rhs: VarRep<R>) -> Self::Output {
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

    fn vec_gte(self, rhs: VarRep<R>) -> Self::Output {
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

    fn vec_lt(self, rhs: VarRep<R>) -> Self::Output {
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

    fn vec_lte(self, rhs: VarRep<R>) -> Self::Output {
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

    fn vec_eq(self, rhs: VarRep<R>) -> Self::Output {
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

    fn vec_neq(self, rhs: VarRep<R>) -> Self::Output {
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
