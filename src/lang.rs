use crate::callable::core::{builtin, Callable};
use crate::error::*;
use crate::internal_err;
use crate::object::types::*;
use crate::object::*;

use core::fmt;
use std::fmt::Display;
use std::rc::Rc;

pub type EvalResult = Result<Obj, Signal>;

#[derive(Debug, Clone, PartialEq)]
pub enum Cond {
    Break,
    Continue,
    Terminate,
}

impl Into<Signal> for Cond {
    fn into(self) -> Signal {
        Signal::Condition(self)
    }
}

impl Into<EvalResult> for Cond {
    fn into(self) -> EvalResult {
        Into::<Signal>::into(self).into()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub enum Signal {
    Condition(Cond),
    Error(RError),
    Return(Obj, bool), // (value, visibility)
    Tail(Expr, bool), // (value expr, visibility)
    Thunk, // used when evaluating null opts like comments
}

impl Into<EvalResult> for Signal {
    fn into(self) -> EvalResult {
        Err(self)
    }
}

impl Display for Signal {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Signal::Return(obj, true) => write!(f, "{obj}\n"),
            Signal::Return(_, false) => Ok(()),
            Signal::Tail(..) => write!(f, "Whoops, a tail is loose!"),
            Signal::Condition(_) => write!(f, "Signal used at top level"),
            Signal::Error(e) => write!(f, "{e}\n"),
            Signal::Thunk => write!(f, ""),
        }
    }
}

impl Obj {
    pub fn with_visibility(self, visibility: bool) -> EvalResult {
        Signal::Return(self, visibility).into()
    }

    pub fn force(self, stack: &mut CallStack) -> EvalResult {
        match self {
            // TODO(feat): 
            // this is quosure behavior, but do we also want closures that 
            // don't evaluate in a new frame, but rather just in originating
            // environment?
            Obj::Closure(expr, env) => {
                stack.add_frame(expr.clone(), env);
                let result = stack.eval(expr);
                stack.pop_frame_and_return(result)
            }
            _ => Ok(self),
        }
    }

    pub fn assign(self, value: Obj) -> EvalResult {
        // TODO(ERROR) cleanup
        let err = RError::Other("Invalid target for assignment".to_string());

        match self {
            Obj::Vector(mut v) => {
                v.assign(value.clone().as_vector()?)?;
                Ok(value)
            }
            Obj::List(mut l) => {
                l.assign(value.clone())?;
                Ok(value)
            }
            _ => Err(err.into()),
        }
    }

    pub fn as_integer(self) -> EvalResult {
        match self {
            Obj::Vector(v) => Ok(Obj::Vector(v.as_integer())),
            Obj::Null => Ok(Obj::Vector(Vector::from(Vec::<Integer>::new()))),
            _ => Err(Signal::Error(RError::CannotBeCoercedToInteger)),
        }
    }

    pub fn as_numeric(self) -> EvalResult {
        match self {
            Obj::Vector(v) => Ok(Obj::Vector(v.as_numeric())),
            Obj::Null => Ok(Obj::Vector(Vector::from(Vec::<Numeric>::new()))),
            _ => RError::CannotBeCoercedToNumeric.into(),
        }
    }

    pub fn as_logical(self) -> EvalResult {
        match self {
            Obj::Vector(v) => Ok(Obj::Vector(v.as_logical())),
            Obj::Null => Ok(Obj::Vector(Vector::from(Vec::<Logical>::new()))),
            atom => unreachable!("{:?} cannot be coerced to logical", atom),
        }
    }

    pub fn as_character(self) -> EvalResult {
        match self {
            Obj::Vector(v) => Ok(Obj::Vector(v.as_character())),
            Obj::Null => Ok(Obj::Vector(Vector::from(Vec::<Character>::new()))),
            atom => unreachable!("{:?} cannot be coerced to character", atom),
        }
    }

