use std::fmt::Debug;
use std::fmt::Display;
use std::iter::repeat;

use super::coercion::{AtomicMode, CoercibleInto, CommonCmp, CommonNum, MinimallyNumeric};
use super::subset::Subset;
use super::subsets::Subsets;
use super::types::*;
use super::{OptionNA, Pow, VecPartialCmp};
use crate::error::Error;
use crate::lang::Signal;
use crate::object::{CowObj, ViewMut};
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
    /// Create an empty `Naming`
    pub fn new() -> Self {
        Naming::default()
    }

    /// Create a naming with the given `capacity`.
    pub fn with_capacity(capacity: usize) -> Self {
        Self {
            map: HashMap::<String, Vec<usize>>::with_capacity(capacity).into(),
            names: CowObj::from(Vec::<Character>::with_capacity(capacity)),
        }
    }

    /// Push a new name onto the `Naming`.
    pub fn push_name(&self, name: OptionNA<String>) {
        self.names.with_inner_mut(|v| v.push(name.clone()));
        if let OptionNA::Some(name) = name {
            let n = self.names.len() - 1;
            self.map.with_inner_mut(|map| {
                let indices = map.entry(name.clone()).or_default();
                if !indices.contains(&n) {
                    indices.push(n);
                };
            });
        };
    }

    /// Get mutable access to the internal data (map and names vector) via the passed closure.
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

        value.iter().enumerate().for_each(|(i, maybe_name)| {
            if let OptionNA::Some(name) = maybe_name {
                let indices = map.entry(name.clone()).or_default();
                if !indices.contains(&i) {
                    indices.push(i);
                };
            };
        });

        Self { map: map.into(), names: value }
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
            RepType::Subset(v, s, n) => RepType::Subset(v.clone(), s.clone(), n.clone()),
        }
    }
}

impl<T: Clone + Default> Default for RepType<T> {
    fn default() -> Self {
        Self::new()
    }
}

impl<T: Clone + Default + ViewMut> RepType<T> {
    /// Get a cloned version of the inner value.
    /// This is used for accessing inner values like `list(1)[[1]]`.
    pub fn try_get_inner(&self, subset: Subset) -> Result<T, Signal> {
        #[allow(clippy::map_clone)]
        self.try_get_inner_mut(subset).map(|x| x.clone())
    }
    /// Retrieve the internal data as a mutable view.
    /// This is important for lists for things like `l$a[1:2] = c(10, 11)`
    pub fn try_get_inner_mut(&self, subset: Subset) -> Result<T, Signal> {
        let new_subset = self.subset(subset);
        match new_subset {
            RepType::Subset(..) => {
                let mut iter = new_subset.iter_subset_indices();

                if let Some(i) = iter.next() {
                    if iter.next().is_some() {
                        return Error::Other("subset has length > 1".to_string()).into();
                    }

                    // TODO: subsetting with NA should not be possible.
                    let i = i.unwrap();

                    Ok(self.with_inner_mut(|values| values[i].view_mut()))
                } else {
                    Error::Other("subset is empty".to_string()).into()
                }
            }
        }
    }
}

pub struct IntoIterableRefNames {
    names: Rc<Vec<Character>>,
    na_name: Character,
    iter: Box<dyn Iterator<Item = Option<usize>>>,
}

pub struct RepTypeIterableNames<'a> {
    names: &'a [Character],
    na_name: &'a Character,
    iter: &'a mut Box<dyn Iterator<Item = Option<usize>>>,
}

impl IntoIterableRefNames {
    pub fn iter(&mut self) -> RepTypeIterableNames<'_> {
        let names = &self.names[..];
        RepTypeIterableNames {
            names,
            na_name: &self.na_name,
            iter: &mut self.iter,
        }
    }
}

impl<'a> Iterator for RepTypeIterableNames<'a> {
    type Item = &'a Character;

    fn next(&mut self) -> Option<Self::Item> {
        if let Some(i) = self.iter.next()? {
            Some(&self.names[i])
        } else {
            Some(self.na_name)
        }
    }
}

pub struct IntoIterableRefValues<T: Clone> {
    values: Rc<Vec<T>>,
    na_value: T,
    iter: Box<dyn Iterator<Item = Option<usize>>>,
}

impl<T: Clone + Default> IntoIterableRefValues<T> {
    pub fn iter(&mut self) -> IterableRefValues<'_, T> {
        let values = &self.values[..];

        IterableRefValues {
            values,
            na_value: &self.na_value,
            iter: &mut self.iter,
        }
    }
}

pub struct IntoIterableRefPairs<T: Clone> {
    values: Rc<Vec<T>>,
    names: Option<Rc<Vec<Character>>>,
    na_value: T,
    na_name: Character,
    iter: Box<dyn Iterator<Item = Option<usize>>>,
}

impl<T: Clone + Default> IntoIterableRefPairs<T> {
    pub fn iter(&mut self) -> IterableRefPairs<'_, T> {
        let values = &self.values[..];

