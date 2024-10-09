use std::cell::RefCell;
use std::ops::Range;
use std::rc::Rc;

use super::{types::*, OptionNA, Vector};
use crate::error::Error;
use crate::lang::Signal;
use crate::object::{CowObj, Obj};

/// Subsets
///
/// Representations of how data views might be specified. Indices are 0-indexed,
/// for direct use against rust-internal representations.
///
#[derive(Debug, Clone, PartialEq)]
pub enum Subset {
    Indices(CowObj<Vec<Integer>>),
    Mask(CowObj<Vec<Logical>>),
    Names(CowObj<Vec<Character>>),
    Range(Range<usize>),
}

impl Subset {
    pub fn get_index_at(&self, index: usize) -> Option<usize> {
        match self {
            Subset::Indices(indices) => indices.clone().borrow().get(index).and_then(|i| match i {
                OptionNA::Some(i) => Some((*i as usize).saturating_sub(1)),
                OptionNA::NA => None,
            }),
            Subset::Range(range) => {
                if range.start <= index && index < range.end {
                    Some(range.start + index)
                } else {
                    None
                }
            }
            Subset::Mask(mask) => mask
                .clone()
                .borrow()
                .iter()
                .enumerate()
                .filter_map(|(i, m)| match m {
                    OptionNA::Some(true) => Some(Some(i)),
                    OptionNA::NA => Some(None),
                    _ => None,
                })
                .nth(index)
                .unwrap_or(None),
            Subset::Names(_) => unimplemented!(),
        }
    }

    pub fn len(&self) -> usize {
        match self {
            Subset::Indices(i) => i.len(),
            Subset::Range(r) => r.end - r.start,
            Subset::Mask(_) => usize::MAX,
            Subset::Names(n) => n.len(),
        }
    }

    #[must_use]
    pub fn is_empty(&self) -> bool {
        self.len() == 0
    }

    pub fn filter<'a, I>(
        &self,
        mut iter: I,
    ) -> Box<dyn Iterator<Item = (usize, Option<usize>)> + 'a>
    where
        I: Iterator<Item = (usize, Option<usize>)> + 'a,
    {
        match self.clone() {
            Subset::Indices(i) => {
                let l = self.len();

                // fastest case, when no indices are selected
                if l == 0 {
                    Box::new(vec![].into_iter())

                // very fast case, when one index is selected
                } else if l == 1 {
                    let msg = "Expected at least one element to index by";
                    if let OptionNA::Some(to_first) = i.clone().borrow().first().expect(msg) {
                        Box::new(iter.skip(*to_first as usize).take(1))
                    } else {
                        let (i_orig, _) = iter.next().unwrap_or((0, None));
                        Box::new(vec![(i_orig, None)].into_iter())
                    }

                // fast case, when indices are already sorted
                } else if i.borrow().windows(2).all(|w| w[0] <= w[1]) {
                    // when sorted, we can keep our existing iterator and
                    // embed the indices, scanning along the iterator
                    // and yielding indices as they are encountered
                    let ic = i.clone();
                    let ib = ic.borrow().clone();

                    Box::new(
                        iter.enumerate()
                            .scan(
                                (ib, 0),
                                |(indices, i), (xi, (xi_orig, x))| -> Option<Vec<(usize, Option<usize>)>> {
                                    if *i >= indices.len() {
                                        return None;
                                    }

                                    let mut n = 0;
                                    while *i < indices.len()
                                        && indices[*i] == OptionNA::Some(xi as i32)
                                    {
                                        n += 1;
                                        *i += 1;
                                    }

                                    Some(vec![(xi_orig, x); n])
                                },
                            )
                            .flatten(),
                    )
                // worst case, indices in random order
                } else {
                    // enumerate indices and swap so it's (index, enumeration)
                    let ic = i.clone();
                    let ib = ic.borrow();

                    let mut order = ib
                        .iter()
                        .map(|i| match i {
                            OptionNA::NA => -1,
                            OptionNA::Some(i) => *i,
                        })
                        .enumerate()
                        .map(|(i, v)| (v, i)) // cast NAs to -1's
                        .collect::<Vec<_>>();

                    // sort by index to get (sorted indices, order)
                    // we'll use this to sample the existing iterator then
                    // permute it back into the original order
                    order.sort();
                    let (mut i, order): (Vec<_>, Vec<_>) = order.into_iter().unzip();

                    // we'll populate this with the sorted indices first
                    let mut indices: Vec<(usize, Option<usize>)> = vec![(0, None); i.len()];
                    let n_na = i.iter().take_while(|&i| *i == -1).count();

                    // populate non-na elements
                    i.insert(n_na, 0);
                    let diffs = i[n_na..].windows(2).map(|w| w[1] - w[0]);

                    let msg = "exhausted iterator";
                    let (mut i_orig, mut last) = iter.nth(0).expect(msg);
                    for (i, diff) in diffs.enumerate() {
                        if diff > 0 {
                            (i_orig, last) = iter.nth((diff - 1) as usize).expect(msg);
                        }
                        indices[order[i + n_na]] = (i_orig, last);
                    }

                    // and finally we convert our new indices into an iterator
                    Box::new(indices.into_iter())
                }
            }
            Subset::Mask(mask) => {
                Box::new(
                    (**mask.borrow())
                        .clone()
                        .into_iter()
                        .cycle()
                        .zip(iter)
                        .filter_map(|(mask, i @ (i_orig, _))| match mask {
                            OptionNA::Some(true) => Some(i),      // accept index
                            OptionNA::NA => Some((i_orig, None)), // accept, but NA
                            _ => None,                            // filter falses
                        }),
                )
            }
            Subset::Range(range) => Box::new(
                iter.skip(range.start)
                    .enumerate()
                    .take_while(move |(i, _)| i < &(range.end - range.start))
                    .map(|(_, v)| v),
            ),
            Subset::Names(_) => unimplemented!(),
        }
    }
}

