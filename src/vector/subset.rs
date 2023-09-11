use std::{ops::Range, cell::RefCell, rc::Rc};

use crate::lang::RSignal;

use super::vectors::{Vector, Integer, OptionNA};

/// Subsets
///
/// Representations of how data views might be specified. Indices are 0-indexed,
/// for direct use against rust-internal representations.
///
#[derive(Debug, Clone, PartialEq)]
pub enum Subset {
    Indices(Rc<RefCell<Vec<Integer>>>),
    Range(Range<usize>)
}

impl Subset {
    pub fn get_index_at(&self, index: usize) -> Option<usize> {
        match self {
            Subset::Indices(indices) => indices.clone().borrow()
                .get(index)
                .and_then(|i| match i {
                    OptionNA::Some(i) => Some((*i as usize).saturating_sub(1)),
                    OptionNA::NA => None,
                }),
            Subset::Range(range) => {
                if range.start <= index && index < range.end {
                    return Some(range.start + index)
                } else {
                    return None
                }
            },
        }
    }

    pub fn len(&self) -> usize {
        match self {
            Subset::Indices(i) => i.clone().borrow().len(),
            Subset::Range(r) => r.end - r.start,
        }
    }
}

impl From<usize> for Subset {
    fn from(value: usize) -> Self {
        Subset::Indices(Rc::new(RefCell::new(
            vec![OptionNA::Some(value as i32)]
        )))
    }
}

impl From<Range<usize>> for Subset {
    fn from(value: Range<usize>) -> Self {
        Subset::Range(value)
    }
}

impl From<Vec<usize>> for Subset {
    fn from(value: Vec<usize>) -> Self {
        Subset::Indices(Rc::new(RefCell::new(
            value.iter()
                .map(|i| OptionNA::Some(*i as i32))
                .collect::<Vec<_>>()
        )))
    }
}

impl TryFrom<Vector> for Subset {
    type Error = RSignal;
    fn try_from(value: Vector) -> Result<Self, Self::Error> {
        match value.as_integer() {
            Vector::Integer(v) => {
                let v = v.inner();

                // convert indices into 0-indexed values
                for i in v.borrow_mut().iter_mut() {
                    match i {
                        OptionNA::NA => (),
                        OptionNA::Some(x) => *x -= 1,
                    }
                }

                Ok(Subset::Indices(v))
            },
            _ => unreachable!()
        }
    }
}
