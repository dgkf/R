use hashbrown::{HashMap, HashSet};

use crate::error::Error;
use crate::lang::EvalResult;
use crate::object::reptype::RepType;
use crate::object::vector::rep::Rep;

use super::*;

type ListNameMap = HashMap<String, Vec<usize>>;

#[derive(Debug, Clone, PartialEq, Default)]
pub struct List {
    pub names: CowObj<ListNameMap>,
    pub values: Rep<(Option<String>, Obj)>,
}

impl From<Vec<(Option<String>, Obj)>> for List {
    fn from(value: Vec<(Option<String>, Obj)>) -> Self {
        let mut result = List {
            values: Rep::from(value),
            ..Default::default()
        };

        result.reindex();
        result
    }
}

impl List {
    pub fn reindex(&mut self) {
        self.names.with_inner_mut(|names| {
            names.drain();

            for (i, (k, _)) in self.values.iter().enumerate() {
                if let Some(name) = k {
                    let indices = names.entry(name.clone()).or_default();
                    if !indices.contains(&i) {
                        indices.push(i)
                    }
                }
            }
        })
    }

    pub fn subset(&self, by: Subset) -> List {
        let values = self.values.view_mut();
        values.subset(by);
        List {
            names: self.names.clone(),
            values,
        }
    }

    pub fn assign(&mut self, value: Obj) -> EvalResult {
        let subsets = match &*self.values.borrow() {
            RepType::Subset(_, s) => s.clone(),
        };
        let n = self.values.len();
        let iter = subsets
            .clone()
            .bind_names(self.names.clone())
            .into_iter()
            .take(n);

        match value {
            // remove elements from list
            Obj::Null => {
                let indices: HashSet<usize> = iter.map(|(i, _)| i).collect();
                self.values.with_inner_mut(|values| {
                    let mut i = 0;
                    values.retain(|_| {
                        let retain = !indices.contains(&i);
                        i += 1;
                        retain
                    });
                });

                self.reindex();

                // TODO(feat): need to return list with NULL elements when
                // index is NA
            }

            // any single length R value
            any if any.len() == Some(1) => {
                self.values.with_inner_mut(|v| {
                    // first check to see if we need to extend
                    if let Some(max) = subsets
                        .clone()
                        .bind_names(self.names.clone())
                        .into_iter()
                        .take(n)
                        .map(|(i, _)| i)
                        .max()
                    {
                        v.reserve(max.saturating_sub(n))
                    }

                    // then assign to indices
                    for (_, i) in iter {
                        if let Some(i) = i {
                            v[i].1 = any.clone()
                        }
                    }
                });
            }
            // vectorized assignment
            // TODO(feature): warn when index recycling does not cycle evenly
            any if any.len() == Some(self.len()) => {
                self.values.with_inner_mut(|v| {
                    // first check to see if we need to extend
                    if let Some(max) = subsets
                        .clone()
                        .bind_names(self.names.clone())
                        .into_iter()
                        .take(n)
                        .map(|(i, _)| i)
                        .max()
                    {
                        v.reserve(max.saturating_sub(n))
                    }

                    // then assign to indices
                    for (any_i, (_, i)) in iter.enumerate() {
                        if let (Some(value), Some(i)) = (any.get(any_i), i) {
                            v[i].1 = value;
                        }
                    }
                });
            }
            other => {
                self.values.with_inner_mut(|v| {
                    // first check to see if we need to extend
                    if let Some(max) = subsets
                        .clone()
                        .bind_names(self.names.clone())
                        .into_iter()
                        .take(n)
                        .map(|(i, _)| i)
                        .max()
                    {
                        v.reserve(max.saturating_sub(n))
                    }

                    // then assign to indices
                    for (_, i) in iter {
                        if let Some(i) = i {
                            v[i].1 = other.clone()
                        }
                    }
                });
            }
        };
        Ok(Obj::List(List {
            names: self.names.clone(),
            values: self.values.clone(),
        }))
    }

    pub fn try_get(&self, index: Obj) -> EvalResult {
        println!("HIIIII");
        let err = Error::Other("Cannot use object for indexing".to_string());
        match index.as_vector()? {
            Obj::Vector(v) => Ok(Obj::List(self.subset(v.try_into()?))),
            _ => Err(err.into()),
        }
    }

