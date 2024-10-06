use r_derive::*;

use crate::callable::core::*;
use crate::formals;
use crate::lang::*;
use crate::object::*;

/// Quote code
///
/// Capture code as a language object instead of evaluating it.
///
/// # In-Language
///
/// ## Usage
///
/// ```custom,{class=r}
/// quote(x)
/// ```
///
/// ## Arguments
///
/// `x`: An expression to capture.
///
/// ## Examples
///
/// ```custom,{class=r-repl}
/// quote(x + y)
/// ```
///
#[doc(alias = "quote")]
#[builtin(sym = "quote")]
#[derive(Debug, Clone, PartialEq)]
pub struct PrimitiveQuote;

formals!(PrimitiveQuote, "(x)");

impl Callable for PrimitiveQuote {
    fn call(&self, args: ExprList, _stack: &mut CallStack) -> EvalResult {
        Ok(Obj::Expr(args.get(0).unwrap_or(Expr::Null)))
    }
}
