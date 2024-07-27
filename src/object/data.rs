use std::cell::{Ref, RefCell, RefMut};
use std::iter::Iterator;
use std::rc::Rc;

/// View an object mutably.
/// This trait drives the assignment into vectors and lists, primarily
/// via the `Data` struct.
pub trait ViewMut {
    fn view_mut(&self) -> Self;
}

/// Internal Data representation for copy-on-write semantics.
#[derive(Debug, PartialEq, Default)]
pub struct Data<T: Clone>(Rc<RefCell<Rc<T>>>);

impl<T: Clone> Clone for Data<T> {
    fn clone(&self) -> Self {
        Self::new(Rc::new(RefCell::new(self.0.borrow().clone())))
    }
}

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

    /// Borrow the internal data immutably.
    pub fn borrow(&self) -> Ref<'_, Rc<T>> {
        self.0.borrow()
    }
}

impl<T: Clone> ViewMut for Data<T> {
    /// Create a mutable view on the data.
    fn view_mut(&self) -> Self {
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

    pub fn iter(&self) -> DataIter<T> {
        self.clone().into_iter()
    }
}

pub type VecData<T> = Data<Vec<T>>;
#[cfg(test)]
mod tests {
    use super::VecData;
    use crate::object::ViewMut;

    #[test]
    fn with_inner_mut() {
        let x = VecData::from(vec![]);
        let _x1 = x.clone();
        let _x2 = x.view_mut();
        x.with_inner_mut(|v| v.push(1));
        assert_eq!(x.0.borrow().first().cloned().unwrap(), 1);
    }
}
