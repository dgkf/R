use r_derive::builtin;

use crate::callable::core::*;
use crate::lang::{CallStack, EvalResult};
use crate::object::*;

/// Get the Current Call Stack
///
/// Returns a list of frames in the call stack.
///
/// # Arguments
///
/// _none_
///
/// # Examples
///
/// ```{.r-repl}
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
impl Callable for PrimitiveCallstack {
    fn call(&self, _args: ExprList, stack: &mut CallStack) -> EvalResult {
        Ok(Obj::List(List::from(
            stack
                .frames
                .iter()
                .skip(1) // skip global frame
                .map(|f| (None, Obj::Expr(f.call.clone())))
                .collect::<Vec<_>>(),
        )))
    }
}
