use crate::callable::core::{builtin, Callable};
use crate::cli::Experiment;
use crate::context::Context;
use crate::error::*;
use crate::internal_err;
use crate::object::types::*;
use crate::object::*;
use crate::session::Session;
use std::collections::HashSet;

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

impl From<Cond> for Signal {
    #[inline]
    fn from(val: Cond) -> Self {
        Signal::Condition(val)
    }
}

impl From<Cond> for EvalResult {
    #[inline]
    fn from(val: Cond) -> Self {
        Into::<Signal>::into(val).into()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub enum Signal {
    Condition(Cond),
    Error(Error),
    Return(Obj, bool), // (value, visibility)
    Tail(Expr, bool),  // (value expr, visibility)
    Thunk,             // used when evaluating null opts like comments
}

impl Display for Signal {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Signal::Return(obj, true) => writeln!(f, "{obj}"),
            Signal::Return(_, false) => Ok(()),
            Signal::Tail(..) => writeln!(f, "Whoops, a tail is loose!"),
            Signal::Condition(_) => writeln!(f, "Signal used at top level"),
            Signal::Error(e) => writeln!(f, "{e}"),
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
            // special case for symbols, which are treated as argument promises
            Obj::Closure(Expr::Symbol(s), mut env) => match env.get(s.clone()) {
                Err(Signal::Error(Error::Missing)) => Err(Error::ArgumentMissing(s).into()),
                Ok(result) => result.force(stack),
                other => other,
            },
            // TODO(feat):
            // this is quosure behavior, but do we also want closures that
            // don't evaluate in a new frame, but rather just in originating
            // environment?
            Obj::Closure(expr, env) => {
                stack.add_frame(expr.clone(), env.clone());
                let result = stack.eval(expr);
                stack.pop_frame_and_return(result)
            }
            _ => Ok(self),
        }
    }

    pub fn assign(self, value: Obj) -> EvalResult {
        // TODO(ERROR) cleanup
        let err = Error::Other("Invalid target for assignment".to_string());

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
            _ => Err(Signal::Error(Error::CannotBeCoercedToInteger)),
        }
    }

    pub fn as_double(self) -> EvalResult {
        match self {
            Obj::Vector(v) => Ok(Obj::Vector(v.as_double())),
            Obj::Null => Ok(Obj::Vector(Vector::from(Vec::<Double>::new()))),
            _ => Error::CannotBeCoercedToDouble.into(),
        }
    }

    pub fn as_logical(self) -> EvalResult {
        match self {
            Obj::Vector(v) => Ok(Obj::Vector(v.as_logical())),
            Obj::Null => Ok(Obj::Vector(Vector::from(Vec::<Logical>::new()))),
            _ => Error::CannotBeCoercedToLogical.into(),
        }
    }

    pub fn as_character(self) -> EvalResult {
        match self {
            Obj::Vector(v) => Ok(Obj::Vector(v.as_character())),
            Obj::Null => Ok(Obj::Vector(Vector::from(Vec::<Character>::new()))),
            _ => Error::CannotBeCoercedToCharacter.into(),
        }
    }

    pub fn as_vector(self) -> EvalResult {
        match self {
            Obj::Null => Ok(Obj::Vector(Vector::from(Vec::<Logical>::new()))),
            Obj::Vector(_) => Ok(self),
            _ => Error::CannotBeCoercedTo("vector").into(),
        }
    }

    pub fn into_usize(&self) -> Result<usize, Signal> {
        use OptionNA::*;
        use Vector::*;
        match self {
            Obj::Vector(rvec) => match rvec {
                Double(v) => match v.inner().clone().borrow()[..] {
                    [Some(x)] => Ok(x as usize),
                    _ => Err(Signal::Error(Error::CannotBeCoercedToInteger)),
                },
                Integer(v) => match v.inner().clone().borrow()[..] {
                    [Some(x)] => Ok(x as usize),
                    _ => Err(Signal::Error(Error::CannotBeCoercedToInteger)),
                },
                Logical(v) => match v.inner().clone().borrow()[..] {
                    [Some(true)] => Ok(1_usize),
                    _ => Err(Signal::Error(Error::CannotBeCoercedToInteger)),
                },
                _ => Err(Signal::Error(Error::CannotBeCoercedToInteger)),
            },
            _ => internal_err!(), // emit an appropriate error message
        }
    }

    pub fn get(&self, index: usize) -> Option<Obj> {
        match self {
            Obj::Vector(v) => v.get(index).map(Obj::Vector),
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
        use Error::{ArgumentMissing, VariableNotFound};
        match self.get_named(name) {
            Some(Obj::Closure(Expr::Missing, _)) => Err(ArgumentMissing(name.into()).into()),
            Some(x) => Ok(x),
            None => Err(VariableNotFound(name.into()).into()),
        }
    }

    // Used for [ ] syntax
    pub fn try_get(&self, index: Obj) -> EvalResult {
        match self {
            Obj::Vector(v) => v.try_get(index),
            Obj::List(l) => l.try_get(index),
            _ => internal_err!(),
        }
    }

    // Used for [[ ]] syntax
    pub fn try_get_inner(&self, index: Obj) -> EvalResult {
        match self {
            Obj::Vector(v) => v.try_get(index),
            Obj::List(l) => l.try_get_inner(index),
            _ => internal_err!(),
        }
    }

    pub fn len(&self) -> Option<usize> {
        match self {
            Obj::Vector(v) => Some(v.len()),
            Obj::List(l) => Some(l.len()),
            _ => None,
        }
    }

    #[must_use]
    pub fn is_empty(&self) -> bool {
        self.len().is_some_and(|i| i > 0)
    }
}

