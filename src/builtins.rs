use crate::ast::*;
use crate::error::*;
use crate::lang::*;
use crate::utils::eval;

use core::fmt;
use std::fmt::Display;
use std::rc::Rc;

fn op_vectorized_recycled<F, T>(f: F, mut e1: Vec<T>, e2: Vec<T>) -> Vec<T>
where
    F: Fn(T, T) -> T,
    T: Clone + Display,
{
    if e2.len() > e1.len() {
        return op_vectorized_recycled(f, e2, e1);
    }

    for i in 0..e1.len() {
        e1[i] = f(e1[i].clone(), e2[i % e2.len()].clone())
    }

    e1
}

fn match_args(formals: RExprList, args: RExprList) -> RExprList {
    use RExprListKey::*;

    let mut assigned = vec![false; formals.values.len()];
    let mut matched_args = formals.clone();

    // assign named args
    for (k, v) in args.keys.iter().zip(args.values.iter()) {
        if let Some(argname) = k {
            let key = Some(Name(argname.clone()));
            let index = matched_args.insert(key, v.clone());
            if index >= assigned.len() {
                assigned.extend(vec![false; index - assigned.len() + 1])
            }
            assigned[index] = true;
        }
    }

    // backfill unnamed args
    for (k, v) in args.keys.iter().zip(args.values.iter()) {
        if let None = k {
            if let Some(next_index) = assigned.iter().position(|&i| !i) {
                let key = if next_index < formals.keys.len() {
                    Some(Name(formals.keys[next_index].clone().unwrap()))
                } else {
                    Some(Index(next_index))
                };
                let index = matched_args.insert(key, v.clone());
                if index >= assigned.len() {
                    assigned.extend(vec![false; index - assigned.len() + 1])
                }
                assigned[index] = true;
            } else {
                let key = Some(Index(matched_args.values.len()));
                let index = matched_args.insert(key, v.clone());
                if index >= assigned.len() {
                    assigned.extend(vec![false; index - assigned.len() + 1])
                }
                assigned[index] = true;
            }
        }
    }

    matched_args
}

impl fmt::Debug for dyn Callable {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.callable_as_str())
    }
}

impl Clone for Box<dyn Callable> {
    fn clone(&self) -> Self {
        self.callable_clone()
    }
}

pub trait Callable {
    fn call(&self, args: RExprList, env: &mut Environment) -> Result<R, RError>;
    fn callable_clone(&self) -> Box<dyn Callable>;
    fn callable_as_str(&self) -> &str;
}

#[derive(Debug, Clone)]
pub struct RExprBlock;

impl Callable for RExprBlock {
    fn call(&self, args: RExprList, env: &mut Environment) -> Result<R, RError> {
        let mut value = Ok(R::Null);
        for expr in args.values {
            let result = eval(expr, env);
            match result {
                Ok(_) => value = result,
                _ => return result,
            }
        }
        value
    }

    fn callable_clone(&self) -> Box<dyn Callable> {
        Box::new(self.clone())
    }

    fn callable_as_str(&self) -> &str {
        "{"
    }
}

#[derive(Debug, Clone)]
pub struct InfixAssign;

impl Callable for InfixAssign {
    fn call(&self, args: RExprList, env: &mut Environment) -> Result<R, RError> {
        if let RExpr::Symbol(s) = &args.values[0] {
            let value = eval(args.values[1].clone(), env)?;
            env.insert(&s, value.clone());
            Ok(value)
        } else {
            unimplemented!()
        }
    }

    fn callable_clone(&self) -> Box<dyn Callable> {
        Box::new(self.clone())
    }

    fn callable_as_str(&self) -> &str {
        "<-"
    }
}

#[derive(Debug, Clone)]
pub struct InfixAdd;

impl Callable for InfixAdd {
    fn call(&self, args: RExprList, env: &mut Environment) -> Result<R, RError> {
        let lhs = eval(args.values[0].clone(), env)?;
        let rhs = eval(args.values[1].clone(), env)?;
        let op = |l, r| l + r;

        // TODO: improve vector type unification prior to math operations
        let res = match (lhs, rhs) {
            (R::Numeric(e1), R::Numeric(e2)) => R::Numeric(op_vectorized_recycled(op, e1, e2)),
            (R::Numeric(e1), R::Integer(e2)) => {
                if let R::Numeric(e2) = R::Integer(e2).as_numeric() {
                    R::Numeric(op_vectorized_recycled(op, e1, e2))
                } else {
                    R::Null
                }
            }
            (R::Integer(e1), R::Numeric(e2)) => {
                if let R::Numeric(e1) = R::Integer(e1).as_numeric() {
                    R::Numeric(op_vectorized_recycled(op, e1, e2))
                } else {
                    R::Null
                }
            }
            (R::Integer(e1), R::Integer(e2)) => {
                R::Integer(op_vectorized_recycled(|l, r| l + r, e1, e2))
            }
            _ => R::Null,
        };

        Ok(res)
    }

    fn callable_clone(&self) -> Box<dyn Callable> {
        Box::new(self.clone())
    }

    fn callable_as_str(&self) -> &str {
        "+"
    }
}

#[derive(Debug, Clone)]
pub struct Name(String);

impl Callable for String {
    fn call(&self, args: RExprList, env: &mut Environment) -> Result<R, RError> {
        if let R::Function(formals, body) = env.get(self.clone())? {
            // set up our local scope, a child environment of calling environment
            let local_scope = Environment::new(Env {
                parent: Some(Rc::clone(env)),
                ..Default::default()
            });

            // match arguments against function signature
            let args = match_args(formals, args);

            // create promises for matched args, do not evaluate until used
            for (k, expr) in args.keys.iter().zip(args.values.iter()) {
                if let Some(formal) = k {
                    local_scope.insert(&formal, R::Closure(expr.clone(), Rc::clone(env)));
                }
            }

            // evaluate body in local scope
            eval(body, &mut Rc::clone(&local_scope))
        } else {
            unimplemented!();
        }
    }

    fn callable_clone(&self) -> Box<dyn Callable> {
        Box::new(self.clone())
    }

    fn callable_as_str(&self) -> &str {
        self.as_str()
    }
}