    pub fn as_vector(self) -> EvalResult {
        match self {
            Obj::Null => Ok(Obj::Vector(Vector::from(Vec::<Logical>::new()))),
            Obj::Vector(_) => Ok(self),
            _ => unimplemented!("cannot coerce object into vector"),
        }
    }

    pub fn into_usize(&self) -> Result<usize, Signal> {
        use OptionNA::*;
        use Vector::*;
        match self {
            Obj::Vector(rvec) => match rvec {
                Numeric(v) => match v.inner().clone().borrow()[..] {
                    [Some(x)] => Ok(x as usize),
                    _ => Err(Signal::Error(RError::CannotBeCoercedToInteger)),
                },
                Integer(v) => match v.inner().clone().borrow()[..] {
                    [Some(x)] => Ok(x as usize),
                    _ => Err(Signal::Error(RError::CannotBeCoercedToInteger)),
                },
                Logical(v) => match v.inner().clone().borrow()[..] {
                    [Some(true)] => Ok(1 as usize),
                    _ => Err(Signal::Error(RError::CannotBeCoercedToInteger)),
                },
                _ => Err(Signal::Error(RError::CannotBeCoercedToInteger)),
            },
            _ => todo!(), // emit an appropriate error message
        }
    }

    pub fn get(&self, index: usize) -> Option<Obj> {
        match self {
            Obj::Vector(v) => {
                if let Some(v) = v.get(index) {
                    Some(Obj::Vector(v))
                } else {
                    None
                }
            }
            Obj::Null => None,
            Obj::List(_) => None,
            Obj::Expr(_) => None,
            Obj::Closure(_, _) => None,
            Obj::Function(_, _, _) => None,
            Obj::Environment(_) => None,
        }
    }

    pub fn get_named(&mut self, name: &str) -> Option<Obj> {
        match self {
            Obj::List(v) => v
                .values
                .borrow()
                .iter()
                .find(|(k, _)| *k == Some(String::from(name)))
                .map(|(_, v)| v.clone()),
            Obj::Environment(e) => match e.get(String::from(name)) {
                Ok(v) => Some(v),
                Err(_) => None,
            },
            _ => None,
        }
    }

    pub fn set_named(&mut self, name: &str, value: Obj) -> EvalResult {
        match self {
            Obj::List(v) => {
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
            Obj::Environment(e) => {
                e.values.borrow_mut().insert(name.into(), value.clone());
                Ok(value)
            }
            _ => Ok(Obj::Null),
        }
    }

    pub fn environment(&self) -> Option<Rc<Environment>> {
        match self {
            Obj::Closure(_, e) | Obj::Function(_, _, e) | Obj::Environment(e) => Some(e.clone()),
            _ => None,
        }
    }

    pub fn try_get_named(&mut self, name: &str) -> EvalResult {
        use RError::{ArgumentMissing, VariableNotFound};
        match self.get_named(name) {
            Some(Obj::Closure(Expr::Missing, _)) => Err(ArgumentMissing(name.into()).into()),
            Some(x) => Ok(x),
            None => Err(VariableNotFound(name.into()).into()),
        }
    }

    pub fn try_get(&self, index: Obj) -> EvalResult {
        match self {
            Obj::Vector(v) => v.try_get(index),
            Obj::List(l) => l.try_get(index),
            _ => todo!(),
        }
    }

    pub fn try_get_inner(&self, index: Obj) -> EvalResult {
        match self {
            Obj::Vector(v) => v.try_get(index),
            Obj::List(l) => l.try_get_inner(index),
            _ => todo!(),
        }
    }

    pub fn len(&self) -> Option<usize> {
        match self {
            Obj::Vector(v) => Some(v.len()),
            Obj::List(l) => Some(l.len()),
            _ => None,
        }
    }
}

impl Default for Obj {
    fn default() -> Self {
        Obj::Null
    }
}

impl TryInto<List> for Obj {
    type Error = Signal;

