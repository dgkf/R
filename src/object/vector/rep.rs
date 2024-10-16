use std::cell::{Ref, RefCell, RefMut};
use std::fmt::{Debug, Display};
use std::iter::repeat;

use super::coercion::{AtomicMode, CoercibleInto, CommonCmp, CommonNum, MinimallyNumeric};
use super::reptype::{
    IntoIterableRefNames, IntoIterableRefPairs, IntoIterableRefValues, IterablePairs,
    IterableValues, Naming, RepType,
};
use super::subset::Subset;
use super::types::*;
use super::{OptionNA, Pow, VecPartialCmp};
use crate::error::Error;
use crate::lang::Signal;
use crate::object::{CowObj, Obj, Subsets, ViewMut};

/// Vector Representation
///
/// The ref-cell is used so vectors can change there internal representation,
/// e.g. by materializing.
#[derive(Debug, PartialEq)]
pub struct Rep<T: Clone>(pub RefCell<RepType<T>>);

impl<T: Clone + Default> Clone for Rep<T> {
    fn clone(&self) -> Self {
        match self.borrow().clone() {
            RepType::Subset(v, s, n) => Rep(RefCell::new(RepType::Subset(
                v.clone(),
                s.clone(),
                n.clone(),
            ))),
        }
    }
}

impl<T: Clone + Default> ViewMut for Rep<T> {
    fn view_mut(&self) -> Self {
        Self(RefCell::new(self.borrow().view_mut()))
    }
}

impl<T: ViewMut + Default + Clone> Rep<T> {
    /// Get the inner value mutably.
    /// This is used for assignments like `list(1)[[1]] = 10`.
    pub fn try_get_inner_mut(&self, subset: Subset) -> Result<T, Signal> {
        self.borrow().try_get_inner_mut(subset)
    }

    /// Get a cloned version of the inner value.
    /// This is used for accessing inner values like `list(1)[[1]]`.
    pub fn try_get_inner(&self, subset: Subset) -> Result<T, Signal> {
        #[allow(clippy::map_clone)]
        self.try_get_inner_mut(subset).map(|x| x.clone())
    }
}

impl<T: Clone + Default + 'static> Rep<T> {
    /// Iterate over the owned names and values of the vector.
    pub fn iter_pairs(&self) -> IterablePairs<T> {
        self.0.borrow().clone().iter_pairs()
    }
}