impl TryInto<List> for Obj {
    type Error = Signal;

    fn try_into(self) -> Result<List, Self::Error> {
        match self {
            Obj::List(l) => Ok(l),
            _ => internal_err!(),
        }
    }
}

impl TryInto<bool> for Obj {
    type Error = Signal;

    fn try_into(self) -> Result<bool, Self::Error> {
        match self {
            Obj::Vector(v) => match TryInto::<bool>::try_into(v) {
                Err(_) => Err(Signal::Error(Error::CannotBeCoercedToLogical)),
                Ok(ok) => Ok(ok),
            },
            _ => Err(Signal::Error(Error::CannotBeCoercedToLogical)),
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
            writeln!(f)?
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
                writeln!(f, "{}", breadcrumbs)?;
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
        match (self.as_double()?, rhs.as_double()?) {
            (Obj::Vector(l), Obj::Vector(r)) => Ok(Obj::Vector(l + r)),
            _ => internal_err!(),
        }
    }
}

impl std::ops::Sub for Obj {
    type Output = EvalResult;

    fn sub(self, rhs: Self) -> Self::Output {
        match (self.as_double()?, rhs.as_double()?) {
            (Obj::Vector(l), Obj::Vector(r)) => Ok(Obj::Vector(l - r)),
            _ => internal_err!(),
        }
    }
}

impl std::ops::Neg for Obj {
    type Output = EvalResult;

    fn neg(self) -> Self::Output {
        match self.as_double()? {
            Obj::Vector(x) => Ok(Obj::Vector(-x)),
            _ => internal_err!(),
        }
    }
}

impl std::ops::Mul for Obj {
    type Output = EvalResult;

    fn mul(self, rhs: Self) -> Self::Output {
        match (self.as_double()?, rhs.as_double()?) {
            (Obj::Vector(l), Obj::Vector(r)) => Ok(Obj::Vector(l * r)),
            _ => internal_err!(),
        }
    }
}

impl std::ops::Div for Obj {
    type Output = EvalResult;

    fn div(self, rhs: Self) -> Self::Output {
        match (self.as_double()?, rhs.as_double()?) {
            (Obj::Vector(l), Obj::Vector(r)) => Ok(Obj::Vector(l / r)),
            _ => internal_err!(),
        }
    }
}

impl super::object::Pow<Obj> for Obj {
    type Output = EvalResult;

    fn power(self, rhs: Self) -> Self::Output {
        match (self.as_double()?, rhs.as_double()?) {
            (Obj::Vector(l), Obj::Vector(r)) => Ok(Obj::Vector(l.power(r))),
            _ => internal_err!(),
        }
    }
}

impl std::ops::Rem for Obj {
    type Output = EvalResult;

