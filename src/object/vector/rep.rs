use std::cell::{Ref, RefCell, RefMut};
use std::fmt::{Debug, Display};

use super::coercion::{AtomicMode, CoercibleInto, CommonCmp, CommonNum, MinimallyNumeric};
use super::iterators::{map_common_numeric, zip_recycle};
use super::reptype::{
    IntoIterableRefNames, IntoIterableRefPairs, IntoIterableRefValues, IterablePairs,
    IterableValues, Naming, RepType,
};
use super::subset::Subset;
use super::types::*;
use super::{OptionNA, Pow, VecPartialCmp};
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

    pub fn assign<R>(&mut self, value: Rep<R>) -> Self
    where
        T: From<R> + Clone,
        R: Clone + Default,
    {
        self.0.borrow_mut().assign(value.0.into_inner()).into()
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

    /// Apply over the vector contents to produce a vector of [std::cmp::Ordering]
    ///
    /// This function is used primarily in support of the implementation of
    /// vectorized comparison operators and likely does not need to be used
    /// outside of that context.
    ///
    /// See [crate::object::vector::VecPartialCmp] for vectorized comparison
    /// operator implementations.
    ///
    pub fn vectorized_partial_cmp<R, C>(self, other: Rep<R>) -> Vec<Option<std::cmp::Ordering>>
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
}

