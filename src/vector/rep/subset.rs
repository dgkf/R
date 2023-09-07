use std::ops::Range;

#[derive(Debug, Clone, PartialEq)]
pub enum Subset {
    Indices(Vec<usize>),
    Range(Range<usize>)
}

impl Subset {
    pub fn get_index_at(&self, index: usize) -> Option<usize> {
        match self {
            Subset::Indices(indices) => indices.get(index).map(|i| *i),
            Subset::Range(range) => {
                if range.start <= index && index < range.end {
                    return Some(range.start + index)
                } else {
                    return None
                }
            },
        }
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