    fn rem(self, rhs: Self) -> Self::Output {
        match (self.as_double()?, rhs.as_double()?) {
            (Obj::Vector(l), Obj::Vector(r)) => Ok(Obj::Vector(l % r)),
            _ => internal_err!(),
        }
    }
}

impl std::ops::BitOr for Obj {
    type Output = EvalResult;

    fn bitor(self, rhs: Self) -> Self::Output {
        match (self.as_logical()?, rhs.as_logical()?) {
            (Obj::Vector(l), Obj::Vector(r)) => Ok(Obj::Vector(l | r)),
            _ => internal_err!(),
        }
    }
}

impl std::ops::BitAnd for Obj {
    type Output = EvalResult;

    fn bitand(self, rhs: Self) -> Self::Output {
        match (self.as_logical()?, rhs.as_logical()?) {
            (Obj::Vector(l), Obj::Vector(r)) => Ok(Obj::Vector(l & r)),
            _ => internal_err!(),
        }
    }
}

impl VecPartialCmp<Obj> for Obj {
    type Output = EvalResult;
    fn vec_gt(self, rhs: Self) -> Self::Output {
        match (self.as_vector()?, rhs.as_vector()?) {
            (Obj::Vector(l), Obj::Vector(r)) => Ok(Obj::Vector(l.vec_gt(r))),
            _ => internal_err!(),
        }
    }

    fn vec_gte(self, rhs: Self) -> Self::Output {
        match (self.as_vector()?, rhs.as_vector()?) {
            (Obj::Vector(l), Obj::Vector(r)) => Ok(Obj::Vector(l.vec_gte(r))),
            _ => internal_err!(),
        }
    }

    fn vec_lt(self, rhs: Self) -> Self::Output {
        match (self.as_vector()?, rhs.as_vector()?) {
            (Obj::Vector(l), Obj::Vector(r)) => Ok(Obj::Vector(l.vec_lt(r))),
            _ => internal_err!(),
        }
    }

    fn vec_lte(self, rhs: Self) -> Self::Output {
        match (self.as_vector()?, rhs.as_vector()?) {
            (Obj::Vector(l), Obj::Vector(r)) => Ok(Obj::Vector(l.vec_lte(r))),
            _ => internal_err!(),
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
                _ => internal_err!(),
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
                _ => internal_err!(),
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

    pub fn new_child_env(&self) -> Box<dyn Context> {
        Box::new(Obj::Environment(Rc::new(Environment {
            parent: Some(self.env().clone()),
            ..Default::default()
        })))
    }
}

impl Display for Frame {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.call)
    }
}

#[derive(Debug, Default, Clone, PartialEq)]
pub struct CallStack {
    pub session: Session,
    pub frames: Vec<Frame>,
}

impl CallStack {
    pub fn with_global_env(mut self, env: Rc<Environment>) -> Self {
        self.frames = vec![Frame::new(Expr::Null, env)];
        self
    }

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
            i => self.frames.get(i as usize),
        }
    }

    pub fn last_frame(&self) -> Frame {
        if let Some(frame) = self.frames.last() {
            frame.clone()
        } else {
            panic!("We've somehow exhausted the entire call stack and are still evaluating")
        }
    }

    pub fn parent_frame(&self) -> Frame {
        if let Some(frame) = self.frame(-1) {
            frame.clone()
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
}

impl Display for CallStack {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        // find most recent frame with same environment
        let origins = self
            .frames
            .iter()
            .enumerate()
            .map(|(i, frame)| {
                self.frames
                    .iter()
                    .take(i)
                    .rev()
                    .position(|prev| {
                        Obj::Environment(prev.env.clone()) == Obj::Environment(frame.env.clone())
                    })
                    // ignore previous frame, std eval
                    .and_then(|n| if n > 1 { Some(n) } else { None })
                    .map(|n| i.saturating_sub(n))
            })
            .collect::<Vec<_>>();

        for (i, frame) in self.frames.iter().enumerate().skip(1) {
            writeln!(f, "{}: {} => {:?}", i, frame.clone(), origins[i])?;
        }

        Ok(())
    }
}

impl From<Session> for CallStack {
    fn from(value: Session) -> Self {
        CallStack {
            session: value,
            frames: vec![],
        }
    }
}