impl<T> Default for Rep<T>
where
    T: Clone + Default,
{
    fn default() -> Self {
        Rep(RefCell::new(RepType::default()))
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

impl From<Vec<OptionNA<f64>>> for Rep<Double> {
    fn from(value: Vec<OptionNA<f64>>) -> Self {
        Rep(RefCell::new(value.into()))
    }
}

impl From<Vec<f64>> for Rep<Double> {
    fn from(value: Vec<f64>) -> Self {
        Rep(RefCell::new(value.into()))
    }
}

impl From<Vec<OptionNA<i32>>> for Rep<Integer> {
    fn from(value: Vec<OptionNA<i32>>) -> Self {
        Rep(RefCell::new(value.into()))
    }
}

impl From<Vec<i32>> for Rep<Integer> {
    fn from(value: Vec<i32>) -> Self {
        Rep(RefCell::new(value.into()))
    }
}

impl From<Vec<OptionNA<bool>>> for Rep<Logical> {
    fn from(value: Vec<OptionNA<bool>>) -> Self {
        Rep(RefCell::new(value.into()))
    }
}

impl From<Vec<bool>> for Rep<Logical> {
    fn from(value: Vec<bool>) -> Self {
        Rep(RefCell::new(value.into()))
    }
}

impl From<Vec<OptionNA<String>>> for Rep<Character> {
    fn from(value: Vec<OptionNA<String>>) -> Self {
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
    type Output = Rep<O>;
    fn neg(self) -> Self::Output {
        let result = -(self.0.into_inner());
        Rep(RefCell::new(result))
    }
}

impl<L, R, C, O, LNum, RNum> std::ops::Add<Rep<R>> for Rep<L>
where
    L: AtomicMode + Default + Clone + MinimallyNumeric<As = LNum> + CoercibleInto<LNum>,
    R: AtomicMode + Default + Clone + MinimallyNumeric<As = RNum> + CoercibleInto<RNum>,
    (LNum, RNum): CommonNum<Common = C>,
    C: Clone + std::ops::Add<Output = O>,
    RepType<C>: From<Vec<O>>,
{
    type Output = Rep<C>;
    fn add(self, rhs: Rep<R>) -> Self::Output {
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

        Rep(RefCell::new(result))
    }
}

impl<L, R, C, O, LNum, RNum> std::ops::Sub<Rep<R>> for Rep<L>
where
    L: AtomicMode + Default + Clone + MinimallyNumeric<As = LNum> + CoercibleInto<LNum>,
    R: AtomicMode + Default + Clone + MinimallyNumeric<As = RNum> + CoercibleInto<RNum>,
    (LNum, RNum): CommonNum<Common = C>,
    C: std::ops::Sub<Output = O>,
    RepType<C>: From<Vec<O>>,
    O: Clone,
    C: Clone,
{
    type Output = Rep<C>;
    fn sub(self, rhs: Rep<R>) -> Self::Output {
        let result = (self.0.into_inner()) - (rhs.0.into_inner());
        Rep(RefCell::new(result))
    }
}

impl<L, R, C, O, LNum, RNum> std::ops::Mul<Rep<R>> for Rep<L>
where
    L: AtomicMode + Default + Clone + MinimallyNumeric<As = LNum> + CoercibleInto<LNum>,
    R: AtomicMode + Default + Clone + MinimallyNumeric<As = RNum> + CoercibleInto<RNum>,
    (LNum, RNum): CommonNum<Common = C>,
    C: std::ops::Mul<Output = O>,
    RepType<C>: From<Vec<O>>,
    O: Clone,
    C: Clone,
{
    type Output = Rep<C>;
    fn mul(self, rhs: Rep<R>) -> Self::Output {
        use std::ops::Mul;
        let result = Mul::mul(self.0.into_inner(), rhs.0.into_inner());

        Rep(RefCell::new(result))
    }
}

impl<L, R, C, O, LNum, RNum> std::ops::Div<Rep<R>> for Rep<L>
where
    L: AtomicMode + Default + Clone + MinimallyNumeric<As = LNum> + CoercibleInto<LNum>,
    R: AtomicMode + Default + Clone + MinimallyNumeric<As = RNum> + CoercibleInto<RNum>,
    (LNum, RNum): CommonNum<Common = C>,
    C: std::ops::Div<Output = O>,
    RepType<C>: From<Vec<O>>,
    O: Clone,
    C: Clone,
{
    type Output = Rep<C>;
    fn div(self, rhs: Rep<R>) -> Self::Output {
        let result = (self.0.into_inner()) / (rhs.0.into_inner());
        Rep(RefCell::new(result))
    }
}

impl<L, R, C, O, LNum, RNum> std::ops::Rem<Rep<R>> for Rep<L>
where
    L: AtomicMode + Default + Clone + MinimallyNumeric<As = LNum> + CoercibleInto<LNum>,
    R: AtomicMode + Default + Clone + MinimallyNumeric<As = RNum> + CoercibleInto<RNum>,
    (LNum, RNum): CommonNum<Common = C>,
    C: std::ops::Rem<Output = O>,
    RepType<C>: From<Vec<O>>,
    L: Clone,
    R: Clone,
    C: Clone,
    O: Clone,
{
    type Output = Rep<C>;
    fn rem(self, rhs: Rep<R>) -> Self::Output {
        pub use std::ops::Rem;
        let result = Rem::rem(self.0.into_inner(), rhs.0.into_inner());
        Rep(RefCell::new(result))
    }
}

impl<L, R, O, LNum, RNum> Pow<Rep<R>> for Rep<L>
where
    L: AtomicMode + Default + Clone + MinimallyNumeric<As = LNum> + CoercibleInto<LNum>,
    R: AtomicMode + Default + Clone + MinimallyNumeric<As = RNum> + CoercibleInto<RNum>,
    LNum: Pow<RNum, Output = O>,
    RepType<O>: From<Vec<O>>,
    L: Clone,
    R: Clone,
    O: Clone,
{
    type Output = Rep<O>;
    fn power(self, rhs: Rep<R>) -> Self::Output {
        let result = Pow::power(self.0.into_inner(), rhs.0.into_inner());
        Rep(RefCell::new(result))
    }
}

impl<L, R, O> std::ops::BitOr<Rep<R>> for Rep<L>
where
    L: AtomicMode + Default + Clone + CoercibleInto<Logical>,
    R: AtomicMode + Default + Clone + CoercibleInto<Logical>,
    Logical: std::ops::BitOr<Logical, Output = O>,
    RepType<O>: From<Vec<O>>,
    L: Clone,
    R: Clone,
    O: Clone,
{
    type Output = Rep<O>;
    fn bitor(self, rhs: Rep<R>) -> Self::Output {
        let result: RepType<O> = (self.0.into_inner()) | (rhs.0.into_inner());
        Rep(RefCell::new(result))
    }
}

impl<L, R, O> std::ops::BitAnd<Rep<R>> for Rep<L>
where
    L: AtomicMode + Default + Clone + CoercibleInto<Logical>,
    R: AtomicMode + Default + Clone + CoercibleInto<Logical>,
    Logical: std::ops::BitAnd<Logical, Output = O>,
    RepType<O>: From<Vec<O>>,
    L: Clone,
    R: Clone,
    O: Clone,
{
    type Output = Rep<O>;
    fn bitand(self, rhs: Rep<R>) -> Self::Output {
        let result: RepType<O> = (self.0.into_inner()) & (rhs.0.into_inner());
        Rep(RefCell::new(result))
    }
}

impl<L, O> std::ops::Not for Rep<L>
where
    L: AtomicMode + Default + Clone + CoercibleInto<Logical>,
    Logical: std::ops::Not<Output = O>,
    RepType<O>: From<Vec<O>>,
    O: Clone,
{
    type Output = Rep<O>;
    fn not(self) -> Self::Output {
        let result: RepType<O> = !self.0.into_inner();
        Rep(RefCell::new(result))
    }
}

impl<L, R, C> VecPartialCmp<Rep<R>> for Rep<L>
where
    L: AtomicMode + Default + Clone + CoercibleInto<C>,
    R: AtomicMode + Default + Clone + CoercibleInto<C>,
    (L, R): CommonCmp<Common = C>,
    C: PartialOrd,
    L: Clone,
    R: Clone,
    C: Clone,
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
