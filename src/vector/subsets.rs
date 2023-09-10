use super::{subset::Subset, vectors::OptionNA};

#[derive(Debug, Clone, PartialEq)]
pub struct Subsets(pub Vec<Subset>);

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
        for subset in subsets.into_iter().rev() {
            match subset.get_index_at(index) {
                Some(i) => index = i,
                None => return None,              
            }
        }
        Some(index)
    }

    pub fn push<T>(self, subset: T) 
    where
        T: Into<Subset>
    {
        let Subsets(mut v) = self;
        v.push(subset.into());
    }
}

impl<T> From<Vec<T>> for Subsets
where
    T: Into<Subset>
{
    fn from(value: Vec<T>) -> Self {
        let v: Vec<Subset> = value.into_iter().map(|i| i.into()).collect();
        Subsets(v)
    }
}

impl IntoIterator for Subsets {
    type Item = Option<usize>;
    type IntoIter = Box<dyn Iterator<Item = Self::Item>>;

    /// Convert Subsets into an iterator if indices
    ///
    /// Builds an iterator of indices from a collection of subsets.
    ///
    /// # Examples
    ///
    /// /// let subsets = Subsets::new()
    /// ///     .push(1..) 
    /// ///     .push(vec![1, 9, 1, 7]);
    /// ///
    /// /// let indices: Vec<_> = subsets.into_iter().collect();
    /// /// assert_eq!(indices, vec![2, 10, 2, 8])
    ///
    fn into_iter(self) -> Self::IntoIter {
        let Subsets(subsets) = self;
        let mut iter = Box::new((0_usize..).map(|i| Some(i)).into_iter()) as Self::IntoIter;
        for subset in subsets {
            let l = subset.len();
            match subset {
                Subset::Indices(i) => {
                    // fastest case, when no indices are selected
                    if l == 0 {
                        return Box::new(vec![].into_iter());

                    // very fast case, when one index is selected
                    } else if l == 1 {
                        let msg = "Expected at least one element to index by";
                        if let OptionNA::Some(to_first) = i.clone().borrow().get(0).expect(msg) {
                            iter = Box::new(iter.skip(*to_first as usize).take(1));
                        } else {
                            iter = Box::new(vec![None].into_iter())
                        }

                    // fast case, when indices are already sorted
                    } else if i.clone().borrow().windows(2).all(|w| w[0] <= w[1]) {
                        // when sorted, we can keep our existing iterator and
                        // embed the indices, scanning along the iterator
                        // and yielding indices as they are encountered
                        let ic = i.clone();
                        let ib = ic.borrow().clone();

                        iter = Box::new(
                            iter.enumerate().scan((ib, 0), |(indices, i), (xi, x)| -> Option<Vec<Option<usize>>> {
                                if *i >= indices.len() { 
                                    return None
                                }

                                let mut n = 0;
                                while *i < indices.len() && indices[*i] == OptionNA::Some(xi as i32) {
                                    n += 1;
                                    *i += 1;
                                };

                                return Some(vec![x; n])
                            })
                            .flat_map(|i| i)
                        )

                    // worst case, indices in random order
                    } else {
                        // enumerate indices and swap so it's (index, enumeration)
                        let ic = i.clone();
                        let ib = ic.borrow();

                        let mut order = ib.iter()
                            .enumerate()
                            .map(|(i, v)| (v, i.clone()))
                            .collect::<Vec<_>>();

                        // sort by index to get (sorted indices, order)
                        // we'll use this to sample the existing iterator then 
                        // permute it back into the original order
                        order.sort();
                        let (mut i, order): (Vec<_>, Vec<_>) = order.into_iter().unzip();

                        // we'll populate this with the sorted indices first
                        let mut indices: Vec<Option<usize>> = vec![None; i.len()];
                        let n_na = i.iter().take_while(|&i| **i == OptionNA::NA).count();

                        // populate non-na elements
                        i.insert(n_na, &OptionNA::Some(0));
                        let diffs = i[n_na..].windows(2)
                            .map(|w| {
                                if let OptionNA::Some(i) = w[1].clone() - w[0].clone() { 
                                    i 
                                } else { 
                                    unreachable!()
                                }
                            });

                        let mut last = iter.nth(0).expect("exhausted iterator"); for (i, diff) in diffs.enumerate() {
                            if diff > 0 {
                                last = iter.nth((diff - 1) as usize).expect("exhausted iterator");
                            }
                            indices[order[i + n_na]] = last;
                        }

                        // and finally we convert our new indices into an iterator
                        iter = Box::new(indices.into_iter())
                    }
                },
                Subset::Range(range) => {
                    iter = Box::new(
                        iter.skip(range.start)
                            .enumerate()
                            .take_while(move |(i, _)| i < &(range.end - range.start))
                            .map(|(_, v)| v)
                    ) as Self::IntoIter
                },
            }
        }

        iter
    }
}

