use std::fmt::Debug;

use super::coercion::{AtomicMode, CoercibleInto, CommonCmp, CommonNum, MinimallyNumeric};
use super::iterators::{map_common_numeric, zip_recycle};
use super::subset::Subset;
use super::subsets::Subsets;
use super::types::*;
use super::{OptionNA, Pow, VecPartialCmp};
use crate::object::{CowObj, Obj, ViewMut};
use hashbrown::HashMap;
use std::cell::RefCell;
use std::rc::Rc;

#[derive(Debug, Clone, PartialEq, Default)]
pub struct Naming {
    // TODO: change this to usize and not Vec<usize> (after making names unique)
    pub map: CowObj<HashMap<String, Vec<usize>>>,
    pub names: CowObj<Vec<OptionNA<String>>>,
}

impl Naming {
    pub fn new() -> Self {
        Naming::default()
    }

    pub fn remove(&self, index: usize) -> Character {
        // FIXME: Already assumes names are unique
        let maybe_name = self.names.with_inner_mut(|names| names.remove(index));
        if let OptionNA::Some(name) = maybe_name {
            self.map.with_inner_mut(|map| map.remove(&name));
            OptionNA::Some(name)
        } else {
            OptionNA::NA
        }
    }

    pub fn push_name(&self, name: OptionNA<String>) {
        self.names.with_inner_mut(|v| v.push(name.clone()));
        if let OptionNA::Some(name) = name {
            let n = self.names.len();
            self.map.with_inner_mut(|map| {
                let indices = map.entry(name.clone()).or_default();
                if !indices.contains(&n) {
                    indices.push(n);
                };
            });
        };
    }

    /// Get mutable access to the internal vector through the passed closure.
    pub fn with_inner_mut<F, R>(&self, f: F) -> R
    where
        F: FnOnce(&mut HashMap<String, Vec<usize>>, &mut Vec<OptionNA<String>>) -> R,
    {
        self.map
            .with_inner_mut(|map| self.names.with_inner_mut(|names| f(map, names)))
    }
}

impl<T: Clone + Default> From<Vec<(Character, T)>> for RepType<T> {
    fn from(value: Vec<(Character, T)>) -> Self {
        let mut names = Vec::with_capacity(value.len());
        let mut values = Vec::with_capacity(value.len());
        for (k, v) in value {
            names.push(k);
            values.push(v);
        }

        RepType::Subset(
            CowObj::new(Rc::new(RefCell::new(Rc::new(values)))),
            Subsets::default(),
            Option::Some(Naming::from(names)),
        )
    }
}

impl From<CowObj<Vec<Character>>> for Naming {
    fn from(value: CowObj<Vec<Character>>) -> Self {
        let mut map: HashMap<String, Vec<usize>> = HashMap::new();

        value.iter().enumerate().map(|(i, maybe_name)| {
            if let OptionNA::Some(name) = maybe_name {
                let indices = map.entry(name.clone()).or_default();
                if !indices.contains(&i) {
                    indices.push(i);
                };
            };
        });

        Self {
            map: map.into(),
            names: value,
        }
    }
}

/// Vector
#[derive(Debug, PartialEq)]
pub enum RepType<T: Clone> {
    // Vector::Subset encompasses a "raw" vector (no subsetting)
    Subset(CowObj<Vec<T>>, Subsets, Option<Naming>),
    // Iterator includes things like ranges 1:Inf, and lazily computed values
    // Iter(Box<dyn Iterator<Item = &T>>)
}