impl<T> Rep<T>
where
    T: Clone + Default,
{
    pub fn as_scalar(&self) -> Option<T> {
        let mut into_iter = self.values_ref();
        let mut iter = into_iter.iter();
        if let Some(x) = iter.next() {
            if iter.next().is_none() {
                return Some(x.clone());
            }
        };
        None
    }

    pub fn borrow(&self) -> Ref<RepType<T>> {
        self.0.borrow()
    }

    pub fn borrow_mut(&mut self) -> RefMut<RepType<T>> {
        self.0.borrow_mut()
    }

    /// Iterate over the (owned) values of the vector.
    pub fn iter_values(&self) -> IterableValues<T> {
        self.0.borrow().iter_values()
    }

    /// Iterate over the names of the vector (if they exist).
    pub fn iter_names(&self) -> Option<IterableValues<Character>> {
        self.0.borrow().iter_names()
    }

    fn materialize_inplace(&self) -> &Self {
        // TODO: Rewrite this to avoid copying unnecessarily
        let new_repr = { self.borrow().materialize() };
        self.0.replace(new_repr);

        self
    }

    /// Reindex the mapping from names to indices using the names vector from the `Naming`.
    pub fn reindex(&mut self) {
        self.borrow_mut().reindex()
    }

    /// Set the names of the vector.
    pub fn set_names(&self, names: CowObj<Vec<OptionNA<String>>>) {
        let new_repr = self.borrow().materialize().set_names(names);
        self.0.replace(new_repr);
    }

    /// Whether the vector representation has names.
    pub fn is_named(&self) -> bool {
        matches!(*self.borrow(), RepType::Subset(.., Some(_)))
    }

    /// Return the names of the vector if there are any.
    pub fn names(&self) -> Option<CowObj<Vec<Character>>> {
        match self.borrow().clone() {
            RepType::Subset(_, s, n) => {
                if s.is_empty() {
                    n.map(|n| n.clone().names)
                } else if n.is_some() {
                    Some(
                        self.iter_names()
                            .expect("checked that names exist")
                            .collect::<Vec<Character>>()
                            .into(),
                    )
                } else {
                    None
                }
            }
        }
    }

    pub fn dedup_last(self) -> Self {
        self.0.into_inner().dedup_last().into()
    }

    /// Constructs a new, empty `Rep<T>` with at least the specified `capacity`.
    /// Names are only include if `names` is true.
    pub fn with_capacity(capacity: usize, names: bool) -> Self {
        let naming = if names {
            Some(Naming::with_capacity(capacity))
        } else {
            None
        };
        Self(RefCell::new(RepType::Subset(
            CowObj::from(Vec::with_capacity(capacity)),
            Subsets::default(),
            naming,
        )))
    }

    /// Get an `RepTypeIntoIterablePairs<T>` which in turn can be converted into an iterator over
    /// pairs of references (&name, &value).
    ///
    /// Directly getting an iterator is not possible due to lifetime issues.
    pub fn pairs_ref(&self) -> IntoIterableRefPairs<T> {
        self.0.borrow().pairs_ref()
    }

    /// Get an `Option<RepTypeIntoIterableValues<T>>` which in turn can be converted into an iterator over
    /// references to the values.
    /// The `None` variant is returned if the `Rep<T>` is not named.
    ///
    /// Directly getting an iterator is not possible due to lifetime issues.
    pub fn values_ref(&self) -> IntoIterableRefValues<T> {
        self.0.borrow().values_ref()
    }

    /// Get an `RepTypeIntoIterableValues<T>` which in turn can be converted into an iterator over
    /// references to the names.
    ///
    /// Directly getting an iterator is not possible due to lifetime issues.
    pub fn names_ref(&self) -> Option<IntoIterableRefNames> {
        self.0.borrow().names_ref()
    }

    pub fn materialize(&self) -> Self {
        self.borrow().materialize().into()
    }

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
        RepType::new().into()
    }

    pub fn inner(&self) -> CowObj<Vec<T>> {
        self.borrow().inner()
    }

    pub fn len(&self) -> usize {
        // TODO: Only materialize when necessary
        self.materialize_inplace();
        self.borrow().len()
    }

    /// Subsetting a Vector
    ///
    /// Introduce a new subset into the aggregate list of subset indices.
    ///
    pub fn subset(&self, subset: Subset) -> Self {
        (*self.borrow()).subset(subset).into()
    }

    #[must_use]
    pub fn is_empty(&self) -> bool {
        self.len() == 0
    }

    pub fn get(&self, index: usize) -> Option<Self> {
        let x = self.borrow().get(index);
        x.map(|x| x.into())
    }

    /// Change a value at the location given by `subset` to the provided `value`.
    /// If the `subset` does not have length `1`, an error is returned.
    pub fn set_subset(&mut self, subset: Subset, value: T) -> Result<T, Signal> {
        // Used for `[[`-assignment.
        self.0.borrow_mut().set_subset(subset, value)
    }

    /// Push a named `value` with a given `name` onto the `Rep<T>`.
    pub fn push_named(&self, name: OptionNA<String>, value: T) {
        self.borrow().push_named(name, value)
    }

    pub fn assign<R>(&mut self, value: Rep<R>) -> Result<Self, Signal>
    where
        T: From<R> + Clone,
        R: Clone + Default,
    {
        self.0
            .borrow_mut()
            .assign(value.0.into_inner())
            .map(|x| x.into())
    }
    /// Test the mode of the internal vector type
    ///
    /// Internally, this is defined by the [crate::object::coercion::AtomicMode]
    /// implementation of the vector's element type.
    ///
    pub fn is_double(&self) -> bool
    where
        T: AtomicMode,
    {
        T::is_double()
    }
    /// See [Self::is_double] for more information
    pub fn is_logical(&self) -> bool
    where
        T: AtomicMode,
    {
        T::is_logical()
    }
    /// See [Self::is_double] for more information
    pub fn is_integer(&self) -> bool
    where
        T: AtomicMode,
    {
        T::is_integer()
    }
    /// See [Self::is_double] for more information
    pub fn is_character(&self) -> bool
    where
        T: AtomicMode,
    {
        T::is_character()
    }

    /// Convert a Vector into a vector of a specific class of internal type
    ///
    /// The internal type only needs to satisfy
    /// [crate::object::coercion::CoercibleInto] for the `Mode`, and for the `Mode`
    /// type to implement [crate::object::coercion::AtomicMode]. Generally,
    /// this is used more directly via [Self::as_logical], [Self::as_integer],
    /// [Self::as_double] and [Self::as_character], which predefine the output
    /// type of the mode.
    ///
    /// ```
    /// use r::object::Vector;
    /// use r::object::OptionNA;
    ///
    /// let x = Vector::from(vec![false, true, true, false]);
    /// let n = x.as_double();
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
        T: CoercibleInto<Mode> + AtomicMode,
        Mode: Clone,
    {
        Rep(RefCell::new(self.borrow().as_mode()))
    }

    /// See [Self::as_mode] for more information
    pub fn as_logical(&self) -> Rep<Logical>
    where
        T: CoercibleInto<Logical> + AtomicMode,
    {
        self.as_mode::<Logical>()
    }

    /// See [Self::as_mode] for more information
    pub fn as_integer(&self) -> Rep<Integer>
    where
        T: CoercibleInto<Integer> + AtomicMode,
    {
        self.as_mode::<Integer>()
    }

    /// See [Self::as_mode] for more information
    pub fn as_double(&self) -> Rep<Double>
    where
        T: CoercibleInto<Double> + AtomicMode,
    {
        self.as_mode::<Double>()
    }

    /// See [Self::as_mode] for more information
    pub fn as_character(&self) -> Rep<Character>
    where
        T: CoercibleInto<Character> + AtomicMode,
    {
        self.as_mode::<Character>()
    }
}