    fn try_into(self) -> Result<List, Self::Error> {
        match self {
            Obj::List(l) => Ok(l),
            _ => unimplemented!(),
        }
    }
}

impl TryInto<bool> for Obj {
    type Error = Signal;

    fn try_into(self) -> Result<bool, Self::Error> {
        match self {
            Obj::Vector(v) => match TryInto::<bool>::try_into(v) {
                Err(_) => Err(Signal::Error(RError::CannotBeCoercedToLogical)),
                Ok(ok) => Ok(ok),
            },
            _ => Err(Signal::Error(RError::CannotBeCoercedToLogical)),
        }
    }
}

impl Display for Obj {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Obj::Vector(v) => write!(f, "{}", v),
            Obj::Null => write!(f, "NULL"),
            Obj::Environment(x) => write!(f, "<environment {:?}>", x.values.as_ptr()),
            Obj::Function(formals, Expr::Primitive(primitive), _) => {
                write!(
                    f,
                    "function({}) .Primitive(\"{}\")",
                    formals,
                    primitive.rfmt()
                )
            }
            Obj::Function(formals, body, parent_env) => {
                let parent_env = Obj::Environment(Rc::clone(parent_env));
                write!(f, "function({}) {}\n{}", formals, body, parent_env)
            }
            Obj::List(vals) => display_list(vals, f, None),
            Obj::Closure(expr, env) => write!(f, "{expr} @ {env}"),
            Obj::Expr(expr) => write!(f, "{}", expr),
        }
    }
}

fn display_list(x: &List, f: &mut fmt::Formatter<'_>, bc: Option<String>) -> fmt::Result {
    let v = x.values.borrow();
    let s = x.subsets.clone();

    for (i, (_, si)) in s
        .bind_names(x.names.clone())
        .into_iter()
        .take(v.len())
        .enumerate()
    {
        let name;
        let value;

        if let Some(i) = si {
            (name, value) = v[i].clone();
        } else {
            return write!(f, "{}", Obj::Null);
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
            Obj::List(nested_values) => {
                write!(f, "{}\n", breadcrumbs)?;
                display_list(&nested_values, f, Some(breadcrumbs))?
            }
            _ => write!(f, "{}\n{}\n", breadcrumbs, value)?,
        }
    }

    Ok(())
}

impl std::ops::Add for Obj {
    type Output = EvalResult;

    fn add(self, rhs: Self) -> Self::Output {
        match (self.as_numeric()?, rhs.as_numeric()?) {
            (Obj::Vector(l), Obj::Vector(r)) => Ok(Obj::Vector(l + r)),
            _ => unreachable!(),
        }
    }
}

impl std::ops::Sub for Obj {
    type Output = EvalResult;

    fn sub(self, rhs: Self) -> Self::Output {
        match (self.as_numeric()?, rhs.as_numeric()?) {
            (Obj::Vector(l), Obj::Vector(r)) => Ok(Obj::Vector(l - r)),
            _ => unreachable!(),
        }
    }
}

impl std::ops::Neg for Obj {
    type Output = EvalResult;

    fn neg(self) -> Self::Output {
        match self.as_numeric()? {
            Obj::Vector(x) => Ok(Obj::Vector(-x)),
            _ => unreachable!(),
        }
    }
}

impl std::ops::Mul for Obj {
    type Output = EvalResult;

    fn mul(self, rhs: Self) -> Self::Output {
        match (self.as_numeric()?, rhs.as_numeric()?) {
            (Obj::Vector(l), Obj::Vector(r)) => Ok(Obj::Vector(l * r)),
            _ => unreachable!(),
        }
    }
}

impl std::ops::Div for Obj {
    type Output = EvalResult;

    fn div(self, rhs: Self) -> Self::Output {
        match (self.as_numeric()?, rhs.as_numeric()?) {
            (Obj::Vector(l), Obj::Vector(r)) => Ok(Obj::Vector(l / r)),
            _ => unreachable!(),
        }
    }
}

