use std::cell::{Ref, RefCell, RefMut};
use std::fmt::{Debug, Display};

use super::coercion::{AtomicMode, CoercibleInto, CommonCmp, CommonNum, MinimallyNumeric};
use super::iterators::{map_common_numeric, zip_recycle};
use super::reptype::{Naming, RepType};
use super::reptype::{RepTypeIter, RepTypeSubsetIterPairsRef};
use super::subset::Subset;
use super::types::*;
use super::{OptionNA, Pow, VecPartialCmp};
use crate::error::Error;
use crate::object::Subsets;
use crate::object::{CowObj, Obj, ViewMut};

use crate::object::reptype::RepTypeSubsetIterPairs;

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
    pub fn try_get_inner_mut(&self, subset: Subset) -> Result<T, Error> {
        match self.borrow().clone() {
            RepType::Subset(values, mut subsets, maybe_naming) => {
                subsets.push(subset);
                if let Some(naming) = maybe_naming {
                    let mut iter = subsets.bind_names(naming.map).into_iter();

                    // Here, the subset must produce exactly one index, i.e. we call next() twice and the second
                    // yielded element must be None
                    if let Some((i, _)) = iter.next() {
                        if let None = iter.next() {
                            values.with_inner_mut(|v| {
                                v.get_mut(i)
                                    .map_or(Err(Error::Other("todo".to_string())), |x| {
                                        Ok(x.view_mut())
                                    })
                            })
                        } else {
                            Err(Error::Other("todo".to_string()))
                        }
                    } else {
                        Err(Error::Other("todo".to_string()))
                    }
                } else {
                    // TODO
                    Err(Error::Other("todo".to_string()))
                }
            }
        }
    }

    pub fn try_get_inner(&self, subset: Subset) -> Result<T, Error> {
        self.try_get_inner_mut(subset).map(|v| v.clone())
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

    pub fn replace(&self, value: Self) {
        // if we do
        // l = list(new.env())
        // l[[1]] = list(99)
        // we need to replace list(1, 2) with list(99)
        let x = value.0.into_inner();
        self.0.replace(x);
    }

    /// Iterate over the (owned) values of the vector.
    // pub fn iter_values(&self) -> Box<dyn Iterator<Item = T> + '_> {
    //     let x = self.0.borrow().clone();
    //     x.into_iter()
    // }

    // /// Iterate over the names of the vector (if they exist).
    // pub fn iter_names(&self) -> Option<Box<dyn Iterator<Item = Character>>> {
    //     self.0.borrow().iter_names()
    // }

    // /// Iterate over the names and values of the vector (if the names exist).
    // pub fn iter_pairs(&self) -> Option<Box<dyn Iterator<Item = (Character, T)>>> {
    //     self.0.borrow().iter_named()
    // }

    fn materialize_inplace(&self) -> &Self {
        // TODO: Rewrite this to avoid copying unnecessarily
        let new_repr = { self.borrow().materialize() };
        self.0.replace(new_repr);

        self
    }

    pub fn remove(&self, index: usize) -> (Character, T) {
        self.borrow().remove(index)
    }

    /// Reindex the mapping from names to indices.
    pub fn reindex(&mut self) {
        self.borrow_mut().reindex()
    }

    /// Set the names of the vector.
    pub fn set_names_(&self, names: CowObj<Vec<OptionNA<String>>>) {
        let new_repr = self.borrow().materialize().set_names(names);
        self.0.replace(new_repr);
    }

    pub fn is_named(&self) -> bool {
        match self.borrow().clone() {
            RepType::Subset(.., Some(_)) => true,
            _ => false,
        }
    }

    pub fn names(&self) -> Option<CowObj<Vec<Character>>> {
        match self.borrow().clone() {
            RepType::Subset(_, s, n) => {
                if s.is_empty() {
                    if let Option::Some(n) = n {
                        Option::Some(n.clone().names)
                    } else {
                        Option::None
                    }
                } else {
                    unimplemented!()
                }
            }
        }
    }

    pub fn dedup_last(self) -> Self {
        self.0.into_inner().dedup_last().into()
    }

    /// Preallocates a
    pub fn with_capacity(n: usize, names: bool) -> Self {
        let naming = if names {
            Some(Naming::with_capacity(n))
        } else {
            None
        };
        Self(RefCell::new(RepType::Subset(
            CowObj::from(Vec::with_capacity(n)),
            Subsets::default(),
            naming,
        )))
    }

    /// Get mutable access to the internal vector through the passed closure.
    pub fn with_inner_mut<F, R>(&self, f: F) -> R
    where
        F: FnOnce(&mut Vec<T>) -> R,
    {
        self.0.borrow().with_inner_mut(f)
    }

    /// Iterates over owned (name, value) tuples.
    pub fn with_pairs<F, R>(&self, f: F)
    where
        F: FnMut(Character, T) -> R,
    {
        self.borrow().with_pairs(f)
    }

    /// Iterates over owned (name, value) tuples.
    pub fn with_iter_pairs<F, R>(&self, f: F) -> R
    where
        F: FnMut(Box<dyn Iterator<Item = (Character, T)>>) -> R,
    {
        todo!()
    }

    pub fn iter_pairs(&self) -> RepTypeSubsetIterPairs<T> {
        let x = self.borrow().clone();

        match x.clone() {
            RepType::Subset(values, _, maybe_naming) => {
                let index_iter = x.iter_subset_indices().map(|(_, i)| i);
                let names = maybe_naming.map(|x| x.names.inner_rc());

                RepTypeSubsetIterPairs {
                    values: values.inner_rc(),
                    names,
                    iter: Box::new(index_iter),
                }
            }
        }
        // // FIXME: This should probably return an Option where None is returned in case there are no names.
        // let x = self.borrow().clone();
        // x.iter_pairs()
    }

    // pub fn iter_subset_indices(&self) -> Box<dyn Iterator<Item = (usize, Option<usize>)>> {
    //     todo!()
    // }

    // /// Iterate over mutable references to the values of the vector by passing a closure.
    // /// The current subsets of the vector are represented.
    // /// This method should be used for vectorized assignment.
    // ///
    // /// It is
    // /// Due to the complex structure of this struct it is not possible to return an iterator that yields
    // /// mutable references to the values of the struct.
    // pub fn with_iter_pairs_mut_ref<'a, F, R>(&'a self, f: F) -> R
    // where
    //      F: FnMut(&mut [T], Option<&mut [Character]>, Box<dyn Iterator<Item = Option<usize>>>) -> R,
    // {
    //     // We cannot easily get an Iterator that yields &mut T, even through a closure.
    //     // This is because we might iterate over the same value multiple times (consider a subset x[c(1, 1)])
    //     // two consecutive calls to .next() might yield the same mutable reference which is illegal.
    //     // Instead we give acces to &mut [T] and &mut [Character] and the index iterator
    //     //
    //     // Maybe this is possible somehow but I am not sure how to satisfy the rust compiler.

    //      when we cannot ensure that each index
    //     for x in self.iter_mut() {}
    //     // FIXME: This is impossible I think.
    //     // It would be possible to pass a closure that receives |&mut T|
    //     // FIXME: I don't think this would be
    //     let iter = todo!();
    //     f(iter)
    // }

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

    pub fn get_inner(&self, index: usize) -> Option<T> {
        self.borrow().get_inner(index)
    }

    pub fn get_inner_named(&self, index: usize) -> Option<(Character, T)> {
        // TODO: I don't think this is really needed.
        self.borrow().get_inner_named(index)
    }

    pub fn values(&self) -> CowObj<Vec<T>> {
        self.materialize_inplace();
        match self.0.borrow().clone() {
            RepType::Subset(values, ..) => values,
        }
    }

    pub fn push_named(&self, name: OptionNA<String>, value: T) {
        self.borrow().push_named(name, value)
    }

    pub fn assign(&mut self, value: Self) -> Self {
        // TODO: Handle names here
        // The assign method from List had a lot more code,
        // check that everything is covered here.
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

    pub fn iter(&self) -> RepIter<T> {
        self.clone().into_iter()
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

pub struct RepIter<T: Clone>(RepTypeIter<T>);

impl<T> IntoIterator for Rep<T>
where
    T: Clone + Default,
{
    type Item = T;
    type IntoIter = RepIter<T>;
    fn into_iter(self) -> Self::IntoIter {
        let x = self.0.into_inner();
        RepIter(x.into_iter())
    }
}

impl<T> Iterator for RepIter<T>
where
    T: Clone + Default,
{
    type Item = T;
    fn next(&mut self) -> Option<Self::Item> {
        self.0.next()
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
