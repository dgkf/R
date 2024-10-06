use r_derive::*;

use crate::callable::core::*;
use crate::object::ExprList;
use crate::{formals, lang::*};

/// Quit
///
/// Quit from the R runtime.
///
/// # In-Language
///
/// ## Usage
///
/// ```custom,{class=r}
/// q()
/// ```
///
/// ## Arguments
///
/// _none_
///
/// ## Examples
///
/// ```custom,{class=r-repl}
/// q()
/// ```
///
#[doc(alias = "q")]
#[builtin(sym = "q")]
#[derive(Debug, Clone, PartialEq)]
pub struct PrimitiveQ;

formals!(PrimitiveQ, "()");

impl Callable for PrimitiveQ {
    fn call(&self, _args: ExprList, _stack: &mut CallStack) -> EvalResult {
        Err(Signal::Condition(Cond::Terminate))
    }
}
