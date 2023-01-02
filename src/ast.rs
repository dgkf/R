use core::fmt;
use std::{iter::Zip, slice::IterMut, vec::IntoIter};

use crate::builtins::*;

#[derive(Debug, Clone)]
pub enum RExpr {
    Null,
    Bool(bool),
    Number(f32),
    Integer(i32),
    String(String),
    Symbol(String),
    List(RExprList),
    Function(RExprList, Box<RExpr>), // TODO: capture environment
    Call(Box<dyn Callable>, RExprList),
    Ellipsis,
}

impl fmt::Display for RExpr {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            RExpr::Null => write!(f, "NULL"),
            RExpr::Bool(true) => write!(f, "TRUE"),
            RExpr::Bool(false) => write!(f, "FALSE"),
            RExpr::Number(x) => write!(f, "{}", x),
            RExpr::Integer(x) => write!(f, "{}L", x),
            RExpr::String(x) => write!(f, "\"{}\"", x),
            RExpr::Symbol(x) => write!(f, "{}", x),
            RExpr::List(x) => write!(f, "{}", x),
            RExpr::Ellipsis => write!(f, "..."),
            x => write!(f, "{:?}", x),
        }
    }
}

#[derive(Debug, Clone, Default)]
pub struct RExprList {
    pub keys: Vec<Option<String>>, // TODO: use Vec<RExprListKey>
    pub values: Vec<RExpr>,
}

impl fmt::Display for RExprList {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let pairs: Vec<String> = self
            .values
            .iter()
            .enumerate()
            .map(|(i, v)| match &self.keys[i] {
                Some(k) => format!("{} = {}", k, v),
                None => format!("{}", v),
            })
            .collect();

        write!(f, "({})", pairs.join(", "))
    }
}

#[derive(Debug, Clone)]
pub struct RExprListItem(Option<String>, RExpr);

impl IntoIterator for RExprList {
    type Item = (Option<String>, RExpr);
    type IntoIter = <Zip<IntoIter<Option<String>>, IntoIter<RExpr>> as IntoIterator>::IntoIter;

    fn into_iter(self) -> Self::IntoIter {
        self.keys.into_iter().zip(self.values.into_iter())
    }
}

impl<'a> IntoIterator for &'a mut RExprList {
    type Item = (&'a mut Option<String>, &'a mut RExpr);
    type IntoIter =
        <Zip<IterMut<'a, Option<String>>, IterMut<'a, RExpr>> as IntoIterator>::IntoIter;

    fn into_iter(self) -> Self::IntoIter {
        self.keys.iter_mut().zip(self.values.iter_mut())
    }
}

impl FromIterator<(Option<String>, RExpr)> for RExprList {
    fn from_iter<T>(iter: T) -> Self
    where
        T: IntoIterator<Item = (Option<String>, RExpr)>,
    {
        let (keys, values) = iter.into_iter().unzip();
        RExprList { keys, values }
    }
}

impl FromIterator<RExpr> for RExprList {
    fn from_iter<T>(iter: T) -> Self
    where
        T: IntoIterator<Item = RExpr>,
    {
        let values: Vec<RExpr> = iter.into_iter().collect();
        RExprList {
            keys: vec![None; values.len()],
            values,
        }
    }
}

impl RExprList {
    pub fn new() -> RExprList {
        RExprList {
            ..Default::default()
        }
    }

    pub fn get_named(&self, key: String) -> Option<RExpr> {
        let first_name_index = self.keys.iter().position(|i| i == &Some(key.clone()));
        match first_name_index {
            Some(index) => Some(self.values[index].clone()),
            _ => None,
        }
    }

    pub fn get(&self, index: usize) -> Option<RExpr> {
        if index < self.values.len() {
            Some(self.values[index].clone())
        } else {
            None
        }
    }

    pub fn pop(&mut self) -> Option<(Option<String>, RExpr)> {
        if let Some(k) = self.keys.pop() {
            if let Some(v) = self.values.pop() {
                return Some((k, v));
            }
        }

        None
    }

    pub fn push(&mut self, pair: (Option<String>, RExpr)) {
        let (key, value) = pair;
        self.keys.push(key);
        self.values.push(value);
    }

    pub fn append(&mut self, mut other: Self) -> &mut RExprList {
        self.keys.append(&mut other.keys);
        self.values.append(&mut other.values);
        self
    }

    pub fn position_ellipsis(&self) -> Option<usize> {
        self.values.iter().position(|i| match i {
            RExpr::Ellipsis => true,
            _ => false,
        })
    }

    pub fn pop_trailing(&mut self) -> RExprList {
        if let Some(index) = self.position_ellipsis() {
            let keys_trailing = self.keys.drain(index..self.keys.len()).collect();
            let vals_trailing = self.values.drain(index..self.values.len()).collect();

            RExprList {
                keys: keys_trailing,
                values: vals_trailing,
            }
        } else {
            RExprList::new()
        }
    }

    pub fn remove_named(&mut self, key: &str) -> Option<(Option<String>, RExpr)> {
        let first_named_index = self.keys.iter().position(|i| i == &Some(key.to_string()));
        if let Some(index) = first_named_index {
            Some((self.keys.remove(index), self.values.remove(index)))
        } else {
            None
        }
    }

    pub fn remove(&mut self, index: usize) -> Option<(Option<String>, RExpr)> {
        if index < self.keys.len() {
            Some((self.keys.remove(index), self.values.remove(index)))
        } else {
            None
        }
    }

    pub fn insert_named(&mut self, key: String, value: RExpr) -> usize {
        if let Some(index) = self.keys.iter().position(|i| i == &Some(key.clone())) {
            self.values[index] = value;
            index
        } else {
            self.keys.push(Some(key.to_string()));
            self.values.push(value);
            self.values.len()
        }
    }

    pub fn insert(&mut self, index: usize, value: RExpr) -> usize {
        if index < self.values.len() {
            self.values.insert(index, value);
            index
        } else {
            let n = index - self.values.len();
            self.keys.extend(vec![None; n]);
            self.values.extend(vec![RExpr::Null; n - 1]);
            self.values.push(value);
            index
        }
    }

    pub fn binary_args(self) -> ((Option<String>, RExpr), (Option<String>, RExpr)) {
        let mut argstream = self.into_iter();
        let Some(lhs) = argstream.next() else {
            unimplemented!()
        };

        let Some(rhs) = argstream.next() else {
            unimplemented!()
        };

        (lhs, rhs)
    }

    pub fn unnamed_binary_args(self) -> (RExpr, RExpr) {
        let mut argstream = self.into_iter();
        let Some((_, lhs)) = argstream.next() else {
            unimplemented!()
        };

        let Some((_, rhs)) = argstream.next() else {
            unimplemented!()
        };

        (lhs, rhs)
    }
}

impl From<Vec<RExpr>> for RExprList {
    fn from(values: Vec<RExpr>) -> Self {
        RExprList {
            keys: vec![None; values.len()],
            values,
        }
    }
}

impl From<RExpr> for RExprList {
    fn from(value: RExpr) -> Self {
        RExprList {
            keys: vec![None],
            values: vec![value],
        }
    }
}

impl From<Vec<(Option<String>, RExpr)>> for RExprList {
    fn from(values: Vec<(Option<String>, RExpr)>) -> Self {
        RExprList::from_iter(values.into_iter())
    }
}