impl<T> Default for Rep<T>
where
    T: Clone + Default,
{
    fn default() -> Self {
        Rep(RefCell::new(RepType::default()))
    }
}

impl<T> From<Vec<T>> for Rep<T>
where
    T: Clone + Default,
{
    fn from(rep: Vec<T>) -> Self {
        Rep(RefCell::new(RepType::from(CowObj::from(rep))))
    }
}

impl<T> From<CowObj<Vec<T>>> for Rep<T>
where
    T: Clone + Default,
{
    fn from(rep: CowObj<Vec<T>>) -> Self {
        Rep(RefCell::new(rep.into()))
    }
}

impl<T> From<RepType<T>> for Rep<T>
where
    T: Clone + Default,
{
    fn from(rep: RepType<T>) -> Self {
        Rep(RefCell::new(rep))
    }
}

// TODO: I think this should err when rep has length > 1
impl<T> TryInto<bool> for Rep<OptionNA<T>>
where
    OptionNA<T>: AtomicMode + Clone + CoercibleInto<OptionNA<bool>>,
    T: 'static,
{
    type Error = ();
    fn try_into(self) -> Result<bool, Self::Error> {
        self.iter_pairs()
            .next()
            .map(|(_, x)| x)
            .map_or(
                Err(()),
                |i| match CoercibleInto::<OptionNA<bool>>::coerce_into(i) {
                    OptionNA::Some(x) => Ok(x),
                    OptionNA::NA => Err(()),
                },
            )
    }
}

impl From<Vec<(Character, Obj)>> for Rep<Obj> {
    fn from(value: Vec<(Character, Obj)>) -> Self {
        Rep(RefCell::new(value.into()))
    }
}

impl From<Vec<f64>> for Rep<Double> {
    fn from(value: Vec<f64>) -> Self {
        Rep(RefCell::new(value.into()))
    }
}

impl From<Vec<i32>> for Rep<Integer> {
    fn from(value: Vec<i32>) -> Self {
        Rep(RefCell::new(value.into()))
    }
}

impl From<Vec<bool>> for Rep<Logical> {
    fn from(value: Vec<bool>) -> Self {
        Rep(RefCell::new(value.into()))
    }
}

impl From<Vec<String>> for Rep<Character> {
    fn from(value: Vec<String>) -> Self {
        Rep(RefCell::new(value.into()))
    }
}