        let names = self.names.as_ref().map(|names| &names[..]);

        IterableRefPairs {
            values,
            names,
            na_value: &self.na_value,
            na_name: &self.na_name,
            iter: &mut self.iter,
        }
    }
}

pub struct IterableRefValues<'a, T: Clone> {
    values: &'a [T],
    na_value: &'a T,
    iter: &'a mut Box<dyn Iterator<Item = Option<usize>>>,
}

pub struct IterableRefPairs<'a, T: Clone> {
    values: &'a [T],
    names: Option<&'a [Character]>,
    na_value: &'a T,
    na_name: &'a Character,
    iter: &'a mut Box<dyn Iterator<Item = Option<usize>>>,
}

impl<'a, T: Clone> Iterator for IterableRefPairs<'a, T> {
    type Item = (&'a Character, &'a T);

    fn next(&mut self) -> Option<Self::Item> {
        if let Some(i) = self.iter.next()? {
            if let Some(names) = self.names {
                Option::Some((&names[i], &self.values[i]))
            } else {
                Option::Some((self.na_name, &self.values[i]))
            }
        } else {
            Option::Some((self.na_name, self.na_value))
        }
    }
}

impl<'a, T: Clone> Iterator for IterableRefValues<'a, T> {
    type Item = &'a T;

