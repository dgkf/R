use hashbrown::HashMap;
use std::cell::RefCell;
use std::rc::Rc;

use crate::error::Error;
use crate::lang::EvalResult;

use super::*;

type ListNameMap = HashMap<String, Vec<usize>>;

#[derive(Debug, Clone, PartialEq, Default)]
pub struct List {
    pub names: Rc<RefCell<ListNameMap>>,
    pub values: VecData<(Option<String>, Obj)>,
    pub subsets: Subsets,
}

impl From<Vec<(Option<String>, Obj)>> for List {
    fn from(value: Vec<(Option<String>, Obj)>) -> Self {
        let mut result = List {
            values: VecData::from(value),
            ..Default::default()
        };

        result.reindex();
        result
    }
}

impl List {
    pub fn reindex(&mut self) {
        let mut names = self.names.borrow_mut();
        names.drain();

        for (i, (k, _)) in self.values.borrow().iter().enumerate() {
            if let Some(name) = k {
                let indices = names.entry(name.clone()).or_default();
                if !indices.contains(&i) {
                    indices.push(i)
                }
            }
        }
    }

    pub fn subset(&self, by: Subset) -> List {
        let Subsets(mut inner) = self.subsets.clone();
        inner.push(by);
        List {
            names: self.names.clone(),
            values: self.values.clone(),
            subsets: Subsets(inner),
        }
    }

    pub fn assign(&mut self, value: Obj) -> EvalResult {
        match value {
            // remove elements from list
            Obj::Null => {
                let n = self.values.len();
                let indices = self
                    .subsets
                    .clone()
                    .bind_names(self.names.clone())
                    .into_iter()
                    .take(n);

                self.values.with_inner_mut(|values| {
                    for (i, _) in indices {
                        values.remove(i);
                    }
                });

                self.reindex();

                // TODO(feat): need to return list with NULL elements when
                // index is NA

                Ok(Obj::List(List {
                    names: self.names.clone(),
                    values: self.values.clone(),
                    subsets: self.subsets.clone(),
                }))
            }

            // any single length R value
            any if any.len() == Some(1) => {
                let n = self.values.len();
                let indices = self
                    .subsets
                    .clone()
                    .bind_names(self.names.clone())
                    .into_iter()
                    .take(n);

                self.values.with_inner_mut(|v| {
                    // first check to see if we need to extend
                    if let Some(max) = self
                        .subsets
                        .clone()
                        .bind_names(self.names.clone())
                        .into_iter()
                        .map(|(i, _)| i)
                        .max()
                    {
                        v.reserve(max.saturating_sub(n))
                    }

                    // then assign to indices
                    for (_, i) in indices {
                        if let Some(i) = i {
                            v[i].1 = any.clone()
                        }
                    }
                });

                Ok(Obj::List(List {
                    names: self.names.clone(),
                    values: self.values.clone(),
                    subsets: self.subsets.clone(),
                }))
            }
            // vectorized assignment
            // TODO(feature): warn when index recycling does not cycle evenly
            any if any.len() == Some(self.len()) => {
                let n = self.values.len();
                let indices = self
                    .subsets
                    .clone()
                    .bind_names(self.names.clone())
                    .into_iter()
                    .take(n);

                self.values.with_inner_mut(|v| {
                    // first check to see if we need to extend
                    if let Some(max) = self
                        .subsets
                        .clone()
                        .bind_names(self.names.clone())
                        .into_iter()
                        .map(|(i, _)| i)
                        .max()
                    {
                        v.reserve(max.saturating_sub(n))
                    }

                    // then assign to indices
                    for (any_i, (_, i)) in indices.enumerate() {
                        if let (Some(value), Some(i)) = (any.get(any_i), i) {
                            v[i].1 = value;
                        }
                    }
                });

                Ok(Obj::List(List {
                    names: self.names.clone(),
                    values: self.values.clone(),
                    subsets: self.subsets.clone(),
                }))
            }
            other => {
                let n = self.len();
                let indices = self
                    .subsets
                    .clone()
                    .bind_names(self.names.clone())
                    .into_iter()
                    .take(n);

                self.values.with_inner_mut(|v| {
                    // first check to see if we need to extend
                    if let Some(max) = self
                        .subsets
                        .clone()
                        .bind_names(self.names.clone())
                        .into_iter()
                        .map(|(i, _)| i)
                        .max()
                    {
                        v.reserve(max.saturating_sub(n))
                    }

                    // then assign to indices
                    for (_, i) in indices {
                        if let Some(i) = i {
                            v[i].1 = other.clone()
                        }
                    }
                });

                Ok(Obj::List(List {
                    names: self.names.clone(),
                    values: self.values.clone(),
                    subsets: self.subsets.clone(),
                }))
            }
        }
    }

    pub fn try_get(&self, index: Obj) -> EvalResult {
        let err = Error::Other("Cannot use object for indexing".to_string());
        match index.as_vector()? {
            Obj::Vector(v) => Ok(Obj::List(self.subset(v.try_into()?))),
            _ => Err(err.into()),
        }
    }

    pub fn try_get_inner(&self, index: Obj) -> EvalResult {
        let err_invalid = Error::Other("Cannot use object for indexing".to_string());
        let err_index_invalid = Error::Other("Index out of bounds".to_string());

        match index.as_vector()? {
            Obj::Vector(v) if v.len() == 1 => {
                let Subsets(mut subsets) = self.subsets.clone();
                subsets.push(v.try_into()?);

                if let Some((i, _)) = Subsets(subsets)
                    .bind_names(self.names.clone())
                    .into_iter()
                    .next()
                {
                    self.values
                        .borrow()
                        .get(i)
                        .map_or(Err(err_index_invalid.into()), |(_, i)| Ok(i.clone()))
                } else {
                    Ok(Obj::Null)
                }
            }
            _ => Err(err_invalid.into()),
        }
    }

    pub fn dedup_last(self) -> Self {
        {
            let names = self.names.borrow();
            let mut dups: Vec<usize> = names
                .iter()
                .flat_map(|(_, indices)| {
                    indices
                        .split_last()
                        .map_or(vec![], |(_, leading_dups)| leading_dups.to_vec())
                })
                .collect();

            dups.sort();

            self.values.with_inner_mut(|vs| {
                for i in dups.into_iter().rev() {
                    vs.remove(i);
                }
            });
        }

        for (_, indices) in self.names.borrow_mut().iter_mut() {
            indices.drain(0..indices.len());
        }

        self
    }

    pub fn len(&self) -> usize {
        let Subsets(inner) = &self.subsets;
        match inner.as_slice() {
            [] => self.values.borrow().len(),
            [.., last] => std::cmp::min(self.values.borrow().len(), last.len()),
        }
    }

    #[must_use]
    pub fn is_empty(&self) -> bool {
        self.len() == 0
    }
}

#[cfg(test)]
mod tests {
    use crate::r;
    #[test]
    fn list_declaration_ambiguity() {
        assert_eq!(r!((a = 1,)), r!(list(a = 1)));
        assert_eq!(r!((a = 1)), r!(1));
        assert_eq!(r!((1)), r!(1));
        assert_eq!(r!((1,)), r!(list(1)));
    }
}
