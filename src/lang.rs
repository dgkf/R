use crate::ast::*;
use crate::error::*;
use crate::utils::eval;

use std::{cell::RefCell, collections::HashMap, rc::Rc};

#[derive(Debug, Clone)]
pub enum R {
    Null,
    Logical(Logical),
    Numeric(Numeric),
    Integer(Integer),
    Character(Character),
    List(List),

    Expr(RExpr),
    Closure(RExpr, Environment),
    Function(RExprList, RExpr),
    Environment(Rc<Environment>),
}

impl R {
    pub fn as_integer(self) -> R {
        match self {
            R::Integer(_) => self,
            R::Numeric(v) => R::Integer(v.iter().map(|&i| i as i32).collect()),
            atom => unreachable!("{:?} cannot be coerced to integer", atom),
        }
    }

    pub fn as_numeric(self) -> R {
        match self {
            R::Numeric(_) => self,
            R::Integer(v) => R::Numeric(v.iter().map(|&i| i as f32).collect()),
            atom => unreachable!("{:?} cannot be coerced to numeric", atom),
        }
    }
}

pub type Logical = Vec<bool>;
pub type Numeric = Vec<f32>;
pub type Integer = Vec<i32>;
pub type Character = Vec<String>;
pub type List = Vec<(Option<String>, R)>;

pub type Environment = Rc<Env>;

#[derive(Debug, Default, Clone)]
pub struct Env {
    pub values: RefCell<HashMap<String, R>>,
    pub parent: Option<Environment>,
}

impl Env {
    pub fn get(&self, name: String) -> Result<R, RError> {
        // search in this environment for value by name
        if let Some(value) = self.values.borrow().get(&name) {
            let result = value.clone();
            return match result {
                R::Closure(expr, env) => eval(expr, &mut Rc::clone(&env)),
                _ => Ok(result),
            };

        // if not found, search through parent if available
        } else if let Some(parent) = &self.parent {
            parent.get(name)

        // otherwise, throw error
        } else {
            Err(RError::VariableNotFound(name))
        }
    }

    pub fn insert(&self, name: &String, value: R) {
        self.values.borrow_mut().insert(name.clone(), value);
    }
}
