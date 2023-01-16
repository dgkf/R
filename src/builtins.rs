use crate::ast::*;
use crate::lang::*;
use crate::r_vector::vectors::*;
use crate::utils::*;

use core::fmt;
use std::fmt::Display;
use std::rc::Rc;

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
    fn call(&self, args: RExprList, env: &mut Environment) -> EvalResult;
    fn callable_as_str(&self) -> &str;
    fn call_as_str(&self, args: &RExprList) -> String;
    fn callable_clone(&self) -> Box<dyn Callable>;
}

impl Display for dyn Callable {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.callable_as_str())
    }
}

#[derive(Debug, Clone)]
pub struct RExprIf;

impl Callable for RExprIf {
    fn call(&self, args: RExprList, env: &mut Environment) -> EvalResult {
        let mut args = args.values.into_iter();

        let cond = eval(args.next().unwrap(), env)?;
        let cond: bool = cond.try_into()?;

        if cond {
            eval(args.next().unwrap(), env)
        } else {
            eval(args.skip(1).next().unwrap_or(RExpr::Null), env)
        }
    }

    fn callable_clone(&self) -> Box<dyn Callable> {
        Box::new(self.clone())
    }

    fn callable_as_str(&self) -> &str {
        "if"
    }

    fn call_as_str(&self, args: &RExprList) -> String {
        if let Some(else_expr) = args.values.get(2) {
            format!(
                "if ({}) {} else {}",
                args.values[0], args.values[1], else_expr
            )
        } else {
            format!("if ({}) {}", args.values[0], args.values[1])
        }
    }
}

#[derive(Debug, Clone)]
pub struct RExprFor;

impl Callable for RExprFor {
    fn call(&self, args: RExprList, env: &mut Environment) -> EvalResult {
        let mut args = args.into_iter();

        let (Some(var), iter_expr) = args.next().unwrap() else {
            unreachable!()
        };

        let (_, body) = args.next().unwrap();
        let iter = eval(iter_expr, env)?;

        let mut eval_result: EvalResult;
        let mut result = R::Null;
        let mut index = 0;

        while let Some(value) = iter.get(index) {
            index += 1;

            env.insert(var.clone(), value);
            eval_result = eval(body.clone(), env);

            // TODO: use std::ops::ControlFlow?
            match eval_result {
                Err(RSignal::Condition(Cond::Break)) => break,
                Err(RSignal::Condition(Cond::Continue)) => continue,
                Err(RSignal::Condition(Cond::Return(_))) => return eval_result,
                Err(RSignal::Error(_)) => return eval_result,
                _ => (),
            }

            result = eval_result.expect("unhandled eval err");
        }

        Ok(result)
    }

    fn callable_clone(&self) -> Box<dyn Callable> {
        Box::new(self.clone())
    }

    fn callable_as_str(&self) -> &str {
        "for"
    }

    fn call_as_str(&self, args: &RExprList) -> String {
        let Some(sym) = &args.keys[0] else {
            unreachable!()
        };

        format!("for ({} in {}) {}", sym, args.values[0], args.values[1])
    }
}

#[derive(Debug, Clone)]
pub struct RExprWhile;

impl Callable for RExprWhile {
    fn call(&self, args: RExprList, env: &mut Environment) -> EvalResult {
        let mut args = args.values.into_iter();
        let cond = args.next().unwrap();
        let body = args.next().unwrap();

        let mut eval_result: EvalResult;
        let mut result = R::Null;

        loop {
            // handle while condition
            let cond_result = eval(cond.clone(), env)?;
            if cond_result.try_into()? {
                eval_result = eval(body.clone(), env);
            } else {
                break;
            }

            // handle control flow signals during execution
            match eval_result {
                Err(RSignal::Condition(Cond::Break)) => break,
                Err(RSignal::Condition(Cond::Continue)) => continue,
                Err(RSignal::Condition(Cond::Return(_))) => return eval_result,
                Err(RSignal::Error(_)) => return eval_result,
                _ => (),
            }

            // update result
            result = eval_result.expect("unhandled eval err");
        }

        Ok(result)
    }

    fn callable_clone(&self) -> Box<dyn Callable> {
        Box::new(self.clone())
    }

    fn callable_as_str(&self) -> &str {
        "while"
    }

