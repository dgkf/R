/// # Design Notes
///
/// When vectors are subset, we can lazily evaluate subsets such that 
/// they may still be assigned to in place without producing intermediate 
/// values.
///
/// For example, 
///
///     x[1:1e6][3:10][[4]] <- 10
///
/// In R, this would first allocate 
///
///     x[1:1e6], which we'll call y, then
///            y[3:10], which we'll call z, then
///                  z[[4]], which we'll call w, then
///                        w <- 3, which we'll assign back to z
///                  z[[4]] <- 4, which we'll assign back to y
///            y[3:10] <- z[[4]], which we'll assign back to x
///     x[1:1e6] <- y
///     
/// This becomes a painfully inefficient method for heavily subset assigment,
/// which intuitively should be fast. Instead, we can pre-calculate exactly
/// which indices should be affected. Only when a subset is assigned to a value
/// do we need to materialize it into a new vector (and even then, possibly 
/// only on-write using Cow)
///
/// # TODOs:
/// - [ ] Vector recycling
/// - [ ] Fail (or warn) when recycled elements don't have a common multiple
/// - [ ] Better bounds checking and defaults when assigning to new indices
///

use std::cell::RefCell;
use std::fmt::{Debug, Display};
use std::ops::{Add, Range};
use std::rc::Rc;

/// Atomic 
///
/// Intended to be a generic trait for all vectors, encompassing all 
/// basic operator behaviors.
/// 
pub trait Atomic: Clone + Add {}
impl Atomic for i32 {}

/// Vector
#[derive(Debug, Clone)]
pub enum Vector<T: Atomic> {
    // Vector::Subset encompasses a "raw" vector (no subsetting)
    Subset(Rc<RefCell<Vec<T>>>, Subsets),

    // Iterator includes things like ranges 1:Inf, and lazily computed values
    // Iter(Box<dyn Iterator<Item = &T>>)
}

#[derive(Debug, Clone)]
pub struct Subsets(Vec<Subset>);

#[derive(Debug, Clone)]
pub enum Subset {
    Indices(Vec<usize>),
    Range(Range<usize>)
}

impl<T: Atomic> Vector<T> {
    /// Subsetting a Vector
    ///
    /// Introduce a new subset into the aggregate list of subset indices.
    ///
    fn subset<S>(&self, subset: S) -> Self
    where
        S: Into<Subset>
    {
        match self {
            Vector::Subset(v, Subsets(subsets)) => {
                let mut subsets = subsets.clone();
                subsets.push(subset.into());
                Vector::Subset(v.clone(), Subsets(subsets))
            },
        }          
    }

    /// Assignment to Subset Indices
    ///
    /// Assignment to a vector from another. The aggregate subsetted indices
    /// are iterated over while performing the assignment.
    ///
    fn assign(&self, value: Self) {
        match (self, value) {
            (Vector::Subset(lv, ls), Vector::Subset(rv, rs)) => {
                let lvc = lv.clone(); let mut lv = lvc.borrow_mut();
                let rvc = rv.clone(); let rv = rvc.borrow();

                let lv_len = lv.len().clone();
                let l_indices = ls.clone().into_iter().take_while(|i| i < &lv_len);
                let r_indices = rs.clone().into_iter().take_while(|i| i < &rv.len());

                for (li, ri) in l_indices.zip(r_indices) {
                    lv[li] = rv[ri].clone()
                }
            },
        }
    }

    /// Materialize a Vector
    ///
    /// Apply subsets and clone values into a new vector.
    ///
    #[allow(dead_code)]
    fn materialize(&self) -> Self {
        match self {
            Vector::Subset(v, subsets) => {      
                let vc = v.clone();
                let vb = vc.borrow();
                let mut res: Vec<T> = vec![];
                let vb_len = vb.len();
                for i in subsets.clone().into_iter().take_while(|i| i < &vb_len) {
                    res.push(vb[i].clone())
                }
                Vector::Subset(Rc::new(RefCell::new(res)), Subsets(vec![]))
            },
        }
    }
}

