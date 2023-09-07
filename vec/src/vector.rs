use std::cell::RefCell;
use std::fmt::{Debug, Display};
use std::rc::Rc;

use crate::coercion::{AsNumeric, AsLogical, CoerceInto, CommonNum, 
    map_common_numeric, zip_recycle, AsMinimallyNumeric, Pow, CommonCmp, OptionNa};
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
    /// Create an empty vector
    ///
    /// The primary use case for this function is to support testing, and there
    /// are few expected use cases outside. It is used for creating a vector
    /// of an explicit atomic type, likely to be tested with 
    /// `SameType::is_same_type_as`.
    ///
    /// ```
    /// use vec::vector::Vector;
    /// use vec::coercion::OptionNa;
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
    fn inner(self) -> Vec<T> {
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
    /// assert_eq!(n, Vector::from(vec![0_f64, 1_f64, 1_f64, 0_f64]))
    /// ```
    ///
    pub fn as_numeric<N, A>(&self) -> Vector<A> 
    where
        T: AsNumeric<As = N> + CoerceInto<N>,
        N: IntoAtomic<Atom = A>,
        A: Atomic
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

    pub fn is_logical(&self) -> bool {
        T::is_logical()
    }
    
    pub fn is_integer(&self) -> bool {
        T::is_integer()
    }

    pub fn is_character(&self) -> bool {
        T::is_character()
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
    pub fn as_logical<N, A>(&self) -> Vector<A> 
    where
        T: AsLogical<As = N>,
        N: IntoAtomic<Atom = A>,
        A: Atomic
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

    pub fn vectorized_partial_cmp<R, C>(self, other: Vector<R>) -> Vec<Option<std::cmp::Ordering>>
    where
        T: Atomic + CoerceInto<C>,
        R: Atomic + CoerceInto<C>,
        (T, R): CommonCmp<Common = C>,
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

impl<T, TNum, N, A> std::ops::Neg for Vector<T> 
where
    T: Atomic + AsMinimallyNumeric<As = TNum>,
    TNum: std::ops::Neg<Output = N>,
    N: IntoAtomic<Atom = A>,
    A: Atomic,
    Vector<A>: From<Vec<A>>,
{
    type Output = Vector<A>;
    fn neg(self) -> Self::Output {
        Vector::from(
            self.inner().into_iter()
                .map(|i| i.as_minimally_numeric().neg().atomic())
                .collect()
        )
    }
}

impl<L, R, C, O, A, LNum, RNum> std::ops::Add<Vector<R>> for Vector<L> 
where
    L: Atomic + AsMinimallyNumeric<As = LNum>,
    R: Atomic + AsMinimallyNumeric<As = RNum>,
    (LNum, RNum): CommonNum<Common = C>,
    C: std::ops::Add<Output = O>,
    O: IntoAtomic<Atom = A>,
    A: Atomic,
    Vector<A>: From<Vec<<C as std::ops::Add>::Output>>,
{
    type Output = Vector<A>;
    fn add(self, rhs: Vector<R>) -> Self::Output {
        Vector::from(
            map_common_numeric(zip_recycle(self.inner(), rhs.inner()))
                .map(|(l, r)| l + r)
                .collect()
        )
    }
}

impl<L, R, C, O, A, LNum, RNum> std::ops::Sub<Vector<R>> for Vector<L> 
where
    L: Atomic + AsMinimallyNumeric<As = LNum>,
    R: Atomic + AsMinimallyNumeric<As = RNum>,
    (LNum, RNum): CommonNum<Common = C>,
    C: std::ops::Sub<Output = O>,
    O: IntoAtomic<Atom = A>,
    A: Atomic,
    Vector<A>: From<Vec<<C as std::ops::Sub>::Output>>,
{
    type Output = Vector<A>;
    fn sub(self, rhs: Vector<R>) -> Self::Output {
        Vector::from(
            map_common_numeric(zip_recycle(self.inner(), rhs.inner()))
                .map(|(l, r)| l - r)
                .collect()
        )
    }
}

impl<L, R, C, O, A, LNum, RNum> std::ops::Mul<Vector<R>> for Vector<L> 
where
    L: Atomic + AsMinimallyNumeric<As = LNum>,
    R: Atomic + AsMinimallyNumeric<As = RNum>,
    (LNum, RNum): CommonNum<Common = C>,
    C: std::ops::Mul<Output = O>,
    O: IntoAtomic<Atom = A>,
    A: Atomic,
    Vector<A>: From<Vec<O>>,
{
    type Output = Vector<A>;
    fn mul(self, rhs: Vector<R>) -> Self::Output {
        Vector::from(
            map_common_numeric(zip_recycle(self.inner(), rhs.inner()))
                .map(|(l, r)| l * r)
                .collect()
        )
    }
}

impl<L, R, C, O, A, LNum, RNum> std::ops::Div<Vector<R>> for Vector<L> 
where
    L: Atomic + AsMinimallyNumeric<As = LNum>,
    R: Atomic + AsMinimallyNumeric<As = RNum>,
    (LNum, RNum): CommonNum<Common = C>,
    C: std::ops::Div<Output = O>,
    O: IntoAtomic<Atom = A>,
    A: Atomic,
    Vector<A>: From<Vec<<C as std::ops::Div>::Output>>,
{
    type Output = Vector<A>;
    fn div(self, rhs: Vector<R>) -> Self::Output {
        Vector::from(
            map_common_numeric(zip_recycle(self.inner(), rhs.inner()))
                .map(|(l, r)| l / r)
                .collect()
        )
    }
}

impl<L, A, LNum> Pow for Vector<L> 
where
    L: Atomic + AsMinimallyNumeric<As = LNum>,
    LNum: IntoAtomic<Atom = A> + Pow,
    A: Atomic,
    Vector<A>: From<Vec<<LNum as Pow>::Output>>,
{
    type Output = Vector<A>;
    fn power(self, rhs: Self) -> Self::Output {
        Vector::from(
            map_common_numeric(zip_recycle(self.inner(), rhs.inner()))
                .map(|(l, r)| l.power(r))
                .collect()
        )
    }
}

impl<L, R, C, O, A, LNum, RNum> std::ops::Rem<Vector<R>> for Vector<L> 
where
    L: Atomic + AsMinimallyNumeric<As = LNum>,
    R: Atomic + AsMinimallyNumeric<As = RNum>,
    (LNum, RNum): CommonNum<Common = C>,
    C: std::ops::Rem<Output = O>,
    O: IntoAtomic<Atom = A>,
    A: Atomic,
    Vector<A>: From<Vec<O>>,
{
    type Output = Vector<A>;
    fn rem(self, rhs: Vector<R>) -> Self::Output {
        Vector::from(
            map_common_numeric(zip_recycle(self.inner(), rhs.inner()))
                .map(|(l, r)| l.rem(r))
                .collect()
        )
    }
}

impl<L, R, O, A, LLog, RLog> std::ops::BitOr<Vector<R>> for Vector<L> 
where
    L: Atomic + AsLogical<As = LLog>,
    R: Atomic + AsLogical<As = RLog>,
    LLog: std::ops::BitOr<RLog, Output = O>,
    O: IntoAtomic<Atom = A>,
    A: Atomic,
{
    type Output = Vector<A>;
    fn bitor(self, rhs: Vector<R>) -> Self::Output {
        Vector::from(
            zip_recycle(self.inner(), rhs.inner())
                .map(|(l, r)| l.as_logical() | r.as_logical())
                .collect::<Vec<_>>()
        )
    }
}

impl<L, R, O, A, LLog, RLog> std::ops::BitAnd<Vector<R>> for Vector<L> 
where
    L: Atomic + AsLogical<As = LLog>,
    R: Atomic + AsLogical<As = RLog>,
    LLog: std::ops::BitAnd<RLog, Output = O>,
    O: IntoAtomic<Atom = A>,
    A: Atomic,
{
    type Output = Vector<A>;
    fn bitand(self, rhs: Vector<R>) -> Self::Output {
        Vector::from(
            zip_recycle(self.inner(), rhs.inner())
                .map(|(l, r)| l.as_logical() & r.as_logical())
                .collect::<Vec<_>>()
        )
    }
}

pub trait VecPartialCmp<Rhs> {
    type Output;
    fn vec_gt(self, rhs: Rhs) -> Self::Output;
    fn vec_gte(self, rhs: Rhs) -> Self::Output;
    fn vec_lt(self, rhs: Rhs) -> Self::Output;
    fn vec_lte(self, rhs: Rhs) -> Self::Output;
    fn vec_eq(self, rhs: Rhs) -> Self::Output;
    fn vec_neq(self, rhs: Rhs) -> Self::Output;
}

impl<L, R, C> VecPartialCmp<Vector<R>> for Vector<L>
where
    L: Atomic + CoerceInto<C>,
    R: Atomic + CoerceInto<C>,
    (L, R): CommonCmp<Common = C>,
    C: PartialOrd,
{
    type Output = Vector<OptionNa<bool>>;

    fn vec_gt(self, rhs: Vector<R>) -> Self::Output {
        use core::cmp::Ordering::*;
        Vector::from(
            self.vectorized_partial_cmp(rhs)
                .into_iter()
                .map(|i| match i {
                    Some(Greater) => OptionNa(Some(true)),
                    Some(_) => OptionNa(Some(false)),
                    None => OptionNa(None),
                })
                .collect::<Vec<_>>()
        )
    }

    fn vec_gte(self, rhs: Vector<R>) -> Self::Output {
        use core::cmp::Ordering::*;
        Vector::from(
            self.vectorized_partial_cmp(rhs)
                .into_iter()
                .map(|i| match i {
                    Some(Greater | Equal) => OptionNa(Some(true)),
                    Some(_) => OptionNa(Some(false)),
                    None => OptionNa(None),
                })
                .collect::<Vec<_>>()
        )
    }

    fn vec_lt(self, rhs: Vector<R>) -> Self::Output {
        use core::cmp::Ordering::*;
        Vector::from(
            self.vectorized_partial_cmp(rhs)
                .into_iter()
                .map(|i| match i {
                    Some(Less) => OptionNa(Some(true)),
                    Some(_) => OptionNa(Some(false)),
                    None => OptionNa(None),
                })
                .collect::<Vec<_>>()
        )
    }

    fn vec_lte(self, rhs: Vector<R>) -> Self::Output {
        use core::cmp::Ordering::*;
        Vector::from(
            self.vectorized_partial_cmp(rhs)
                .into_iter()
                .map(|i| match i {
                    Some(Less | Equal) => OptionNa(Some(true)),
                    Some(_) => OptionNa(Some(false)),
                    None => OptionNa(None),
                })
                .collect::<Vec<_>>()
        )
    }

    fn vec_eq(self, rhs: Vector<R>) -> Self::Output {
        use core::cmp::Ordering::*;
        Vector::from(
            self.vectorized_partial_cmp(rhs)
                .into_iter()
                .map(|i| match i {
                    Some(Equal) => OptionNa(Some(true)),
                    Some(_) => OptionNa(Some(false)),
                    None => OptionNa(None),
                })
                .collect::<Vec<_>>()
        )
    }

    fn vec_neq(self, rhs: Vector<R>) -> Self::Output {
        use core::cmp::Ordering::*;
        Vector::from(
            self.vectorized_partial_cmp(rhs)
                .into_iter()
                .map(|i| match i {
                    Some(Equal) => OptionNa(Some(false)),
                    Some(_) => OptionNa(Some(true)),
                    None => OptionNa(None),
                })
                .collect::<Vec<_>>()
        )
    }
}


#[cfg(test)]
mod test{
    use super::*;
    use crate::utils::SameType;
    use crate::coercion::OptionNa;

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
