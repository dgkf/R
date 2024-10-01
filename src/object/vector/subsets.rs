use crate::object::CowObj;
use hashbrown::HashMap;

use super::Subset;

#[derive(Debug, Clone, PartialEq, Default)]
pub struct Subsets(pub Vec<Subset>);

pub struct NamedSubsets {
    subsets: Subsets,
    names: CowObj<HashMap<String, Vec<usize>>>,
}

impl Subsets {
    pub fn new() -> Self {
        Subsets(Vec::new())
    }

    /// Get the raw index of a index applied to a subset
    ///
    /// Provided a vector with multiple subsets applied, determine which
    /// original index corresponds with the index applied to the subset.
    ///
    pub fn get_index_at(&self, mut index: usize) -> Option<usize> {
        let Subsets(subsets) = self;
        for subset in subsets.iter().rev() {
            match subset.get_index_at(index) {
                Some(i) => index = i,
                None => return None,
            }
        }
        Some(index)
    }

    pub fn len(&self) -> usize {
        self.0.len()
    }

    #[must_use]
    pub fn is_empty(&self) -> bool {
        self.len() == 0
    }

    pub fn push<T>(&mut self, subset: T)
    where
        T: Into<Subset>,
    {
        self.0.push(subset.into());
    }

    pub fn bind_names(self, names: CowObj<HashMap<String, Vec<usize>>>) -> NamedSubsets {
        NamedSubsets {
            subsets: self,
            names,
        }
    }
}

impl<T> From<Vec<T>> for Subsets
where
    T: Into<Subset>,
{
    fn from(value: Vec<T>) -> Self {
        let v: Vec<Subset> = value.into_iter().map(|i| i.into()).collect();
        Subsets(v)
    }
}

impl IntoIterator for NamedSubsets {
    type Item = (usize, Option<usize>);
    type IntoIter = Box<dyn Iterator<Item = Self::Item>>;

    fn into_iter(self) -> Self::IntoIter {
        let mut iter = Box::new((0_usize..).map(|i| (i, Some(i)))) as Self::IntoIter;
        let Subsets(subsets) = self.subsets;
        for subset in subsets {
            match subset {
                Subset::Names(names) => {
                    use super::OptionNA;
                    let snames = self.names.borrow();

                    // grab indices within subset to find first named index
                    let (_, hint_n_max) = iter.size_hint();
                    let subset_indices: Vec<_> = match hint_n_max {
                        Some(n) => iter.map(|(i, _)| i).take(n).collect(),
                        None => {
                            // figure out the absolute maximum value we may require
                            let mut n = 0_usize;
                            for name in names.borrow().iter() {
                                let OptionNA::Some(name) = name else { continue };
                                let name_max = snames
                                    .get(name)
                                    .and_then(|name| name.iter().reduce(|l, r| std::cmp::max(l, r)))
                                    .unwrap_or(&0);

                                n = std::cmp::max(n, *name_max)
                            }
                            iter.map(|(i, _)| i).take(n + 1).collect()
                        }
                    };

                    // for each name, find the first index in the subset
                    let named_indices = names
                        .borrow()
                        .iter()
                        .filter_map(|name| match name {
                            OptionNA::NA => None,
                            OptionNA::Some(name) => snames
                                .get(name)
                                .and_then(|name_indices| {
                                    for i in name_indices {
                                        if subset_indices.contains(i) {
                                            return Some(Some((*i, Some(*i))));
                                        }
                                    }
                                    None
                                })
                                .unwrap_or(None),
                        })
                        .collect::<Vec<_>>();

                    iter = Box::new(named_indices.into_iter()) as Self::IntoIter
                }
                _ => iter = subset.filter(iter),
            }
        }
        iter
    }
}

impl IntoIterator for Subsets {
    type Item = (usize, Option<usize>);
    type IntoIter = Box<dyn Iterator<Item = Self::Item>>;

    /// Convert Subsets into an iterator of indices
    ///
    /// Builds an iterator of indices from a collection of subsets. Iterators
    /// will provide the maximum number of indices, meaning that ranges
    /// and masks may be infinite.
    ///
    fn into_iter(self) -> Self::IntoIter {
        let Subsets(subsets) = self;
        let mut iter = Box::new((0_usize..).map(|i| (i, Some(i)))) as Self::IntoIter;
        for subset in subsets {
            iter = subset.filter(iter);
        }
        iter
    }
}

#[cfg(test)]
mod test {
    use std::borrow::BorrowMut;

    use crate::object::Vector;

    #[test]
    fn subset_range() {
        let x: Vector = (1..=10).collect::<Vec<_>>().into();
        let result = x.subset((2..6).into()).materialize();
        let expect = Vector::from(vec![3, 4, 5, 6]);
        assert_eq!(result, expect)
    }

    #[test]
    fn subset_sequential_indices() {
        let x: Vector = (1..=10).collect::<Vec<_>>().into();
        let result = x.subset(vec![2, 3, 4, 5].into()).materialize();
        let expect = Vector::from(vec![3, 4, 5, 6]);
        assert_eq!(result, expect)
    }

