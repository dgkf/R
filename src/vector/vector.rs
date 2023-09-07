use std::cell::RefCell;
use std::fmt::{Debug, Display};
use std::rc::Rc;

use super::coercion::*;
use super::types::atomic::{Atomic, IntoAtomic};
use super::types::modes::*;
use super::rep::{Subsets, Subset};

/// Vector
#[derive(Debug, Clone, PartialEq)]
pub enum Vector<T: Atomic> {
    // Vector::Subset encompasses a "raw" vector (no subsetting)
    Subset(Rc<RefCell<Vec<T>>>, Subsets),

    // Iterator includes things like ranges 1:Inf, and lazily computed values
    // Iter(Box<dyn Iterator<Item = &T>>)
}

impl<T: Atomic> Vector<T> {
    /// Create an empty vector
    ///
    /// The primary use case for this function is to support testing, and there
    /// are few expected use cases outside. It is used for creating a vector
    /// of an explicit atomic type, likely to be tested with 
    /// `SameType::is_same_type_as`.
    ///
    /// ```
    /// use vec::vector::Vector;
    /// use vec::types::OptionNa;
    /// use vec::utils::SameType;
    ///
    /// let result = Vector::from(vec![1, 2, 3]);
    /// let expect = Vector::<OptionNa<i32>>::new();
    /// assert!(result.is_same_type_as(&expect));
    /// ```
    /// 
    pub fn new() -> Self {
        Vector::Subset(Rc::new(RefCell::new(Vec::new())), Subsets(Vec::new()))
    }

    /// Access the internal vector
    pub fn inner(self) -> Vec<T> {
        match self.materialize() {
            Vector::Subset(v, _) => v.clone().borrow().to_vec()
        }
    }

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