impl<T: Clone> From<Vec<(Option<String>, T)>> for Rep<T> {
    fn from(value: Vec<(Option<String>, T)>) -> Self {
        Rep(RefCell::new(value.into()))
    }
}

impl<T> Display for Rep<T>
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
        // calculate how many characters are printed per value.
        // The iteraror yields the characters needed for a specific item.
        fn element_width(iter: impl Iterator<Item = usize>) -> usize {
            let mut elt_width = 1_usize;
            for (i, width) in iter.enumerate() {
                elt_width = std::cmp::max(elt_width, width);
                if elt_width * (i + 1) >= 20 * 80 {
                    break;
                }
            }
            elt_width
        }

        if !self.is_named() {
            let elt_width =
                element_width(self.values_ref().iter().map(|x| format!("{:?}", x).len()));

            let mut values_ref = self.values_ref();
            let x_strs = values_ref.iter().map(|xi| format!("{:?}", xi));

            let mut col = 0;
            let gutterlen = 2 + nlen + 1;

            // hard coded max print & console width
            // we print at most 20 rows
            let maxprint = 20 * ((80 - gutterlen) / (elt_width + 1));

            x_strs
                .take(maxprint)
                .enumerate()
                .try_for_each(|(i, x_str)| {
                    if i == 0 {
                        col = gutterlen + elt_width;
                        write!(
                            f,
                            "{:>3$}[{}] {:>4$}",
                            "",
                            i + 1,
                            x_str,
                            nlen - 1,
                            elt_width
                        )
                    } else if col + 1 + elt_width > 80 {
                        col = gutterlen + elt_width;
                        let i_str = format!("{}", i + 1);
                        let gutter = nlen - i_str.len();
                        write!(
                            f,
                            "\n{:>3$}[{}] {:>4$}",
                            "", i_str, x_str, gutter, elt_width
                        )
                    } else {
                        col += 1 + elt_width;
                        write!(f, " {:>1$}", x_str, elt_width)
                    }
                })?;

            if n > maxprint {
                write!(f, "\n[ omitting {} entries ]", n - maxprint)?;
            }
        } else {
            let elt_width = element_width(
                self.pairs_ref()
                    .iter()
                    .map(|x| std::cmp::max(format!("{:}", x.0).len(), format!("{:?}", x.1).len())),
            );
            let mut values_ref = self.values_ref();
            let mut names_ref = self
                .names_ref()
                .expect("already checked existence of names");

            let mut values_strs = values_ref.iter().map(|x| format!("{:?}", x));
            let mut names_strs = names_ref.iter().map(|x| format!("{:}", x));

            // hard coded max print & console width
            // we print at most 20 rows
            let elts_per_line = 80 / (elt_width + 1);

            'lines: for _ in 1..=20 {
                for _ in 1..=elts_per_line {
                    if let Some(name) = names_strs.next() {
                        write!(f, "{:}{:>2$}", name, " ", elt_width - name.len())?;
                    } else {
                        break;
                    }
                }
                writeln!(f)?;
                for _ in 1..=elts_per_line {
                    if let Some(value) = values_strs.next() {
                        write!(f, "{:}{:>2$}", value, " ", elt_width - value.len())?;
                    } else {
                        break 'lines;
                    }
                }
                writeln!(f)?;
            }
        }
        Ok(())
    }
}

impl<L, LNum, O> std::ops::Neg for Rep<L>
where
    L: AtomicMode + Default + Clone + MinimallyNumeric<As = LNum> + CoercibleInto<LNum>,
    LNum: std::ops::Neg<Output = O>,
    RepType<O>: From<Vec<O>>,
    O: Clone,
{
    type Output = Result<Rep<O>, Signal>;
    fn neg(self) -> Self::Output {
        let result: Vec<O> = self
            .iter_values()
            .map(|x| -(CoercibleInto::<LNum>::coerce_into(x)))
            .collect();
        Ok(Rep(RefCell::new(result.into())))
    }
}