    pub fn try_get_inner_mut(&self, index: Obj) -> EvalResult {
        let err_invalid = Error::Other("Cannot use object for indexing".to_string());
        let err_index_invalid = Error::Other("Index out of bounds".to_string());

        match index.as_vector()? {
            Obj::Vector(v) if v.len() == 1 => {
                let values = self.values.subset(v.try_into()?);
                // self.values.get_inner(i)

                let subsets = match &*values.borrow() {
                    RepType::Subset(_, Subsets(v)) => v.clone(),
                };

                if let Some((i, _)) = Subsets(subsets)
                    .bind_names(self.names.clone())
                    .into_iter()
                    .next()
                {
                    self.values.with_inner_mut(|v| {
                        v.get_mut(i)
                            .map_or(Err(err_index_invalid.into()), |(_, i)| Ok(i.view_mut()))
                    })
                } else {
                    Ok(Obj::Null)
                }
            }
            _ => Err(err_invalid.into()),
        }
    }

    pub fn try_get_inner(&self, index: Obj) -> EvalResult {
        #[allow(clippy::map_clone)]
        self.try_get_inner_mut(index).map(|v| v.clone())
    }

    pub fn dedup_last(self) -> Self {
        self.names.with_inner_mut(|names| {
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
        });

        self.names.with_inner_mut(|names| {
            for (_, indices) in names.iter_mut() {
                indices.drain(0..indices.len());
            }
        });

        self
    }

    pub fn len(&self) -> usize {
        self.values.len()
    }

    #[must_use]
    pub fn is_empty(&self) -> bool {
        self.len() == 0
    }
}

#[cfg(test)]
mod tests {
    use crate::{r, r_expect};
    #[test]
    fn list_declaration_ambiguity() {
        assert_eq!(r!((a = 1,)), r!(list(a = 1)));
        assert_eq!(r!((a = 1)), r!(1));
        assert_eq!(r!((1)), r!(1));
        assert_eq!(r!((1,)), r!(list(1)));
    }

    #[test]
    fn copy_on_write_single_bracket() {
        r_expect! {{"
            l1 = (1,)
            l2 = l1
            l1[1] = 2
            l1[[1]] == 2 & l2[[1]] == 1
        "}}
    }
    #[test]
    fn copy_on_write_bracket_names() {
        r_expect! {{r#"
            l1 = (a = 1,)
            l2 = l1
            l1["a"] = 2
            l1$a == 2 & l2$a == 1
        "#}}
    }
    #[test]
    fn copy_on_write_slice_names() {
        r_expect! {{r#"
            l = (a = 1, b = 2, c = 3)
            l1 = l
            l1[c("a", "b")] = c(10, 20)

            l1$a == 10 && l1$b == 20 & l$a == 1 & l$b == 2
        "#}}
    }
    #[test]
    fn copy_on_write_slice_indices() {
        r_expect! {{"
            l = (1, 2)
            l1 = l
            l1[1:2] = [10, 20]
            l1[[1]] == 10 && l1[[2]] == 20 & l[[1]] == 1 & l[[2]] == 2
        "}}
    }

    #[test]
    fn copy_on_write_index() {
        r_expect! {{"
            l = (1, 2)
            l_cow = l  # at this point, a copy-on-write reference
            l_cow[[2]] = 20
            l_cow[[1]] == 1 && l_cow[[2]] == 20 && l[[1]] == 1 && l[[2]] == 2
        "}}
    }

    #[test]
    fn nested_double_bracket_index() {
        r_expect! {{"
            l = ((1,),)
            l_cow = l
            l_cow[[1]][[1]] = 20
            l_cow[[1]][[1]] == 20 && l[[1]][[1]] == 1
        "}}
    }
    #[test]
    fn nested_double_bracket_names() {
        r_expect! {{r#"
            l = (a = (b = 1,),)
            l_cow = l
            l_cow[["a"]][["b"]] = 20
            l_cow[["a"]][["b"]] == 20 && l[["a"]][["b"]] == 1
        "#}}
    }
    #[test]
    fn nested_double_bracket_mixed() {
        r_expect! {{r#"
            l = (a = (1,),)
            l_cow = l
            l_cow[["a"]][[1]] = 20
            l_cow[["a"]][[1]] == 20 && l[["a"]][[1]] == 1
        "#}}
    }
    #[test]
    fn assign_list_to_list_slice() {
        r_expect! {{r#"
            l = (1, 2, 3)
            l[1:2] = (10, 20)
            l[[1]] == 10 & l[[2]] == 20
        "#}}
    }
}
