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
/// quote(1)
/// ```
/// ## Differentes to the R implementation
/// While R treats literals as expressions this implementation of `quote` differentiates between
/// the literal `1` and the length-1 vector "`c(1)`".
/// Thereby the return type of `quote()` can be expected to be an object of type `Expression`.
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