impl<T: Atomic> Display for Vector<T> 
where
    T: Display
{
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "[")?;
        match self {
            Vector::Subset(v, s) => {
                let vc = v.clone();
                let v = vc.borrow();
                for (e, i) in s.clone().into_iter().take_while(|i| i < &v.len()).enumerate() {
                    if e > 0 { write!(f, ", ")? }
                    write!(f, "{}", v[i])?;
                }
            }
        }
        write!(f, "]")?;

        Ok(())
    }
}

impl IntoIterator for Subsets {
    type Item = usize;
    type IntoIter = Box<dyn Iterator<Item = Self::Item>>;

    fn into_iter(self) -> Self::IntoIter {
        let Subsets(subsets) = self;
        let mut iter = Box::new((0_usize..).into_iter()) as Self::IntoIter;
        for subset in subsets {
            match subset {
                Subset::Indices(mut i) => {
                    // fasttest case, when no indices are selected
                    if i.len() == 0 {
                        return Box::new((0..0).into_iter());

                    // very fast case, when one index is selected
                    } else if i.len() == 1 {
                        iter = Box::new(iter.skip(i[0]).take(1));
                        
                    // fast case, when indices are already sorted
                    } else if i.windows(2).all(|w| w[0] < w[1]) {
                        i.insert(0, 0);
                        let mut diffs = i.windows(2).map(|w| w[1] - w[2]);
                        loop {
                            let Some(diff) = diffs.next() else {
                                break
                            };
                            
                            if diff > 1 {
                                iter = Box::new(iter.skip(diff - 1));
                            }

                            if let Some(n) = diffs.position(|diff| diff != 1) {
                                diffs.nth(n.saturating_sub(1));
                                iter = Box::new(iter.take(n));
                            }
                        }

                    // worst case, indices in random order
                    } else {
                        // enumerate indices and swap so it's (index, enumeration)
                        let mut order = i.into_iter().enumerate()
                            .map(|(i, v)| (v, i))
                            .collect::<Vec<_>>();

                        // sort by index to get (sorted indices, order)
                        // we'll use this to sample the existing iterator then 
                        // permute it back into the original order
                        order.sort();
                        let (mut i, order): (Vec<_>, Vec<_>) = order.into_iter().unzip();

                        // we'll populate this with the sorted indices first
                        let mut indices: Vec<usize> = vec![0; i.len()];

                        i.insert(0, 0);
                        let diffs = i.windows(2).map(|w| w[1] - w[0]);

                        let mut last = iter.nth(0).expect("exhausted iterator");
                        for (i, diff) in diffs.enumerate() {
                            if diff > 0 {
                                last = iter.nth(diff - 1).expect("exhausted iterator");
                            }
                            indices[order[i]] = last;
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

impl<T: Atomic> From<Vec<T>> for Vector<T> {
    fn from(value: Vec<T>) -> Self {
        Vector::Subset(Rc::new(RefCell::new(value)), Subsets(Vec::new()))
    }
}

impl From<usize> for Subset {
    fn from(value: usize) -> Self {
        Subset::Indices(vec![value])
    }
}

impl From<Range<usize>> for Subset {
    fn from(value: Range<usize>) -> Self {
        Subset::Range(value)
    }
}

impl From<Vec<usize>> for Subset {
    fn from(value: Vec<usize>) -> Self {
        Subset::Indices(value)
    }
}

fn main() {
    // print out equivalent R code to mock up an interface

    let x: Vector<_> = vec![1, 2, 3, 4, 5, 6, 7, 8, 9, 10].into();
    let y: Vector<_> = vec![10, 9].into();
    println!("> x <- {}", x);
    println!("> y <- {}", y);

    // subset x by some indices
    let x_subset = x.subset(2..8).subset(vec![4, 2, 2]);
    println!("> x[3:8][c(5, 3, 3)]\n{}", x_subset);

    // modify
    x_subset.assign(y);
    println!("x[3:8][c(5, 3)] <- y");
    println!("> x\n{}", x);
}
