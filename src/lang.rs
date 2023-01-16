use crate::ast::*;
use crate::error::*;
use crate::r_vector::vectors::*;
use crate::utils::eval;

use core::fmt;
use std::fmt::Display;
use std::{cell::RefCell, collections::HashMap, rc::Rc};

pub type EvalResult = Result<R, RSignal>;

#[derive(Debug, Clone)]
pub enum R {
    // Data structures
    Vector(RVector),
    Null,
    List(List),

    // Metaprogramming primitives
    Expr(RExpr),
    Closure(RExpr, Environment),
    Function(RExprList, RExpr, Environment),
    Environment(Environment),
}

#[derive(Debug, Clone)]
pub enum Cond {
    Break,
    Continue,
    Return(R),
}

#[derive(Debug, Clone)]
pub enum RSignal {
    Condition(Cond),
    Error(RError),
}

impl Display for RSignal {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            RSignal::Condition(_) => write!(f, "Signal used at top level"),
            RSignal::Error(e) => write!(f, "{}", e),
        }
    }
}

impl R {
    pub fn as_integer(self) -> EvalResult {
        match self {
            R::Vector(v) => Ok(R::Vector(v.as_integer())),
            _ => Err(RSignal::Error(RError::CannotBeCoercedToInteger)),
        }
    }

    pub fn as_numeric(self) -> R {
        match self {
            R::Vector(v) => R::Vector(v.as_numeric()),
            atom => unreachable!("{:?} cannot be coerced to numeric", atom),
        }
    }

    pub fn into_usize(&self) -> Result<usize, RSignal> {
        use OptionNA::*;
        use RVector::*;
        match self {
            R::Vector(rvec) => match rvec {
                Numeric(v) => match v[..] {
                    [Some(x)] => Ok(x as usize),
                    _ => Err(RSignal::Error(RError::CannotBeCoercedToInteger)),
                },
                Integer(v) => match v[..] {
                    [Some(x)] => Ok(x as usize),
                    _ => Err(RSignal::Error(RError::CannotBeCoercedToInteger)),
                },
                Logical(v) => match v[..] {
                    [Some(true)] => Ok(1 as usize),
                    _ => Err(RSignal::Error(RError::CannotBeCoercedToInteger)),
                },
                _ => Err(RSignal::Error(RError::CannotBeCoercedToInteger)),
            },
            _ => todo!(), // emit an appropriate error message
        }
    }

    pub fn get(&self, index: usize) -> Option<R> {
        match self {
            R::Vector(v) => {
                if let Some(v) = v.get(index) {
                    Some(R::Vector(v))
                } else {
                    None
                }
            }
            R::Null => None,
            R::List(_) => None,
            R::Expr(_) => None,
            R::Closure(_, _) => None,
            R::Function(_, _, _) => None,
            R::Environment(_) => None,
        }
    }

    pub(crate) fn try_get(&self, index: R) -> EvalResult {
        let i = index.into_usize()?;
        match self {
            R::Vector(rvec) => match rvec.get(i) {
                Some(v) => Ok(R::Vector(v)),
                None => Err(RSignal::Error(RError::Other("out of bounds".to_string()))),
            },
            _ => todo!(),
        }
    }
}

impl TryInto<bool> for R {
    type Error = RSignal;

    fn try_into(self) -> Result<bool, Self::Error> {
        match self {
            R::Vector(v) => match TryInto::<bool>::try_into(v) {
                Err(_) => Err(RSignal::Error(RError::CannotBeCoercedToLogical)),
                Ok(ok) => Ok(ok),
            },
            _ => Err(RSignal::Error(RError::CannotBeCoercedToLogical)),
        }
    }
}

impl fmt::Display for R {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            R::Vector(v) => write!(f, "{}", v),
            R::Null => write!(f, "NULL"),
            R::Environment(x) => write!(f, "<environment {:?}>", x.values.as_ptr()),
            R::Function(formals, body, parent) => {
                let parent_env = R::Environment(Rc::clone(parent));
                write!(f, "function({}) {}\n{}", formals, body, parent_env)
            }
            x => write!(f, "{:?}", x),
        }
    }
}

pub type List = Vec<(Option<String>, R)>;

pub type Environment = Rc<Env>;

#[derive(Debug, Default, Clone)]
pub struct Env {
    pub values: RefCell<HashMap<String, R>>,
    pub parent: Option<Environment>,
}

impl Env {
    pub fn get(&self, name: String) -> EvalResult {
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
            Err(RSignal::Error(RError::VariableNotFound(name)))
        }
    }

    pub fn get_ellipsis(&self) -> EvalResult {
        if let Ok(ellipsis) = self.get("...".to_string()) {
            Ok(ellipsis)
        } else {
            Err(RSignal::Error(RError::IncorrectContext("...".to_string())))
        }
    }

    pub fn insert(&self, name: String, value: R) {
        self.values.borrow_mut().insert(name, value);
    }

    pub fn append(&self, values: R) {
        match values {
            R::List(x) => {
                for (key, value) in x {
                    if let Some(name) = key {
                        self.insert(name, value)
                    } else {
                        println!("Dont' know what to do with value...")
                    }
                }
            }
            _ => unimplemented!(),
        }
    }
}