impl<T: Clone> Clone for RepType<T> {
    fn clone(&self) -> Self {
        match self {
            // FIXME: should this reall call .view_mut()? should add comment
            RepType::Subset(v, s, n) => RepType::Subset(v.view_mut(), s.clone(), n.clone()),
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
    pub fn try_get_inner_mut(&self, index: usize) -> Option<T> {
        match self {
            RepType::Subset(v, subsets, _) => {
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

// FIXME: Don't make this an enum but simply different structs for eah reptype.
// We yield Box<dyn Iterators> anyway.
pub enum RepTypeIter<T: Clone> {
    SubsetIter(RepType<T>, usize, usize),
}

pub struct RepTypeIterPairs<T: Clone> {
    x: RepType<T>,
    index: usize,
    length: usize,
}

// FIXME: This will be extremely inefficient for Named subsets so optimize this.
impl<T: Clone + Default> Iterator for RepTypeIterPairs<T> {
    type Item = (Character, T);
    fn next(&mut self) -> Option<Self::Item> {
        if self.index < self.length {
            let x = self.x.get_inner_named(self.index);
            self.index += 1;
            x
        } else {
            None
        }
    }
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
            RepType::Subset(v, s, n) => RepType::Subset(v.view_mut(), s.clone(), n.clone()),
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
        RepType::Subset(
            Vec::new().into(),
            Subsets(Vec::new()),
            Some(Naming::default()),
        )
    }

    /// Preallocates a RepType with the given capacity.
    pub fn with_capacity(n: usize) {
        todo!()
    }

    pub fn ensure_named(&self) {
        todo!()
    }

    pub fn with_iter_pairs<F, R>(&self, f: F)
    where
        F: FnMut(Box<dyn Iterator<Item = (Character, T)>>) -> R,
    {
        todo!()
    }

    /// Iterates over owned (name, value) tuples.
    pub fn with_pairs<F, R>(&self, f: F)
    where
        F: FnMut(Character, T) -> R,
    {
        todo!()
    }

    /// Iterates over owned (name, value) tuples.
    pub fn with_iter_pairs_mut<F, R>(&self, f: F)
    where
        F: Fn(&mut Character, &mut T) -> R,
    {
        todo!()
    }

    /// Iterates over name, value pairs
    pub fn with_iter_values<F, R>(&self, f: F)
    where
        F: Fn(&T) -> R,
    {
        todo!()
    }

    /// Iterates over name, value pairs
    pub fn with_iter_values_mut<F, R>(&self, f: F)
    where
        F: Fn(&T) -> R,
    {
        todo!()
    }

    /// Iterates over name, value pairs
    pub fn with_iter_names<F, R>(&self, f: F)
    where
        F: Fn(&Character) -> R,
    {
        todo!()
    }

    pub fn with_iter_ref<F, R>(&self, f: F)
    where
        F: FnOnce(&T) -> R,
    {
        todo!()
    }

    pub fn with_iter_ref_mut<F, R>(&self, f: F)
    where
        F: FnOnce(&T) -> R,
    {
        todo!()
    }

    pub fn push_value(&self, value: T) {
        self.push_named(Character::NA, value);
    }

    pub fn remove(&self, index: usize) -> (Character, T) {
        match self {
            RepType::Subset(values, Subsets(subsets), maybe_naming) => {
                if let [] = subsets.as_slice() {
                    let value = values.with_inner_mut(|values| values.remove(index));

                    let name = if let Some(naming) = maybe_naming {
                        naming.remove(index)
                    } else {
                        OptionNA::NA
                    };
                    return (name, value);
                } else {
                    unimplemented!()
                }
            }
        }
    }

    pub fn push_named(&self, name: OptionNA<String>, value: T) {
        match self {
            RepType::Subset(values, Subsets(subsets), maybe_naming) => match subsets.as_slice() {
                [] => {
                    values.with_inner_mut(|values| values.push(value));
                    if let Some(naming) = maybe_naming {
                        naming.push_name(name)
                    }
                }
                _ => unimplemented!(),
            },
        }
    }

    pub fn iter_values<'a>(&'a self) -> Box<dyn Iterator<Item = T> + 'a> {
        let iter = self.iter_subset_indices();
        match self.clone() {
            RepType::Subset(values, ..) => Box::new(iter.map(move |(_, i)| {
                let i = i.unwrap();
                let vb = values.borrow();
                (&vb[i]).clone()
            })),
        }
    }

    pub fn iter_names(&self) -> Option<Box<dyn Iterator<Item = Character>>> {
        let iter = self.iter_subset_indices();
        match self.clone() {
            RepType::Subset(.., None) => None,
            RepType::Subset(.., Some(naming)) => Some(Box::new(iter.map(move |(_, i)| {
                let i = i.unwrap();
                let vb = naming.names.borrow();
                (&vb[i]).clone()
            }))),
        }
    }

    // FIXME: Do we really need iter_named and iter_pairs?
    pub fn iter_named<'a>(&'a self) -> Option<Box<dyn Iterator<Item = (Character, T)> + 'a>> {
        let iter_names = self.iter_names()?;
        Some(Box::new(iter_names.zip(self.iter_values())))
    }

    pub fn iter_pairs<'a>(&'a self) -> Option<Box<dyn Iterator<Item = (Character, T)> + 'a>> {
        todo!()
    }

    // TODO(refactor): This is internal implementation detail and should not be exposed via the API.
    pub fn iter_subset_indices(&self) -> Box<dyn Iterator<Item = (usize, Option<usize>)>> {
        match self.clone() {
            RepType::Subset(_, subsets, Some(naming)) => {
                Box::new(subsets.bind_names(naming.map).into_iter())
            }
            RepType::Subset(_, subsets, None) => Box::new(subsets.into_iter()),
        }
    }

    /// Reindex the mapping from names to indices.
    pub fn reindex(&mut self) {
        match self {
            RepType::Subset(.., Some(naming)) => naming.map.with_inner_mut(|map| {
                map.drain();

                for (i, maybe_name) in naming.names.borrow().iter().enumerate() {
                    if let OptionNA::Some(name) = maybe_name {
                        let indices = map.entry(name.clone()).or_default();
                        if !indices.contains(&i) {
                            indices.push(i)
                        }
                    }
                }
            }),
            _ => (),
        }
    }

    pub fn dedup_last(self) -> Self {
        // TODO(docu): What exactly does this? I think there might still be a bug here because we drain ALL indices (?)
        match self {
            RepType::Subset(values, subsets, Some(naming)) => {
                naming.with_inner_mut(|map, names| {
                    let mut dups: Vec<usize> = map
                        .iter()
                        .flat_map(|(_, indices)| {
                            indices
                                .split_last()
                                .map_or(vec![], |(_, leading_dups)| leading_dups.to_vec())
                        })
                        .collect();

                    dups.sort();

                    values.with_inner_mut(|vs| {
                        for i in dups.into_iter().rev() {
                            vs.remove(i);
                            names.remove(i);
                        }
                    });

                    for (_, indices) in map.iter_mut() {
                        indices.drain(0..indices.len());
                    }
                });
                RepType::Subset(values, subsets, Some(naming))
            }
            RepType::Subset(.., None) => return self,
        }
    }

    pub fn set_names(&self, names: CowObj<Vec<Character>>) -> Self {
        match self {
            RepType::Subset(v, s, _) => {
                RepType::Subset(v.clone(), s.clone(), Option::Some(names.into()))
            }
        }
    }

    /// Access a lazy copy of the internal vector data
    pub fn inner(&self) -> CowObj<Vec<T>> {
        match self.materialize() {
            RepType::Subset(v, ..) => v.clone(),
        }
    }

    /// Get mutable access to the internal vector through the passed closure.
    pub fn with_inner_mut<F, R>(&self, f: F) -> R
    where
        F: FnOnce(&mut Vec<T>) -> R,
    {
        match self {
            RepType::Subset(v, ..) => v.with_inner_mut(f),
        }
    }

    /// Get mutable access to the internal vector through the passed closure.
    pub fn with_inner<F, R>(&self, f: F) -> R
    where
        F: FnOnce(&Vec<T>) -> R,
    {
        match self {
            RepType::Subset(v, ..) => v.with_inner(f),
        }
    }

    /// Subsetting a Vector
    ///
    /// Introduce a new subset into the aggregate list of subset indices.
    pub fn subset(&self, subset: Subset) -> Self {
        match self {
            RepType::Subset(v, Subsets(subsets), n) => {
                let mut subsets = subsets.clone();
                subsets.push(subset);
                RepType::Subset(v.view_mut(), Subsets(subsets), n.clone())
            }
        }
    }

    pub fn len(&self) -> usize {
        match self {
            RepType::Subset(v, Subsets(s), _) => match s.as_slice() {
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
            RepType::Subset(v, subsets, _) => {
                let vb = v.borrow();
                let index = subsets.get_index_at(index)?;
                let elem = vb.get(index)?;
                Some(RepType::Subset(
                    vec![elem.clone()].into(),
                    Subsets::new(),
                    Option::Some(Naming::new()),
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
        // TODO(feature): here we should also throw an error if the recycling rules are violated.
        // TODO(refactor): I don't think we should ever iterate over the indices.
        // Instead the API of RepType should conveniently, i.e. offer Iterators that yield mutable / immutable references to the Ts.
        let l_indices = self.iter_subset_indices();
        let r_indices = value.iter_subset_indices();
        match (self, value) {
            (RepType::Subset(lv, ls, ln), RepType::Subset(rv, rs, _)) => {
                lv.with_inner_mut(|lvb| {
                    let rvc = rv.clone();
                    let rvb = rvc.borrow();

                    let lv_len = lvb.len();

                    for ((_, li), (_, ri)) in l_indices.zip(r_indices) {
                        match (li, ri) {
                            (Some(li), None) => lvb[li] = T::default(),
                            (Some(li), Some(ri)) => lvb[li] = rvb[ri % rvb.len()].clone(),
                            _ => (),
                        }
                    }
                });

                RepType::Subset(lv.clone(), ls.clone(), ln.clone())
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
            RepType::Subset(v, subsets, naming) => {
                // early exit when there is nothing to do
                match subsets {
                    Subsets(s) => {
                        if s.as_slice().is_empty() {
                            return self.clone();
                        }
                    }
                }

                let vc = v.clone();
                let vb = vc.borrow();
                let mut res: Vec<T> = vec![];
                let vb_len = vb.len();

                let new_naming = Naming::new();

                let iter = subsets.clone().into_iter().take_while(|(i, _)| i < &vb_len);

                for (_, i) in iter {
                    match i {
                        Some(i) => {
                            res.push(vb[i].clone());
                            if let Option::Some(n) = naming {
                                new_naming.push_name(n.names.borrow()[i].clone())
                            };
                        }
                        // default is NA
                        None => {
                            res.push(T::default());
                            // When we subset with NA, there is no name for this entry;
                            new_naming.push_name(OptionNA::NA);
                        }
                    }
                }

                RepType::Subset(res.into(), Subsets(vec![]), Option::None)
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
            RepType::Subset(v, subsets, naming) => {
                let vc = v.clone();
                let vb = vc.borrow();

                let num_vec: Vec<Mode> = vb.iter().map(|i| (*i).clone().coerce_into()).collect();

                RepType::Subset(num_vec.into(), subsets.clone(), naming.clone())
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
            RepType::Subset(v, subsets, n) => {
                if n.is_some() {
                    unimplemented!()
                }
                let vb = v.borrow();
                let index = subsets.get_index_at(index)?;
                vb.get(index).cloned()
            }
        }
    }
    pub fn get_inner_named(&self, index: usize) -> Option<(OptionNA<String>, T)> {
        match &self {
            RepType::Subset(.., maybe_naming) => {
                let x = self.get_inner(index)?;
                if let Some(naming) = maybe_naming {
                    Some((
                        OptionNA::Some(naming.names.borrow().get(index).unwrap().to_string()),
                        x,
                    ))
                } else {
                    Some((OptionNA::NA, x))
                }
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

impl From<Vec<Character>> for Naming {
    fn from(value: Vec<Character>) -> Self {
        let naming = Naming::new();
        for k in value {
            naming.push_name(k);
        }
        naming
    }
}

impl<T: Clone> From<CowObj<Vec<T>>> for RepType<T> {
    fn from(value: CowObj<Vec<T>>) -> Self {
        RepType::Subset(value, Subsets::default(), Option::None)
    }
}

impl<T: Clone> From<Vec<(Option<String>, T)>> for RepType<T> {
    fn from(value: Vec<(Option<String>, T)>) -> Self {
        let mut names = Vec::with_capacity(value.len());
        let mut values = Vec::with_capacity(value.len());
        for (k, v) in value.into_iter() {
            names.push(k.map_or(Character::NA, |x| Character::Some(x)));
            values.push(v)
        }
        let naming = Naming::from(names);
        RepType::Subset(values.into(), Subsets::default(), Some(naming))
    }
}

impl From<Vec<OptionNA<f64>>> for RepType<Double> {
    fn from(value: Vec<OptionNA<f64>>) -> Self {
        let value: Vec<_> = value.into_iter().map(|i| i.coerce_into()).collect();
        RepType::Subset(value.into(), Subsets(Vec::new()), Option::None)
    }
}

impl From<Vec<f64>> for RepType<Double> {
    fn from(value: Vec<f64>) -> Self {
        let value: Vec<_> = value.into_iter().map(|i| i.coerce_into()).collect();
        RepType::Subset(value.into(), Subsets(Vec::new()), Option::None)
    }
}

impl From<Vec<OptionNA<i32>>> for RepType<Integer> {
    fn from(value: Vec<OptionNA<i32>>) -> Self {
        let value: Vec<_> = value.into_iter().map(|i| i.coerce_into()).collect();
        RepType::Subset(value.into(), Subsets(Vec::new()), Option::None)
    }
}

impl From<Vec<i32>> for RepType<Integer> {
    fn from(value: Vec<i32>) -> Self {
        let value: Vec<_> = value.into_iter().map(|i| i.coerce_into()).collect();
        RepType::Subset(value.into(), Subsets(Vec::new()), Option::None)
    }
}

impl From<Vec<OptionNA<bool>>> for RepType<Logical> {
    fn from(value: Vec<OptionNA<bool>>) -> Self {
        let value: Vec<_> = value.into_iter().map(|i| i.coerce_into()).collect();
        RepType::Subset(value.into(), Subsets(Vec::new()), Option::None)
    }
}

impl From<Vec<bool>> for RepType<Logical> {
    fn from(value: Vec<bool>) -> Self {
        let value: Vec<_> = value.into_iter().map(|i| i.coerce_into()).collect();
        RepType::Subset(value.into(), Subsets(Vec::new()), Option::None)
    }
}

impl From<Vec<OptionNA<String>>> for RepType<Character> {
    fn from(value: Vec<OptionNA<String>>) -> Self {
        let value: Vec<_> = value.into_iter().map(|i| i.coerce_into()).collect();
        RepType::Subset(value.into(), Subsets(Vec::new()), Option::None)
    }
}

impl From<Vec<String>> for RepType<Character> {
    fn from(value: Vec<String>) -> Self {
        let value: Vec<_> = value.into_iter().map(|i| i.coerce_into()).collect();
        RepType::Subset(value.into(), Subsets(Vec::new()), Option::None)
    }
}

impl<F, T> From<(Vec<F>, Subsets)> for RepType<T>
where
    RepType<T>: From<Vec<F>>,
    T: Clone,
{
    fn from(value: (Vec<F>, Subsets)) -> Self {
        match Self::from(value.0) {
            RepType::Subset(v, ..) => RepType::Subset(v, value.1, Option::None),
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