impl super::object::Pow<Obj> for Obj {
    type Output = EvalResult;

    fn power(self, rhs: Self) -> Self::Output {
        match (self.as_numeric()?, rhs.as_numeric()?) {
            (Obj::Vector(l), Obj::Vector(r)) => Ok(Obj::Vector(l.power(r))),
            _ => unreachable!(),
        }
    }
}

impl std::ops::Rem for Obj {
    type Output = EvalResult;

    fn rem(self, rhs: Self) -> Self::Output {
        match (self.as_numeric()?, rhs.as_numeric()?) {
            (Obj::Vector(l), Obj::Vector(r)) => Ok(Obj::Vector(l % r)),
            _ => unreachable!(),
        }
    }
}

impl std::ops::BitOr for Obj {
    type Output = EvalResult;

    fn bitor(self, rhs: Self) -> Self::Output {
        match (self.as_logical()?, rhs.as_logical()?) {
            (Obj::Vector(l), Obj::Vector(r)) => Ok(Obj::Vector(l | r)),
            _ => unreachable!(),
        }
    }
}

impl std::ops::BitAnd for Obj {
    type Output = EvalResult;

    fn bitand(self, rhs: Self) -> Self::Output {
        match (self.as_logical()?, rhs.as_logical()?) {
            (Obj::Vector(l), Obj::Vector(r)) => Ok(Obj::Vector(l & r)),
            _ => unreachable!(),
        }
    }
}

impl VecPartialCmp<Obj> for Obj {
    type Output = EvalResult;
    fn vec_gt(self, rhs: Self) -> Self::Output {
        match (self.as_vector()?, rhs.as_vector()?) {
            (Obj::Vector(l), Obj::Vector(r)) => Ok(Obj::Vector(l.vec_gt(r))),
            _ => unreachable!(),
        }
    }

    fn vec_gte(self, rhs: Self) -> Self::Output {
        match (self.as_vector()?, rhs.as_vector()?) {
            (Obj::Vector(l), Obj::Vector(r)) => Ok(Obj::Vector(l.vec_gte(r))),
            _ => unreachable!(),
        }
    }

    fn vec_lt(self, rhs: Self) -> Self::Output {
        match (self.as_vector()?, rhs.as_vector()?) {
            (Obj::Vector(l), Obj::Vector(r)) => Ok(Obj::Vector(l.vec_lt(r))),
            _ => unreachable!(),
        }
    }

    fn vec_lte(self, rhs: Self) -> Self::Output {
        match (self.as_vector()?, rhs.as_vector()?) {
            (Obj::Vector(l), Obj::Vector(r)) => Ok(Obj::Vector(l.vec_lte(r))),
            _ => unreachable!(),
        }
    }

    fn vec_eq(self, rhs: Self) -> Self::Output {
        match (self, rhs) {
            (lhs @ Obj::Expr(_), rhs @ Obj::Expr(_)) => Ok((lhs == rhs).into()),
            (lhs @ Obj::Closure(..), rhs @ Obj::Closure(..)) => Ok((lhs == rhs).into()),
            (lhs @ Obj::Function(..), rhs @ Obj::Function(..)) => Ok((lhs == rhs).into()),
            (lhs @ Obj::Environment(_), rhs @ Obj::Environment(_)) => Ok((lhs == rhs).into()),
            (lhs, rhs) => match (lhs.as_vector()?, rhs.as_vector()?) {
                (Obj::Vector(l), Obj::Vector(r)) => Ok(Obj::Vector(l.vec_eq(r))),
                _ => unreachable!(),
            },
        }
    }

