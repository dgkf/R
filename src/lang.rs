use crate::ast::*;
use crate::error::*;
use crate::utils::eval;

use core::fmt;
use std::iter::repeat;
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
            R::Logical(v) => R::Integer(v.iter().map(|&i| i as i32).collect()),
            R::Numeric(v) => R::Integer(v.iter().map(|&i| i as i32).collect()),
            atom => unreachable!("{:?} cannot be coerced to integer", atom),
        }
    }

    pub fn as_numeric(self) -> R {
        match self {
            R::Numeric(_) => self,
            R::Logical(v) => R::Numeric(v.iter().map(|&i| i as i32 as f32).collect()),
            R::Integer(v) => R::Numeric(v.iter().map(|&i| i as f32).collect()),
            atom => unreachable!("{:?} cannot be coerced to numeric", atom),
        }
    }

    pub fn format_numeric(f: &mut fmt::Formatter<'_>, x: &Numeric) -> fmt::Result {
        let n = x.len();
        let nlen = format!("{}", n).len();

        if n == 0 {
            write!(f, "numeric(0)")?;
        } else {
            let pad: String = repeat(' ').take(nlen - 1).collect();
            write!(f, "{}[{}] ", pad, "1")?;
        }

        let mut col = nlen + 3;
        for (i, v) in x.iter().enumerate() {
            let rep = format!("{} ", v);
            col += rep.len();
            if col > 80 {
                col = nlen + 3 + rep.len();
                let pad: String = repeat(' ').take(nlen - rep.len()).collect();
                write!(f, "\n{}[{}] {}", pad, i, rep)?;
            } else {
                write!(f, "{}", rep)?;
            }
        }

        Ok(())
    }
}

impl fmt::Display for R {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            R::Null => write!(f, "NULL"),
            R::Logical(x) => write!(f, "[1] {}", x[0]),
            R::Numeric(x) => R::format_numeric(f, x),
            R::Integer(x) => write!(f, "[1] {}", x[0]),
            R::Character(x) => write!(f, "[1] \"{}\"", x[0]),
            // R::Function(formals, _) => write!(f, "<function({})>", formals.keys.len()),
            x => write!(f, "{:?}", x),
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
