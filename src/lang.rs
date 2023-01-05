use crate::ast::*;
use crate::error::*;
use crate::utils::eval;

use core::fmt;
use std::cmp::max;
use std::fmt::Display;
use std::{cell::RefCell, collections::HashMap, rc::Rc};

pub type EvalResult = Result<R, RSignal>;

#[derive(Debug, Clone)]
pub enum R {
    // Atomic types
    Logical(Logical),
    Numeric(Numeric),
    Integer(Integer),
    Character(Character),
    // Complex(Complex),
    // Raw(Raw),

    // Data structures
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

    pub fn get(&self, index: usize) -> Option<R> {
        match self {
            R::Logical(x) => {
                if let Some(val) = x.get(index) {
                    Some(R::Logical(vec![*val]))
                } else {
                    None
                }
            }
            R::Numeric(x) => {
                if let Some(val) = x.get(index) {
                    Some(R::Numeric(vec![*val]))
                } else {
                    None
                }
            }
            R::Integer(x) => {
                if let Some(val) = x.get(index) {
                    Some(R::Integer(vec![*val]))
                } else {
                    None
                }
            }
            R::Character(x) => {
                if let Some(val) = x.get(index) {
                    Some(R::Character(vec![val.clone()]))
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

    pub fn format_numeric(f: &mut fmt::Formatter<'_>, x: &Numeric) -> fmt::Result {
        let n = x.len();
        let nlen = format!("{}", n).len();

        if n == 0 {
            return write!(f, "numeric(0)");
        }

        let x_strs = x.iter().map(|xi| format!("{}", xi));
        let max_len = x_strs.clone().fold(0, |max_len, xi| max(max_len, xi.len()));

        let mut col = 0;
        x_strs.enumerate().try_for_each(|(i, x_str)| {
            col += max_len + 1;
            if i == 0 {
                write!(f, "{:>3$}[{}] {:>4$} ", "", i + 1, x_str, nlen - 1, max_len)
            } else if col > 80 - nlen - 3 {
                col = 0;
                let i_str = format!("{}", i + 1);
                let gutter = nlen - i_str.len();
                write!(f, "\n{:>3$}[{}] {:>4$} ", "", i_str, x_str, gutter, max_len)
            } else {
                write!(f, "{:>1$} ", x_str, max_len)
            }
        })
    }
}

impl TryInto<bool> for R {
    type Error = RSignal;

    fn try_into(self) -> Result<bool, Self::Error> {
        match self {
            R::Logical(vec) => match vec[..] {
                [true] => Ok(true),
                [false] => Ok(false),
                _ => Err(RSignal::Error(RError::ConditionIsNotScalar)),
            },
            R::Numeric(vec) => match vec[..] {
                [i] if i == 0.0 => Ok(true),
                [_] => Ok(false),
                _ => Err(RSignal::Error(RError::ConditionIsNotScalar)),
            },
            R::Integer(vec) => match vec[..] {
                [0] => Ok(true),
                [_] => Ok(false),
                _ => Err(RSignal::Error(RError::ConditionIsNotScalar)),
            },
            R::Character(vec) => match &vec[..] {
                [x] if x.as_str() == "TRUE" => Ok(true),
                [x] if x.as_str() == "FALSE" => Ok(false),
                [_] => Ok(false), // by R standards, this is "NA"
                _ => Err(RSignal::Error(RError::ConditionIsNotScalar)),
            },
            _ => Err(RSignal::Error(RError::CannotBeCoercedToLogical)),
        }
    }
}

impl From<Logical> for R {
    fn from(v: Logical) -> Self {
        R::Logical(v)
    }
}

impl From<Numeric> for R {
    fn from(v: Numeric) -> Self {
        R::Numeric(v)
    }
}

impl From<Integer> for R {
    fn from(v: Integer) -> Self {
        R::Integer(v)
    }
}

impl From<Character> for R {
    fn from(v: Character) -> Self {
        R::Character(v)
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
            R::Environment(x) => write!(f, "<environment {:?}>", x.values.as_ptr()),
            R::Function(formals, body, parent) => {
                let parent_env = R::Environment(Rc::clone(parent));
                write!(f, "function{} {}\n{}", formals, body, parent_env)
            }
            x => write!(f, "{:?}", x),
        }
    }
}

// // TODO: allow different internal types?
//
// pub enum MaybeNA<T> {
//     NA,
//     A(T)
// }
//
// pub enum MaybeFinite<T> {
//     NA,
//     NaN,
//     Inf,
//     NegInf,
//     Finite(T),
// }
//
// pub enum Numeric<T> {
//     Scalar(    MaybeNA<MaybeFinite<T>> ),
//     Vector(Vec<MaybeNA<MaybeFinite<T>>>),
// }

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
