use std::cell::{Ref, RefCell, RefMut};
use std::rc::Rc;

#[derive(Debug, PartialEq, Clone, Default)]
pub struct VecData<T>(Rc<RefCell<Rc<Vec<T>>>>);

pub struct VecDataIter<T> {
    data: VecData<T>,
    index: usize,
}

impl<T> IntoIterator for VecData<T>
where
    T: Clone + Default,
{
    type Item = T;
    type IntoIter = VecDataIter<T>;
    fn into_iter(self) -> Self::IntoIter {
        VecDataIter {
            data: self,
            index: 0,
        }
    }
}

impl<T> VecDataIter<T> {
    pub fn new(data: VecData<T>) -> Self {
        VecDataIter { data, index: 0 }
    }
}

impl<T: Clone> Iterator for VecDataIter<T> {
    type Item = T;

    fn next(&mut self) -> Option<Self::Item> {
        let vec = self.data.0.borrow();
        if self.index < vec.len() {
            let item = vec[self.index].clone();
            self.index += 1;
            Some(item)
        } else {
            None
        }
    }
}

/// Vector Data
///
/// This data structure allows to both get mutable views of the vector and to create lazy copies of
/// the vector.
/// The former is used to modify vectors in-place, while the latter is used to create new vectors
/// with an internal copy-on-write mechanism to avoid unnecessary clones.
impl<T: Clone> VecData<T> {
    /// Create a new instance
    pub fn new(x: Rc<RefCell<Rc<Vec<T>>>>) -> Self {
        VecData(x)
    }

    pub fn len(&self) -> usize {
        self.0.borrow().len()
    }

    /// Get mutable access to the internal vector.
    /// In case more than one reference to the internal data exists,
    /// the vector is cloned.
    pub fn with_inner_mut<F, R>(&self, f: F) -> R
    where
        F: FnOnce(&mut Vec<T>) -> R,
    {
        let VecData(x) = self;
        let x1 = &mut *x.borrow_mut();
        let vals = Rc::make_mut(x1);
        f(vals)
    }

    /// Create a (lazy) copy of the vector data by cloning the *inner* Rc.
    pub fn lazy_copy(&self) -> Self {
        Self::new(Rc::new(RefCell::new(self.0.borrow().clone())))
    }
    /// Create a mutable view on the data.
    pub fn mutable_view(&self) -> Self {
        Self::new(Rc::clone(&self.0))
    }
    /// Get a mutable access to the data.
    pub fn borrow_mut(&self) -> RefMut<Rc<Vec<T>>> {
        self.0.borrow_mut()
    }
    /// Borrow the internal data immutably.
    pub fn borrow(&self) -> Ref<'_, Rc<Vec<T>>> {
        self.0.borrow()
    }
}

impl<T: Clone> From<Vec<T>> for VecData<T> {
    fn from(x: Vec<T>) -> Self {
        VecData::new(Rc::new(RefCell::new(Rc::new(x))))
    }
}

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
        assert_eq!(x.0.borrow().get(0).cloned().unwrap(), 1);
    }
}
