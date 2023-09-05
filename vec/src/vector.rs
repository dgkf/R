use std::cell::RefCell;
use std::fmt::{Debug, Display};
use std::rc::Rc;

use crate::coercion::{IntoNumeric, IntoLogical};
use super::atomic::{Atomic, IntoAtomic};
use super::subsets::{Subsets, Subset};

/// Vector
#[derive(Debug, Clone, PartialEq)]
pub enum Vector<T: Atomic> {
    // Vector::Subset encompasses a "raw" vector (no subsetting)
    Subset(Rc<RefCell<Vec<T>>>, Subsets),

    // Iterator includes things like ranges 1:Inf, and lazily computed values
    // Iter(Box<dyn Iterator<Item = &T>>)
}

impl<T: Atomic> Vector<T> {
    /// Subsetting a Vector
    ///
    /// Introduce a new subset into the aggregate list of subset indices.
    ///
    pub fn subset<S>(&self, subset: S) -> Self
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
    pub fn assign(&self, value: Self) {
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
    pub fn materialize(&self) -> Self {
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

    /// Convert a Vector into a Numeric type
    ///
    /// The numeric type is defined by the implementation of IntoNumeric,
    /// defining how the elements of Vector `self` map into a numeric
    /// representation.
    ///
    /// ```
    /// use vec::vector::Vector;
    ///
    /// let x = Vector::from(vec![false, true, true, false]);
    /// let n = x.as_numeric();
    ///
    /// assert_eq!(n, Vector::from(vec![0, 1, 1, 0]))
    /// ```
    ///
    pub fn as_numeric<N>(&self) -> Vector<N> 
    where
        T: IntoNumeric<Output = N>,
        N: Atomic
    {
        match self {
            Vector::Subset(v, subsets) => {
                let vc = v.clone();
                let vb = vc.borrow();

                let num_vec: Vec<N> = vb.iter()
                    .map(|i| i.clone().as_numeric())
                    .collect();

                Vector::from((num_vec, subsets.clone()))
            },
        }        
    }

    /// Test whether a Vector if of a Numeric Type
    ///
    /// Internally, this is defined by the AtomicMode implementation of 
    /// The vector's elements.
    ///
    /// ```
    /// use vec::vector::Vector;
    /// let v = Vector::from(vec![0_f32, 100_f32]);
    /// assert!(v.is_numeric())
    /// ```
    ///
    pub fn is_numeric(&self) -> bool {
        T::is_numeric()
    }

    /// Convert a Vector into a Logical type
    ///
    /// The logical type is defined by the implementation of IntoLogical, 
    /// defining how the elements of Vector `self` map into a logical 
    /// representation.
    ///
    /// ```
    /// use vec::vector::Vector;
    ///
    /// let x = Vector::from(vec![42_f64, 0_f64, -20_f64, 101_f64]);
    /// let l = x.as_logical();
    ///
    /// assert_eq!(l, Vector::from(vec![true, false, true, true]))
    /// ```
    ///
    pub fn as_logical<N, O>(&self) -> Vector<O> 
    where
        T: IntoLogical<Output = N>,
        N: IntoAtomic<Output = O>,
        O: Atomic
    {
        match self {
            Vector::Subset(v, subsets) => {
                let vc = v.clone();
                let vb = vc.borrow();

                let num_vec: Vec<N> = vb.iter()
                    .map(|i| i.clone().as_logical())
                    .collect();

                Vector::from((num_vec, subsets.clone()))
            },
        }        
    }
}

impl<F, T> From<Vec<F>> for Vector<T> 
where
    F: IntoAtomic<Output = T>,
    T: Atomic,
{
    fn from(value: Vec<F>) -> Self {
        let value: Vec<_> = value.into_iter().map(|i| i.into()).collect();
        Vector::Subset(Rc::new(RefCell::new(value)), Subsets(Vec::new()))
    }
}

impl<F, T> From<(Vec<F>, Subsets)> for Vector<T> 
where
    T: Atomic,
    Vector<T>: From<Vec<F>>
{
    fn from(value: (Vec<F>, Subsets)) -> Self {
        match Self::from(value.0) {
            Vector::Subset(v, _) => Vector::Subset(v, value.1),
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
