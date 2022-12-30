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
}

#[derive(Debug, Clone, Default)]
pub struct RExprList {
    pub keys: Vec<Option<String>>,
    pub values: Vec<RExpr>,
}

#[derive(Debug, Clone)]
pub enum RExprListKey {
    Name(String),
    Index(usize),
}

impl RExprList {
    pub fn get(&self, key: RExprListKey) -> Option<RExpr> {
        use RExprListKey::*;
        match key {
            Name(s) => {
                if let Some(index) = self.keys.iter().position(|i| i == &Some(s.clone())) {
                    Some(self.values[index].clone())
                } else {
                    None
                }
            }
            Index(i) => Some(self.values[i].clone()),
        }
    }

    pub fn insert(&mut self, key: Option<RExprListKey>, value: RExpr) -> usize {
        use RExprListKey::*;
        match key {
            Some(Name(s)) => {
                if let Some(index) = self.keys.iter().position(|i| i == &Some(s.clone())) {
                    self.values[index] = value;
                    index
                } else {
                    self.keys.push(Some(s));
                    self.values.push(value);
                    self.values.len()
                }
            }
            Some(Index(index)) => {
                if index < self.values.len() {
                    self.values[index] = value;
                    index
                } else {
                    let n = index - self.values.len();
                    self.keys.extend(vec![None; n]);
                    self.values.extend(vec![RExpr::Null; n - 1]);
                    self.values.push(value);
                    self.values.len()
                }
            }
            None => {
                self.keys.push(None);
                self.values.push(value);
                self.values.len()
            }
        }
    }
}

impl From<Vec<RExpr>> for RExprList {
    fn from(values: Vec<RExpr>) -> Self {
        let mut l = RExprList {
            ..Default::default()
        };
        for value in values {
            l.insert(None, value);
        }
        l
    }
}

impl From<Vec<(Option<String>, RExpr)>> for RExprList {
    fn from(values: Vec<(Option<String>, RExpr)>) -> Self {
        let mut l = RExprList {
            ..Default::default()
        };
        for (key, value) in values {
            match key {
                Some(s) => l.insert(Some(RExprListKey::Name(s)), value),
                None => l.insert(None, value),
            };
        }
        l
    }
}

impl FromIterator<RExpr> for RExprList {
    fn from_iter<T: IntoIterator<Item = RExpr>>(iter: T) -> Self {
        let mut l = RExprList {
            ..Default::default()
        };
        for value in iter {
            l.insert(None, value);
        }
        l
    }
}

impl FromIterator<(Option<String>, RExpr)> for RExprList {
    fn from_iter<T: IntoIterator<Item = (Option<String>, RExpr)>>(iter: T) -> Self {
        let mut l = RExprList {
            ..Default::default()
        };
        for (key, value) in iter {
            match key {
                Some(s) => l.insert(Some(RExprListKey::Name(s)), value),
                None => l.insert(None, value),
            };
        }
        l
    }
}
