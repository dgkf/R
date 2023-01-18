use crate::ast::*;
use crate::builtins::Callable;
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
    Null,
    Vector(RVector),
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
            R::Null => Ok(R::Vector(RVector::Integer(vec![]))),
            _ => Err(RSignal::Error(RError::CannotBeCoercedToInteger)),
        }
    }

    pub fn as_numeric(self) -> EvalResult {
        match self {
            R::Vector(v) => Ok(R::Vector(v.as_numeric())),
            R::Null => Ok(R::Vector(RVector::Numeric(vec![]))),
            atom => unreachable!("{:?} cannot be coerced to numeric", atom),
        }
    }

    pub fn as_logical(self) -> EvalResult {
        match self {
            R::Vector(v) => Ok(R::Vector(v.as_logical())),
            R::Null => Ok(R::Vector(RVector::Logical(vec![]))),
            atom => unreachable!("{:?} cannot be coerced to logical", atom),
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

impl Default for R {
    fn default() -> Self {
        R::Null
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

impl std::ops::Add for R {
    type Output = EvalResult;

    fn add(self, rhs: Self) -> Self::Output {
        match (self.as_numeric()?, rhs.as_numeric()?) {
            (R::Vector(l), R::Vector(r)) => Ok(R::Vector(l + r)),
            _ => unreachable!(),
        }
    }
}

impl std::ops::Sub for R {
    type Output = EvalResult;

    fn sub(self, rhs: Self) -> Self::Output {
        match (self.as_numeric()?, rhs.as_numeric()?) {
            (R::Vector(l), R::Vector(r)) => Ok(R::Vector(l - r)),
            _ => unreachable!(),
        }
    }
}

impl std::ops::Neg for R {
    type Output = EvalResult;

    fn neg(self) -> Self::Output {
        match self.as_numeric()? {
            R::Vector(x) => Ok(R::Vector(-x)),
            _ => unreachable!(),
        }
    }
}

impl std::ops::Mul for R {
    type Output = EvalResult;

    fn mul(self, rhs: Self) -> Self::Output {
        match (self.as_numeric()?, rhs.as_numeric()?) {
            (R::Vector(l), R::Vector(r)) => Ok(R::Vector(l * r)),
            _ => unreachable!(),
        }
    }
}

impl std::ops::Div for R {
    type Output = EvalResult;

    fn div(self, rhs: Self) -> Self::Output {
        match (self.as_numeric()?, rhs.as_numeric()?) {
            (R::Vector(l), R::Vector(r)) => Ok(R::Vector(l / r)),
            _ => unreachable!(),
        }
    }
}

impl super::r_vector::vectors::Pow for R {
    type Output = EvalResult;

    fn power(self, rhs: Self) -> Self::Output {
        match (self.as_numeric()?, rhs.as_numeric()?) {
            (R::Vector(l), R::Vector(r)) => Ok(R::Vector(l.power(r))),
            _ => unreachable!(),
        }
    }
}

impl std::ops::Rem for R {
    type Output = EvalResult;

    fn rem(self, rhs: Self) -> Self::Output {
        match (self.as_numeric()?, rhs.as_numeric()?) {
            (R::Vector(l), R::Vector(r)) => Ok(R::Vector(l % r)),
            _ => unreachable!(),
        }
    }
}

impl std::ops::BitOr for R {
    type Output = EvalResult;

    fn bitor(self, rhs: Self) -> Self::Output {
        match (self.as_logical()?, rhs.as_logical()?) {
            (R::Vector(l), R::Vector(r)) => Ok(R::Vector(l | r)),
            _ => unreachable!(),
        }
    }
}

impl std::ops::BitAnd for R {
    type Output = EvalResult;

    fn bitand(self, rhs: Self) -> Self::Output {
        match (self.as_logical()?, rhs.as_logical()?) {
            (R::Vector(l), R::Vector(r)) => Ok(R::Vector(l & r)),
            _ => unreachable!(),
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

pub trait Context {
    fn eval(&mut self, expr: RExpr) -> EvalResult;
    fn eval_binary(&mut self, exprs: (RExpr, RExpr)) -> Result<(R, R), RSignal>;
    fn eval_list(&mut self, l: RExprList) -> EvalResult;
}

impl Context for Environment {
    fn eval(&mut self, expr: RExpr) -> EvalResult {
        use RVector::*;
        use R::*;

        match expr {
            RExpr::Null => Ok(Null),
            RExpr::NA => Ok(Vector(Logical(vec![OptionNA::NA]))),
            RExpr::Inf => Ok(Vector(Numeric(vec![OptionNA::Some(f64::INFINITY)]))),
            RExpr::Number(x) => Ok(Vector(RVector::from(vec![x]))),
            RExpr::Integer(x) => Ok(Vector(RVector::from(vec![x]))),
            RExpr::Bool(x) => Ok(Vector(Logical(vec![OptionNA::Some(x)]))),
            RExpr::String(x) => Ok(Vector(Character(vec![OptionNA::Some(x)]))),
            RExpr::Function(formals, body) => Ok(Function(formals, *body, Rc::clone(self))),
            RExpr::Symbol(name) => self.get(name),
            RExpr::List(x) => Ok(self.eval_list(x)?),
            RExpr::Break => Err(RSignal::Condition(Cond::Break)),
            RExpr::Continue => Err(RSignal::Condition(Cond::Continue)),
            RExpr::Call(what, args) => match *what {
                RExpr::Primitive(what) => Ok(what.call(args, self)?),
                RExpr::String(what) | RExpr::Symbol(what) => Ok(what.call(args, self)?),
                rexpr => (eval(rexpr, self)?).call(args, self),
            },
            x => unimplemented!("eval({})", x),
        }
    }

    fn eval_binary(&mut self, exprs: (RExpr, RExpr)) -> Result<(R, R), RSignal> {
        Ok((self.eval(exprs.0)?, self.eval(exprs.1)?))
    }

    fn eval_list(&mut self, l: RExprList) -> EvalResult {
        Ok(R::List(
            l.into_iter()
                .flat_map(|pair| match pair {
                    (_, RExpr::Ellipsis) => {
                        if let Ok(R::List(ellipsis)) = self.get_ellipsis() {
                            ellipsis.into_iter()
                        } else {
                            vec![].into_iter()
                        }
                    }
                    (k, e @ (RExpr::Call(..) | RExpr::Symbol(..))) => {
                        let elem = vec![(k, R::Closure(e, Rc::clone(self)))];
                        elem.into_iter()
                    }
                    (k, v) => {
                        if let Ok(elem) = eval(v, &mut Rc::clone(self)) {
                            vec![(k, elem)].into_iter()
                        } else {
                            unreachable!()
                        }
                    }
                })
                .collect(),
        ))
    }
}