impl TryFrom<Obj> for Subset {
    type Error = Signal;
    fn try_from(value: Obj) -> Result<Self, Self::Error> {
        let err = Error::Other("Cannot use object for indexing".to_string());
        match value {
            Obj::Vector(v) => Subset::try_from(v),
            _ => err.into(),
        }
    }
}

impl From<usize> for Subset {
    fn from(value: usize) -> Self {
        Subset::Indices(vec![OptionNA::Some(value as i32)].into())
    }
}

impl From<Range<usize>> for Subset {
    fn from(value: Range<usize>) -> Self {
        Subset::Range(value)
    }
}

impl From<Vec<usize>> for Subset {
    fn from(value: Vec<usize>) -> Self {
        Subset::Indices(CowObj::new(Rc::new(RefCell::new(Rc::new(
            value
                .iter()
                .map(|i| OptionNA::Some(*i as i32))
                .collect::<Vec<_>>(),
        )))))
    }
}

impl TryFrom<Vector> for Subset {
    type Error = Signal;
    fn try_from(value: Vector) -> Result<Self, Self::Error> {
        match value {
            value @ Vector::Double(_) => Subset::try_from(value.as_integer()),
            Vector::Integer(v) => {
                let y = v
                    .iter_values()
                    .map(|i| match i {
                        OptionNA::Some(x) => OptionNA::Some(x - 1),
                        OptionNA::NA => OptionNA::NA,
                    })
                    .collect();

                Ok(Subset::Indices(CowObj::new(Rc::new(RefCell::new(
                    Rc::new(y),
                )))))
            }
            Vector::Logical(v) => {
                let all_false = v
                    .inner()
                    .clone()
                    .borrow()
                    .iter()
                    .all(|i| i == &OptionNA::Some(false));

                // special case when all are false, treat it as no indices
                if all_false {
                    Ok(Subset::Indices(Vec::new().into()))
                } else {
                    Ok(Subset::Mask(v.inner()))
                }
            }
            Vector::Character(v) => Ok(Subset::Names(v.inner())),
        }
    }
}
