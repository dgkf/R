use crate::ast::*;
use crate::callable::builtins::BUILTIN;
use crate::callable::core::{builtin, Callable};
use crate::error::*;
use crate::vector::subset::Subset;
use crate::vector::subsets::Subsets;
use crate::vector::vectors::*;

use core::fmt;
use std::fmt::Display;
use std::{cell::RefCell, collections::HashMap, rc::Rc};

pub type EvalResult = Result<R, RSignal>;

#[derive(Debug, Clone)]
pub enum R {
    // Data structures
    Null,
    Vector(Vector),
    List(List),

    // Metaprogramming structures
    Expr(Expr),
    Closure(Expr, Rc<Environment>),
    Function(ExprList, Expr, Rc<Environment>),
    Environment(Rc<Environment>),
}

impl PartialEq for R {
    fn eq(&self, other: &Self) -> bool {
        match (self, other) {
            (R::Null, R::Null) => true,
            (R::List(l), R::List(r)) => {
                let lb = l.values.borrow();
                let rb = r.values.borrow();
                let liter = lb.iter();
                let riter = rb.iter();
                liter
                    .zip(riter)
                    .all(|((lk, lv), (rk, rv))| lk == rk && lv == rv)
            }
            (R::Expr(l), R::Expr(r)) => l == r,
            (R::Closure(lc, lenv), R::Closure(rc, renv)) => lc == rc && lenv == renv,
            (R::Function(largs, lbody, lenv), R::Function(rargs, rbody, renv)) => {
                largs == rargs && lbody == rbody && lenv == renv
            }
            (R::Environment(l), R::Environment(r)) => {
                l.values.as_ptr() == r.values.as_ptr()
                    && (match (&l.parent, &r.parent) {
                        (None, None) => true,
                        (Some(lp), Some(rp)) => {
                            Rc::<Environment>::as_ptr(&lp) == Rc::<Environment>::as_ptr(&rp)
                        }
                        _ => false,
                    })
            }
            (R::Vector(lv), R::Vector(rv)) => match (lv, rv) {
                (Vector::Numeric(l), Vector::Numeric(r)) => l == r,
                (Vector::Integer(l), Vector::Integer(r)) => l == r,
                (Vector::Logical(l), Vector::Logical(r)) => l == r,
                (Vector::Character(l), Vector::Character(r)) => l == r,
                _ => false,
            },
            _ => false,
        }
    }
}

impl TryInto<i32> for R {
    type Error = RSignal;
    fn try_into(self) -> Result<i32, Self::Error> {
        use RError::CannotBeCoercedToInteger;

        let R::Vector(Vector::Integer(v)) = self.as_integer()? else {
            unreachable!();
        };

        match v.inner().clone().borrow()[..] {
            [OptionNA::Some(i), ..] => Ok(i),
            _ => Err(CannotBeCoercedToInteger.into()),
        }
    }
}

impl<T> From<T> for R
where
    Vector: From<T>,
{
    fn from(value: T) -> Self {
        R::Vector(Vector::from(value))
    }
}

impl TryInto<f64> for R {
    type Error = RSignal;
    fn try_into(self) -> Result<f64, Self::Error> {
        use RError::CannotBeCoercedToNumeric;

        let R::Vector(Vector::Numeric(v)) = self.as_numeric()? else {
            unreachable!();
        };

        match v.inner().clone().borrow()[..] {
            [OptionNA::Some(i), ..] => Ok(i),
            _ => Err(CannotBeCoercedToNumeric.into()),
        }
    }
}

impl TryInto<Vec<f64>> for R {
    type Error = RSignal;
    fn try_into(self) -> Result<Vec<f64>, Self::Error> {
        let R::Vector(Vector::Numeric(v)) = self.as_numeric()? else {
            unreachable!();
        };

        Ok(v.inner()
            .clone()
            .borrow()
            .iter()
            .map(|vi| match vi {
                OptionNA::Some(i) => *i,
                OptionNA::NA => f64::NAN,
            })
            .collect())
    }
}

