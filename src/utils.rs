// TODO: move eval to builtin accepting R::Expr to remove ast dependence, using just lang
use crate::ast::RExpr;
use crate::error::*;
use crate::lang::*;

pub fn eval(expr: RExpr, env: &mut Environment) -> Result<R, RError> {
    match expr {
        RExpr::Null => Ok(R::Null),
        RExpr::Number(x) => Ok(R::Numeric(vec![x])),
        RExpr::Integer(x) => Ok(R::Integer(vec![x])),
        RExpr::Bool(x) => Ok(R::Logical(vec![x])),
        RExpr::String(x) => Ok(R::Character(vec![x])),
        RExpr::Function(formals, body) => Ok(R::Function(formals, *body)),
        RExpr::Call(what, list) => Ok(what.call(list, env)?),
        RExpr::Symbol(name) => env.get(name),
        _ => unimplemented!(),
    }
}