impl<L, R, C, O, LNum, RNum> std::ops::Add<Rep<R>> for Rep<L>
where
    L: AtomicMode + Default + Clone + MinimallyNumeric<As = LNum> + CoercibleInto<LNum>,
    R: AtomicMode + Default + Clone + MinimallyNumeric<As = RNum> + CoercibleInto<RNum>,
    (LNum, RNum): CommonNum<Common = C>,
    C: Clone + std::ops::Add<Output = O> + Default,
    Rep<C>: From<Vec<O>>,
    O: Clone + Default,
{
    type Output = Result<Rep<C>, Signal>;
    fn add(self, rhs: Rep<R>) -> Self::Output {
        try_binary_num_op(self, rhs, |x, y| x + y)
    }
}

impl<L, R, C, O, LNum, RNum> std::ops::Sub<Rep<R>> for Rep<L>
where
    L: AtomicMode + Default + Clone + MinimallyNumeric<As = LNum> + CoercibleInto<LNum>,
    R: AtomicMode + Default + Clone + MinimallyNumeric<As = RNum> + CoercibleInto<RNum>,
    (LNum, RNum): CommonNum<Common = C>,
    C: Clone + std::ops::Sub<Output = O> + Default,
    Rep<C>: From<Vec<O>>,
    O: Clone + Default,
{
    type Output = Result<Rep<C>, Signal>;
    fn sub(self, rhs: Rep<R>) -> Self::Output {
        try_binary_num_op(self, rhs, |x, y| x - y)
    }
}

impl<L, R, C, O, LNum, RNum> std::ops::Mul<Rep<R>> for Rep<L>
where
    L: AtomicMode + Default + Clone + MinimallyNumeric<As = LNum> + CoercibleInto<LNum>,
    R: AtomicMode + Default + Clone + MinimallyNumeric<As = RNum> + CoercibleInto<RNum>,
    (LNum, RNum): CommonNum<Common = C>,
    C: Clone + std::ops::Mul<Output = O> + Default,
    Rep<C>: From<Vec<O>>,
    O: Clone + Default,
{
    type Output = Result<Rep<C>, Signal>;
    fn mul(self, rhs: Rep<R>) -> Self::Output {
        try_binary_num_op(self, rhs, |x, y| x * y)
    }
}

impl<L, R, C, O, LNum, RNum> std::ops::Div<Rep<R>> for Rep<L>
where
    L: AtomicMode + Default + Clone + MinimallyNumeric<As = LNum> + CoercibleInto<LNum>,
    R: AtomicMode + Default + Clone + MinimallyNumeric<As = RNum> + CoercibleInto<RNum>,
    (LNum, RNum): CommonNum<Common = C>,
    C: Clone + std::ops::Div<Output = O> + Default,
    Rep<C>: From<Vec<O>>,
    O: Clone + Default,
{
    type Output = Result<Rep<C>, Signal>;
    fn div(self, rhs: Rep<R>) -> Self::Output {
        try_binary_num_op(self, rhs, |x, y| x / y)
    }
}

impl<L, R, C, O, LNum, RNum> std::ops::Rem<Rep<R>> for Rep<L>
where
    L: AtomicMode + Default + Clone + MinimallyNumeric<As = LNum> + CoercibleInto<LNum>,
    R: AtomicMode + Default + Clone + MinimallyNumeric<As = RNum> + CoercibleInto<RNum>,
    (LNum, RNum): CommonNum<Common = C>,
    C: Clone + std::ops::Rem<Output = O> + Default,
    Rep<C>: From<Vec<O>>,
    O: Clone + Default,
{
    type Output = Result<Rep<C>, Signal>;
    fn rem(self, rhs: Rep<R>) -> Self::Output {
        try_binary_num_op(self, rhs, |x, y| x % y)
    }
}

impl<L, R, O, LNum, RNum> Pow<Rep<R>> for Rep<L>
where
    L: AtomicMode + Default + Clone + MinimallyNumeric<As = LNum> + CoercibleInto<LNum>,
    R: AtomicMode + Default + Clone + MinimallyNumeric<As = RNum> + CoercibleInto<RNum>,
    (LNum, RNum): CommonNum<Common = O>,
    O: Pow<O, Output = O>,
    Rep<O>: From<Vec<O>>,
    O: Default,
    L: Clone,
    R: Clone,
    O: Clone,
{
    type Output = Result<Rep<O>, Signal>;
    fn power(self, rhs: Rep<R>) -> Self::Output {
        try_binary_num_op(self, rhs, |x, y| Pow::power(x, y))
    }
}