#[derive(Debug, Clone, PartialEq)]
pub enum Cond {
    Break,
    Continue,
    Terminate,
    Return(R),
}

#[derive(Debug, Clone, PartialEq)]
pub enum RSignal {
    Condition(Cond),
    Error(RError),
    Thunk, // used when evaluating null opts like comments
}

impl Display for RSignal {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            RSignal::Condition(_) => write!(f, "Signal used at top level"),
            RSignal::Error(e) => write!(f, "{}", e),
            RSignal::Thunk => write!(f, ""),
        }
    }
}

impl R {
    pub fn force(self, stack: &mut CallStack) -> EvalResult {
        match self {
            R::Closure(expr, env) => {
                stack.add_frame(expr.clone(), env);
                let result = stack.eval(expr);
                stack.pop_frame_after(result)
            }
            _ => Ok(self),
        }
    }

    pub fn assign(self, value: R) -> EvalResult {
        // TODO(ERROR) cleanup
        let err = RError::Other("Invalid target for assignment".to_string());

        match self {
            R::Vector(mut v) => {
                v.assign(value.clone().as_vector()?)?;
                Ok(value)
            }
            R::List(mut l) => {
                l.assign(value.clone())?;
                Ok(value)
            }
            _ => Err(err.into()),
        }
    }

    pub fn as_integer(self) -> EvalResult {
        match self {
            R::Vector(v) => Ok(R::Vector(v.as_integer())),
            R::Null => Ok(R::Vector(Vector::from(Vec::<Integer>::new()))),
            _ => Err(RSignal::Error(RError::CannotBeCoercedToInteger)),
        }
    }

    pub fn as_numeric(self) -> EvalResult {
        match self {
            R::Vector(v) => Ok(R::Vector(v.as_numeric())),
            R::Null => Ok(R::Vector(Vector::from(Vec::<Numeric>::new()))),
            _ => RError::CannotBeCoercedToNumeric.into(),
        }
    }

    pub fn as_logical(self) -> EvalResult {
        match self {
            R::Vector(v) => Ok(R::Vector(v.as_logical())),
            R::Null => Ok(R::Vector(Vector::from(Vec::<Logical>::new()))),
            atom => unreachable!("{:?} cannot be coerced to logical", atom),
        }
    }

    pub fn as_character(self) -> EvalResult {
        match self {
            R::Vector(v) => Ok(R::Vector(v.as_character())),
            R::Null => Ok(R::Vector(Vector::from(Vec::<Character>::new()))),
            atom => unreachable!("{:?} cannot be coerced to character", atom),
        }
    }

    pub fn as_vector(self) -> EvalResult {
        match self {
            R::Null => Ok(R::Vector(Vector::from(Vec::<Logical>::new()))),
            R::Vector(_) => Ok(self),
            _ => unimplemented!("cannot coerce object into vector"),
        }
    }

