use core::fmt;
use std::{iter::Zip, slice::IterMut, vec::IntoIter};

use crate::builtins::Primitive;

#[derive(Debug, Clone)]
pub enum Expr {
    Null,
    NA,
    Inf,
    Continue,
    Break,
    Ellipsis,
    Missing,
    Bool(bool),
    Number(f64),
    Integer(i32),
    String(String),
    Symbol(String),
    List(ExprList),
    Function(ExprList, Box<Expr>),
    Call(Box<Expr>, ExprList),
    Primitive(Box<dyn Primitive>),
}

impl Expr {
    pub fn as_primitive<T>(x: T) -> Self
    where
        T: Primitive + 'static,
    {
        Self::Primitive(Box::new(x))
    }

    pub fn new_primitive_call<T>(x: T, args: ExprList) -> Self
    where
        T: Primitive + 'static,
    {
        let p = Self::as_primitive(x);
        Self::Call(Box::new(p), args)
    }
}

impl fmt::Display for Expr {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Expr::Null => write!(f, "NULL"),
            Expr::Missing => write!(f, ""),
            Expr::Break => write!(f, "break"),
            Expr::Continue => write!(f, "continue"),
            Expr::Bool(true) => write!(f, "TRUE"),
            Expr::Bool(false) => write!(f, "FALSE"),
            Expr::Number(x) => write!(f, "{}", x),
            Expr::Integer(x) => write!(f, "{}L", x),
            Expr::String(x) => write!(f, "\"{}\"", x),
            Expr::Symbol(x) => write!(f, "{}", x),
            Expr::List(x) => write!(f, "{}", x),
            Expr::Ellipsis => write!(f, "..."),
            Expr::Call(what, args) => match &**what {
                Expr::Primitive(p) => write!(f, "{}", p.rfmt_call(args)),
                rexpr => write!(f, "{}({})", rexpr, args),
            },
            x => write!(f, "{:?}", x),
        }
    }
}

#[derive(Debug, Clone, Default)]
pub struct ExprList {
    pub keys: Vec<Option<String>>, // TODO: use Vec<RExprListKey>
    pub values: Vec<Expr>,
}

impl fmt::Display for ExprList {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let pairs: Vec<String> = self
            .values
            .iter()
            .enumerate()
            .map(|(i, v)| match (&self.keys[i], v) {
                (Some(k), Expr::Missing) => format!("{}", k),
                (Some(k), _) => format!("{} = {}", k, v),
                (None, v) => format!("{}", v),
            })
            .collect();

        write!(f, "{}", pairs.join(", "))
    }
}

#[derive(Debug, Clone)]
pub struct RExprListItem(Option<String>, Expr);

impl IntoIterator for ExprList {
    type Item = (Option<String>, Expr);
    type IntoIter = <Zip<IntoIter<Option<String>>, IntoIter<Expr>> as IntoIterator>::IntoIter;

    fn into_iter(self) -> Self::IntoIter {
        self.keys.into_iter().zip(self.values.into_iter())
    }
}

impl<'a> IntoIterator for &'a mut ExprList {
    type Item = (&'a mut Option<String>, &'a mut Expr);
    type IntoIter = <Zip<IterMut<'a, Option<String>>, IterMut<'a, Expr>> as IntoIterator>::IntoIter;

    fn into_iter(self) -> Self::IntoIter {
        self.keys.iter_mut().zip(self.values.iter_mut())
    }
}

impl FromIterator<(Option<String>, Expr)> for ExprList {
    fn from_iter<T>(iter: T) -> Self
    where
        T: IntoIterator<Item = (Option<String>, Expr)>,
    {
        let (keys, values) = iter.into_iter().unzip();
        ExprList { keys, values }
    }
}

impl FromIterator<Expr> for ExprList {
    fn from_iter<T>(iter: T) -> Self
    where
        T: IntoIterator<Item = Expr>,
    {
        let values: Vec<Expr> = iter.into_iter().collect();
        ExprList {
            keys: vec![None; values.len()],
            values,
        }
    }
}

impl ExprList {
    pub fn new() -> ExprList {
        ExprList {
            ..Default::default()
        }
    }

    pub fn get_named(&self, key: &String) -> Option<Expr> {
        let first_name_index = self.keys.iter().position(|i| i.as_ref() == Some(key));
        match first_name_index {
            Some(index) => Some(self.values[index].clone()),
            _ => None,
        }
    }

