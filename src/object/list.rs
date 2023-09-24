use std::rc::Rc;
use std::cell::RefCell;

use crate::error::RError;
use crate::lang::EvalResult;

use super::*;

#[derive(Debug, Clone, PartialEq, Default)]
pub struct List {
    pub values: Rc<RefCell<Vec<(Option<String>, Obj)>>>,
    pub subsets: Subsets,
}

impl From<Vec<(Option<String>, Obj)>> for List {
    fn from(value: Vec<(Option<String>, Obj)>) -> Self {
        List {
            values: Rc::new(RefCell::new(value)),
            ..Default::default()
        }
    }
}

impl List {
    pub fn subset(&self, by: Subset) -> List {
        let Subsets(mut inner) = self.subsets.clone();
        inner.push(by);
        List {
            values: self.values.clone(),
            subsets: Subsets(inner),
        }
    }

    pub fn assign(&mut self, value: Obj) -> EvalResult {
        // TODO(performance): Avoid having to split vector and collect into 
        // separate names vec for binding during subsetting. Ideally just
        // need a reference.
        let names: Vec<_> = self.values.borrow().clone().into_iter().map(|(n, _)| n).collect();
        match value {
            // remove elements from list
            Obj::Null => {
                let mut v = self.values.borrow_mut();
                let n = v.len();
                let indices = self.subsets.clone().bind_names(names).into_iter().take(n);
                for (i, _) in indices {
                    v.remove(i);
                }
                Ok(Obj::List(List {
                    values: self.values.clone(),
                    subsets: self.subsets.clone(),
                }))
            }
            // any single length R value
            any if any.len() == Some(1) => {
                let mut v = self.values.borrow_mut();
                let n = v.len();
                let indices = self.subsets.clone().bind_names(names.clone()).into_iter().take(n);

                // first check to see if we need to extend
                if let Some(max) = self.subsets.clone().bind_names(names).into_iter().map(|(i, _)| i).max() {
                    v.reserve(max.saturating_sub(n))
                }

                // then assign to indices
                for (_, i) in indices {
                    if let Some(i) = i {
                        v[i].1 = any.clone()
                    }
                }

                Ok(Obj::List(List {
                    values: self.values.clone(),
                    subsets: self.subsets.clone(),
                }))
            }
            // vectorized assignment
            // TODO(feature): warn when index recycling does not cycle evenly
            any if any.len() == Some(self.len()) => {
                let mut v = self.values.borrow_mut();
                let n = v.len();
                let indices = self.subsets.clone().bind_names(names.clone()).into_iter().take(n);

                // first check to see if we need to extend
                if let Some(max) = self.subsets.clone().bind_names(names).into_iter().map(|(i, _)| i).max() {
                    v.reserve(max.saturating_sub(n))
                }

                // then assign to indices
                for (any_i, (_, i)) in indices.enumerate() {
                    if let (Some(value), Some(i)) = (any.get(any_i), i) {
                        v[i].1 = value;
                    }
                }

                Ok(Obj::List(List {
                    values: self.values.clone(),
                    subsets: self.subsets.clone(),
                }))
            }
            other => {
                let mut v = self.values.borrow_mut();
                let n = v.len();
                let indices = self.subsets.clone().bind_names(names.clone()).into_iter().take(n);

                // first check to see if we need to extend
                if let Some(max) = self.subsets.clone().bind_names(names).into_iter().map(|(i, _)| i).max() {
                    v.reserve(max.saturating_sub(n))
                }

                // then assign to indices
                for (_, i) in indices {
                    if let Some(i) = i {
                        v[i].1 = other.clone()
                    }
                }

                Ok(Obj::List(List {
                    values: self.values.clone(),
                    subsets: self.subsets.clone(),
                }))
            }
        }
    }

    pub fn try_get(&self, index: Obj) -> EvalResult {
        let err = RError::Other("Cannot use object for indexing.".to_string());
        match index.as_vector()? {
            Obj::Vector(v) => Ok(Obj::List(self.subset(v.try_into()?))),
            _ => Err(err.into()),
        }
    }

    pub fn try_get_inner(&self, index: Obj) -> EvalResult {
        let err = RError::Other("Cannot use object for indexing.".to_string());
        let names: Vec<_> = self.values.borrow().clone().into_iter().map(|(n, _)| n).collect();
        match index.as_vector()? {
            Obj::Vector(v) if v.len() == 1 => {
                let Subsets(mut subsets) = self.subsets.clone();
                subsets.push(v.try_into()?);

                if let Some((i, _)) = Subsets(subsets).bind_names(names).into_iter().next() {
                    self.values
                        .borrow()
                        .get(i)
                        .map_or(Err(err.into()), |(_, i)| Ok(i.clone()))
                } else {
                    Ok(Obj::Null)
                }
            }
            _ => Err(err.into()),
        }
    }

    pub fn len(&self) -> usize {
        let Subsets(inner) = &self.subsets;
        match inner.as_slice() {
            [] => self.values.borrow().len(),
            [.., last] => std::cmp::min(self.values.borrow().len(), last.len()),
        }
    }
}

