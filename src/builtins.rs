use crate::ast::*;
use crate::error::*;
use crate::lang::*;
use crate::utils::*;

use core::fmt;
use std::cmp::max;
use std::fmt::Display;
use std::rc::Rc;

fn op_vectorized_recycled<F, O, T>(f: F, e1: Vec<O>, e2: Vec<T>) -> Vec<O>
where
    F: Fn((&O, &T)) -> O,
    T: Clone + Display,
{
    let n = max(e1.len(), e2.len());
    e1.iter()
        .cycle()
        .zip(e2.iter().cycle())
        .take(n)
        .map(f)
        .collect()
}

fn match_args(
    mut formals: RExprList,
    mut args: Vec<(Option<String>, R)>,
    env: &Environment,
) -> (Vec<(Option<String>, R)>, Vec<(Option<String>, R)>) {
    let mut ellipsis: Vec<(Option<String>, R)> = vec![];
    let mut matched_args: Vec<(Option<String>, R)> = vec![];

    // assign named args to corresponding formals
    let mut i: usize = 0;
    while i < args.len() {
        match &args[i].0 {
            Some(argname) => {
                if let Some((Some(_), _)) = formals.remove_named(&argname) {
                    matched_args.push(args.remove(i));
                    continue;
                }
            }
            _ => (),
        }
        i += 1;
    }

    // remove any Ellipsis param, and any trailing unassigned params
    formals.pop_trailing();

    // backfill unnamed args, populating ellipsis with overflow
    for (key, value) in args.into_iter() {
        match key {
            // named args go directly to ellipsis, they did not match a formal
            Some(arg) => {
                ellipsis.push((Some(arg), value));
            }

            // unnamed args populate next formal, or ellipsis if formals exhausted
            None => {
                if let Some((Some(param), _)) = formals.remove(0) {
                    matched_args.push((Some(param), value));
                } else {
                    ellipsis.push((None, value));
                }
            }
        }
    }

    // add back in parameter defaults that weren't filled with args
    for (param, default) in formals.into_iter() {
        matched_args.push((param, R::Closure(default, Rc::clone(env))));
    }

    (matched_args, ellipsis)
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
    fn call_as_str(&self, args: &RExprList) -> String;
}

impl Display for dyn Callable {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.callable_as_str())
    }
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

    fn call_as_str(&self, args: &RExprList) -> String {
        format!(
            "{{\n{}\n}}",
            args.clone()
                .into_iter()
                .map(|(_, v)| format!("  {}", v))
                .collect::<Vec<String>>()
                .join("\n")
        )
    }
}

#[derive(Debug, Clone)]
pub struct InfixAssign;

impl Callable for InfixAssign {
    fn call(&self, args: RExprList, env: &mut Environment) -> Result<R, RError> {
        let (RExpr::Symbol(s), value) = args.unnamed_binary_args() else {
            unimplemented!()
        };

        let value = eval(value, env)?;
        env.insert(s, value.clone());
        Ok(value)
    }

    fn callable_clone(&self) -> Box<dyn Callable> {
        Box::new(self.clone())
    }

    fn callable_as_str(&self) -> &str {
        "<-"
    }

    fn call_as_str(&self, args: &RExprList) -> String {
        format!(
            "{} {} {}",
            args.values[0],
            self.callable_as_str(),
            args.values[1]
        )
    }
}

#[derive(Debug, Clone)]
pub struct InfixAdd;

impl Callable for InfixAdd {
    fn call(&self, mut args: RExprList, env: &mut Environment) -> Result<R, RError> {
        // TODO: emit proper error
        let rhs = args.values.pop().unwrap_or(RExpr::Number(0.0));
        let lhs = args.values.pop().unwrap_or(RExpr::Number(0.0));

        let lhs = eval(lhs, env)?;
        let rhs = eval(rhs, env)?;

        // TODO: improve vector type unification prior to math operations
        let res = match (lhs, rhs) {
            (R::Numeric(e1), R::Numeric(e2)) => {
                R::Numeric(op_vectorized_recycled(|(&l, &r)| l + r, e1, e2))
            }
            (R::Numeric(e1), R::Integer(e2)) => {
                if let R::Numeric(e2) = R::Integer(e2).as_numeric() {
                    R::Numeric(op_vectorized_recycled(|(&l, &r)| l + r as f32, e1, e2))
                } else {
                    R::Null
                }
            }
            (R::Integer(e1), R::Numeric(e2)) => {
                if let R::Numeric(e1) = R::Integer(e1).as_numeric() {
                    R::Numeric(op_vectorized_recycled(|(&l, &r)| l as f32 + r, e1, e2))
                } else {
                    R::Null
                }
            }
            (R::Integer(e1), R::Integer(e2)) => {
                R::Integer(op_vectorized_recycled(|(&l, &r)| l + r, e1, e2))
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

    fn call_as_str(&self, args: &RExprList) -> String {
        format!(
            "{} {} {}",
            args.values[0],
            self.callable_as_str(),
            args.values[1]
        )
    }
}

#[derive(Debug, Clone)]
pub struct Name(String);

pub fn primitive(
    name: &str,
) -> Option<Box<dyn Fn(RExprList, &mut Environment) -> Result<R, RError>>> {
    match name {
        "c" => Some(Box::new(c)),
        _ => None,
    }
}

pub fn c(args: RExprList, env: &mut Environment) -> Result<R, RError> {
    let R::List(vals) = eval_rexprlist(args, env)? else {
        unreachable!()
    };

    let mut output = vec![0.0; 0];
    for (_, val) in vals {
        match val {
            R::Numeric(mut v) => output.append(&mut v),
            _ => unimplemented!(),
        }
    }

    Ok(R::Numeric(output))
}

impl Callable for String {
    fn call(&self, args: RExprList, env: &mut Environment) -> Result<R, RError> {
        if let Some(f) = primitive(self) {
            f(args, env)
        } else if let R::Function(formals, body, _) = env.get(self.clone())? {
            // set up our local scope, a child environment of calling environment
            let local_scope = Environment::new(Env {
                parent: Some(Rc::clone(env)),
                ..Default::default()
            });

            let R::List(args) = eval_rexprlist(args, env)? else {
                unreachable!();
            };

            // match arguments against function signature
            let (args, ellipsis) = match_args(formals, args, env);

            // add closures to local scope
            local_scope.insert("...".to_string(), R::List(ellipsis));
            local_scope.append(R::List(args));

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

    fn call_as_str(&self, args: &RExprList) -> String {
        format!("{}{}", self.callable_as_str(), args)
    }
}