    fn call_as_str(&self, args: &RExprList) -> String {
        format!("while ({}) {}", args.values[0], args.values[1])
    }
}

#[derive(Debug, Clone)]
pub struct RExprRepeat;

impl Callable for RExprRepeat {
    fn call(&self, args: RExprList, env: &mut Environment) -> EvalResult {
        let mut args = args.values.into_iter();
        let body = args.next().unwrap();

        let mut eval_result: EvalResult;
        let mut result = R::Null;

        loop {
            eval_result = eval(body.clone(), env);

            // handle control flow signals during execution
            match eval_result {
                Err(RSignal::Condition(Cond::Break)) => break,
                Err(RSignal::Condition(Cond::Continue)) => continue,
                Err(RSignal::Condition(Cond::Return(_))) => return eval_result,
                Err(RSignal::Error(_)) => return eval_result,
                _ => (),
            }

            // update result
            result = eval_result.expect("unhandled eval err");
        }

        Ok(result)
    }

    fn callable_clone(&self) -> Box<dyn Callable> {
        Box::new(self.clone())
    }

    fn callable_as_str(&self) -> &str {
        "repeat"
    }

    fn call_as_str(&self, args: &RExprList) -> String {
        format!("repeat {}", args.values[1])
    }
}

#[derive(Debug, Clone)]
pub struct RExprBlock;