    pub fn get(&self, index: usize) -> Option<Expr> {
        if index < self.values.len() {
            Some(self.values[index].clone())
        } else {
            None
        }
    }

    pub fn pop(&mut self) -> Option<(Option<String>, Expr)> {
        if let Some(k) = self.keys.pop() {
            if let Some(v) = self.values.pop() {
                return Some((k, v));
            }
        }

        None
    }

    pub fn push(&mut self, pair: (Option<String>, Expr)) {
        let (key, value) = pair;
        self.keys.push(key);
        self.values.push(value);
    }

    pub fn append(&mut self, mut other: Self) -> &mut ExprList {
        self.keys.append(&mut other.keys);
        self.values.append(&mut other.values);
        self
    }

    pub fn position_ellipsis(&self) -> Option<usize> {
        self.values.iter().position(|i| match i {
            Expr::Ellipsis => true,
            _ => false,
        })
    }

    pub fn pop_trailing(&mut self) -> ExprList {
        if let Some(index) = self.position_ellipsis() {
            let keys_trailing = self.keys.drain(index..self.keys.len()).collect();
            let vals_trailing = self.values.drain(index..self.values.len()).collect();

            ExprList {
                keys: keys_trailing,
                values: vals_trailing,
            }
        } else {
            ExprList::new()
        }
    }

    pub fn remove_named(&mut self, key: &str) -> Option<(Option<String>, Expr)> {
        let first_named_index = self.keys.iter().position(|i| i == &Some(key.to_string()));
        if let Some(index) = first_named_index {
            Some((self.keys.remove(index), self.values.remove(index)))
        } else {
            None
        }
    }

    pub fn remove(&mut self, index: usize) -> Option<(Option<String>, Expr)> {
        if index < self.keys.len() {
            Some((self.keys.remove(index), self.values.remove(index)))
        } else {
            None
        }
    }

    pub fn insert_named(&mut self, key: String, value: Expr) -> usize {
        if let Some(index) = self.keys.iter().position(|i| i == &Some(key.clone())) {
            self.values[index] = value;
            index
        } else {
            self.keys.push(Some(key.to_string()));
            self.values.push(value);
            self.values.len()
        }
    }

    pub fn insert(&mut self, index: usize, value: Expr) -> usize {
        if index < self.values.len() {
            self.keys.insert(index, None);
            self.values.insert(index, value);
            index
        } else {
            let n = index - self.values.len();
            self.keys.extend(vec![None; n + 1]);
            self.values.extend(vec![Expr::Null; n]);
            self.values.push(value);
            index
        }
    }

    pub fn binary_args(self) -> ((Option<String>, Expr), (Option<String>, Expr)) {
        let mut argstream = self.into_iter();
        let Some(lhs) = argstream.next() else {
            unimplemented!()
        };

        let Some(rhs) = argstream.next() else {
            unimplemented!()
        };

        (lhs, rhs)
    }

    pub fn unnamed_binary_args(self) -> (Expr, Expr) {
        let mut argstream = self.into_iter();
        let Some((_, lhs)) = argstream.next() else {
            unimplemented!()
        };

        let Some((_, rhs)) = argstream.next() else {
            unimplemented!()
        };

        (lhs, rhs)
    }

    pub fn unnamed_unary_arg(self) -> Expr {
        let mut argstream = self.into_iter();
        let Some((_, lhs)) = argstream.next() else {
            unimplemented!()
        };

        lhs
    }

    /// Converts unnamed value symbols into missing named parameters
    pub fn as_formals(self) -> ExprList {
        self.into_iter()
            .map(|(k, v)| match (k, v) {
                (None, Expr::Symbol(param)) => (Some(param), Expr::Missing),
                other => other,
            })
            .collect()
    }

    pub fn len(&self) -> usize {
        self.values.len()
    }
}

impl From<Vec<Expr>> for ExprList {
    fn from(values: Vec<Expr>) -> Self {
        ExprList {
            keys: vec![None; values.len()],
            values,
        }
    }
}

impl From<Expr> for ExprList {
    fn from(value: Expr) -> Self {
        ExprList {
            keys: vec![None],
            values: vec![value],
        }
    }
}

impl From<Vec<(Option<String>, Expr)>> for ExprList {
    fn from(values: Vec<(Option<String>, Expr)>) -> Self {
        ExprList::from_iter(values.into_iter())
    }
}
