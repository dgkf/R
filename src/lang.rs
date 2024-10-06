use crate::callable::builtins::BUILTIN;
use crate::callable::core::Callable;
use crate::cli::Experiment;
use crate::context::Context;
use crate::error::*;
use crate::internal_err;
use crate::object::types::*;
use crate::object::List;
use crate::object::*;
use crate::parser::LocalizedParser;
use crate::parser::ParseResult;
use crate::session::{Session, SessionParserConfig};
use hashbrown::HashSet;

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

impl ViewMut for Obj {
    fn view_mut(&self) -> Self {
        match self {
            Obj::Vector(v) => Obj::Vector(match v {
                Vector::Double(v) => Vector::Double(v.view_mut()),
                Vector::Character(v) => Vector::Character(v.view_mut()),
                Vector::Integer(v) => Vector::Integer(v.view_mut()),
                Vector::Logical(v) => Vector::Logical(v.view_mut()),
            }),

            Obj::List(l) => Obj::List(l.view_mut()),
            // FIXME: this needs to be implemented for all objects that can be mutated
            x => x.clone(),
        }
    }
}

impl Obj {
    pub fn with_visibility(self, visibility: bool) -> EvalResult {
        Signal::Return(self, visibility).into()
    }

    pub fn force(self, stack: &mut CallStack) -> EvalResult {
        match self {
            Obj::Promise(None, expr, env) => {
                stack.add_frame(expr.clone(), env.clone());
                let result = stack.eval_and_finalize(expr);
                stack.pop_frame_and_return(result)
            }
            Obj::Promise(Some(value), ..) => Ok(*value),
            _ => Ok(self),
        }
    }

    // this is vectorized assignment.
    pub fn assign(self, value: Obj) -> EvalResult {
        // TODO(ERROR) cleanup
        let err = Error::Other("Invalid target for assignment".to_string());

        match self {
            Obj::Vector(mut v) => {
                v.assign(value.clone().as_vector()?)?;
                Ok(value)
            }
            Obj::List(mut l) => {
                match value.clone() {
                    Obj::List(r) => {
                        l.assign(r);
                    }
                    Obj::Vector(r) => match r {
                        Vector::Integer(r) => {
                            l.assign(r);
                        }
                        Vector::Character(r) => {
                            l.assign(r);
                        }
                        Vector::Logical(r) => {
                            l.assign(r);
                        }
                        Vector::Double(r) => {
                            l.assign(r);
                        }
                    },
                    _ => return Err(err.into()),
                };

                Ok(value)
            }
            _ => Err(err.into()),
        }
    }

    pub fn as_list(&self) -> EvalResult {
        match self {
            Obj::Null => Ok(Obj::List(List::new())),
            Obj::Vector(_v) => internal_err!(),
            Obj::List(l) => Ok(Obj::List(l.clone())),
            Obj::Expr(e) => match e {
                Expr::List(exprlist) => Ok(Obj::List(List::from(
                    exprlist
                        .clone()
                        .into_iter()
                        .map(|(k, v)| (k, Obj::Expr(v)))
                        .collect::<Vec<_>>(),
                ))),
                Expr::Function(_, _) => internal_err!(),
                Expr::Call(what, args) => Ok(Obj::List(List::from(
                    vec![(None, (**what).clone())]
                        .into_iter()
                        .chain((*args).clone())
                        .map(|(k, v)| (k, Obj::Expr(v)))
                        .collect::<Vec<_>>(),
                ))),
                other => Ok(Obj::List(List::from(vec![(
                    None,
                    Obj::Expr(other.clone()),
                )]))),
            },
            Obj::Promise(..) => internal_err!(),
            Obj::Function(..) => internal_err!(),
            Obj::Environment(..) => internal_err!(),
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
            Obj::List(v) => v.get(index).map(Obj::List),
            Obj::Null => None,
            Obj::Expr(..) => None,
            Obj::Promise(..) => None,
            Obj::Function(..) => None,
            Obj::Environment(..) => None,
        }
    }

    pub fn environment(&self) -> Option<Rc<Environment>> {
        match self {
            Obj::Promise(.., e) | Obj::Function(.., e) | Obj::Environment(e) => Some(e.clone()),
            _ => None,
        }
    }