impl Context for CallStack {
    fn assign_lazy(&mut self, to: Expr, from: Expr) -> EvalResult {
        const LIST: &str = "list";
        let err = Err(Signal::Error(Error::IncorrectContext("<-".to_string())));

        if let Expr::Call(what, mut args) = to {
            match *what {
                // special case for list() calls
                Expr::String(s) | Expr::Symbol(s) if s == LIST => {
                    let result = self.eval_and_finalize(from)?;
                    return self.assign(Expr::List(args), result);
                }
                Expr::String(s) | Expr::Symbol(s) => {
                    args.insert(0, from);
                    let s = format!("{}<-", s);
                    return self.eval(Expr::Call(Box::new(Expr::Symbol(s)), args));
                }
                Expr::Primitive(p) => return p.call_assign(from, args, self),
                _ => return err,
            }
        }

        let result = self.eval_and_finalize(from);
        self.assign(to, result?)
    }

    fn assign(&mut self, to: Expr, from: Obj) -> EvalResult {
        let err = Err(Signal::Error(Error::IncorrectContext("<-".to_string())));

        match (to, from) {
            (Expr::String(s) | Expr::Symbol(s), from) => {
                self.env().insert(s, from.clone());
                Ok(from)
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
                        }
                        // TODO(feature): allow arbitrary right-side expressions
                        // evaluated with list as additional data-frame
                        (Some(n), Expr::String(s) | Expr::Symbol(s)) => {
                            let value = args.try_get_inner(Obj::Vector(Vector::from(vec![s])))?;
                            self.assign(Expr::Symbol(n), value)?;
                        }
                        _ => return internal_err!(),
                    }
                }
                Ok(Obj::List(args))
            }
            _ => err,
        }
    }

    fn env(&self) -> Rc<Environment> {
        self.last_frame().env().clone()
    }

    fn eval_call(&mut self, expr: Expr) -> EvalResult {
        let Expr::Call(what, args) = expr.clone() else {
            return internal_err!();
        };

        match *what {
            Expr::Primitive(f) if f.is_transparent() => f.call(args, self),
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
                use Signal::*;

                // look up our call target
                let obj = self.env().get(name.clone())?;

                // ensure our call target expression has an encapsulating environment
                let Some(env) = obj.environment() else {
                    return internal_err!();
                };

                // introduce a new call frame and evaluate body in new frame
                self.add_child_frame(expr, env.clone());

                // handle tail call recursion
                let mut result = obj.call(args, self);

                // intercept and rearrange call stack to handle tail calls
                if self.session.experiments.contains(&Experiment::TailCalls) {
                    while let Err(Tail(Expr::Call(what, args), _vis)) = result {
                        let tail = Expr::Call(what.clone(), args.clone());

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
                            continue;
                        }

                        result = self.eval_call(tail);
                    }
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
        match expr {
            List(x) => self.eval_list_lazy(x),
            Symbol(s) => self.get(s),
            Call(..) => self.eval_call(expr),
            Function(formals, body) => Ok(Obj::Function(
                assert_formals(&self.session, formals)?,
                *body,
                self.env().clone(),
            )),
            _ => self.last_frame().eval(expr),
        }
    }

    fn eval_and_finalize(&mut self, expr: Expr) -> EvalResult {
        let mut result = self.eval(expr);

        // evaluate any lingering tail calls in the current frame
        use Signal::Tail;
        while let Err(Tail(expr, _vis)) = result {
            result = self.eval(expr);
        }

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
            Err(Signal::Error(Error::VariableNotFound(name)))
        }
    }

    // NOTE:
    // eval_list_lazy and force_closures are often used together to greedily
    // evaluated arguments. This pattern can be specialized in the case of a
    // CallStack to cut out the creation of intermediate closures. Need to
    // lift EvalResult over Context::eval_list_lazy's flat_map by implementing
    // Try.
}

impl Context for Frame {
    fn env(&self) -> Rc<Environment> {
        self.env.clone()
    }
}

