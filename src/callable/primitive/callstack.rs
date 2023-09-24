use r_derive::builtin;

use crate::callable::core::*;
use crate::lang::{CallStack, EvalResult};
use crate::object::*;

#[derive(Debug, Clone, PartialEq)]
#[builtin(sym = "callstack")]
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