    /// Used for `$`-assignment.
    pub fn try_set_named(&mut self, name: &str, value: Obj) -> EvalResult {
        match self {
            Obj::List(l) => {
                let subset = Subset::Names(vec![Character::Some(name.to_string())].into());
                Ok(l.set_subset(subset, value)?)
            }
            Obj::Environment(e) => {
                e.values.borrow_mut().insert(name.into(), value.clone());
                Ok(value)
            }
            _ => internal_err!(),
        }
    }

    /// Used for `$`-assignment.
    pub fn try_get_named_mut(&mut self, name: &str) -> EvalResult {
        match self {
            Obj::List(l) => {
                let subset = Subset::Names(vec![Character::Some(name.to_string())].into());
                Ok(l.try_get_inner_mut(subset)?)
            }
            Obj::Environment(e) => match e.get_mut(String::from(name)) {
                Ok(v) => Ok(v),
                Err(_) => Err(Error::VariableNotFound(name.into()).into()),
            },
            _ => internal_err!(),
        }
    }

    /// Used for `$`-access.
    pub fn try_get_named(&mut self, name: &str) -> EvalResult {
        match self {
            Obj::List(l) => {
                let subset = Subset::Names(vec![Character::Some(name.to_string())].into());
                Ok(l.try_get_inner(subset)?)
            }
            Obj::Environment(e) => match e.get(String::from(name)) {
                Ok(v) => Ok(v),
                Err(_) => Err(Error::VariableNotFound(name.into()).into()),
            },
            _ => internal_err!(),
        }
    }

    // Used for [ ] syntax
    pub fn try_get(&self, index: Obj) -> EvalResult {
        let index = index.as_vector()?;
        match self {
            Obj::Vector(v) => v.try_get(index),
            Obj::List(l) => {
                let subset = Subset::try_from(index)?;
                let x = l.subset(subset);
                EvalResult::Ok(Obj::List(List::from(x)))
            }
            obj => obj.as_list()?.try_get(index),
        }
    }

    // Used for `[[`-access.
    pub fn try_get_inner(&self, index: Obj) -> EvalResult {
        match self {
            Obj::Vector(v) => v.try_get(index),
            Obj::List(l) => EvalResult::Ok(l.try_get_inner(index.try_into()?)?),
            // To access environments use try_get_named
            Obj::Environment(_) => internal_err!(),
            obj => obj.as_list()?.try_get_inner(index),
        }
    }

    // Used for assignment to [[ ]]
    pub fn try_get_inner_mut(&self, index: Obj) -> EvalResult {
        match self {
            Obj::Vector(v) => v.try_get(index),
            Obj::List(l) => EvalResult::Ok(l.try_get_inner_mut(index.try_into()?)?),
            obj => obj.as_list()?.try_get_inner_mut(index),
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
            Obj::Promise(None, expr, env) => write!(f, "{expr} @ {env}"),
            Obj::Promise(Some(obj), ..) => write!(f, "{obj}"),
            Obj::Expr(expr) => write!(f, "{}", expr),
        }
    }
}

