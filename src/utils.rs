use std::rc::Rc;

// TODO: move eval to builtin accepting R::Expr to remove ast dependence, using just lang
use crate::ast::{RExpr, RExprList};
use crate::error::*;
use crate::lang::*;

pub fn eval(expr: RExpr, env: &mut Environment) -> Result<R, RError> {
    match expr {
        RExpr::Null => Ok(R::Null),
        RExpr::Number(x) => Ok(R::Numeric(vec![x])),
        RExpr::Integer(x) => Ok(R::Integer(vec![x])),
        RExpr::Bool(x) => Ok(R::Logical(vec![x])),
        RExpr::String(x) => Ok(R::Character(vec![x])),
        RExpr::Function(formals, body) => Ok(R::Function(formals, *body, Rc::clone(env))),
        RExpr::Call(what, list) => Ok(what.call(list, env)?),
        RExpr::Symbol(name) => env.get(name),
        RExpr::List(x) => Ok(eval_rexprlist(x, &mut Rc::clone(env))?),
        x => unimplemented!("eval({})", x),
    }
}

// pub fn expand_ellipsis(args: R, env: &Environment) -> Result<RExprList, RError> {
//     if let Some(i) = args.position_ellipsis() {
//         let mut args = args.clone();
//         args.remove(i);
//
//         let R::List(ellipsis) = env.get_ellipsis()? else {
//             unreachable!();
//         };
//
//         args.keys.splice(i..i, ellipsis.keys);
//         args.values.splice(i..i, ellipsis.values);
//
//         Ok(args)
//     } else {
//         Ok(args)
//     }
// }

pub fn eval_rexprlist(x: RExprList, env: &mut Environment) -> Result<R, RError> {
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
