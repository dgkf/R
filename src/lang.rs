use crate::ast::*;
use crate::error::*;
use crate::callable::core::{Callable, string_as_primitive};
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
            (R::List(l), R::List(r)) => l.iter().zip(r.iter()).all(|((lk, lv), (rk, rv))| lk == rk && lv == rv),
            (R::Expr(l), R::Expr(r)) => l == r,
            (R::Closure(lc, lenv), R::Closure(rc, renv)) => lc == rc && lenv == renv,
            (R::Function(largs, lbody, lenv), R::Function(rargs, rbody, renv)) => largs == rargs && lbody == rbody && lenv == renv,
            (R::Environment(l), R::Environment(r)) => l == r,
            (R::Vector(lv), R::Vector(rv)) => match (lv, rv) {
                (Vector::Numeric(l), Vector::Numeric(r)) => l == r,
                (Vector::Integer(l), Vector::Integer(r)) => l == r,
                (Vector::Logical(l), Vector::Logical(r)) => l == r,
                (Vector::Character(l), Vector::Character(r)) => l == r,
                _ => false,
            },
            _ => false
        }
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
    pub fn force(self, stack: &mut CallStack) -> EvalResult {
        match self {
            R::Closure(expr, env) => {
                stack.add_frame(expr.clone(), env);
                let result = stack.eval(expr);
                stack.pop_frame_after(result)
            },
            _ => Ok(self),
        }
    }

    pub fn as_integer(self) -> EvalResult {
        match self {
            R::Vector(v) => Ok(R::Vector(v.as_integer())),
            R::Null => Ok(R::Vector(Vector::Integer(vec![]))),
            _ => Err(RSignal::Error(RError::CannotBeCoercedToInteger)),
        }
    }

    pub fn as_numeric(self) -> EvalResult {
        match self {
            R::Vector(v) => Ok(R::Vector(v.as_numeric())),
            R::Null => Ok(R::Vector(Vector::Numeric(vec![]))),
            atom => unreachable!("{:?} cannot be coerced to numeric", atom),
        }
    }

    pub fn as_logical(self) -> EvalResult {
        match self {
            R::Vector(v) => Ok(R::Vector(v.as_logical())),
            R::Null => Ok(R::Vector(Vector::Logical(vec![]))),
            atom => unreachable!("{:?} cannot be coerced to logical", atom),
        }
    }

    pub fn as_character(self) -> EvalResult {
        match self {
            R::Vector(v) => Ok(R::Vector(v.as_character())),
            R::Null => Ok(R::Vector(Vector::Character(vec![]))),
            atom => unreachable!("{:?} cannot be coerced to character", atom),
        }
    }

    pub fn as_vector(self) -> EvalResult {
        match self {
            R::Null => Ok(R::Vector(Vector::Logical(vec![]))),
            R::Vector(_) => Ok(self),
            _ => unimplemented!("cannot coerce object into vector"),
        }
    }

    pub fn into_usize(&self) -> Result<usize, RSignal> {
        use OptionNA::*;
        use Vector::*;
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

    pub fn environment(&self) -> Option<Rc<Environment>> {
        match self {
            R::Closure(_, e) | R::Function(_, _, e) | R::Environment(e) => Some(e.clone()),
            _ => None
        }        
    }

    pub fn try_get(&self, index: R) -> EvalResult {
        let i = index.into_usize()?;
        match self {
            R::Vector(rvec) => match rvec.get(i) {
                Some(v) => Ok(R::Vector(v)),
                None => Err(RSignal::Error(RError::Other("out of bounds".to_string()))),
            },
            R::List(lvec) => Ok(lvec[i - 1].1.clone()),
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

impl Display for R {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            R::Vector(v) => write!(f, "{}", v),
            R::Null => write!(f, "NULL"),
            R::Environment(x) => write!(f, "<environment {:?}>", x.values.as_ptr()),
            R::Function(formals, Expr::Primitive(primitive), _) => {
                write!(f, "function({}) .Primitive(\"{}\")", formals, primitive.rfmt())
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
    for (i, (name, value)) in x.iter().enumerate() {
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
                display_list(nested_values, f, Some(breadcrumbs))?
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

impl super::vector::vectors::Pow for R {
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

impl VecPartialCmp for R {
    type CmpOutput = Vec<Option<std::cmp::Ordering>>;
    type Output = EvalResult;

    fn vec_partial_cmp(self, rhs: Self) -> Self::CmpOutput {
        let Ok(sv) = self.as_vector() else {
            unimplemented!()
        };

        let Ok(rv) = rhs.as_vector() else {
            unimplemented!()
        };

        match (sv, rv) {
            (R::Vector(l), R::Vector(r)) => l.vec_partial_cmp(r),
            _ => unimplemented!(),
        }
    }

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
        match (self.as_vector()?, rhs.as_vector()?) {
            (R::Vector(l), R::Vector(r)) => Ok(R::Vector(l.vec_eq(r))),
            _ => unreachable!(),
        }
    }

    fn vec_neq(self, rhs: Self) -> Self::Output {
        match (self.as_vector()?, rhs.as_vector()?) {
            (R::Vector(l), R::Vector(r)) => Ok(R::Vector(l.vec_neq(r))),
            _ => unreachable!(),
        }
    }
}

pub type List = Vec<(Option<String>, R)>;

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
            _ => unreachable!()
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
            },
            error => error
        }
    }

    pub fn new() -> CallStack {
        CallStack::from(Frame { call: Expr::Missing, env: Rc::new(Environment::default())})
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
        CallStack { frames: vec![Frame { call: Expr::Missing, env: value.clone() }] }
    }
}

#[derive(Debug, Default, Clone, PartialEq)]
pub struct Environment {
    pub values: RefCell<HashMap<String, R>>,
    pub parent: Option<Rc<Environment>>,
}

impl Environment {
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
    fn get(&self, name: String) -> EvalResult {
        (*self).env().get(name)
    }

    fn get_ellipsis(&self) -> EvalResult {
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
        Ok(R::List(
            l.into_iter()
                .flat_map(|pair| match pair {
                    (_, Expr::Ellipsis) => {
                        if let Ok(R::List(ellipsis)) = self.get_ellipsis() {
                            ellipsis.into_iter()
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
                .collect(),
        ))
    }

    fn eval_list_greedy(&mut self, l: ExprList) -> EvalResult {
        Ok(R::List(
            l.into_iter()
                .flat_map(|pair| match pair {
                    (_, Expr::Ellipsis) => {
                        if let Ok(R::List(ellipsis)) = self.get_ellipsis() {
                            ellipsis.into_iter()
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
                .collect()
        ))
    }
}

impl Context for CallStack {
    fn env(&self) -> Rc<Environment> {
        self.last_frame().env.clone()
    }

    fn eval(&mut self, expr: Expr) -> EvalResult {
        if let Expr::List(x) = expr {
            Ok(self.eval_list_lazy(x)?)
        } else if let Expr::Call(what, args) = expr.clone() {
            match *what {
                Expr::Primitive(what) => {
                    self.add_frame(expr, self.last_frame().env.clone());
                    let result = what.call(args, self);
                    return self.pop_frame_after(result)
                },
                Expr::String(what) | Expr::Symbol(what) => {
                    // builtin primitives do not introduce a new call onto the stack
                    if let Ok(f) = string_as_primitive(&what) {
                        self.add_frame(expr, self.last_frame().env.clone());
                        let result = f.call(args, self);
                        return self.pop_frame_after(result)
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
                },
                _ => {
                    self.add_frame(expr, self.last_frame().env.clone());
                    let result = (self.eval(*what)?).call(args, self);
                    return self.pop_frame_after(result)
                }
            }
        } else {
            self.last_frame().eval(expr)
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
        use Vector::*;
        match expr {
            Expr::Null => Ok(R::Null),
            Expr::NA => Ok(R::Vector(Logical(vec![OptionNA::NA]))),
            Expr::Inf => Ok(R::Vector(Numeric(vec![OptionNA::Some(f64::INFINITY)]))),
            Expr::Number(x) => Ok(R::Vector(Vector::from(vec![x]))),
            Expr::Integer(x) => Ok(R::Vector(Vector::from(vec![x]))),
            Expr::Bool(x) => Ok(R::Vector(Logical(vec![OptionNA::Some(x)]))),
            Expr::String(x) => Ok(R::Vector(Character(vec![OptionNA::Some(x)]))),
            Expr::Function(formals, body) => Ok(R::Function(formals, *body, self.clone())),
            Expr::Symbol(name) => self.get(name),
            Expr::Break => Err(RSignal::Condition(Cond::Break)),
            Expr::Continue => Err(RSignal::Condition(Cond::Continue)),
            x => unimplemented!("Context::eval(Rc<Environment>, {})", x),
        }
    }

    fn get(&self, name: String) -> EvalResult {
        // search in this environment for value by name
        if let Some(value) = self.values.borrow().get(&name) {
            let result = value.clone();
            return match result {
                R::Closure(expr, mut env) => env.eval(expr),
                _ => Ok(result),
            };

        // if not found, search through parent if available
        } else if let Some(parent) = &self.parent {
            parent.get(name)

        // if we're at the top level, fall back to primitives if available
        } else if let Ok(prim) = name.as_str().try_into() {
            Ok(R::Function(ExprList::new(), Expr::Primitive(prim), self.env()))
            
        // otherwise, throw error
        } else {
            Err(RSignal::Error(RError::VariableNotFound(name)))
        }
    }
}