    /// Get a single element from a vector
    ///
    /// Access a single element without materializing a new vector
    ///
    pub fn get(&self, index: usize) -> Option<Vector<T>> {
        match self {
            Vector::Subset(v, subsets) => {
                let vc = v.clone();
                let vb = vc.borrow();
                let index = subsets.get_index_at(index)?;
                let elem = vb.get(index)?;
                Some(Vector::Subset(Rc::new(RefCell::new(vec![elem.clone()])), Subsets::new()))
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

    /// Test the mode of the internal vector type
    ///
    /// Internally, this is defined by the [crate::types::atomic::AtomicMode] 
    /// implementation of the vector's element type.
    ///
    /// ```
    /// use vec::vector::Vector;
    /// let v = Vector::from(vec![0_f32, 100_f32]);
    /// assert!(v.is_numeric())
    /// ```
    ///
    pub fn is_numeric(&self) -> bool {
        self.is_numeric()
    }

    /// See [Self::is_numeric] for more information
    pub fn is_logical(&self) -> bool {
        self.is_logical()
    }
    
    /// See [Self::is_numeric] for more information
    pub fn is_integer(&self) -> bool {
        self.is_integer()
    }

    /// See [Self::is_numeric] for more information
    pub fn is_character(&self) -> bool {
        self.is_character()
    }

    /// Convert a Vector into a vector of a specific class of internal type
    ///
    /// The internal type only needs to satisfy [CoerceInto] for the `Mode`,
    /// and for the `Mode` type to implement [Atomic]. Generally, this
    /// is used more directly via [Self::as_logical], [Self::as_integer],
    /// [Self::as_numeric] and [Self::as_character], which predefine the output
    /// type of the mode.
    ///
    /// ```
    /// use vec::vector::Vector;
    ///
    /// let x = Vector::from(vec![false, true, true, false]);
    /// let n = x.as_numeric();
    ///
    /// assert_eq!(n, Vector::from(vec![0_f64, 1_f64, 1_f64, 0_f64]))
    /// ```
    ///
    pub fn as_mode<Mode>(&self) -> Vector<Mode> 
    where
        T: CoerceInto<Mode>,
        Mode: IntoAtomic<Atom = Mode> + Atomic,
    {
        match self {
            Vector::Subset(v, subsets) => {
                let vc = v.clone();
                let vb = vc.borrow();

                let num_vec: Vec<Mode> = vb.iter()
                    .map(|i| i.clone().coerce())
                    .collect();

                Vector::from((num_vec, subsets.clone()))
            },
        }
    }

    /// See [Self::as_mode] for more information
    pub fn as_logical(&self) -> Vector<Logical> where T: CoerceInto<Logical> {
        self.as_mode::<Logical>()
    }

    /// See [Self::as_mode] for more information
    pub fn as_integer(&self) -> Vector<Integer> where T: CoerceInto<Integer> {
        self.as_mode::<Integer>()
    }

    /// See [Self::as_mode] for more information
    pub fn as_numeric(&self) -> Vector<Numeric> where T: CoerceInto<Numeric> {
        self.as_mode::<Numeric>()
    }

    /// See [Self::as_mode] for more information
    pub fn as_character(&self) -> Vector<Character> where T: CoerceInto<Character> {
        self.as_mode::<Character>()
    }

    /// Apply over the vector contents to produce a vector of [std::cmp::Ordering]
    ///
    /// This function is used primarily in support of the implementation of
    /// vectorized comparison operators and likely does not need to be used
    /// outside of that context. 
    ///
    /// See [crate::vecops::VecPartialCmp] for vectorized comparison operator
    /// implementations.
    ///
    pub fn vectorized_partial_cmp<R, C>(self, other: Vector<R>) -> Vec<Option<std::cmp::Ordering>>
    where
        T: Atomic + CoerceInto<C>,
        R: Atomic + CoerceInto<C>,
        (T, R): CommonLog<Common = C>,
        C: PartialOrd,
    {
        zip_recycle(self.inner(), other.inner())
            .map(|(l, r)| {
                let lc = CoerceInto::<C>::coerce(l);
                let rc = CoerceInto::<C>::coerce(r);
                lc.partial_cmp(&rc)
            })
            .collect()
    }
}

impl<F, T> From<Vec<F>> for Vector<T> 
where
    F: IntoAtomic<Atom = T>,
    T: Atomic,
{
    fn from(value: Vec<F>) -> Self {
        let value: Vec<_> = value.into_iter().map(|i| i.atomic()).collect();
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

#[cfg(test)]
mod test{
    use super::*;
    use crate::vector::utils::SameType;
    use crate::vector::types::OptionNa;
    use crate::vector::vecops::*;

    #[test]
    fn vector_add() {
        let x = Vector::from((1..=10).into_iter().collect::<Vec<_>>());
        let y = Vector::from(vec![2, 5, 6, 2, 3]);

        let z = x + y;
        assert_eq!(z, Vector::from(vec![3, 7, 9, 6, 8, 8, 12, 14, 11, 13]));

        let expected_type = Vector::<OptionNa<i32>>::new();
        assert!(z.is_same_type_as(&expected_type));
        assert!(z.is_integer());
    }

    #[test]
    fn vector_mul() {
        let x = Vector::from((1..=10).into_iter().collect::<Vec<_>>());
        let y = Vector::from(vec![
            OptionNa(Some(2)), OptionNa(None), OptionNa(Some(6)), 
            OptionNa(None), OptionNa(Some(3))
        ]);

        let z = x * y;
        assert_eq!(z, Vector::from(vec![
            OptionNa(Some(2)), OptionNa(None), OptionNa(Some(18)), 
            OptionNa(None), OptionNa(Some(15)), OptionNa(Some(12)), 
            OptionNa(None), OptionNa(Some(48)), OptionNa(None), 
            OptionNa(Some(30))
        ]));

        let expected_type = Vector::<OptionNa<i32>>::new();
        assert!(z.is_same_type_as(&expected_type));
        assert!(z.is_integer());
    }

    #[test]
    fn vector_common_mul_f32_na() {
        // expect that f32's do not get coerced into an OptionNa, instead
        // using std::f32::NAN as NA representation.

        let x = Vector::from(vec![0_f32, std::f32::NAN, 10_f32]);
        let y = Vector::from(vec![100, 10]);

        let z = x * y;
        // assert_eq!(z, Vector::from(vec![0_f32, std::f32::NAN, 1_000_f32]));
        // comparing floats is error prone

        let expected_type = Vector::<f32>::new();
        assert!(z.is_same_type_as(&expected_type));
        assert!(z.is_numeric());
    }

    #[test]
    fn vector_and() {
        // expect that f32's do not get coerced into an OptionNa, instead
        // using std::f32::NAN as NA representation.

        let x = Vector::from(vec![0_f32, std::f32::NAN, 10_f32]);
        let y = Vector::from(vec![100, 10]);

        let z = x & y;
        assert_eq!(z, Vector::from(vec![OptionNa(Some(false)), OptionNa(None), OptionNa(Some(true))]));

        let expected_type = Vector::<OptionNa<bool>>::new();
        assert!(z.is_same_type_as(&expected_type));
        assert!(z.is_logical());
    }

    #[test]
    fn vector_gt() {
        // expect that f32's do not get coerced into an OptionNa, instead
        // using std::f32::NAN as NA representation.

        let x = Vector::from(vec![0_f32, std::f32::NAN, 10000_f32]);
        let y = Vector::from(vec![100, 10]);

        let z = x.vec_gt(y);
        assert_eq!(z, Vector::from(vec![
            OptionNa(Some(false)), 
            OptionNa(None), 
            OptionNa(Some(true))
        ]));

        let expected_type = Vector::<OptionNa<bool>>::new();
        assert!(z.is_same_type_as(&expected_type));
        assert!(z.is_logical());
    }
}
