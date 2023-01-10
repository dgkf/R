use std::rc::Rc;

// TODO: move eval to builtin accepting R::Expr to remove ast dependence, using just lang
use crate::ast::{RExpr, RExprList};
use crate::lang::*;
use crate::r_vector::vectors::*;

pub fn eval(expr: RExpr, env: &mut Environment) -> EvalResult {
    match expr {
        RExpr::Null => Ok(R::Null),
        RExpr::NA => Ok(R::Vector(RVector::Logical(vec![OptionNA::NA]))),
        RExpr::Number(x) => Ok(R::Vector(RVector::from(vec![x]))),
        RExpr::Integer(x) => Ok(R::Vector(RVector::from(vec![x]))),
        RExpr::Bool(x) => Ok(R::Vector(RVector::Logical(vec![OptionNA::Some(x)]))),
        RExpr::String(x) => Ok(R::Vector(RVector::Character(vec![OptionNA::Some(x)]))),
        RExpr::Function(formals, body) => Ok(R::Function(formals, *body, Rc::clone(env))),
        RExpr::Call(what, args) => Ok(what.call(args, env)?),
        RExpr::Symbol(name) => env.get(name),
        RExpr::List(x) => Ok(eval_rexprlist(x, &mut Rc::clone(env))?),
        RExpr::Break => Err(RSignal::Condition(Cond::Break)),
        RExpr::Continue => Err(RSignal::Condition(Cond::Continue)),
        x => unimplemented!("eval({})", x),
    }
}

pub fn force(val: R) -> EvalResult {
    match val {
        R::Closure(expr, mut env) => eval(expr, &mut env),
        _ => Ok(val),
    }
}

pub fn eval_rexprlist(x: RExprList, env: &mut Environment) -> EvalResult {
    Ok(R::List(
        x.into_iter()
            .flat_map(|pair| match pair {
                (_, RExpr::Ellipsis) => {
                    if let Ok(R::List(ellipsis)) = env.get_ellipsis() {
                        ellipsis.into_iter()
                    } else {
                        vec![].into_iter()
                    }
                }
                (k, e @ (RExpr::Call(..) | RExpr::Symbol(..))) => {
                    let elem = vec![(k, R::Closure(e, Rc::clone(env)))];
                    elem.into_iter()
                }
                (k, v) => {
                    if let Ok(elem) = eval(v, &mut Rc::clone(env)) {
                        vec![(k, elem)].into_iter()
                    } else {
                        unreachable!()
                    }
                }
            })
            .collect(),
    ))
}
