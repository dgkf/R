use r_derive::*;

use crate::callable::core::*;
use crate::lang::*;
use crate::object::*;

#[derive(Debug, Clone, PartialEq)]
#[builtin(sym = "quote")]
pub struct PrimitiveQuote;
impl Callable for PrimitiveQuote {
    fn call(&self, args: ExprList, _stack: &mut CallStack) -> EvalResult {
        Ok(Obj::Expr(args.get(0).unwrap_or(Expr::Null)))
    }
}