    #[test]
    fn subset_sequential_repeating_indices() {
        let x: Vector = (1..=10).collect::<Vec<_>>().into();
        let result = x.subset(vec![2, 3, 3, 3, 5, 5].into()).materialize();
        let expect = Vector::from(vec![3, 4, 4, 4, 6, 6]);
        assert_eq!(result, expect)
    }

    #[test]
    fn subset_indices_with_gap() {
        let x: Vector = (1..=10).collect::<Vec<_>>().into();
        let result = x.subset(vec![2, 8].into()).materialize();
        let expect = Vector::from(vec![3, 9]);
        assert_eq!(result, expect);
    }

    #[test]
    fn subset_empty_indices() {
        let x: Vector = (1..=10).collect::<Vec<_>>().into();
        let result = x.subset(vec![].into()).materialize();
        let expect = Vector::from(Vec::new() as Vec<i32>);
        assert_eq!(result, expect);
    }

    #[test]
    fn subset_single_index() {
        let x: Vector = (1..=10).collect::<Vec<_>>().into();
        let result = x.subset(vec![6].into()).materialize();
        let expect = Vector::from(vec![7]);
        assert_eq!(result, expect);
    }

    #[test]
    fn subset_unsorted_indices() {
        let x: Vector = (1..=10).collect::<Vec<_>>().into();
        let result = x.subset(vec![6, 2, 1, 4].into()).materialize();
        let expect = Vector::from(vec![7, 3, 2, 5]);
        assert_eq!(result, expect);
    }

    #[test]
    fn subset_repeated_indices() {
        let x: Vector = (1..=10).collect::<Vec<_>>().into();
        let result = x.subset(vec![6, 2, 6, 6].into()).materialize();
        let expect = Vector::from(vec![7, 3, 7, 7]);
        assert_eq!(result, expect);
    }

    #[test]
    fn subset_by_range() {
        let x: Vector = (1..=10).collect::<Vec<_>>().into();
        let result = x.subset((3..6).into()).materialize();
        let expect = Vector::from(vec![4, 5, 6]);
        assert_eq!(result, expect);
    }

    #[test]
    fn nested_subsets() {
        let x: Vector = (1..=10).collect::<Vec<_>>().into();
        let result = x
            .subset((3..6).into())
            .subset(vec![2, 1].into())
            .materialize();
        let expect = Vector::from(vec![6, 5]);
        assert_eq!(result, expect);
    }

    #[test]
    fn subset_assignment() {
        let x: Vector = (1..=10).collect::<Vec<_>>().into();
        let mut subset = x.subset((3..6).into()).subset(vec![2, 1].into());
        let y: Vector = vec![101, 102].into();
        let _ = subset.assign(crate::object::Obj::Vector(y));
        let expect = Vector::from(vec![1, 2, 3, 4, 102, 101, 7, 8, 9, 10]);
        assert_eq!(x, expect)
    }

    #[test]
    fn iter_named_subsets() {
        use crate::object::reptype::Naming;
        use crate::object::reptype::RepType;
        use crate::object::types::{Character, Integer, Logical};
        use crate::object::CowObj;
        use crate::object::{Subset, Subsets};
        use hashbrown::HashMap;
        use std::cell::RefCell;
        use std::rc::Rc;

        let v = CowObj::new(Rc::new(RefCell::new(Rc::new(
            vec![1, 2, 3]
                .into_iter()
                .map(|i| Integer::Some(i))
                .collect(),
        ))));
        let n = CowObj::new(Rc::new(RefCell::new(Rc::new(
            vec!["a".to_string(), "b".to_string(), "c".to_string()]
                .into_iter()
                .map(|i| Character::Some(i))
                .collect(),
        ))));
        let s_names: CowObj<Vec<Character>> = CowObj::new(Rc::new(RefCell::new(Rc::new(
            vec!["a".to_string()]
                .into_iter()
                .map(|i| Character::Some(i))
                .collect(),
        ))));
        let s_indices: CowObj<Vec<Integer>> = CowObj::new(Rc::new(RefCell::new(Rc::new(
            vec![1, 2].into_iter().map(|i| Integer::Some(i)).collect(),
        ))));
        let s_logical: CowObj<Vec<Logical>> = CowObj::new(Rc::new(RefCell::new(Rc::new(
            vec![true, true, true]
                .into_iter()
                .map(|i| Logical::Some(i))
                .collect(),
        ))));
        let mut m1: HashMap<String, Vec<usize>> = HashMap::new();
        m1.insert("a".to_string(), vec![0 as usize]);
        m1.insert("b".to_string(), vec![1 as usize]);
        m1.insert("c".to_string(), vec![2 as usize]);
        let m: CowObj<HashMap<String, Vec<usize>>> =
            CowObj::new(Rc::new(RefCell::new(Rc::new(m1))));
        let naming = Naming { map: m, names: n };
        let x = RepType::Subset(v, Subsets(vec![Subset::Indices(s_indices)]), Option::None);
        for i in x.iter_subset_indices().take(3) {
            println!("{}", i.unwrap());
        }
        // let x: CowObj::from(vec![1, 2, 3]);
        // // let x = RepType::Sub
    }

    // table:
    // -- has_names: true
    // - range:   currently unused
    // - mask:    correct
    // - names:   correct
    // - indices: false
    // -- has_names: false
    // - range:   currently unused
    // - mask:    correct
    // - names:   unimplemented
    // - indices: false
}