fn display_list(x: &List, f: &mut fmt::Formatter<'_>, bc: Option<String>) -> fmt::Result {
    for (i, (maybe_name, value)) in x.pairs_ref().iter().enumerate() {
        if i > 0 {
            writeln!(f)?
        }

        let bc_elem = if let Character::Some(name) = maybe_name {
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
                display_list(nested_values, f, Some(breadcrumbs))?
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

impl std::ops::Not for Obj {
    type Output = EvalResult;

    fn not(self) -> Self::Output {
        match self.as_logical()? {
            Obj::Vector(x) => Ok(Obj::Vector(!x)),
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
            (lhs @ Obj::Promise(..), rhs @ Obj::Promise(..)) => Ok((lhs == rhs).into()),
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
            (lhs @ Obj::Promise(..), rhs @ Obj::Promise(..)) => Ok((lhs != rhs).into()),
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

#[derive(Debug, Clone, PartialEq)]
pub struct CallStack {
    pub session: Session,
    pub frames: Vec<Frame>,
}

impl CallStack {
    pub fn parse(&self, input: &str) -> ParseResult {
        let config: SessionParserConfig = self.session.clone().into();
        config.parse_input(input)
    }

    pub fn parse_and_eval(mut self, input: &str) -> EvalResult {
        let expr = self.parse(input)?;
        self.eval_and_finalize(expr)
    }
}

impl Default for CallStack {
    fn default() -> Self {
        let global_env = Rc::new(Environment {
            parent: Some(Environment::from_builtins()),
            ..Default::default()
        });

        CallStack {
            session: Session::default(),
            frames: vec![Frame::new(Expr::Null, global_env)],
        }
    }
}

impl CallStack {
    pub fn with_global_env(mut self, env: Rc<Environment>) -> Self {
        self.frames = vec![Frame::new(Expr::Null, env)];
        self
    }

    pub fn map_session(mut self, f: impl Fn(Session) -> Session) -> Self {
        self.session = f(self.session);
        self
    }

    /// Find an object in the current environment or one of its parents and return a mutable view
    /// of the object, as well as the environment in which it was found.
    /// None is returned if the value was not found.
    fn find(&mut self, name: String) -> Result<(Obj, Rc<Environment>), Signal> {
        let mut env = self.env();
        loop {
            // search in this environment for value by name
            let Some(value) = env.values.borrow_mut().get(&name).map(|x| (*x).view_mut()) else {
                // if not found, search through parent if available
                if let Some(parent) = &env.parent {
                    env = parent.clone();
                    continue;
                } else {
                    break;
                }
            };

            return match value {
                // evaluate promises
                Obj::Promise(None, expr, p_env) => {
                    let result = Obj::Promise(None, expr.clone(), p_env.clone()).force(self)?;
                    let value = Some(Box::new(result.view_mut()));
                    env.insert(name, Obj::Promise(value, expr, p_env.clone()));
                    Result::Ok((result, env))
                }
                _ => Result::Ok((value, env)),
            };
        }

        if let Some(prim) = BUILTIN.get(name.as_str()) {
            Result::Ok((
                Obj::Function(ExprList::new(), Expr::Primitive(prim.clone()), self.env()),
                env,
            ))
        } else {
            Result::Err(Signal::Error(Error::VariableNotFound(name)))
        }
    }

    pub fn add_frame(&mut self, call: Expr, env: Rc<Environment>) -> usize {
        self.frames.push(Frame::new(call, env));
        self.frames.len()
    }

    pub fn add_child_frame(&mut self, call: Expr, env: Rc<Environment>) -> usize {
        let local_env = Rc::new(Environment { parent: Some(env.clone()), ..Default::default() });

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
        CallStack::default().map_session(|_| value.clone())
    }
}

impl Context for CallStack {
    #[inline]
    fn eval_binary(&mut self, exprs: (Expr, Expr)) -> Result<(Obj, Obj), Signal> {
        Ok((
            self.eval_and_finalize(exprs.0)?.force(self)?,
            self.eval_and_finalize(exprs.1)?.force(self)?,
        ))
    }

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
                            let value = args.try_get_inner(index.try_into()?)?;
                            self.assign(Expr::Symbol(s), value)?;
                            i += 1;
                        }
                        // TODO(feature): allow arbitrary right-side expressions
                        // evaluated with list as additional data-frame
                        (Some(n), Expr::String(s) | Expr::Symbol(s)) => {
                            let value =
                                args.try_get_inner(Obj::Vector(Vector::from(vec![s])).try_into()?)?;
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

    fn eval_call_mut(&mut self, expr: Expr) -> EvalResult {
        eval_call(self, expr, true)
    }

    fn eval_call(&mut self, expr: Expr) -> EvalResult {
        eval_call(self, expr, false)
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

    #[inline]
    fn eval_mut(&mut self, expr: Expr) -> EvalResult {
        match expr {
            Expr::Symbol(x) => self.get_mut(x),
            Expr::Call(..) => self.eval_call_mut(expr),
            e => Error::CannotEvaluateAsMutable(e).into(),
        }
    }

    fn get(&mut self, name: String) -> EvalResult {
        let (obj, _) = self.find(name.clone())?;
        Ok(obj.clone())
    }

    fn get_mut(&mut self, name: String) -> EvalResult {
        let (obj, obj_source_env) = self.find(name.clone())?;

        let objc = match (self.env() == obj_source_env, obj) {
            // when accessed mutably, promises are always masked by materialized value
            (_, Obj::Promise(Some(x), ..)) => *x.clone(),
            (true, obj) => return Ok(obj),
            (false, obj) => obj.clone(),
        };

        self.env().insert(name, objc.view_mut());
        Ok(objc)
    }

    // NOTE:
    // eval_list_lazy and force_promises are often used together to greedily
    // evaluated arguments. This pattern can be specialized in the case of a
    // CallStack to cut out the creation of intermediate closures. Need to
    // lift EvalResult over Context::eval_list_lazy's flat_map by implementing
    // Try.
}

fn eval_call(callstack: &mut CallStack, expr: Expr, mutable: bool) -> EvalResult {
    let Expr::Call(what, args) = expr.clone() else {
        return internal_err!();
    };

    match *what {
        Expr::Primitive(f) if f.is_transparent() => {
            if mutable {
                f.call_mut(args, callstack)
            } else {
                f.call(args, callstack)
            }
        }
        Expr::Primitive(f) => {
            callstack.add_frame(expr, callstack.last_frame().env().clone());
            let result = if mutable {
                f.call_mut(args, callstack)
            } else {
                f.call(args, callstack)
            };
            callstack.pop_frame_and_return(result)
        }
        Expr::String(name) | Expr::Symbol(name) if BUILTIN.contains_key(name.as_str()) => {
            let f = BUILTIN
                .get(name.as_str())
                .ok_or(Error::VariableNotFound(name))?;
            callstack.add_frame(expr, callstack.last_frame().env().clone());
            let result = if mutable {
                f.call_mut(args, callstack)
            } else {
                f.call(args, callstack)
            };
            callstack.pop_frame_and_return(result)
        }
        Expr::String(name) | Expr::Symbol(name) => {
            if mutable {
                // currently, things like names(x) = "a" is anyway not supported
                return internal_err!();
            }
            use Signal::*;

            // look up our call target
            let obj = callstack.env().get(name.clone())?;

            // ensure our call target expression has an encapsulating environment
            let Some(env) = obj.environment() else {
                return internal_err!();
            };

            // introduce a new call frame and evaluate body in new frame
            callstack.add_child_frame(expr, env.clone());

            // handle tail call recursion
            let mut result = obj.call(args, callstack);

            // intercept and rearrange call stack to handle tail calls
            if callstack
                .session
                .experiments
                .contains(&Experiment::TailCalls)
            {
                while let Err(Tail(Expr::Call(what, args), _vis)) = result {
                    let tail = Expr::Call(what.clone(), args.clone());

                    // tail is recursive call if it calls out to same object
                    // that was called to enter current frame
                    let what_obj = callstack.eval(*what)?;
                    if what_obj == callstack.last_frame().to {
                        // eagerly evaluate and match argument expressions in tail frame
                        let args: List = callstack.eval_list_eager(args)?.try_into()?;
                        let (args, ellipsis) = what_obj.match_args(args, callstack)?;

                        // pop tail frame and add a new local frame
                        callstack.frames.pop();
                        callstack.add_child_frame(tail, env.clone());

                        // call with pre-matched args
                        result = what_obj.call_matched(args, ellipsis, callstack);
                        continue;
                    }

                    result = callstack.eval_call(tail);
                }
            }

            // evaluate any lingering tail calls in the current frame
            while let Err(Tail(expr, _vis)) = result {
                result = callstack.eval(expr)
            }

            callstack.pop_frame_and_return(result)
        }
        _ => {
            callstack.add_frame(expr, callstack.last_frame().env().clone());
            let result = (callstack.eval(*what)?).call_mut(args, callstack);
            callstack.pop_frame_and_return(result)
        }
    }
}
impl Context for Frame {
    fn env(&self) -> Rc<Environment> {
        self.env.clone()
    }
    fn eval_mut(&mut self, expr: Expr) -> EvalResult {
        self.env().eval_mut(expr)
    }
}

impl Context for Obj {
    fn env(&self) -> Rc<Environment> {
        match self {
            Obj::Environment(e) => e.clone(),
            _ => unimplemented!(),
        }
    }

    fn eval_mut(&mut self, expr: Expr) -> EvalResult {
        self.env().eval_mut(expr)
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
            Obj::List(l) => Ok(l.try_get_inner(Obj::Vector(Vector::from(vec![name])).try_into()?)?),
            Obj::Environment(e) => e.get(name),
            _ => unimplemented!(),
        }
    }
}

impl Context for Rc<Environment> {
    fn env(&self) -> Rc<Environment> {
        self.clone()
    }

    /// Evaluates an expression mutably.
    /// This is used for things like `x[1:10] <- 2:11`
    fn eval_mut(&mut self, expr: Expr) -> EvalResult {
        match expr {
            Expr::Symbol(name) => self.get_mut(name),
            expr => self.eval(expr),
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

    fn get_mut(&mut self, name: String) -> EvalResult {
        Environment::get_mut(self, name)
    }
}

pub fn assert_formals(session: &Session, formals: ExprList) -> Result<ExprList, Signal> {
    let allow_rest_args = session.experiments.contains(&Experiment::RestArgs);
    let mut ellipsis: u8 = 0;
    let mut set: HashSet<&str> = HashSet::new();

    for (key, value) in formals.keys.iter().zip(formals.values.iter()) {
        match *value {
            Expr::Ellipsis(_) => match value.clone() {
                Expr::Ellipsis(None) => ellipsis += 1,
                Expr::Ellipsis(Some(x)) if x == "." => ellipsis += 1,
                Expr::Ellipsis(Some(_)) if allow_rest_args => ellipsis += 1,
                Expr::Ellipsis(Some(_)) => return Error::FeatureDisabledRestArgs.into(),
                _ => unreachable!(),
            },
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
        if allow_rest_args {
            return Error::DuplicatedParameter("...".to_string()).into();
        } else {
            return Error::DuplicatedMoreParameter().into();
        }
    }

    Ok(formals.clone())
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
            EvalResult::Err(Signal::Error(Error::DuplicatedMoreParameter()))
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
    fn fn_rest_arg_ellipsis() {
        assert_eq!(
            CallStack::default()
                .map_session(|s| s.with_experiments(vec![Experiment::RestArgs]))
                .parse_and_eval(
                    "
                    f <- fn(...) { . }
                    f(1, 2, 3)
                    ",
                ),
            r! { list(1, 2, 3) }
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
    fn fn_assign_doesnt_cause_binding() {
        r_expect! {{"
            x <- 1
            f <- fn(x) {x; null}
            f(x = 2)
            x == 1
        "}}
    }

    #[test]
    fn fn_assign_in_arg_forces_all_promises() {
        r_expect! {{"
            f <- fn(x) { x }
            z <- f({ y <- 123 })
            z == 123 && y == 123
        "}}
    }

    #[test]
    fn fn_assign_curly_causes_binding() {
        r_expect! {{"
            f = fn(x) x
            f({y = 2})
            y == 2
        "}}
    }

    #[test]
    fn binding_promise_evaluates_parenthesis() {
        r_expect! {{"
            f = fn(x) x
            a = f((y = 2))
            a == 2 
        "}}
    }

    #[test]
    fn binding_promise_binds_parenthesis() {
        r_expect! {{"
            f = fn(x) x
            f((y = 2))
            y == 2 
        "}}
    }

    #[test]
    fn curly_causes_binding() {
        r_expect! {{"
            x = {a = 2}
            x == 2
        "}}
    }

    #[test]
    fn expr_indexing() {
        r_expect! { quote(x(1, 2, 3))[[1]] == quote(x) }
        r_expect! { quote(x(1, 2, 3))[[3]] == quote(2) }
        assert_eq! { r! { quote(x(1, 2, 3))[1] }, r! { list(quote(x)) } }
    }

    #[test]
    fn dont_mutate_vec_inplace_after_assignment() {
        r_expect! {{"
            x = 1
            y = x
            y[1] = 2
            (y == 2 & x == 1)
        "}}
    }
    #[test]
    fn vectors_are_mutable() {
        r_expect! {{"
            x = 1
            x[1] = 2
            x == 2
        "}}
    }

    #[test]
    fn dont_mutate_value_from_parent() {
        r_expect! {{"
            f = fn() x[1] <- -99
            x = 10
            f()
            x == 10
        "}}

        r_expect! {{"
             f = fn(x) {
               x[1] <- -99
               x
             }
             x1 = 10
             x2 = f(x1)
             (x1 == 10) && x2 == -99
        "}}
    }

    #[test]
    fn promises_can_be_mutated() {
        r_expect! {{"
            f = fn(x) {
              x[1] = -99
              x
            }
            x2 = f(c(1, 2))
            (x2[1] == -99) && (x2[2] == 2)
        "}}
    }

    #[test]
    fn nested_promises_can_be_mutated() {
        r_expect! {{"
            inc = fn(x) { x[1] = x[1] + 1; x }
            add_two = fn(x) { inc(inc(x)) }
            add_two(1) == 3
        "}}
    }
}