    pub fn into_usize(&self) -> Result<usize, RSignal> {
        use OptionNA::*;
        use Vector::*;
        match self {
            R::Vector(rvec) => match rvec {
                Numeric(v) => match v.inner().clone().borrow()[..] {
                    [Some(x)] => Ok(x as usize),
                    _ => Err(RSignal::Error(RError::CannotBeCoercedToInteger)),
                },
                Integer(v) => match v.inner().clone().borrow()[..] {
                    [Some(x)] => Ok(x as usize),
                    _ => Err(RSignal::Error(RError::CannotBeCoercedToInteger)),
                },
                Logical(v) => match v.inner().clone().borrow()[..] {
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

    pub fn get_named(&mut self, name: &str) -> Option<R> {
        match self {
            R::List(v) => v
                .values
                .borrow()
                .iter()
                .find(|(k, _)| *k == Some(String::from(name)))
                .map(|(_, v)| v.clone()),
            R::Environment(e) => match e.get(String::from(name)) {
                Ok(v) => Some(v),
                Err(_) => None,
            },
            _ => None,
        }
    }

    pub fn set_named(&mut self, name: &str, value: R) -> EvalResult {
        match self {
            R::List(v) => {
                let mut vb = v.values.borrow_mut();

                let loc = vb
                    .iter()
                    .enumerate()
                    .find(|(_, (k, _))| *k == Some(name.into()))
                    .map(|(i, _)| i);

                match loc {
                    Some(i) => vb[i].1 = value.clone(),
                    None => vb.push((Some(name.into()), value.clone())),
                }

                Ok(value)
            }
            R::Environment(e) => {
                e.values.borrow_mut().insert(name.into(), value.clone());
                Ok(value)
            }
            _ => Ok(R::Null),
        }
    }

    pub fn environment(&self) -> Option<Rc<Environment>> {
        match self {
            R::Closure(_, e) | R::Function(_, _, e) | R::Environment(e) => Some(e.clone()),
            _ => None,
        }
    }

    pub fn try_get_named(&mut self, name: &str) -> EvalResult {
        use RError::{ArgumentMissing, VariableNotFound};
        match self.get_named(name) {
            Some(R::Closure(Expr::Missing, _)) => Err(ArgumentMissing(name.into()).into()),
            Some(x) => Ok(x),
            None => Err(VariableNotFound(name.into()).into()),
        }
    }

    pub fn try_get(&self, index: R) -> EvalResult {
        match self {
            R::Vector(v) => v.try_get(index),
            R::List(l) => l.try_get(index),
            _ => todo!(),
        }
    }

    pub fn try_get_inner(&self, index: R) -> EvalResult {
        match self {
            R::Vector(v) => v.try_get(index),
            R::List(l) => l.try_get_inner(index),
            _ => todo!(),
        }
    }

    fn len(&self) -> Option<usize> {
        match self {
            R::Vector(v) => Some(v.len()),
            R::List(l) => Some(l.len()),
            _ => None,
        }
    }
}

impl Default for R {
    fn default() -> Self {
        R::Null
    }
}

impl TryInto<List> for R {
    type Error = RSignal;

    fn try_into(self) -> Result<List, Self::Error> {
        match self {
            R::List(l) => Ok(l),
            _ => unimplemented!(),
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

impl Display for R {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            R::Vector(v) => write!(f, "{}", v),
            R::Null => write!(f, "NULL"),
            R::Environment(x) => write!(f, "<environment {:?}>", x.values.as_ptr()),
            R::Function(formals, Expr::Primitive(primitive), _) => {
                write!(
                    f,
                    "function({}) .Primitive(\"{}\")",
                    formals,
                    primitive.rfmt()
                )
            }
            R::Function(formals, body, parent_env) => {
                let parent_env = R::Environment(Rc::clone(parent_env));
                write!(f, "function({}) {}\n{}", formals, body, parent_env)
            }
            R::List(vals) => display_list(vals, f, None),
            R::Closure(expr, env) => write!(f, "{expr} @ {env}"),
            R::Expr(expr) => write!(f, "{}", expr),
        }
    }
}

fn display_list(x: &List, f: &mut fmt::Formatter<'_>, bc: Option<String>) -> fmt::Result {
    let v = x.values.borrow();
    let s = x.subsets.clone();

    let names: Vec<_> = x.values.borrow().clone().into_iter().map(|(n, _)| n).collect();
    for (i, (_, si)) in s.bind_names(names).into_iter().take(v.len()).enumerate() {
        let name;
        let value;

        if let Some(i) = si {
            (name, value) = v[i].clone();
        } else {
            return write!(f, "{}", R::Null);
        }

        if i > 0 {
            write!(f, "\n")?
        }

        let bc_elem = if let Some(name) = name {
            format!("${}", name)
        } else {
            format!("[[{}]]", i + 1)
        };

        let breadcrumbs = match bc.clone() {
            Some(bc_prev) => format!("{}{}", bc_prev, bc_elem),
            _ => bc_elem,
        };

        match value {
            R::List(nested_values) => {
                write!(f, "{}\n", breadcrumbs)?;
                display_list(&nested_values, f, Some(breadcrumbs))?
            }
            _ => write!(f, "{}\n{}\n", breadcrumbs, value)?,
        }
    }

    Ok(())
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

impl super::vector::vectors::Pow<R> for R {
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

impl VecPartialCmp<R> for R {
    type Output = EvalResult;
    fn vec_gt(self, rhs: Self) -> Self::Output {
        match (self.as_vector()?, rhs.as_vector()?) {
            (R::Vector(l), R::Vector(r)) => Ok(R::Vector(l.vec_gt(r))),
            _ => unreachable!(),
        }
    }

    fn vec_gte(self, rhs: Self) -> Self::Output {
        match (self.as_vector()?, rhs.as_vector()?) {
            (R::Vector(l), R::Vector(r)) => Ok(R::Vector(l.vec_gte(r))),
            _ => unreachable!(),
        }
    }

    fn vec_lt(self, rhs: Self) -> Self::Output {
        match (self.as_vector()?, rhs.as_vector()?) {
            (R::Vector(l), R::Vector(r)) => Ok(R::Vector(l.vec_lt(r))),
            _ => unreachable!(),
        }
    }

    fn vec_lte(self, rhs: Self) -> Self::Output {
        match (self.as_vector()?, rhs.as_vector()?) {
            (R::Vector(l), R::Vector(r)) => Ok(R::Vector(l.vec_lte(r))),
            _ => unreachable!(),
        }
    }

    fn vec_eq(self, rhs: Self) -> Self::Output {
        match (self, rhs) {
            (lhs @ R::Expr(_), rhs @ R::Expr(_)) => Ok((lhs == rhs).into()),
            (lhs @ R::Closure(..), rhs @ R::Closure(..)) => Ok((lhs == rhs).into()),
            (lhs @ R::Function(..), rhs @ R::Function(..)) => Ok((lhs == rhs).into()),
            (lhs @ R::Environment(_), rhs @ R::Environment(_)) => Ok((lhs == rhs).into()),
            (lhs, rhs) => match (lhs.as_vector()?, rhs.as_vector()?) {
                (R::Vector(l), R::Vector(r)) => Ok(R::Vector(l.vec_eq(r))),
                _ => unreachable!(),
            },
        }
    }

    fn vec_neq(self, rhs: Self) -> Self::Output {
        match (self, rhs) {
            (lhs @ R::Expr(_), rhs @ R::Expr(_)) => Ok((lhs != rhs).into()),
            (lhs @ R::Closure(..), rhs @ R::Closure(..)) => Ok((lhs != rhs).into()),
            (lhs @ R::Function(..), rhs @ R::Function(..)) => Ok((lhs != rhs).into()),
            (lhs @ R::Environment(_), rhs @ R::Environment(_)) => Ok((lhs != rhs).into()),
            (lhs, rhs) => match (lhs.as_vector()?, rhs.as_vector()?) {
                (R::Vector(l), R::Vector(r)) => Ok(R::Vector(l.vec_neq(r))),
                _ => unreachable!(),
            },
        }
    }
}

#[derive(Debug, Clone, PartialEq, Default)]
pub struct List {
    pub values: Rc<RefCell<Vec<(Option<String>, R)>>>,
    pub subsets: Subsets,
}

impl From<Vec<(Option<String>, R)>> for List {
    fn from(value: Vec<(Option<String>, R)>) -> Self {
        List {
            values: Rc::new(RefCell::new(value)),
            ..Default::default()
        }
    }
}

impl List {
    pub fn subset(&self, by: Subset) -> List {
        let Subsets(mut inner) = self.subsets.clone();
        inner.push(by);
        List {
            values: self.values.clone(),
            subsets: Subsets(inner),
        }
    }

    pub fn assign(&mut self, value: R) -> EvalResult {
        let names: Vec<_> = self.values.borrow().clone().into_iter().map(|(n, _)| n).collect();
        match value {
            // remove elements from list
            R::Null => {
                let mut v = self.values.borrow_mut();
                let n = v.len();
                let indices = self.subsets.clone().bind_names(names).into_iter().take(n);
                for (i, _) in indices {
                    v.remove(i);
                }
                Ok(R::List(List {
                    values: self.values.clone(),
                    subsets: self.subsets.clone(),
                }))
            }
            // any single length R value
            any if any.len() == Some(1) => {
                let mut v = self.values.borrow_mut();
                let n = v.len();
                let indices = self.subsets.clone().bind_names(names.clone()).into_iter().take(n);

                // first check to see if we need to extend
                if let Some(max) = self.subsets.clone().bind_names(names).into_iter().map(|(i, _)| i).max() {
                    v.reserve(max.saturating_sub(n))
                }

                // then assign to indices
                for (_, i) in indices {
                    if let Some(i) = i {
                        v[i].1 = any.clone()
                    }
                }

                Ok(R::List(List {
                    values: self.values.clone(),
                    subsets: self.subsets.clone(),
                }))
            }
            // vectorized assignment
            // TODO(feature): warn when index recycling does not cycle evenly
            any if any.len() == Some(self.len()) => {
                let mut v = self.values.borrow_mut();
                let n = v.len();
                let indices = self.subsets.clone().bind_names(names.clone()).into_iter().take(n);

                // first check to see if we need to extend
                if let Some(max) = self.subsets.clone().bind_names(names).into_iter().map(|(i, _)| i).max() {
                    v.reserve(max.saturating_sub(n))
                }

                // then assign to indices
                for (any_i, (_, i)) in indices.enumerate() {
                    if let (Some(value), Some(i)) = (any.get(any_i), i) {
                        v[i].1 = value;
                    }
                }

                Ok(R::List(List {
                    values: self.values.clone(),
                    subsets: self.subsets.clone(),
                }))
            }
            other => {
                let mut v = self.values.borrow_mut();
                let n = v.len();
                let indices = self.subsets.clone().bind_names(names.clone()).into_iter().take(n);

                // first check to see if we need to extend
                if let Some(max) = self.subsets.clone().bind_names(names).into_iter().map(|(i, _)| i).max() {
                    v.reserve(max.saturating_sub(n))
                }

                // then assign to indices
                for (_, i) in indices {
                    if let Some(i) = i {
                        v[i].1 = other.clone()
                    }
                }

                Ok(R::List(List {
                    values: self.values.clone(),
                    subsets: self.subsets.clone(),
                }))
            }
        }
    }

    fn try_get(&self, index: R) -> EvalResult {
        let err = RError::Other("Cannot use object for indexing.".to_string());
        match index.as_vector()? {
            R::Vector(v) => Ok(R::List(self.subset(v.try_into()?))),
            _ => Err(err.into()),
        }
    }

    fn try_get_inner(&self, index: R) -> EvalResult {
        let err = RError::Other("Cannot use object for indexing.".to_string());
        let names: Vec<_> = self.values.borrow().clone().into_iter().map(|(n, _)| n).collect();
        match index.as_vector()? {
            R::Vector(v) if v.len() == 1 => {
                let Subsets(mut subsets) = self.subsets.clone();
                subsets.push(v.try_into()?);

                if let Some((i, _)) = Subsets(subsets).bind_names(names).into_iter().next() {
                    self.values
                        .borrow()
                        .get(i)
                        .map_or(Err(err.into()), |(_, i)| Ok(i.clone()))
                } else {
                    Ok(R::Null)
                }
            }
            _ => Err(err.into()),
        }
    }

    fn len(&self) -> usize {
        let Subsets(inner) = &self.subsets;
        match inner.as_slice() {
            [] => self.values.borrow().len(),
            [.., last] => std::cmp::min(self.values.borrow().len(), last.len()),
        }
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct Frame {
    pub call: Expr,
    pub env: Rc<Environment>,
}

impl Frame {
    pub fn new(call: Expr, env: Environment) -> Frame {
        Self {
            call: call.clone(),
            env: Rc::new(env),
        }
    }

    pub fn new_child_env(&self) -> Rc<Environment> {
        Rc::new(Environment {
            parent: Some(self.env.clone()),
            ..Default::default()
        })
    }
}

impl Display for Frame {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.call)
    }
}

#[derive(Debug, Default, Clone, PartialEq)]
pub struct CallStack {
    pub frames: Vec<Frame>,
}

impl CallStack {
    pub fn add_frame(&mut self, call: Expr, env: Rc<Environment>) -> usize {
        self.frames.push(Frame {
            call,
            env: env.clone(),
        });

        self.frames.len()
    }

    pub fn frame(&self, n: i32) -> Option<&Frame> {
        match n {
            i if i <= 0 => self.frames.get((self.frames.len() as i32 - 1 + i) as usize),
            i if i > 0 => self.frames.get(i as usize),
            _ => unreachable!(),
        }
    }

    pub fn last_frame(&self) -> &Frame {
        if let Some(frame) = self.frames.last() {
            frame
        } else {
            panic!("We've somehow exhausted the entire call stack and are still evaluating")
        }
    }

    pub fn parent_frame(&self) -> &Frame {
        if let Some(frame) = self.frame(-1) {
            frame
        } else {
            panic!("Attempting access to parent frame at top level")
        }
    }

    pub fn pop_frame_after(&mut self, result: EvalResult) -> EvalResult {
        match result {
            Ok(..) => {
                self.frames.pop();
                result
            }
            error => error,
        }
    }

    pub fn new() -> CallStack {
        CallStack::from(Frame {
            call: Expr::Null,
            env: Rc::new(Environment::default()),
        })
    }
}

impl Display for CallStack {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        for (i, frame) in self.frames.iter().enumerate().skip(1) {
            writeln!(f, "{}: {} {}", i, frame.call, frame.env.clone())?;
        }
        Ok(())
    }
}

impl From<Frame> for CallStack {
    fn from(frame: Frame) -> Self {
        Self {
            frames: vec![frame],
        }
    }
}

impl From<Rc<Environment>> for CallStack {
    fn from(value: Rc<Environment>) -> Self {
        CallStack {
            frames: vec![Frame {
                call: Expr::Null,
                env: value.clone(),
            }],
        }
    }
}

#[derive(Debug, Default, Clone, PartialEq)]
pub struct Environment {
    pub values: RefCell<HashMap<String, R>>,
    pub parent: Option<Rc<Environment>>,
}

impl Environment {
    pub fn from_builtins() -> Rc<Environment> {
        let env = Rc::new(Environment::default());
        for (name, builtin) in BUILTIN.iter() {
            let builtin_fn = R::Function(
                ExprList::new(),
                Expr::Primitive(builtin.clone()),
                env.clone(),
            );

            env.insert(String::from(*name), builtin_fn);
        }
        env
    }

    pub fn insert(&self, name: String, value: R) {
        self.values.borrow_mut().insert(name, value);
    }

    pub fn append(&self, values: R) {
        match values {
            R::List(x) => {
                for (key, value) in x.values.borrow().iter() {
                    if let Some(name) = key {
                        self.values.borrow_mut().insert(name.clone(), value.clone());
                    } else {
                        println!("Dont' know what to do with value...")
                    }
                }
            }
            _ => unimplemented!(),
        }
    }
}

impl Display for Environment {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "<environment {:?}", self.values.as_ptr())?;

        // // print defined variable names
        // if self.values.borrow().len() > 0 { write!(f, " [")?; }
        // for (i, k) in self.values.borrow().keys().enumerate() {
        //     if i > 0 { write!(f, ", ")?; }
        //     write!(f, "{}", k)?;
        // }
        // if self.values.borrow().len() > 0 { write!(f, "]")?; }

        write!(f, ">")?;
        Ok(())
    }
}

pub trait Context {
    fn get(&mut self, name: String) -> EvalResult {
        (*self).env().get(name)
    }

    fn get_ellipsis(&mut self) -> EvalResult {
        let err = Err(RSignal::Error(RError::IncorrectContext("...".to_string())));
        self.get("...".to_string()).or(err)
    }

    fn env(&self) -> Rc<Environment>;

    fn eval(&mut self, expr: Expr) -> EvalResult {
        self.env().eval(expr)
    }

    fn eval_binary(&mut self, exprs: (Expr, Expr)) -> Result<(R, R), RSignal> {
        Ok((self.eval(exprs.0)?, self.eval(exprs.1)?))
    }

    fn eval_list_lazy(&mut self, l: ExprList) -> EvalResult {
        Ok(R::List(List::from(
            l.into_iter()
                .flat_map(|pair| match pair {
                    (_, Expr::Ellipsis) => {
                        if let Ok(R::List(ellipsis)) = self.get_ellipsis() {
                            ellipsis.values.borrow_mut().clone().into_iter()
                        } else {
                            vec![].into_iter()
                        }
                    }
                    (k, e @ (Expr::Call(..) | Expr::Symbol(..))) => {
                        let elem = vec![(k, R::Closure(e, self.env()))];
                        elem.into_iter()
                    }
                    (k, v) => {
                        if let Ok(elem) = self.eval(v) {
                            vec![(k, elem)].into_iter()
                        } else {
                            unreachable!()
                        }
                    }
                })
                .collect::<Vec<_>>(),
        )))
    }

    fn eval_list_eager(&mut self, l: ExprList) -> EvalResult {
        Ok(R::List(List::from(
            l.into_iter()
                .flat_map(|pair| match pair {
                    (_, Expr::Ellipsis) => {
                        if let Ok(R::List(ellipsis)) = self.get_ellipsis() {
                            ellipsis.values.borrow_mut().clone().into_iter()
                        } else {
                            vec![].into_iter()
                        }
                    }
                    (k, v) => {
                        if let Ok(elem) = self.eval(v) {
                            vec![(k, elem)].into_iter()
                        } else {
                            unreachable!()
                        }
                    }
                })
                .collect::<Vec<_>>(),
        )))
    }
}

impl Context for CallStack {
    fn env(&self) -> Rc<Environment> {
        self.last_frame().env.clone()
    }

    fn eval(&mut self, expr: Expr) -> EvalResult {
        if let Expr::List(x) = expr {
            Ok(self.eval_list_lazy(x)?)
        } else if let Expr::Symbol(what) = expr {
            let what = self.get(what);
            what
        } else if let Expr::Call(what, args) = expr.clone() {
            match *what {
                Expr::Primitive(what) => {
                    use crate::callable::keywords::KeywordBlock;
                    // block expressions do not add to the call stack
                    if what.dyn_eq(&KeywordBlock) {
                        what.call(args, self)
                    } else {
                        self.add_frame(expr.clone(), self.last_frame().env.clone());
                        let result = what.call(args, self);
                        return self.pop_frame_after(result);
                    }
                }
                Expr::String(what) | Expr::Symbol(what) => {
                    // builtin primitives do not introduce a new call onto the stack
                    if let Ok(f) = builtin(&what) {
                        self.add_frame(expr, self.last_frame().env.clone());
                        let result = f.call(args, self);
                        return self.pop_frame_after(result);
                    }

                    // look up our call target
                    let rwhat = self.env().get(what.clone())?;

                    // ensure our call target expression has an encapsulating environment
                    let Some(env) = rwhat.environment() else {
                        unimplemented!("can't call non-function")
                    };

                    // create a new environment to evaluate the function body within
                    let local_env = Rc::new(Environment {
                        parent: Some(env.clone()),
                        ..Default::default()
                    });

                    // introduce a new call frame and evaluate body in new frame
                    self.add_frame(expr, local_env);

                    // evaluate call and handle errors if they arise
                    let result = rwhat.call(args, self);
                    self.pop_frame_after(result)
                }
                _ => {
                    self.add_frame(expr, self.last_frame().env.clone());
                    let result = (self.eval(*what)?).call(args, self);
                    return self.pop_frame_after(result);
                }
            }
        } else {
            self.last_frame().eval(expr)
        }
    }

    fn get(&mut self, name: String) -> EvalResult {
        let mut env = self.env();
        loop {
            // search in this environment for value by name
            if let Some(value) = env.values.borrow().get(&name) {
                let result = value.clone();
                return match result {
                    R::Closure(expr, env) => {
                        self.add_frame(expr.clone(), env.clone());
                        let result = self.eval(expr.clone());
                        return self.pop_frame_after(result);
                    }
                    _ => Ok(result),
                };
            }

            // if not found, search through parent if available
            if let Some(parent) = &env.parent {
                env = parent.clone();
            } else {
                break;
            }
        }

        if let Ok(prim) = builtin(name.as_str()) {
            Ok(R::Function(
                ExprList::new(),
                Expr::Primitive(prim),
                self.env(),
            ))
        } else {
            Err(RSignal::Error(RError::VariableNotFound(name)))
        }
    }

    // NOTE:
    // eval_list_lazy and force_closures are often used together to greedily
    // evaluated arguments. This pattern can be specialized in the case of a
    // CallStack to cut out the creation of intermediate closures. Need to
    // lift EvalResult over Context::eval_list_lazy's flat_map by implementing
    // Try.
}

impl Context for &Frame {
    fn env(&self) -> Rc<Environment> {
        self.env.clone()
    }
}

impl Context for Rc<Environment> {
    fn env(&self) -> Rc<Environment> {
        self.clone()
    }

    fn eval(&mut self, expr: Expr) -> EvalResult {
        match expr {
            Expr::Null => Ok(R::Null),
            Expr::NA => Ok(R::Vector(Vector::from(vec![OptionNA::NA as Logical]))),
            Expr::Inf => Ok(R::Vector(Vector::from(vec![OptionNA::Some(f64::INFINITY)]))),
            Expr::Number(x) => Ok(R::Vector(Vector::from(vec![x]))),
            Expr::Integer(x) => Ok(R::Vector(Vector::from(vec![x]))),
            Expr::Bool(x) => Ok(R::Vector(Vector::from(vec![OptionNA::Some(x)]))),
            Expr::String(x) => Ok(R::Vector(Vector::from(vec![OptionNA::Some(x)]))),
            Expr::Function(formals, body) => Ok(R::Function(formals, *body, self.clone())),
            Expr::Symbol(name) => self.get(name),
            Expr::Break => Err(RSignal::Condition(Cond::Break)),
            Expr::Continue => Err(RSignal::Condition(Cond::Continue)),
            x => unimplemented!("Context::eval(Rc<Environment>, {})", x),
        }
    }

    fn get(&mut self, name: String) -> EvalResult {
        // search in this environment for value by name
        if let Some(value) = self.values.borrow().get(&name) {
            let result = value.clone();
            return match result {
                R::Closure(expr, mut env) => env.eval(expr),
                _ => Ok(result),
            };

        // if not found, search through parent if available
        } else if let Some(parent) = &self.parent {
            parent.clone().get(name)

        // if we're at the top level, fall back to primitives if available
        } else if let Ok(prim) = name.as_str().try_into() {
            Ok(R::Function(
                ExprList::new(),
                Expr::Primitive(prim),
                self.env(),
            ))

        // otherwise, throw error
        } else {
            Err(RSignal::Error(RError::VariableNotFound(name)))
        }
    }
}