impl Callable for RExprBlock {
    fn call(&self, args: RExprList, env: &mut Environment) -> EvalResult {
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
    fn call(&self, args: RExprList, env: &mut Environment) -> EvalResult {
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
    fn call(&self, mut args: RExprList, env: &mut Environment) -> EvalResult {
        // TODO: emit proper error
        let rhs = args.values.pop().unwrap_or(RExpr::Number(0.0));
        let lhs = args.values.pop().unwrap_or(RExpr::Number(0.0));

        let lhs = eval(lhs, env)?;
        let rhs = eval(rhs, env)?;

        let res = match (lhs, rhs) {
            (R::Vector(l), R::Vector(r)) => R::Vector(l + r),
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
pub struct InfixSub;

impl Callable for InfixSub {
    fn call(&self, mut args: RExprList, env: &mut Environment) -> EvalResult {
        // TODO: emit proper error
        let rhs = args.values.pop().unwrap_or(RExpr::Number(0.0));
        let lhs = args.values.pop().unwrap_or(RExpr::Number(0.0));

        let lhs = eval(lhs, env)?;
        let rhs = eval(rhs, env)?;

        let res = match (lhs, rhs) {
            (R::Vector(l), R::Vector(r)) => R::Vector(l - r),
            _ => R::Null,
        };

        Ok(res)
    }

    fn callable_clone(&self) -> Box<dyn Callable> {
        Box::new(self.clone())
    }

    fn callable_as_str(&self) -> &str {
        "-"
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
pub struct PrefixSub;

impl Callable for PrefixSub {
    fn call(&self, mut args: RExprList, env: &mut Environment) -> EvalResult {
        // TODO: emit proper error
        let what = args.values.pop().unwrap();
        let what = eval(what, env)?;
        let res = match what {
            R::Vector(l) => R::Vector(-l),
            _ => R::Null,
        };

        Ok(res)
    }

    fn callable_clone(&self) -> Box<dyn Callable> {
        Box::new(self.clone())
    }

    fn callable_as_str(&self) -> &str {
        "-"
    }

    fn call_as_str(&self, args: &RExprList) -> String {
        format!("{}{}", self.callable_as_str(), args.values[0])
    }
}

#[derive(Debug, Clone)]
pub struct InfixMul;

impl Callable for InfixMul {
    fn call(&self, mut args: RExprList, env: &mut Environment) -> EvalResult {
        // TODO: emit proper error
        let rhs = args.values.pop().unwrap_or(RExpr::Number(0.0));
        let lhs = args.values.pop().unwrap_or(RExpr::Number(0.0));

        let lhs = eval(lhs, env)?;
        let rhs = eval(rhs, env)?;

        let res = match (lhs, rhs) {
            (R::Vector(l), R::Vector(r)) => R::Vector(l * r),
            _ => R::Null,
        };

        Ok(res)
    }

    fn callable_clone(&self) -> Box<dyn Callable> {
        Box::new(self.clone())
    }

    fn callable_as_str(&self) -> &str {
        "*"
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
pub struct InfixDiv;

impl Callable for InfixDiv {
    fn call(&self, mut args: RExprList, env: &mut Environment) -> EvalResult {
        // TODO: emit proper error
        let rhs = args.values.pop().unwrap_or(RExpr::Number(0.0));
        let lhs = args.values.pop().unwrap_or(RExpr::Number(0.0));

        let lhs = eval(lhs, env)?;
        let rhs = eval(rhs, env)?;

        let res = match (lhs, rhs) {
            (R::Vector(l), R::Vector(r)) => R::Vector(l / r),
            _ => R::Null,
        };

        Ok(res)
    }

    fn callable_clone(&self) -> Box<dyn Callable> {
        Box::new(self.clone())
    }

    fn callable_as_str(&self) -> &str {
        "/"
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
pub struct InfixPow;

impl Callable for InfixPow {
    fn call(&self, mut args: RExprList, env: &mut Environment) -> EvalResult {
        // TODO: emit proper error
        let rhs = args.values.pop().unwrap_or(RExpr::Number(0.0));
        let lhs = args.values.pop().unwrap_or(RExpr::Number(0.0));

        let lhs = eval(lhs, env)?;
        let rhs = eval(rhs, env)?;

        let res = match (lhs, rhs) {
            (R::Vector(l), R::Vector(r)) => R::Vector(l.power(r)),
            _ => R::Null,
        };

        Ok(res)
    }

    fn callable_clone(&self) -> Box<dyn Callable> {
        Box::new(self.clone())
    }

    fn callable_as_str(&self) -> &str {
        "^"
    }

    fn call_as_str(&self, args: &RExprList) -> String {
        format!(
            "{}{}{}",
            args.values[0],
            self.callable_as_str(),
            args.values[1]
        )
    }
}

#[derive(Debug, Clone)]
pub struct InfixMod;

impl Callable for InfixMod {
    fn call(&self, mut args: RExprList, env: &mut Environment) -> EvalResult {
        // TODO: emit proper error
        let rhs = args.values.pop().unwrap_or(RExpr::Number(0.0));
        let lhs = args.values.pop().unwrap_or(RExpr::Number(0.0));

        let lhs = eval(lhs, env)?;
        let rhs = eval(rhs, env)?;

        let res = match (lhs, rhs) {
            (R::Vector(l), R::Vector(r)) => R::Vector(l % r),
            _ => R::Null,
        };

        Ok(res)
    }

    fn callable_clone(&self) -> Box<dyn Callable> {
        Box::new(self.clone())
    }

    fn callable_as_str(&self) -> &str {
        "%%"
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
pub struct PostfixIndex;

impl Callable for PostfixIndex {
    fn call(&self, args: RExprList, env: &mut Environment) -> EvalResult {
        let mut args = args.into_iter();
        let (_, what) = args.next().unwrap();
        let (_, index) = args.next().unwrap();

        let what = eval(what, env)?;
        let index = eval(index, env)?;

        what.try_get(index)
    }

    fn callable_as_str(&self) -> &str {
        "[["
    }

    fn call_as_str(&self, args: &RExprList) -> String {
        let what = &args.values[0];

        let args = RExprList {
            keys: args.keys[1..].to_owned(),
            values: args.values[1..].to_owned(),
        };

        format!("{}[[{}]]", what, args)
    }

    fn callable_clone(&self) -> Box<dyn Callable> {
        Box::new(self.clone())
    }
}

#[derive(Debug, Clone)]
pub struct PostfixVecIndex;

impl Callable for PostfixVecIndex {
    fn call(&self, args: RExprList, env: &mut Environment) -> EvalResult {
        let mut args = args.into_iter();
        let (_, what) = args.next().unwrap();
        let (_, index) = args.next().unwrap();

        let what = eval(what, env)?;
        let index = eval(index, env)?;

        what.try_get(index)
    }

    fn callable_as_str(&self) -> &str {
        "["
    }

    fn call_as_str(&self, args: &RExprList) -> String {
        format!("{}[{}]", args.values[0], args.values[1])
    }

    fn callable_clone(&self) -> Box<dyn Callable> {
        Box::new(self.clone())
    }
}

#[derive(Debug, Clone)]
pub struct Name(String);

pub fn primitive(name: &str) -> Option<Box<dyn Fn(RExprList, &mut Environment) -> EvalResult>> {
    match name {
        "c" => Some(Box::new(c)),
        _ => None,
    }
}

pub fn c(args: RExprList, env: &mut Environment) -> EvalResult {
    // this can be cleaned up quite a bit, but I just need it working with
    // more types for now to test vectorized operators using different types

    let R::List(vals) = eval_rexprlist(args, env)? else {
        unreachable!()
    };

    // force any closures that were created during call
    let vals: Vec<_> = vals
        .into_iter()
        .map(|(k, v)| (k, force(v).unwrap_or(R::Null))) // TODO: raise this error
        .collect();

    // until there's a better way of handling type hierarchy, this will do
    let t: u8 = vals
        .iter()
        .map(|(_, v)| match v {
            R::Null => 0,
            R::Vector(vec) => match vec {
                RVector::Logical(_) => 1,
                RVector::Integer(_) => 2,
                RVector::Numeric(_) => 3,
                RVector::Character(_) => 4,
            },
            R::List(_) => 5,
            _ => 0,
        })
        .fold(0, std::cmp::max);

    match t {
        0 => Ok(R::Null),
        // Coerce everything into logical
        1 => {
            let mut output = vec![OptionNA::Some(true); 0];
            for (_, val) in vals {
                match val {
                    R::Null => continue,
                    R::Vector(RVector::Logical(mut v)) => output.append(&mut v),
                    _ => unimplemented!(),
                }
            }
            Ok(R::Vector(RVector::Logical(output)))
        }
        // Coerce everything into integer
        2 => {
            let mut output = vec![OptionNA::Some(0); 0];
            for (_, val) in vals {
                match val {
                    R::Null => continue,
                    R::Vector(RVector::Integer(mut v)) => output.append(&mut v),
                    R::Vector(RVector::Logical(v)) => {
                        output.append(&mut RVector::vec_coerce::<bool, i32>(&v))
                    }
                    _ => unimplemented!(),
                }
            }
            Ok(R::Vector(RVector::Integer(output)))
        }
        // Coerce everything into numeric
        3 => {
            let mut output = vec![OptionNA::Some(0.0); 0];
            for (_, val) in vals {
                match val {
                    R::Null => continue,
                    R::Vector(RVector::Numeric(mut v)) => output.append(&mut v),
                    R::Vector(RVector::Integer(v)) => {
                        output.append(&mut RVector::vec_coerce::<i32, f64>(&v))
                    }
                    R::Vector(RVector::Logical(v)) => {
                        output.append(&mut RVector::vec_coerce::<bool, f64>(&v))
                    }
                    _ => {
                        println!("{:#?}", val);
                        unimplemented!()
                    }
                }
            }
            Ok(R::Vector(RVector::Numeric(output)))
        }
        _ => Ok(R::Null),
    }
}

impl Callable for String {
    fn call(&self, args: RExprList, env: &mut Environment) -> EvalResult {
        if let Some(f) = primitive(self) {
            return f(args, env);
        }

        (env.get(self.clone())?).call(args, env)
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

impl Callable for R {
    fn call(&self, args: RExprList, env: &mut Environment) -> EvalResult {
        let R::Function(formals, body, fn_env) = self else {
            unimplemented!("can't call non-function")
        };

        // set up our local scope, a child environment of the function environment
        let local_scope = Environment::new(Env {
            parent: Some(Rc::clone(fn_env)),
            ..Default::default()
        });

        // evaluate arguments in calling environment
        let R::List(args) = eval_rexprlist(args, env)? else {
            unreachable!();
        };

        // match arguments against function signature
        let (args, ellipsis) = match_args(formals.clone(), args, env);

        // add closures to local scope
        local_scope.insert("...".to_string(), R::List(ellipsis));
        local_scope.append(R::List(args));

        // evaluate body in local scope
        eval(body.clone(), &mut Rc::clone(&local_scope))
    }

    fn callable_as_str(&self) -> &str {
        todo!()
    }

    fn call_as_str(&self, args: &RExprList) -> String {
        todo!()
    }

    fn callable_clone(&self) -> Box<dyn Callable> {
        todo!()
    }
}