    fn next(&mut self) -> Option<Self::Item> {
        if let Some(i) = self.iter.next()? {
            Some(&self.values[i])
        } else {
            Some(self.na_value)
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

pub struct IterableValues<T: Clone> {
    values: Rc<Vec<T>>,
    iter: Box<dyn Iterator<Item = Option<usize>>>,
}

impl<T: Clone> Iterator for IterableValues<T> {
    type Item = T;
    fn next(&mut self) -> Option<Self::Item> {
        // FIXME: Already assumes no indexing with NA
        let i = self.iter.next()?.unwrap();
        Some(self.values[i].clone())
    }
}

pub struct IterablePairs<T> {
    values: Rc<Vec<T>>,
    names: Option<Rc<Vec<Character>>>,
    iter: Box<dyn Iterator<Item = Option<usize>>>,
}

impl<T: Clone> Iterator for IterablePairs<T> {
    type Item = (Character, T);
    fn next(&mut self) -> Option<Self::Item> {
        // FIXME: Already assumes no indexing with NA
        let i = self.iter.next()?.unwrap();
        let value = self.values[i].clone();
        let name = if let Some(names) = &self.names {
            names[i].clone()
        } else {
            Character::NA
        };
        Some((name, value))
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

    /// Whether the vector representation has names.
    pub fn is_named(&self) -> bool {
        matches!(self, RepType::Subset(.., Some(_)))
    }

    /// Return the names of the vector if there are any.
    pub fn names(&self) -> Option<CowObj<Vec<Character>>> {
        match self.clone() {
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

    // fn materialize_inplace(&self) -> &Self {
    //     // TODO: Rewrite this to avoid copying unnecessarily
    //     let new_repr = { self.borrow().materialize() };
    //     self.0.replace(new_repr);

    //     self
    // }

    pub fn set_subset(&mut self, subset: Subset, value: T) -> Result<T, Signal> {
        match &self {
            RepType::Subset(..) => {
                let err = Error::Other("subset must have length 1".to_string());

                let mut iter = self.clone().subset(subset).iter_subset_indices();
                let i1 = iter.next();

                // check that subset has exactly length 1
                // assumes no indexing with NA (unwrap the option)
                let i = if let Some(i) = i1 {
                    if iter.next().is_some() {
                        return err.into();
                    }
                    i
                } else {
                    return err.into();
                }
                .unwrap();

                self.with_inner_mut(|v| v[i] = value.clone());
                Ok(value.clone())
            }
        }
    }

    pub fn values_ref(&self) -> IntoIterableRefValues<T> {
        match self.clone() {
            RepType::Subset(values, ..) => {
                let iter = Box::new(self.iter_subset_indices());
                for x in iter {
                    dbg!(&x);
                }
                let iter = Box::new(self.iter_subset_indices());
                let values = values.inner_rc();

                IntoIterableRefValues { values, na_value: T::default(), iter }
            }
        }
    }

    pub fn names_ref(&self) -> Option<IntoIterableRefNames> {
        match self.clone() {
            RepType::Subset(.., naming) => {
                let iter = Box::new(self.iter_subset_indices());
                let naming = naming?;
                let names = naming.names.inner_rc();

                Some(IntoIterableRefNames { names, na_name: Character::default(), iter })
            }
        }
    }

    pub fn pairs_ref(&self) -> IntoIterableRefPairs<T> {
        match self.clone() {
            RepType::Subset(values, _, maybe_naming) => {
                let iter = Box::new(self.iter_subset_indices());
                let values = values.inner_rc();
                let names = maybe_naming.map(|x| x.names.inner_rc());

                IntoIterableRefPairs {
                    values,
                    names,
                    na_value: T::default(),
                    na_name: Character::NA,
                    iter,
                }
            }
        }
    }

    pub fn iter_pairs(&self) -> IterablePairs<T> {
        match self.clone() {
            RepType::Subset(values, _, maybe_naming) => {
                let iter = Box::new(self.iter_subset_indices());
                let values = values.inner_rc();
                let names = maybe_naming.map(|x| x.names.inner_rc());

                IterablePairs { values, names, iter }
            }
        }
    }

    pub fn iter_values(&self) -> IterableValues<T> {
        match self.clone() {
            RepType::Subset(values, ..) => {
                let iter = Box::new(self.iter_subset_indices());
                IterableValues { values: values.inner_rc(), iter }
            }
        }
    }

    pub fn iter_names(&self) -> Option<IterableValues<Character>> {
        match self.clone() {
            RepType::Subset(.., maybe_naming) => {
                let iter = Box::new(self.iter_subset_indices());
                let names = maybe_naming.map(|x| x.names.inner_rc())?;

                Some(IterableValues { values: names, iter })
            }
        }
    }

    pub fn push_value(&self, value: T) {
        self.push_named(Character::NA, value);
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

    pub fn iter_subset_indices_exact(&self) -> ExactIterSubsetIndices {
        // TODO(performance): Avoid the vector allocation
        let iter = self.iter_subset_indices();
        let len = iter.count();
        let iter = self.iter_subset_indices();
        ExactIterSubsetIndices { iter, len }
    }

    pub fn iter_subset_indices(&self) -> Box<dyn Iterator<Item = Option<usize>>> {
        match self.clone() {
            RepType::Subset(vals, subsets, maybe_naming) => {
                if subsets.is_empty() {
                    return Box::new((0_usize..vals.len()).map(Some));
                }

                if let Some(naming) = maybe_naming {
                    Box::new(subsets.bind_names(naming.map).into_iter().map(|(_, y)| y))
                } else {
                    Box::new(subsets.into_iter().map(|(_, y)| y))
                }
            }
        }
    }

    /// Reindex the mapping from names to indices.
    pub fn reindex(&mut self) {
        if let RepType::Subset(.., Some(naming)) = self {
            naming.map.with_inner_mut(|map| {
                map.drain();

                for (i, maybe_name) in naming.names.borrow().iter().enumerate() {
                    if let OptionNA::Some(name) = maybe_name {
                        let indices = map.entry(name.clone()).or_default();
                        if !indices.contains(&i) {
                            indices.push(i)
                        }
                    }
                }
            })
        }
    }

    pub fn dedup_last(self) -> Self {
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
                        indices.drain(0..(indices.len()));
                    }
                });
                RepType::Subset(values, subsets, Some(naming))
            }
            RepType::Subset(.., None) => self,
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
                _ => {
                    dbg!(&s[0]);
                    self.values_ref().iter().count(); // _ => self.materialize_inplace().len(),
                    println!("Wupsie");
                    todo!()
                }
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
    pub fn assign<R>(&mut self, value: RepType<R>) -> Result<Self, Signal>
    where
        T: Clone + Default + From<R>,
        R: Default + Clone,
    {
        let l_indices = self.iter_subset_indices_exact();
        let mut r_indices = value.iter_subset_indices_exact();

        // TODO(performance): When we clone the interior data of self (to which we write)
        // we don't have to perform recycling checks
        // and just start iterating. We can always discard the result afterwards again
        // Maybe implement filter_exact on (named)subsets
        if r_indices.len() == 1 {
            // get the element from reptype value
            let index = r_indices
                .next()
                .expect("index should exist")
                .expect("No NA for subsetting");
            let elem = value.get_inner(index).expect("element should exist");
            match (self, value) {
                (RepType::Subset(lv, ls, ln), RepType::Subset(..)) => {
                    lv.with_inner_mut(|lvb| {
                        for li in l_indices {
                            lvb[li.unwrap()] = elem.clone().into();
                        }
                    });
                    return Ok(RepType::Subset(lv.clone(), ls.clone(), ln.clone()));
                }
            }
        }

        if l_indices.len() != r_indices.len() {
            return Err(Signal::Error(Error::NonRecyclableLengths(
                l_indices.len(),
                r_indices.len(),
            )));
        }

        match (self, value) {
            (RepType::Subset(lv, ls, ln), RepType::Subset(rv, ..)) => {
                lv.with_inner_mut(|lvb| {
                    let rvc = rv.clone();
                    let rvb = rvc.borrow();

                    for (li, ri) in l_indices.zip(r_indices) {
                        match (li, ri) {
                            (Some(li), None) => lvb[li] = T::default(),
                            (Some(li), Some(ri)) => lvb[li] = rvb[ri % rvb.len()].clone().into(),
                            _ => (),
                        }
                    }
                });

                Ok(RepType::Subset(lv.clone(), ls.clone(), ln.clone()))
            }
        }
    }

    // implement materialize_inplace
    fn materialize_inplace(&mut self) {
        *self = self.materialize();
    }

    /// Return the only value if the vector has length 1.
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

    pub fn get_inner(&self, index: usize) -> Option<T> {
        match self {
            RepType::Subset(v, subsets, maybe_naming) => {
                if maybe_naming.is_some() {
                    // TODO(NOW)
                    unimplemented!()
                }
                let vb = v.borrow();
                let index = subsets.get_index_at(index)?;
                vb.get(index).cloned()
            }
        }
    }
}

pub struct ExactIterSubsetIndices {
    iter: Box<dyn Iterator<Item = Option<usize>>>,
    len: usize,
}

impl ExactSizeIterator for ExactIterSubsetIndices {
    fn len(&self) -> usize {
        self.len
    }
}

impl Iterator for ExactIterSubsetIndices {
    type Item = Option<usize>;
    fn next(&mut self) -> Option<Self::Item> {
        self.iter.next()
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
            names.push(k.map_or(Character::NA, Character::Some));
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

impl<T> Display for RepType<T>
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

impl<L, LNum, O> std::ops::Neg for RepType<L>
where
    L: AtomicMode + Default + Clone + MinimallyNumeric<As = LNum> + CoercibleInto<LNum>,
    LNum: std::ops::Neg<Output = O>,
    RepType<O>: From<Vec<O>>,
    O: Clone,
{
    type Output = Result<RepType<O>, Signal>;
    fn neg(self) -> Self::Output {
        let result: Vec<O> = self
            .iter_values()
            .map(|x| -(CoercibleInto::<LNum>::coerce_into(x)))
            .collect();
        Ok(result.into())
    }
}

impl<L, R, C, O, LNum, RNum> std::ops::Add<RepType<R>> for RepType<L>
where
    L: AtomicMode + Default + Clone + MinimallyNumeric<As = LNum> + CoercibleInto<LNum>,
    R: AtomicMode + Default + Clone + MinimallyNumeric<As = RNum> + CoercibleInto<RNum>,
    (LNum, RNum): CommonNum<Common = C>,
    C: Clone + std::ops::Add<Output = O> + Default,
    RepType<C>: From<Vec<O>>,
    O: Clone + Default,
{
    type Output = Result<RepType<C>, Signal>;
    fn add(self, rhs: RepType<R>) -> Self::Output {
        try_binary_num_op(self, rhs, |x, y| x + y)
    }
}

impl<L, R, C, O, LNum, RNum> std::ops::Sub<RepType<R>> for RepType<L>
where
    L: AtomicMode + Default + Clone + MinimallyNumeric<As = LNum> + CoercibleInto<LNum>,
    R: AtomicMode + Default + Clone + MinimallyNumeric<As = RNum> + CoercibleInto<RNum>,
    (LNum, RNum): CommonNum<Common = C>,
    C: Clone + std::ops::Sub<Output = O> + Default,
    RepType<C>: From<Vec<O>>,
    O: Clone + Default,
{
    type Output = Result<RepType<C>, Signal>;
    fn sub(self, rhs: RepType<R>) -> Self::Output {
        try_binary_num_op(self, rhs, |x, y| x - y)
    }
}

impl<L, R, C, O, LNum, RNum> std::ops::Mul<RepType<R>> for RepType<L>
where
    L: AtomicMode + Default + Clone + MinimallyNumeric<As = LNum> + CoercibleInto<LNum>,
    R: AtomicMode + Default + Clone + MinimallyNumeric<As = RNum> + CoercibleInto<RNum>,
    (LNum, RNum): CommonNum<Common = C>,
    C: Clone + std::ops::Mul<Output = O> + Default,
    RepType<C>: From<Vec<O>>,
    O: Clone + Default,
{
    type Output = Result<RepType<C>, Signal>;
    fn mul(self, rhs: RepType<R>) -> Self::Output {
        try_binary_num_op(self, rhs, |x, y| x * y)
    }
}

impl<L, R, C, O, LNum, RNum> std::ops::Div<RepType<R>> for RepType<L>
where
    L: AtomicMode + Default + Clone + MinimallyNumeric<As = LNum> + CoercibleInto<LNum>,
    R: AtomicMode + Default + Clone + MinimallyNumeric<As = RNum> + CoercibleInto<RNum>,
    (LNum, RNum): CommonNum<Common = C>,
    C: Clone + std::ops::Div<Output = O> + Default,
    RepType<C>: From<Vec<O>>,
    O: Clone + Default,
{
    type Output = Result<RepType<C>, Signal>;
    fn div(self, rhs: RepType<R>) -> Self::Output {
        try_binary_num_op(self, rhs, |x, y| x / y)
    }
}

impl<L, R, C, O, LNum, RNum> std::ops::Rem<RepType<R>> for RepType<L>
where
    L: AtomicMode + Default + Clone + MinimallyNumeric<As = LNum> + CoercibleInto<LNum>,
    R: AtomicMode + Default + Clone + MinimallyNumeric<As = RNum> + CoercibleInto<RNum>,
    (LNum, RNum): CommonNum<Common = C>,
    C: Clone + std::ops::Rem<Output = O> + Default,
    RepType<C>: From<Vec<O>>,
    O: Clone + Default,
{
    type Output = Result<RepType<C>, Signal>;
    fn rem(self, rhs: RepType<R>) -> Self::Output {
        try_binary_num_op(self, rhs, |x, y| x % y)
    }
}

impl<L, R, O, LNum, RNum> Pow<RepType<R>> for RepType<L>
where
    L: AtomicMode + Default + Clone + MinimallyNumeric<As = LNum> + CoercibleInto<LNum>,
    R: AtomicMode + Default + Clone + MinimallyNumeric<As = RNum> + CoercibleInto<RNum>,
    (LNum, RNum): CommonNum<Common = O>,
    O: Pow<O, Output = O>,
    RepType<O>: From<Vec<O>>,
    O: Default,
    L: Clone,
    R: Clone,
    O: Clone,
{
    type Output = Result<RepType<O>, Signal>;
    fn power(self, rhs: RepType<R>) -> Self::Output {
        try_binary_num_op(self, rhs, |x, y| Pow::power(x, y))
    }
}

impl<L, R> std::ops::BitOr<RepType<R>> for RepType<L>
where
    L: AtomicMode + Default + Clone + CoercibleInto<Logical>,
    R: AtomicMode + Default + Clone + CoercibleInto<Logical>,
{
    type Output = Result<RepType<Logical>, Signal>;
    fn bitor(self, rhs: RepType<R>) -> Self::Output {
        try_binary_lgl_op(self, rhs, |x, y| x | y)
    }
}

impl<L, R> std::ops::BitAnd<RepType<R>> for RepType<L>
where
    L: AtomicMode + Default + Clone + CoercibleInto<Logical>,
    R: AtomicMode + Default + Clone + CoercibleInto<Logical>,
{
    type Output = Result<RepType<Logical>, Signal>;
    fn bitand(self, rhs: RepType<R>) -> Self::Output {
        try_binary_lgl_op(self, rhs, |x, y| x & y)
    }
}

impl<L> std::ops::Not for RepType<L>
where
    L: AtomicMode + Default + Clone + CoercibleInto<Logical>,
{
    type Output = Result<RepType<Logical>, Signal>;
    fn not(self) -> Self::Output {
        let result: Vec<Logical> = self
            .iter_values()
            .map(|x| !(CoercibleInto::<Logical>::coerce_into(x)))
            .collect();
        Ok(result.into())
    }
}

impl<L, R, C> VecPartialCmp<RepType<R>> for RepType<L>
where
    L: AtomicMode + Default + Clone + CoercibleInto<C> + Clone,
    R: AtomicMode + Default + Clone + CoercibleInto<C> + Clone,
    (L, R): CommonCmp<Common = C>,
    C: PartialOrd + Clone + Default,
{
    type Output = Result<RepType<Logical>, Signal>;

    fn vec_gt(self, rhs: RepType<R>) -> Self::Output {
        use std::cmp::Ordering::*;
        try_binary_cmp_op(self, rhs, |i| match i {
            Some(Greater) => OptionNA::Some(true),
            Some(_) => OptionNA::Some(false),
            None => OptionNA::NA,
        })
    }

    fn vec_gte(self, rhs: RepType<R>) -> Self::Output {
        use std::cmp::Ordering::*;
        try_binary_cmp_op(self, rhs, |i| match i {
            Some(Greater | Equal) => OptionNA::Some(true),
            Some(_) => OptionNA::Some(false),
            None => OptionNA::NA,
        })
    }

    fn vec_lt(self, rhs: RepType<R>) -> Self::Output {
        use std::cmp::Ordering::*;
        try_binary_cmp_op(self, rhs, |i| match i {
            Some(Less) => OptionNA::Some(true),
            Some(_) => OptionNA::Some(false),
            None => OptionNA::NA,
        })
    }

    fn vec_lte(self, rhs: RepType<R>) -> Self::Output {
        use std::cmp::Ordering::*;
        try_binary_cmp_op(self, rhs, |i| match i {
            Some(Less | Equal) => OptionNA::Some(true),
            Some(_) => OptionNA::Some(false),
            None => OptionNA::NA,
        })
    }

    fn vec_eq(self, rhs: RepType<R>) -> Self::Output {
        use std::cmp::Ordering::*;
        try_binary_cmp_op(self, rhs, |i| match i {
            Some(Equal) => OptionNA::Some(true),
            Some(_) => OptionNA::Some(false),
            None => OptionNA::NA,
        })
    }

    fn vec_neq(self, rhs: RepType<R>) -> Self::Output {
        use std::cmp::Ordering::*;
        try_binary_cmp_op(self, rhs, |i| match i {
            Some(Equal) => OptionNA::Some(false),
            Some(_) => OptionNA::Some(true),
            None => OptionNA::NA,
        })
    }
}

/// This function applies a function `g` to pairs from lhs and rhs.
/// The function returns an error when the lengths are not compatible.
fn try_recycle_then<L, R, O, F, A>(
    lhs: RepType<L>,
    rhs: RepType<R>,
    g: F,
) -> Result<RepType<A>, Signal>
where
    L: Clone + Default,
    R: Clone + Default,
    RepType<A>: From<Vec<O>>,
    O: Clone + Default,
    A: Clone,
    F: Fn(L, R) -> O,
{
    match (lhs.as_scalar(), rhs.as_scalar()) {
        (Some(l), Some(r)) => {
            let result: Vec<O> = vec![g(l, r)];
            Ok(RepType::from(result))
        }
        (Some(l), None) => {
            let result: Vec<O> = repeat(l)
                .zip(rhs.iter_values())
                .map(|(l, r)| g(l, r))
                .collect();
            if result.is_empty() {
                return Err(Signal::Error(Error::NonRecyclableLengths(1, 0)));
            }
            Ok(RepType::from(result))
        }
        (None, Some(r)) => {
            let result: Vec<O> = lhs
                .iter_values()
                .zip(repeat(r))
                .map(|(l, r)| g(l, r))
                .collect();
            if result.is_empty() {
                return Err(Signal::Error(Error::NonRecyclableLengths(0, 1)));
            }
            Ok(RepType::from(result))
        }
        (None, None) => {
            let mut lc = lhs.iter_values();
            let mut rc = rhs.iter_values();

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
                    }
                    (None, Some(_)) => {
                        return Err(Signal::Error(Error::NonRecyclableLengths(
                            result.len(),
                            result.len() + 1 + rc.count(),
                        )));
                    }
                    (None, None) => return Ok(RepType::from(result)),
                }
            }
        }
    }
}

fn try_binary_num_op<L, R, C, O, LNum, RNum, F>(
    lhs: RepType<L>,
    rhs: RepType<R>,
    f: F,
) -> Result<RepType<C>, Signal>
where
    L: Default + Clone + MinimallyNumeric<As = LNum> + CoercibleInto<LNum>,
    R: Default + Clone + MinimallyNumeric<As = RNum> + CoercibleInto<RNum>,
    C: Default + Clone,
    (LNum, RNum): CommonNum<Common = C>,
    RepType<C>: From<Vec<O>>,
    O: Clone + Default,
    F: Fn(C, C) -> O,
    C: Clone + Default,
{
    try_recycle_then(lhs, rhs, |x, y| {
        let (c1, c2) = (
            CoercibleInto::<LNum>::coerce_into(x),
            CoercibleInto::<RNum>::coerce_into(y),
        )
            .into_common();
        f(c1, c2)
    })
}

// FIXME(performance): equality with references for characters
fn try_binary_cmp_op<L, R, C, F>(
    lhs: RepType<L>,
    rhs: RepType<R>,
    f: F,
) -> Result<RepType<Logical>, Signal>
where
    L: AtomicMode + Default + Clone + CoercibleInto<C> + Clone,
    R: AtomicMode + Default + Clone + CoercibleInto<C> + Clone,
    (L, R): CommonCmp<Common = C>,
    C: PartialOrd + Clone + Default,
    F: Fn(Option<std::cmp::Ordering>) -> Logical,
{
    try_recycle_then(lhs, rhs, |x, y| {
        let c1: C = x.coerce_into();
        let c2: C = y.coerce_into();
        let ordering = c1.partial_cmp(&c2);
        f(ordering)
    })
}

pub fn try_binary_lgl_op<L, R, F>(
    lhs: RepType<L>,
    rhs: RepType<R>,
    f: F,
) -> Result<RepType<Logical>, Signal>
where
    L: AtomicMode + Default + Clone + CoercibleInto<Logical>,
    R: AtomicMode + Default + Clone + CoercibleInto<Logical>,
    F: Fn(Logical, Logical) -> Logical,
{
    try_recycle_then(lhs, rhs, |x, y| {
        let (c1, c2) = (
            CoercibleInto::<Logical>::coerce_into(x),
            CoercibleInto::<Logical>::coerce_into(y),
        );
        f(c1, c2)
    })
}
#[cfg(test)]
mod test {
    use super::OptionNA::*;
    use crate::object::reptype::RepType;
    use crate::object::{types::*, OptionNA, VecPartialCmp};
    use crate::r;
    use crate::utils::SameType;

    #[test]
    fn vector_add() {
        let x = RepType::<Integer>::from((1..=5).collect::<Vec<_>>());
        let y = RepType::<Integer>::from(vec![2, 5, 6, 2, 3]);

        let z = (x + y).unwrap();
        assert_eq!(z, RepType::from(vec![3, 7, 9, 6, 8]));

        let expected_type = RepType::<Integer>::new();
        assert!(z.is_same_type_as(&expected_type));
        assert!(z.is_integer());
    }

    #[test]
    fn vector_mul() {
        let x = RepType::<Integer>::from((1..=5).collect::<Vec<_>>());
        let y = RepType::<Integer>::from(vec![Some(2), NA, Some(6), NA, Some(3)]);

        let z = (x * y).unwrap();
        assert_eq!(z, RepType::from(vec![Some(2), NA, Some(18), NA, Some(15),]));

        let expected_type = RepType::<Integer>::new();
        assert!(z.is_same_type_as(&expected_type));
        assert!(z.is_integer());
    }

    #[test]
    fn vector_common_mul_f32_na() {
        // expect that f32's do not get coerced into an OptionNA:: instead
        // using std::f32::NAN as NA representation.

        let x = RepType::<Double>::from(vec![Some(0_f64), NA, Some(10_f64)]);
        let y = RepType::<Integer>::from(vec![100, 10, 1]);

        let z = (x * y).unwrap();
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

        let x = RepType::<Double>::from(vec![Some(0_f64), NA, Some(10_f64)]);
        let y = RepType::<Integer>::from(vec![100, 10, 1]);

        let z = (x & y).unwrap();
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
        let y = RepType::<Integer>::from(vec![100, 10, 1]);

        let z = x.vec_gt(y).unwrap();
        assert_eq!(z, RepType::from(vec![Some(false), NA, Some(true)]));

        let expected_type = RepType::<Logical>::new();
        assert!(z.is_same_type_as(&expected_type));
        assert!(z.is_logical());
    }

    #[test]
    fn test_iter_values() {
        // Create values as Vec<i32>
        let values = vec![1, 2, 3, 4, 5];

        // Create RepType<Integer> from values
        let rep = RepType::from(values.clone());

        // Use iter_values to get an iterator and collect values
        let collected_values: Vec<Integer> = rep.iter_values().collect();

        // Expected values as Vec<OptionNA<i32>>
        let expected_values: Vec<Integer> = values.into_iter().map(OptionNA::Some).collect();

        // Assert collected values match expected values
        assert_eq!(collected_values, expected_values);
    }

    #[test]
    fn test_iter_names() {
        // Create values with names
        let values_with_names = vec![
            (Character::Some(String::from("a")), 1),
            (Character::Some(String::from("b")), 2),
            (Character::NA, 3),
            (Character::Some(String::from("d")), 4),
            (Character::NA, 5),
        ];

        // Create RepType<Integer> from values with names
        let rep = RepType::from(values_with_names.clone());

        // Use iter_names to get an iterator
        let names_iter = rep.iter_names();

        // Ensure iter_names is Some iterator
        assert!(names_iter.is_some());

        // Collect names
        let collected_names: Vec<Character> = names_iter.unwrap().collect();

        // Expected names
        let expected_names: Vec<Character> = values_with_names
            .iter()
            .map(|(name_opt, _)| match name_opt {
                Some(name) => Character::Some(name.clone()),
                Character::NA => Character::NA,
            })
            .collect();

        // Assert collected names match expected names
        assert_eq!(collected_names, expected_names);
    }

    use crate::object::{Obj, Vector};
    // The tests below don't test the subsetting mechanism, which is instead tested in subsets.rs
    #[test]
    fn iter_pairs_mixed_names() {
        let x = r!(c(a = 1, 2)).unwrap();

        let mut x = if let Obj::Vector(Vector::Double(r)) = x {
            r.iter_pairs()
        } else {
            unreachable!()
        };

        assert_eq!(
            x.next().unwrap(),
            (Character::Some("a".to_string()), Double::Some(1.0))
        );
        assert_eq!(x.next().unwrap(), (Character::NA, Double::Some(2.0)));
        assert_eq!(x.next(), None);
    }

    #[test]
    fn iter_pairs_no_names() {
        let x = r!(c(1, 2)).unwrap();

        let mut x = if let Obj::Vector(Vector::Double(r)) = x {
            r.iter_pairs()
        } else {
            unreachable!()
        };

        assert_eq!(x.next().unwrap(), (Character::NA, Double::Some(1.0)));
        assert_eq!(x.next().unwrap(), (Character::NA, Double::Some(2.0)));
        assert_eq!(x.next(), None);
    }

    #[test]
    fn iter_values() {
        let x = r!(c(1, 2)).unwrap();

        let mut x = if let Obj::Vector(Vector::Double(r)) = x {
            r.iter_values()
        } else {
            unreachable!()
        };

        assert_eq!(x.next().unwrap(), Double::Some(1.0));
        assert_eq!(x.next().unwrap(), Double::Some(2.0));
        assert_eq!(x.next(), None);
    }

    #[test]
    fn iter_names_none() {
        let x = r!(c(1, 2)).unwrap();

        let x = if let Obj::Vector(Vector::Double(r)) = x {
            r.iter_names()
        } else {
            unreachable!()
        };

        assert!(x.is_none())
    }

    #[test]
    fn iter_names_some() {
        let x = r!(c(1, b = 2)).unwrap();

        let mut x = if let Obj::Vector(Vector::Double(r)) = x {
            r.iter_names().unwrap()
        } else {
            unreachable!()
        };

        assert_eq!(x.next().unwrap(), Character::NA);
        assert_eq!(x.next().unwrap(), Character::Some("b".to_string()));
        assert_eq!(x.next(), None);
    }

    #[test]
    fn names_ref_iter_some() {
        let x = r!(c(1, b = 2)).unwrap();

        let mut x = if let Obj::Vector(Vector::Double(r)) = x {
            r.names_ref().unwrap()
        } else {
            unreachable!()
        };

        let mut x = x.iter();

        assert_eq!(x.next().unwrap(), &Character::NA);
        assert_eq!(x.next().unwrap(), &Character::Some("b".to_string()));
        assert_eq!(x.next(), None);
    }

    #[test]
    #[should_panic]
    fn names_ref_iter_none() {
        let x = r!(c(1, 2)).unwrap();

        if let Obj::Vector(Vector::Double(r)) = x {
            r.names_ref().unwrap()
        } else {
            unreachable!()
        };
    }

    #[test]
    fn values_ref_iter() {
        let x = r!(c(1, b = 2)).unwrap();

        let mut x = if let Obj::Vector(Vector::Double(r)) = x {
            r.values_ref()
        } else {
            unreachable!()
        };

        let mut x = x.iter();

        assert_eq!(x.next().unwrap(), &Double::Some(1.0));
        assert_eq!(x.next().unwrap(), &Double::Some(2.0));
        assert_eq!(x.next(), None);
    }

    #[test]
    fn pairs_ref_iter() {
        let x = r!(c(1, b = 2)).unwrap();

        let mut x = if let Obj::Vector(Vector::Double(r)) = x {
            r.pairs_ref()
        } else {
            unreachable!()
        };

        let mut x = x.iter();

        assert_eq!(x.next().unwrap(), (&Character::NA, &Double::Some(1.0)));
        assert_eq!(
            x.next().unwrap(),
            (&Character::Some("b".to_string()), &Double::Some(2.0))
        );
        assert_eq!(x.next(), None);
    }

    use crate::error::Error;
    use crate::lang::Signal;

    #[test]
    fn assign_recycle_incompatible() {
        let mut x = RepType::<Integer>::from(vec![1, 2, 3]);
        let y = RepType::<Integer>::from(vec![99, 99]);
        let result = x.assign(y);
        assert_eq!(
            result.unwrap_err(),
            Signal::Error(Error::NonRecyclableLengths(3, 2))
        );
    }
    #[test]
    fn assign_recycle_length_one() {
        let x = RepType::<Integer>::from(vec![1, 2, 3]);
        let y = RepType::<Integer>::from(vec![99]);
        let mut xview = x.subset(vec![0, 1].into());
        let _ = xview.assign(y).unwrap();
        let result_vec: Vec<_> = x.iter_values().collect();
        assert_eq!(result_vec, vec![Some(99), Some(99), Some(3)])
    }
    #[test]
    fn non_recyclable_lengths_3_2() {
        let x = RepType::<Integer>::from(vec![1, 2, 3]);
        let y = RepType::<Integer>::from(vec![99, 99]);
        let result = x + y;
        assert_eq!(
            result.unwrap_err(),
            Signal::Error(Error::NonRecyclableLengths(3, 2))
        );
    }
    #[test]
    fn non_recyclable_lengths_4_2() {
        let x = RepType::<Integer>::from(vec![1, 2, 3, 4]);
        let y = RepType::<Integer>::from(vec![99, 99]);
        let result = x + y;
        assert_eq!(
            result.unwrap_err(),
            Signal::Error(Error::NonRecyclableLengths(4, 2))
        );
    }
    #[test]
    fn non_recyclable_lengths_2_3() {
        let x = RepType::<Integer>::from(vec![1, 2]);
        let y = RepType::<Integer>::from(vec![99, 99, 99]);
        let result = x + y;
        assert_eq!(
            result.unwrap_err(),
            Signal::Error(Error::NonRecyclableLengths(2, 3))
        );
    }
    #[test]
    fn non_recyclable_lengths_2_4() {
        let x = RepType::<Integer>::from(vec![1, 2]);
        let y = RepType::<Integer>::from(vec![99, 99, 99, 99]);
        let result = x + y;
        assert_eq!(
            result.unwrap_err(),
            Signal::Error(Error::NonRecyclableLengths(2, 4))
        );
    }
    #[test]
    fn non_recyclable_lengths_0_1() {
        let x = RepType::<Integer>::from(Vec::<Integer>::new());
        let y = RepType::<Integer>::from(vec![99]);
        let result = x + y;
        assert_eq!(
            result.unwrap_err(),
            Signal::Error(Error::NonRecyclableLengths(0, 1))
        );
    }
    #[test]
    fn non_recyclable_lengths_1_0() {
        let x = RepType::<Integer>::from(vec![99]);
        let y = RepType::<Integer>::from(Vec::<Integer>::new());
        let result = x + y;
        assert_eq!(
            result.unwrap_err(),
            Signal::Error(Error::NonRecyclableLengths(1, 0))
        );
    }
}
