use r_derive::Primitive;

use crate::ast::ExprList;
use crate::lang::{CallStack, EvalResult, R};
use crate::callable::core::*;

#[derive(Debug, Clone, Primitive)]
pub struct PrimitiveCallstack;

impl PrimitiveSYM for PrimitiveCallstack {
    const SYM: &'static str = "callstack";
}

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
