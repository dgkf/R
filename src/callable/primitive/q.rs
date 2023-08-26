use r_derive::Primitive;

use crate::ast::*;
use crate::lang::*;
use crate::callable::core::*;

#[derive(Debug, Clone, Primitive)]
pub struct PrimitiveQ;

impl PrimitiveSYM for PrimitiveQ {
    const SYM: &'static str = "q";
}

impl Callable for PrimitiveQ {
    fn call(&self, _args: ExprList, _stack: &mut CallStack) -> EvalResult {
        Err(RSignal::Condition(Cond::Terminate))
    }
}