impl<L, R> std::ops::BitOr<Rep<R>> for Rep<L>
where
    L: AtomicMode + Default + Clone + CoercibleInto<Logical>,
    R: AtomicMode + Default + Clone + CoercibleInto<Logical>,
{
    type Output = Result<Rep<Logical>, Signal>;
    fn bitor(self, rhs: Rep<R>) -> Self::Output {
        try_binary_lgl_op(self, rhs, |x, y| x | y)
    }
}

impl<L, R> std::ops::BitAnd<Rep<R>> for Rep<L>
where
    L: AtomicMode + Default + Clone + CoercibleInto<Logical>,
    R: AtomicMode + Default + Clone + CoercibleInto<Logical>,
{
    type Output = Result<Rep<Logical>, Signal>;
    fn bitand(self, rhs: Rep<R>) -> Self::Output {
        try_binary_lgl_op(self, rhs, |x, y| x & y)
    }
}

impl<L> std::ops::Not for Rep<L>
where
    L: AtomicMode + Default + Clone + CoercibleInto<Logical>,
{
    type Output = Result<Rep<Logical>, Signal>;
    fn not(self) -> Self::Output {
        let result: Vec<Logical> = self
            .iter_values()
            .map(|x| !(CoercibleInto::<Logical>::coerce_into(x)))
            .collect();
        Ok(Rep(RefCell::new(result.into())))
    }
}

impl<L, R, C> VecPartialCmp<Rep<R>> for Rep<L>
where
    L: AtomicMode + Default + Clone + CoercibleInto<C> + Clone,
    R: AtomicMode + Default + Clone + CoercibleInto<C> + Clone,
    (L, R): CommonCmp<Common = C>,
    C: PartialOrd + Clone + Default,
{
    type Output = Result<Rep<Logical>, Signal>;

    fn vec_gt(self, rhs: Rep<R>) -> Self::Output {
        use std::cmp::Ordering::*;
        try_binary_cmp_op(self, rhs, |i| match i {
            Some(Greater | Equal) => OptionNA::Some(true),
            Some(_) => OptionNA::Some(false),
            None => OptionNA::NA,
        })
    }

    fn vec_gte(self, rhs: Rep<R>) -> Self::Output {
        use std::cmp::Ordering::*;
        try_binary_cmp_op(self, rhs, |i| match i {
            Some(Greater | Equal) => OptionNA::Some(true),
            Some(_) => OptionNA::Some(false),
            None => OptionNA::NA,
        })
    }

    fn vec_lt(self, rhs: Rep<R>) -> Self::Output {
        use std::cmp::Ordering::*;
        try_binary_cmp_op(self, rhs, |i| match i {
            Some(Less) => OptionNA::Some(true),
            Some(_) => OptionNA::Some(false),
            None => OptionNA::NA,
        })
    }

    fn vec_lte(self, rhs: Rep<R>) -> Self::Output {
        use std::cmp::Ordering::*;
        try_binary_cmp_op(self, rhs, |i| match i {
            Some(Less | Equal) => OptionNA::Some(true),
            Some(_) => OptionNA::Some(false),
            None => OptionNA::NA,
        })
    }

    fn vec_eq(self, rhs: Rep<R>) -> Self::Output {
        use std::cmp::Ordering::*;
        try_binary_cmp_op(self, rhs, |i| match i {
            Some(Equal) => OptionNA::Some(true),
            Some(_) => OptionNA::Some(false),
            None => OptionNA::NA,
        })
    }

    fn vec_neq(self, rhs: Rep<R>) -> Self::Output {
        use std::cmp::Ordering::*;
        try_binary_cmp_op(self, rhs, |i| match i {
            Some(Equal) => OptionNA::Some(false),
            Some(_) => OptionNA::Some(true),
            None => OptionNA::NA,
        })
    }
}