impl Context for Obj {
    fn env(&self) -> Rc<Environment> {
        match self {
            Obj::Environment(e) => e.clone(),
            _ => unimplemented!(),
        }
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
            Expr::Function(formals, body) => Ok(Obj::Function(formals, *body, self.env().clone())),
            Expr::Symbol(name) => self.get(name),
            Expr::Break => Err(Signal::Condition(Cond::Break)),
            Expr::Continue => Err(Signal::Condition(Cond::Continue)),
            Expr::Primitive(p) => Ok(Obj::Function(
                p.formals(),
                Expr::Primitive(p),
                self.environment().unwrap(),
            )),
            Expr::More => Ok(Obj::Null),

            // bubbles up to where a symbol can be attached for context
            Expr::Missing => Err(Error::Missing.into()),

            x => internal_err!(format!("Can't evaluate Context::eval(Obj, {x:?})")),
        }
    }

    fn get(&mut self, name: String) -> EvalResult {
        match self {
            Obj::List(l) => l.try_get_inner(Obj::Vector(Vector::from(vec![name]))),
            Obj::Environment(e) => e.get(name),
            _ => unimplemented!(),
        }
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
            Expr::Function(formals, body) => Ok(Obj::Function(
                assert_formals(&Session::default(), formals)?,
                *body,
                self.env().clone(),
            )),
            Expr::Symbol(name) => self.get(name),
            Expr::Break => Err(Signal::Condition(Cond::Break)),
            Expr::Continue => Err(Signal::Condition(Cond::Continue)),
            Expr::Primitive(p) => Ok(Obj::Function(p.formals(), Expr::Primitive(p), self.clone())),
            Expr::More => Ok(Obj::Null),

            // bubbles up to where a symbol can be attached for context
            Expr::Missing => Err(Error::Missing.into()),

            x => internal_err!(format!(
                "Can't evaluate Context::eval(Rc<Envrionment>, {x:?})"
            )),
        }
    }

    fn get(&mut self, name: String) -> EvalResult {
        Environment::get(self, name)
    }
}

pub fn assert_formals(session: &Session, formals: ExprList) -> Result<ExprList, Signal> {
    let allow_rest_args = session.experiments.contains(&Experiment::RestArgs);
    let mut ellipsis: u8 = 0;
    let mut set: HashSet<&str> = HashSet::new();

    for (key, value) in formals.keys.iter().zip(formals.values.iter()) {
        match *value {
            Expr::Ellipsis(None) => ellipsis += 1,
            Expr::Ellipsis(Some(_)) if allow_rest_args => ellipsis += 1,
            Expr::Ellipsis(Some(_)) => return Error::FeatureDisabledRestArgs.into(),
            _ if key.is_none() => return Error::InvalidFunctionParameter(value.clone()).into(),
            _ => (),
        }
        if let Some(key) = key {
            if !set.insert(key.as_str()) {
                return Error::DuplicatedParameter(key.to_string()).into();
            }
        }
    }

    if ellipsis > 1 {
        return Error::DuplicatedParameter("...".to_string()).into();
    }

    Ok(formals)
}

#[cfg(test)]
mod test {
    use super::*;
    use crate::{r, r_expect};

    #[test]
    fn assign_from_tail_call() {
        assert_eq!(r! { x <- if (TRUE) 1 else 2; x }, r! { 1 });
    }

    #[test]
    fn fn_multiple_ellipsis() {
        assert_eq!(
            r! { fn(..., ...) {} },
            EvalResult::Err(Signal::Error(Error::DuplicatedParameter("...".to_string())))
        );
    }

    #[test]
    fn fn_rest_args() {
        let formals = ExprList::from(vec![(None, Expr::Ellipsis(Some("a".to_string())))]);
        assert_eq!(
            assert_formals(&Session::default(), formals),
            Result::Err(Signal::Error(Error::FeatureDisabledRestArgs))
        )
    }

    #[test]
    fn fn_duplicated_parameters() {
        assert_eq!(
            r! { fn(x, x) {} },
            EvalResult::Err(Signal::Error(Error::DuplicatedParameter("x".to_string())))
        );
    }

    #[test]
    fn fn_exprs_as_names() {
        assert_eq!(
            r! { fn(1) {} },
            EvalResult::Err(Signal::Error(Error::InvalidFunctionParameter(
                Expr::Number(1.0)
            )))
        );
    }

    #[test]
    fn fn_assign_dont_causes_binding() {
        r_expect! {{"
            x <- 1
            f <- fn(x) {x; null}
            f(x = 2)
            x == 1
        "}}
    }

    #[test]
    fn fn_assign_curly_causes_binding() {
        r_expect! {{"
            x <- 1
            f <- fn(x) {x; null}
            f(x = {x = 2})
            x == 2
        "}}
    }
}
