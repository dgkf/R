use std::cell::{Ref, RefCell, RefMut};
use std::iter::Iterator;
use std::rc::Rc;

/// Internal Data representation for copy-on-write semantics.
#[derive(Debug, PartialEq, Default, Clone)]
pub struct Data<T: Clone>(Rc<RefCell<Rc<T>>>);

impl<T: Clone> From<T> for Data<T> {
    fn from(x: T) -> Self {
        Data::new(Rc::new(RefCell::new(Rc::new(x))))
    }
}

pub struct DataIter<T: Clone> {
    data: Rc<Vec<T>>,
    index: usize,
}

impl<T: Clone> Iterator for DataIter<T> {
    type Item = T;

    fn next(&mut self) -> Option<Self::Item> {
        if self.index < self.data.len() {
            let item = self.data[self.index].clone();
            self.index += 1;
            Some(item)
        } else {
            None
        }
    }
}

impl<T: Clone> IntoIterator for Data<Vec<T>> {
    type Item = T;
    type IntoIter = DataIter<T>;

    fn into_iter(self) -> Self::IntoIter {
        let x = self.borrow().clone();
        DataIter { data: x, index: 0 }
    }
}
impl<T: Clone> Data<T> {
    /// Create a new instance
    pub fn new(x: Rc<RefCell<Rc<T>>>) -> Self {
        Data(x)
    }

    /// Get mutable access to the internal vector.
    /// In case more than one reference to the internal data exists,
    /// the vector is cloned.
    pub fn with_inner_mut<F, R>(&self, f: F) -> R
    where
        F: FnOnce(&mut T) -> R,
    {
        let Data(x) = self;
        let x1 = &mut *x.borrow_mut();
        let vals = Rc::make_mut(x1);
        f(vals)
    }

    /// Get a mutable access to the data.
    pub fn borrow_mut(&self) -> RefMut<Rc<T>> {
        self.0.borrow_mut()
    }
    /// Borrow the internal data immutably.
    pub fn borrow(&self) -> Ref<'_, Rc<T>> {
        self.0.borrow()
    }

    // TODO: remove both methods below
    pub fn lazy_copy(&self) -> Self {
        Self::new(Rc::new(RefCell::new(self.0.borrow().clone())))
    }
    /// Create a mutable view on the data.
    pub fn mutable_view(&self) -> Self {
        Self::new(Rc::clone(&self.0))
    }
}

impl<T: Clone> Data<Vec<T>> {
    pub fn len(&self) -> usize {
        self.0.borrow().len()
    }

    pub fn is_empty(&self) -> bool {
        self.len() == 0
    }
}

pub type VecData<T> = Data<Vec<T>>;
#[cfg(test)]
mod tests {
    use super::VecData;
    use std::ptr;
    use std::rc::Rc;

    #[test]
    fn lazy_copy() {
        let x1: VecData<i32> = vec![1].into();

        let x2: VecData<i32> = x1.lazy_copy();
        let x3: VecData<i32> = x1.lazy_copy();
        let x4 = x1.mutable_view();

        // all point to the same data
        assert!(ptr::eq(x1.borrow().as_ref(), x2.borrow().as_ref()));
        assert!(ptr::eq(x2.borrow().as_ref(), x3.borrow().as_ref()));
        assert!(ptr::eq(x3.borrow().as_ref(), x4.borrow().as_ref()));

        // mutate x4 and thereby also x1
        {
            let mut b = x4.borrow_mut();
            let xmm = Rc::make_mut(&mut b);

            xmm[0] = 10;
        }
        assert_eq!(x1.borrow().as_ref()[0], 10);
        assert_eq!(x4.borrow().as_ref()[0], 10);

        // they still point to the same data
        assert!(ptr::eq(x1.borrow().as_ref(), x4.borrow().as_ref()));

        // now we modify x2, which should not modify anything else
        {
            let mut b = x2.borrow_mut();
            let xmm = Rc::make_mut(&mut b);
            xmm[0] = -10;
        }
        assert_eq!(x2.borrow().as_ref()[0], -10);
        assert_eq!(x3.borrow().as_ref()[0], 1);
    }

    #[test]
    fn with_inner_mut() {
        let x = VecData::from(vec![]);
        let _x1 = x.lazy_copy();
        let _x2 = x.mutable_view();
        x.with_inner_mut(|v| v.push(1));
        assert_eq!(x.0.borrow().first().cloned().unwrap(), 1);
    }
}
