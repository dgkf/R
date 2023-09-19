use r_derive::builtin;

use crate::ast::ExprList;
use crate::callable::core::*;
use crate::lang::{CallStack, EvalResult, List, R};

#[derive(Debug, Clone, PartialEq)]
#[builtin(sym = "callstack")]
pub struct PrimitiveCallstack;
impl Callable for PrimitiveCallstack {
    fn call(&self, _args: ExprList, stack: &mut CallStack) -> EvalResult {
        Ok(R::List(List::from(
            stack
                .frames
                .iter()
                .skip(1) // skip global frame
                .map(|f| (None, R::Expr(f.call.clone())))
                .collect::<Vec<_>>(),
        )))
    }
}
