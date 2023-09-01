use r_derive::*;

use crate::ast::*;
use crate::lang::*;
use crate::callable::core::*;

#[derive(Debug, Clone, PartialEq)]
#[builtin(sym = "q")]
pub struct PrimitiveQ;
impl Callable for PrimitiveQ {
    fn call(&self, _args: ExprList, _stack: &mut CallStack) -> EvalResult {
        Err(RSignal::Condition(Cond::Terminate))
    }
}
