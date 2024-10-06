use r_derive::builtin;

use crate::callable::core::*;
use crate::lang::{CallStack, EvalResult};
use crate::{formals, object::*};

/// Get the Current Call Stack
///
/// Returns a list of frames in the call stack.
///
/// # In-Language
///
/// ## Usage
///
/// ```custom,{class=r}
/// callstack()
/// ```
///
/// ## Arguments
///
/// _none_
///
/// ## Examples
///
/// ```custom,{class=r-repl}
/// h <- fn() callstack()
/// g <- fn() h()
/// f <- fn() g()
/// f()
/// ```
///
#[doc(alias = "callstack")]
#[builtin(sym = "callstack")]
#[derive(Debug, Clone, PartialEq)]
pub struct PrimitiveCallstack;

formals!(PrimitiveCallstack, "()");

impl Callable for PrimitiveCallstack {
    fn call(&self, _args: ExprList, stack: &mut CallStack) -> EvalResult {
        Ok(Obj::List(List::from(
            stack
                .frames
                .iter()
                .skip(1) // skip global frame
                .map(|f| (OptionNA::NA, Obj::Expr(f.call.clone())))
                .collect::<Vec<_>>(),
        )))
    }
}