// #[cfg(test)]
//  mod test {
//     use crate::vector::Vector;

//     #[test]
//     fn subset_range() {
//         let x: Vector<_> = (1..=10).into_iter().collect::<Vec<_>>().into();
//         let result = x.subset(2..6).materialize();
//         let expect = Vector::from(vec![3, 4, 5, 6]);
//         assert_eq!(result, expect)
//     }

//     #[test]
//     fn subset_sequential_indices() {
//         let x: Vector<_> = (1..=10).into_iter().collect::<Vec<_>>().into();
//         let result = x.subset(vec![2, 3, 4, 5]).materialize();
//         let expect = Vector::from(vec![3, 4, 5, 6]);
//         assert_eq!(result, expect)
//     }

//     #[test]
//     fn subset_sequential_repeating_indices() {
//         let x: Vector<_> = (1..=10).into_iter().collect::<Vec<_>>().into();
//         let result = x.subset(vec![2, 3, 3, 3, 5, 5]).materialize();
//         let expect = Vector::from(vec![3, 4, 4, 4, 6, 6]);
//         assert_eq!(result, expect)
//     }

//     #[test]
//     fn subset_indices_with_gap() {
//         let x: Vector<_> = (1..=10).into_iter().collect::<Vec<_>>().into();
//         let result = x.subset(vec![2, 8]).materialize();
//         let expect = Vector::from(vec![3, 9]);
//         assert_eq!(result, expect);
//     }

//     #[test]
//     fn subset_empty_indices() {
//         let x: Vector<_> = (1..=10).into_iter().collect::<Vec<_>>().into();
//         let result = x.subset(vec![]).materialize();
//         let expect = Vector::from(Vec::new() as Vec<i32>);
//         assert_eq!(result, expect);
//     }

//     #[test]
//     fn subset_single_index() {
//         let x: Vector<_> = (1..=10).into_iter().collect::<Vec<_>>().into();
//         let result = x.subset(vec![6]).materialize();
//         let expect = Vector::from(vec![7]);
//         assert_eq!(result, expect);
//     }

//     #[test]
//     fn subset_unsorted_indices() {
//         let x: Vector<_> = (1..=10).into_iter().collect::<Vec<_>>().into();
//         let result = x.subset(vec![6, 2, 1, 4]).materialize();
//         let expect = Vector::from(vec![7, 3, 2, 5]);
//         assert_eq!(result, expect);
//     }

//     #[test]
//     fn subset_repeated_indices() {
//         let x: Vector<_> = (1..=10).into_iter().collect::<Vec<_>>().into();
//         let result = x.subset(vec![6, 2, 6, 6]).materialize();
//         let expect = Vector::from(vec![7, 3, 7, 7]);
//         assert_eq!(result, expect);
//     }

//     #[test]
//     fn subset_by_range() {
//         let x: Vector<_> = (1..=10).into_iter().collect::<Vec<_>>().into();
//         let result = x.subset(3..6).materialize();
//         let expect = Vector::from(vec![4, 5, 6]);
//         assert_eq!(result, expect);
//     }

//     #[test]
//     fn nested_subsets() {
//         let x: Vector<_> = (1..=10).into_iter().collect::<Vec<_>>().into();
//         let result = x.subset(3..6).subset(vec![2, 1]).materialize();
//         let expect = Vector::from(vec![6, 5]);
//         assert_eq!(result, expect);
//     }

//     #[test]
//     fn subset_assignment() {
//         let x: Vector<_> = (1..=10).into_iter().collect::<Vec<_>>().into();
//         let subset = x.subset(3..6).subset(vec![2, 1]);
//         let y: Vector<_> = vec![101, 102].into();
//         subset.assign(y);
//         let expect = Vector::from(vec![1, 2, 3, 4, 102, 101, 7, 8, 9, 10]);
//         assert_eq!(x, expect)
//     }
// }