    fn vec_neq(self, rhs: Self) -> Self::Output {
        match (self, rhs) {
            (lhs @ Obj::Expr(_), rhs @ Obj::Expr(_)) => Ok((lhs != rhs).into()),
            (lhs @ Obj::Closure(..), rhs @ Obj::Closure(..)) => Ok((lhs != rhs).into()),
            (lhs @ Obj::Function(..), rhs @ Obj::Function(..)) => Ok((lhs != rhs).into()),
            (lhs @ Obj::Environment(_), rhs @ Obj::Environment(_)) => Ok((lhs != rhs).into()),
            (lhs, rhs) => match (lhs.as_vector()?, rhs.as_vector()?) {
                (Obj::Vector(l), Obj::Vector(r)) => Ok(Obj::Vector(l.vec_neq(r))),
                _ => unreachable!(),
            },
        }
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct Frame {
    // The expression that was evaluated to introduce this frame
    pub call: Expr,
    // The target of the call that prompted the new frame
    pub to: Obj, 
    // The evaluation environment for the frame
    pub env: Rc<Environment>,
}

impl Frame {
    pub fn new(call: Expr, mut env: Rc<Environment>) -> Frame {
        let to = match call.clone() {
            Expr::Call(what, _) => env.eval(*what).unwrap_or_default(),
            _ => Obj::Null,
        };

        Self { call, to, env }
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
        self.frames.push(Frame::new(call, env));
        self.frames.len()
    }

    pub fn add_child_frame(&mut self, call: Expr, env: Rc<Environment>) -> usize {       
        let local_env = Rc::new(Environment {
            parent: Some(env.clone()),
            ..Default::default()
        });

        self.add_frame(call, local_env)
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

    pub fn pop_frame_and_return(&mut self, result: EvalResult) -> EvalResult {
        match result {
            Ok(..) => {
                self.frames.pop();
                result
            }
            error => error,
        }
    }

    pub fn new() -> CallStack {
        CallStack::from(Frame::new(Expr::Null, Rc::new(Environment::default())))
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
            frames: vec![Frame::new(Expr::Null, value.clone())],
        }
    }
}

pub trait Context {
    fn get(&mut self, name: String) -> EvalResult {
        (*self).env().get(name)
    }

    fn get_ellipsis(&mut self) -> EvalResult {
        let err = Err(Signal::Error(RError::IncorrectContext("...".to_string())));
        self.get("...".to_string()).or(err)
    }

    fn assign_lazy(&mut self, _to: Expr, _from: Expr) -> EvalResult {
        Err(Signal::Error(RError::IncorrectContext("<-".to_string())))
    }

    fn assign(&mut self, _to: Expr, _from: Obj) -> EvalResult {
        Err(Signal::Error(RError::IncorrectContext("<-".to_string())))
    }

    fn env(&self) -> Rc<Environment>;

    fn eval_call(&mut self, expr: Expr) -> EvalResult {
        self.eval(expr)
    }

    fn eval(&mut self, expr: Expr) -> EvalResult {
        self.env().eval(expr)
    }

    fn eval_and_finalize(&mut self, expr: Expr) -> EvalResult {
        self.eval(expr)
    }

    fn eval_binary(&mut self, exprs: (Expr, Expr)) -> Result<(Obj, Obj), Signal> {
        Ok((self.eval(exprs.0)?, self.eval(exprs.1)?))
    }

    fn eval_list_lazy(&mut self, l: ExprList) -> EvalResult {
        Ok(Obj::List(List::from(
            l.into_iter()
                .flat_map(|pair| match pair {
                    (_, Expr::Ellipsis) => {
                        if let Ok(Obj::List(ellipsis)) = self.get_ellipsis() {
                            ellipsis.values.borrow_mut().clone().into_iter()
                        } else {
                            vec![].into_iter()
                        }
                    }
                    (k, e @ (Expr::Call(..) | Expr::Symbol(..))) => {
                        let elem = vec![(k, Obj::Closure(e, self.env()))];
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
        Ok(Obj::List(List::from(
            l.into_iter()
                .flat_map(|pair| match pair {
                    (_, Expr::Ellipsis) => {
                        if let Ok(Obj::List(ellipsis)) = self.get_ellipsis() {
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
    fn assign_lazy(&mut self, to: Expr, from: Expr) -> EvalResult {
        const LIST: &str = "list";
        let err = Err(Signal::Error(RError::IncorrectContext("<-".to_string())));

        if let Expr::Call(what, mut args) = to { 
            match *what {
                // special case for list() calls
                Expr::String(s) | Expr::Symbol(s) if s == LIST => {
                    let result = self.eval(from)?;
                    return self.assign(Expr::List(args), result);
                }
                Expr::String(s) | Expr::Symbol(s) => {
                    args.insert(0, from);
                    let s = format!("{}<-", s);
                    return self.eval(Expr::Call(Box::new(Expr::Symbol(s)), args))
                }
                Expr::Primitive(p) => return p.call_assign(from, args, self),
                _ => return err,
            }
        }
        
        let result = self.eval(from)?;
        self.assign(to, result)
    }

    fn assign(&mut self, to: Expr, from: Obj) -> EvalResult {
        use Signal::*;
        let err = Err(Signal::Error(RError::IncorrectContext("<-".to_string())));

        match (to, from) {          
            (Expr::String(s) | Expr::Symbol(s), from) => {
                self.env().insert(s, from.clone());
                Return(from, false).into()
            }
            (Expr::List(l), Obj::List(args)) => {
                let mut i = 1;
                for item in l.into_iter() {
                    match item {
                        (None, Expr::String(s) | Expr::Symbol(s)) => {
                            let index = Obj::Vector(Vector::from(vec![i]));
                            let value = args.try_get_inner(index)?;
                            self.assign(Expr::Symbol(s), value)?;
                            i += 1;
                        },
                        // TODO: allow arbitrary right-side expressions
                        // evaluated with list as additional data-frame
                        (Some(n), Expr::String(s) | Expr::Symbol(s)) => {
                            let value = args.try_get_inner(Obj::Vector(Vector::from(vec![s])))?;
                            self.assign(Expr::Symbol(n), value)?;                            
                        }
                        _ => unimplemented!(),
                    }
                }

                Return(Obj::List(args), false).into()
            },
            _ => err,
        }
    }

    fn env(&self) -> Rc<Environment> {
        self.last_frame().env.clone()
    }

    fn eval_call(&mut self, expr: Expr) -> EvalResult {
        let Expr::Call(what, args) = expr.clone() else { return internal_err!() };

        match *what {
            Expr::Primitive(f) if f.is_transparent() => {
                f.call(args, self)
            }
            Expr::Primitive(f) => {
                self.add_frame(expr, self.last_frame().env().clone());
                let result = f.call(args, self);
                self.pop_frame_and_return(result)
            }
            Expr::String(name) | Expr::Symbol(name) if builtin(&name).is_ok() => {
                let f = builtin(&name)?;
                self.add_frame(expr, self.last_frame().env().clone());
                let result = f.call(args, self);
                self.pop_frame_and_return(result)
            }
            Expr::String(name) | Expr::Symbol(name) => {
                // look up our call target
                let obj = self.env().get(name.clone())?;

                // ensure our call target expression has an encapsulating environment
                let Some(env) = obj.environment() else { return internal_err!() };

                // introduce a new call frame and evaluate body in new frame
                self.add_child_frame(expr, env.clone());

                // handle tail call recursion
                let mut result = obj.call(args, self);

                use Signal::*;
                while let Err(Tail(Expr::Call(what, args), _vis)) = result {
                    let tail = Expr::Call(what.clone(), args.clone());

                    #[cfg(feature = "tail-call-optimization")]
                    {
                        // below allows for tail recursion optimization,
                        // disabled for now because of idiosyncracies with
                        // standard evaluation expectations due to eagerly
                        // evaluated arguments

                        // tail is recursive call if it calls out to same object
                        // that was called to enter current frame
                        let what_obj = self.eval(*what)?;
                        if what_obj == self.last_frame().to {
                            // eagerly evaluate and match argument expressions in tail frame
                            let args: List = self.eval_list_eager(args)?.try_into()?;
                            let (args, ellipsis) = what_obj.match_args(args, self)?;

                            // pop tail frame and add a new local frame
                            self.frames.pop();
                            self.add_child_frame(tail, env.clone());

                            // call with pre-matched args
                            result = what_obj.call_matched(args, ellipsis, self);
                            continue
                        }
                    }

                    result = self.eval_call(tail);
                }

                // evaluate any lingering tail calls in the current frame
                while let Err(Tail(expr, _vis)) = result {
                    result = self.eval(expr)
                }

                self.pop_frame_and_return(result)
            }
            _ => {
                self.add_frame(expr, self.last_frame().env().clone());
                let result = (self.eval(*what)?).call(args, self);
                self.pop_frame_and_return(result)
            }
        }
    }

    fn eval(&mut self, expr: Expr) -> EvalResult {
        use Expr::*;
        let result = match expr {
            List(x) => self.eval_list_lazy(x),
            Symbol(s) => self.get(s),
            Call(..) => self.eval_call(expr),
            _ => self.last_frame().eval(expr)
        };


        result
    }

    fn eval_and_finalize(&mut self, expr: Expr) -> EvalResult {
        let mut result = self.eval(expr);

        // evaluate any lingering tail calls in the current frame
        use Signal::Tail;
        while let Err(Tail(expr, _vis)) = result {
            result = self.eval(expr)
        };

        result
    }

    fn get(&mut self, name: String) -> EvalResult {
        let mut env = self.env();
        loop {
            // search in this environment for value by name
            if let Some(value) = env.values.borrow().get(&name) {
                let result = value.clone();
                return match result {
                    c @ Obj::Closure(..) => c.force(self),
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
            Ok(Obj::Function(
                ExprList::new(),
                Expr::Primitive(prim),
                self.env(),
            ))
        } else {
            Err(Signal::Error(RError::VariableNotFound(name)))
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
            Expr::Null => Ok(Obj::Null),
            Expr::NA => Ok(Obj::Vector(Vector::from(vec![OptionNA::NA as Logical]))),
            Expr::Inf => Ok(Obj::Vector(Vector::from(vec![OptionNA::Some(
                f64::INFINITY,
            )]))),
            Expr::Number(x) => Ok(Obj::Vector(Vector::from(vec![x]))),
            Expr::Integer(x) => Ok(Obj::Vector(Vector::from(vec![x]))),
            Expr::Bool(x) => Ok(Obj::Vector(Vector::from(vec![OptionNA::Some(x)]))),
            Expr::String(x) => Ok(Obj::Vector(Vector::from(vec![OptionNA::Some(x)]))),
            Expr::Function(formals, body) => Ok(Obj::Function(formals, *body, self.clone())),
            Expr::Symbol(name) => self.get(name),
            Expr::Break => Err(Signal::Condition(Cond::Break)),
            Expr::Continue => Err(Signal::Condition(Cond::Continue)),
            Expr::Primitive(p) => Ok(Obj::Function(p.formals(), Expr::Primitive(p), self.clone())),
            x => unimplemented!("Context::eval(Rc<Environment>, {})", x),
        }
    }

    fn get(&mut self, name: String) -> EvalResult {
        // search in this environment for value by name
        if let Some(value) = self.values.borrow().get(&name) {
            let result = value.clone();
            return match result {
                Obj::Closure(expr, mut env) => env.eval(expr),
                _ => Ok(result),
            };

        // if not found, search through parent if available
        } else if let Some(parent) = &self.parent {
            parent.clone().get(name)

        // if we're at the top level, fall back to primitives if available
        } else if let Ok(prim) = name.as_str().try_into() {
            Ok(Obj::Function(
                ExprList::new(),
                Expr::Primitive(prim),
                self.env(),
            ))

        // otherwise, throw error
        } else {
            Err(Signal::Error(RError::VariableNotFound(name)))
        }
    }
}