// New function: try_recycle_then
fn try_recycle_then<L, R, O, F, A>(
    lhs: Rep<L>,
    rhs: Rep<R>,
    g: F,
) -> Result<Rep<A>, Signal>
where
    L: Clone + Default,
    R: Clone + Default,
    Rep<A>: From<Vec<O>>,
    O: Clone + Default,
    A: Clone,
    F: Fn(L, R) -> O,
{
    match (lhs.as_scalar(), rhs.as_scalar()) {
        (Some(l), Some(r)) => {
            let result: Vec<O> = vec![g(l, r)];
            Ok(Rep::from(result))
        }
        (Some(l), None) => {
            let result: Vec<O> = repeat(l)
                .zip(rhs.iter_values())
                .map(|(l, r)| g(l, r))
                .collect();
            if result.is_empty() {
                return Err(Signal::Error(Error::NonRecyclableLengths(
                    1, 0
                )));
            }
            Ok(Rep::from(result))
        }
        (None, Some(r)) => {
            let result: Vec<O> = lhs.iter_values()
                .zip(repeat(r))
                .map(|(l, r)| g(l, r))
                .collect();
            if result.is_empty() {
                return Err(Signal::Error(Error::NonRecyclableLengths(
                    0, 1
                )));
            }
            Ok(Rep::from(result))
        }
        (None, None) => {
            let mut lc = lhs.iter_values();
            let mut rc = rhs.iter_values();

            // get the maximum size hint of the two iterators lc and rc
            let max_size = std::cmp::max(lc.size_hint().0, rc.size_hint().0);

            let mut result: Vec<O> = Vec::with_capacity(max_size);

            loop {
                match (lc.next(), rc.next()) {
                    (Some(l), Some(r)) => result.push(g(l, r)),
                    (Some(_), None) => {
                        return Err(Signal::Error(Error::NonRecyclableLengths(
                            result.len() + 1 + lc.count(),
                            result.len(),
                        )));
                    },
                    (None, Some(_)) => {
                        return Err(Signal::Error(Error::NonRecyclableLengths(
                            result.len(),
                            result.len() + 1 + rc.count(),
                        )));
                    }
                    (None, None) => {
                        return Ok(Rep::from(result))
                    },
                }
            }
        }
    }
}

// things like x + y
fn try_binary_num_op<L, R, C, O, LNum, RNum, F>(
    lhs: Rep<L>,
    rhs: Rep<R>,
    f: F,
) -> Result<Rep<C>, Signal>
where
    L: Default + Clone + MinimallyNumeric<As = LNum> + CoercibleInto<LNum>,
    R: Default + Clone + MinimallyNumeric<As = RNum> + CoercibleInto<RNum>,
    C: Default + Clone,
    (LNum, RNum): CommonNum<Common = C>,
    Rep<C>: From<Vec<O>>,
    O: Clone + Default,
    F: Fn(C, C) -> O,
    C: Clone + Default,
{
    try_recycle_then(
        lhs,
        rhs,
        |x, y| {
            let (c1, c2) = (
                CoercibleInto::<LNum>::coerce_into(x),
                CoercibleInto::<RNum>::coerce_into(y),
            )
                .into_common();
            f(c1, c2)
        }
    )
}

// FIXME(performance): equality with references for characters
fn try_binary_cmp_op<L, R, C, F>(lhs: Rep<L>, rhs: Rep<R>, f: F) -> Result<Rep<Logical>, Signal>
where
    L: AtomicMode + Default + Clone + CoercibleInto<C> + Clone,
    R: AtomicMode + Default + Clone + CoercibleInto<C> + Clone,
    (L, R): CommonCmp<Common = C>,
    C: PartialOrd + Clone + Default,
    F: Fn(Option<std::cmp::Ordering>) -> Logical,
{
    try_recycle_then(
        lhs,
        rhs,
        |x, y| {
        let c1: C = x.coerce_into();
        let c2: C = y.coerce_into();
        let ordering = c1.partial_cmp(&c2);
        f(ordering)
        }
    )
}

pub fn try_binary_lgl_op<L, R, F>(lhs: Rep<L>, rhs: Rep<R>, f: F) -> Result<Rep<Logical>, Signal>
where
    L: AtomicMode + Default + Clone + CoercibleInto<Logical>,
    R: AtomicMode + Default + Clone + CoercibleInto<Logical>,
    F: Fn(Logical, Logical) -> Logical,
{
    try_recycle_then(
        lhs,
        rhs,
        |x, y| {
            let (c1, c2) = (
                CoercibleInto::<Logical>::coerce_into(x),
                CoercibleInto::<Logical>::coerce_into(y),
            );
            f(c1, c2)
        }
    )
}
