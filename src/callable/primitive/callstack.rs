use r_derive::builtin;

use crate::ast::ExprList;
use crate::lang::{CallStack, EvalResult, R};
use crate::callable::core::*;

#[derive(Debug, Clone, PartialEq)]
#[builtin(sym = "callstack")]
pub struct PrimitiveCallstack;
impl Callable for PrimitiveCallstack {
    fn call(&self, _args: ExprList, stack: &mut CallStack) -> EvalResult {
        Ok(R::List(
            stack.frames.iter()
                .skip(1) // skip global frame
                .map(|f| (None, R::Expr(f.call.clone())))
                .collect()
        ))
    }
}
